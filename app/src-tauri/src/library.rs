//! Tauri side of the game library (`emudome_core::library`): the database and covers on disk,
//! the commands the interface calls (through `app/src/emu.ts` only), background scans with
//! `library://scan` progress, `library://changed` after every change, play time, covers served on
//! `emu://cover/<id>?v=<rev>`, and folder watching (`library.watch`, desktop only).
//!
//! Files, all in `<app local data>` because paths are per device: `library/library.json` and
//! `covers/` (ROM icons as PNG, the user's images).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use emudome_core::library::{
    self, cover_colour, CoverKind, GameRecord, Library, ScanInput, ScanReport,
};
use emudome_core::settings::{ConsoleId, Scope, Value};
use emudome_core::ErrorCode;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::settings::{self as app_settings, SettingsState};
use crate::{AppState, GameInfo};

/// Largest image accepted as a cover.
const MAX_COVER_BYTES: u64 = 16 * 1024 * 1024;

pub struct LibraryState {
    lib: Mutex<Library>,
    covers: PathBuf,
    scanning: AtomicBool,
    /// A scan was asked for while one was running: run another when it ends.
    rescan: AtomicBool,
    /// Files dropped on the window, for the next scan.
    pending: Mutex<Vec<PathBuf>>,
    /// The library game being played, and since when.
    playing: Mutex<Option<(String, Instant)>>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

/// Opens the library database and the covers folder.
pub fn open(app: &tauri::App) -> tauri::Result<LibraryState> {
    let local = app.path().app_local_data_dir()?;
    let dir = local.join("library");
    log::info!("library in {}", dir.display());
    Ok(LibraryState {
        lib: Mutex::new(Library::open(&dir)),
        covers: local.join("covers"),
        scanning: AtomicBool::new(false),
        rescan: AtomicBool::new(false),
        pending: Mutex::new(Vec::new()),
        playing: Mutex::new(None),
        watcher: Mutex::new(None),
    })
}

/// At startup: a scan if `library.scanstart`, and the folder watcher if `library.watch`.
pub fn startup(app: &AppHandle) {
    if setting_bool(app, "library.scanstart") {
        start_scan(app);
    }
    refresh_watch(app);
}

// ---- DTOs (mirrors in app/src/emu.ts) ----

/// Mirror: `LibraryGame`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameDto {
    id: String,
    console: ConsoleId,
    title: String,
    path: String,
    folder: Option<String>,
    size: u64,
    available: bool,
    play_time_secs: u64,
    last_played: Option<u64>,
    added_at: u64,
    cover: CoverKind,
    cover_url: Option<String>,
    has_icon: bool,
    colour: &'static str,
    favourite: bool,
    hidden: bool,
    last_state_thumb_url: Option<String>,
}

impl GameDto {
    fn new(g: &GameRecord) -> Self {
        let cover_url = match g.cover {
            CoverKind::Generated => None,
            CoverKind::Icon | CoverKind::User => Some(emudome_platform::protocol_url(
                "emu",
                &format!("cover/{}?v={}", g.id.replace(':', "%3A"), g.cover_rev),
            )),
        };
        Self {
            id: g.id.clone(),
            console: g.console,
            title: g.title.clone(),
            path: g.path.clone(),
            folder: g.folder.clone(),
            size: g.size,
            available: g.available,
            play_time_secs: g.play_time_secs,
            last_played: g.last_played,
            added_at: g.added_at,
            cover: g.cover,
            cover_url,
            has_icon: g.has_icon,
            colour: cover_colour(&g.id),
            favourite: g.favourite,
            hidden: g.hidden,
            // TODO(save state thumbnails): states have no thumbnail yet.
            last_state_thumb_url: None,
        }
    }
}

/// Mirror: `LibraryFolder`.
#[derive(Serialize)]
struct FolderDto {
    path: String,
    reachable: bool,
}

/// Mirror: `Library`.
#[derive(Serialize)]
pub struct LibraryDto {
    games: Vec<GameDto>,
    folders: Vec<FolderDto>,
    scanning: bool,
}

/// Payload of `library://scan`. Mirror: `LibraryScanEvent`.
#[derive(Clone, Serialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
enum ScanEvent {
    Started,
    Progress {
        checked: usize,
        found: usize,
    },
    Done {
        found: usize,
        added: usize,
        unreachable: Vec<String>,
    },
}

// ---- Settings ----

fn setting(app: &AppHandle, key: &str) -> Option<Value> {
    let state = app.try_state::<SettingsState>()?;
    let resolved = state.0.lock().resolve(key, &Scope::Global);
    match resolved {
        Ok(r) => r.value,
        Err(e) => {
            log::warn!("{key} unavailable: {e:?}");
            None
        }
    }
}

fn setting_bool(app: &AppHandle, key: &str) -> bool {
    matches!(setting(app, key), Some(Value::Bool(true)))
}

/// `library.folders` of this device.
fn folders(app: &AppHandle) -> Vec<String> {
    match setting(app, "library.folders") {
        Some(Value::List(list)) => list,
        Some(Value::Bool(_) | Value::Int(_) | Value::Text(_)) | None => Vec::new(),
    }
}

/// Stores `library.folders` (Global level; the entry is per device, so it goes to this
/// device's file) and tells the interface.
fn set_folders(app: &AppHandle, list: Vec<String>) -> Result<(), ErrorCode> {
    let state = app.state::<SettingsState>();
    let level = state
        .0
        .lock()
        .set("library.folders", &Scope::Global, Value::List(list))?;
    app_settings::emit_changed(app, &["library.folders"], level);
    Ok(())
}

/// A library setting changed from the settings pages.
pub fn on_setting_changed(app: &AppHandle, key: &str) {
    match key {
        "library.folders" => {
            refresh_watch(app);
            start_scan(app);
        }
        "library.watch" => refresh_watch(app),
        _ => {}
    }
}

// ---- Events, time ----

fn emit_changed(app: &AppHandle) {
    if let Err(e) = app.emit("library://changed", ()) {
        log::warn!("library://changed: {e}");
    }
}

fn emit_scan(app: &AppHandle, event: ScanEvent) {
    if let Err(e) = app.emit("library://scan", event) {
        log::warn!("library://scan: {e}");
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or_default()
}

// ---- Scan ----

/// Starts a background scan (or queues one after the running scan).
pub fn start_scan(app: &AppHandle) {
    let Some(state) = app.try_state::<LibraryState>() else {
        return;
    };
    if state.scanning.swap(true, Ordering::SeqCst) {
        state.rescan.store(true, Ordering::SeqCst);
        return;
    }
    let thread_app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("library-scan".into())
        .spawn(move || {
            let app = thread_app;
            let state = app.state::<LibraryState>();
            loop {
                state.rescan.store(false, Ordering::SeqCst);
                run_scan(&app, &state);
                state.scanning.store(false, Ordering::SeqCst);
                // A request that came in meanwhile, and nobody else took over.
                if !state.rescan.load(Ordering::SeqCst)
                    || state.scanning.swap(true, Ordering::SeqCst)
                {
                    break;
                }
            }
        });
    if let Err(e) = spawned {
        log::error!("library scan thread: {e}");
        state.scanning.store(false, Ordering::SeqCst);
    }
}

fn run_scan(app: &AppHandle, state: &LibraryState) {
    emit_scan(app, ScanEvent::Started);
    let input = {
        let lib = state.lib.lock();
        let mut files = lib.single_files();
        files.append(&mut state.pending.lock());
        ScanInput {
            folders: folders(app).into_iter().map(PathBuf::from).collect(),
            files,
            cache: lib.cache().clone(),
        }
    };
    let started = Instant::now();
    let mut last: Option<Instant> = None;
    let outcome = library::scan(input, &mut |checked, found| {
        // About ten progress events a second.
        if last.is_none_or(|t| t.elapsed() >= Duration::from_millis(100)) {
            last = Some(Instant::now());
            emit_scan(app, ScanEvent::Progress { checked, found });
        }
    });
    let report = state.lib.lock().apply_scan(outcome, now_ms());
    let report = report.unwrap_or_else(|e| {
        log::error!("library scan not saved: {e:?}");
        ScanReport {
            found: 0,
            added: 0,
            unreachable: Vec::new(),
        }
    });
    log::info!(
        "library scan: {} games, {} added, {} folders unreachable, {:?}",
        report.found,
        report.added,
        report.unreachable.len(),
        started.elapsed()
    );
    cache_icons(state);
    emit_scan(
        app,
        ScanEvent::Done {
            found: report.found,
            added: report.added,
            unreachable: report.unreachable,
        },
    );
    emit_changed(app);
}

// ---- Covers ----

/// The cover file name of a game id (`:` is not allowed in Windows file names).
fn file_stem(id: &str) -> String {
    id.replace(':', "-")
}

fn icon_file(covers: &Path, id: &str) -> PathBuf {
    covers.join(format!("{}.icon.png", file_stem(id)))
}

/// Extracts the icon of a game into the covers folder, if it is not there yet.
fn cache_icon(covers: &Path, game: &GameRecord) -> Option<PathBuf> {
    let file = icon_file(covers, &game.id);
    if file.is_file() {
        return Some(file);
    }
    let icon = match library::read_icon(Path::new(&game.path)) {
        Ok(Some(icon)) => icon,
        Ok(None) => return None,
        Err(e) => {
            log::warn!("icon of {}: {e}", game.id);
            return None;
        }
    };
    let png = icon
        .to_png()
        .inspect_err(|e| log::warn!("icon of {}: {e}", game.id))
        .ok()?;
    let written = std::fs::create_dir_all(covers).and_then(|()| std::fs::write(&file, png));
    match written {
        Ok(()) => Some(file),
        Err(e) => {
            log::warn!("icon of {} not cached: {e}", game.id);
            None
        }
    }
}

/// After a scan: the icons of the reachable games, so they show even when the ROM is offline.
fn cache_icons(state: &LibraryState) {
    let games: Vec<GameRecord> = state
        .lib
        .lock()
        .games()
        .filter(|g| g.has_icon && g.available)
        .cloned()
        .collect();
    for game in &games {
        cache_icon(&state.covers, game);
    }
}

/// `emu://cover/<id>`: the game's icon or user image.
pub fn cover_response(app: &AppHandle, path: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let id = path.replace("%3A", ":").replace("%3a", ":");
    let found = app.try_state::<LibraryState>().and_then(|state| {
        let game = state.lib.lock().game(&id).ok().cloned()?;
        let file = match game.cover {
            CoverKind::Icon => cache_icon(&state.covers, &game)?,
            CoverKind::User => state.covers.join(game.user_cover.as_deref()?),
            CoverKind::Generated => return None,
        };
        let bytes = std::fs::read(&file)
            .inspect_err(|e| log::warn!("cover {}: {e}", file.display()))
            .ok()?;
        Some((bytes, image_type(&file)))
    });
    let builder = Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    let response = match found {
        // The URL changes with every cover change (`?v=`): it can be cached for good.
        Some((bytes, mime)) => builder
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(bytes),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.unwrap_or_else(crate::response_error)
}

fn image_type(file: &Path) -> &'static str {
    let ext = file
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    }
}

/// The image format of `bytes` from its signature, as a file extension.
fn image_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

/// Copies the user's image into the covers folder; returns its file name there.
fn import_cover(covers: &Path, id: &str, rev: u32, source: &Path) -> Result<String, ErrorCode> {
    let invalid = |detail: String| ErrorCode::CoverInvalid { detail };
    let size = std::fs::metadata(source)
        .map_err(|e| invalid(e.to_string()))?
        .len();
    if size > MAX_COVER_BYTES {
        return Err(invalid(format!("{size} bytes, limit {MAX_COVER_BYTES}")));
    }
    let bytes = std::fs::read(source).map_err(|e| invalid(e.to_string()))?;
    let ext = image_ext(&bytes).ok_or_else(|| invalid("not a PNG, JPEG or WebP image".into()))?;
    let name = format!("{}.user-{rev}.{ext}", file_stem(id));
    std::fs::create_dir_all(covers)
        .and_then(|()| std::fs::write(covers.join(&name), &bytes))
        .map_err(|e| ErrorCode::LibraryWriteFailed {
            detail: e.to_string(),
        })?;
    Ok(name)
}

// ---- Native pickers (desktop; Android has no file-path pickers: TODO with the SAF) ----

#[cfg(desktop)]
fn pick_folder(app: &AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("picked folder: {e}"))
        .ok()
}

#[cfg(desktop)]
fn pick_image(app: &AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
        .blocking_pick_file()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("picked image: {e}"))
        .ok()
}

#[cfg(mobile)]
fn pick_folder(_app: &AppHandle) -> Option<PathBuf> {
    log::warn!("folder picker not available on this platform yet");
    None
}

#[cfg(mobile)]
fn pick_image(_app: &AppHandle) -> Option<PathBuf> {
    log::warn!("image picker not available on this platform yet");
    None
}

// ---- Watching (desktop only) ----

/// (Re)creates the folder watcher from `library.watch` and `library.folders`.
pub fn refresh_watch(app: &AppHandle) {
    use notify::Watcher;
    let Some(state) = app.try_state::<LibraryState>() else {
        return;
    };
    let mut slot = state.watcher.lock();
    // Dropping the old watcher closes its channel, which ends its debounce thread.
    *slot = None;
    if emudome_platform::PHONE || !setting_bool(app, "library.watch") {
        return;
    }
    let (tx, rx) = mpsc::channel::<()>();
    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        // Reads (our own hashing included) are not changes.
        if res.is_ok_and(|e| !e.kind.is_access()) {
            let _ = tx.send(());
        }
    });
    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            log::warn!("library watch unavailable: {e}");
            return;
        }
    };
    for folder in folders(app) {
        if let Err(e) = watcher.watch(Path::new(&folder), notify::RecursiveMode::Recursive) {
            log::warn!("library watch {folder}: {e}");
        }
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("library-watch".into())
        .spawn(move || {
            while rx.recv().is_ok() {
                // Wait for two quiet seconds (a copy in progress sends many events).
                while rx.recv_timeout(Duration::from_secs(2)).is_ok() {}
                start_scan(&app);
            }
        });
    match spawned {
        Ok(_) => *slot = Some(watcher),
        Err(e) => log::warn!("library watch thread: {e}"),
    }
}

// ---- Play time ----

/// A library game started: remember it and when, record «last played».
pub fn begin_play(app: &AppHandle, id: &str) {
    let Some(state) = app.try_state::<LibraryState>() else {
        return;
    };
    *state.playing.lock() = Some((id.to_owned(), Instant::now()));
    if let Err(e) = state.lib.lock().mark_played(id, now_ms()) {
        log::warn!("last played of {id}: {e:?}");
    }
    emit_changed(app);
}

/// The game closed (or the app exits): add the time since it opened.
/// TODO: paused time and time in the quick menu count too, for now.
pub fn end_play(app: &AppHandle) {
    let Some(state) = app.try_state::<LibraryState>() else {
        return;
    };
    let Some((id, since)) = state.playing.lock().take() else {
        return;
    };
    let secs = since.elapsed().as_secs();
    if let Err(e) = state.lib.lock().add_play_time(&id, secs) {
        log::warn!("play time of {id}: {e:?}");
    }
    emit_changed(app);
}

/// The library game whose file is `path`: its id, title and console.
pub fn game_at(app: &AppHandle, path: &str) -> Option<(String, String, ConsoleId)> {
    let state = app.try_state::<LibraryState>()?;
    let lib = state.lib.lock();
    lib.game_at(path)
        .map(|g| (g.id.clone(), g.title.clone(), g.console))
}

// ---- Commands ----

/// The whole library for the interface.
#[tauri::command]
pub fn library_get(app: AppHandle, state: State<'_, LibraryState>) -> LibraryDto {
    let games: Vec<GameDto> = state.lib.lock().games().map(GameDto::new).collect();
    LibraryDto {
        games,
        folders: folders(&app)
            .into_iter()
            .map(|path| FolderDto {
                reachable: Path::new(&path).is_dir(),
                path,
            })
            .collect(),
        scanning: state.scanning.load(Ordering::SeqCst),
    }
}

/// Native folder picker: the folder joins `library.folders` and is scanned. `None` = cancelled.
#[tauri::command]
pub async fn library_choose_folder(app: AppHandle) -> Result<Option<String>, ErrorCode> {
    let Some(folder) = pick_folder(&app) else {
        return Ok(None);
    };
    let folder = folder.to_string_lossy().into_owned();
    add_folders(&app, std::slice::from_ref(&folder))?;
    start_scan(&app);
    Ok(Some(folder))
}

fn add_folders(app: &AppHandle, new: &[String]) -> Result<(), ErrorCode> {
    let mut list = folders(app);
    let before = list.len();
    for folder in new {
        if !list.contains(folder) {
            list.push(folder.clone());
        }
    }
    if list.len() != before {
        set_folders(app, list)?;
        refresh_watch(app);
    }
    Ok(())
}

/// Removes a folder from `library.folders`; its games leave the library (their play time and
/// flags are kept for when it comes back).
#[tauri::command]
pub fn library_remove_folder(
    app: AppHandle,
    state: State<'_, LibraryState>,
    path: String,
) -> Result<(), ErrorCode> {
    let list: Vec<String> = folders(&app).into_iter().filter(|f| *f != path).collect();
    set_folders(&app, list)?;
    state.lib.lock().unlist_folder(&path)?;
    refresh_watch(&app);
    emit_changed(&app);
    Ok(())
}

/// Drag and drop: folders join `library.folders`, files are added one by one, then a scan.
#[tauri::command]
pub fn library_add_paths(
    app: AppHandle,
    state: State<'_, LibraryState>,
    paths: Vec<String>,
) -> Result<(), ErrorCode> {
    let mut new_folders = Vec::new();
    for path in paths {
        let p = PathBuf::from(&path);
        if p.is_dir() {
            new_folders.push(path);
        } else if library::identity::is_rom(&p) {
            state.pending.lock().push(p);
        }
    }
    add_folders(&app, &new_folders)?;
    start_scan(&app);
    Ok(())
}

/// Rescans every folder in the background.
#[tauri::command]
pub fn library_scan(app: AppHandle) {
    start_scan(&app);
}

/// Opens a library game and counts its play time until it closes.
#[tauri::command]
pub fn library_open(
    app: AppHandle,
    app_state: State<'_, AppState>,
    state: State<'_, LibraryState>,
    id: String,
) -> Result<GameInfo, ErrorCode> {
    let (path, title, console) = {
        let lib = state.lib.lock();
        let game = lib.game(&id)?;
        (game.path.clone(), game.title.clone(), game.console)
    };
    if !Path::new(&path).is_file() {
        state.lib.lock().set_unavailable(&id)?;
        emit_changed(&app);
        return Err(ErrorCode::LibraryGameUnavailable { id });
    }
    crate::open_library_game(&app, &app_state, &id, PathBuf::from(path), title, console)
}

/// Changes the cover (L12). `user` opens an image picker first: `false` if it was cancelled.
#[tauri::command]
pub async fn library_set_cover(
    app: AppHandle,
    id: String,
    kind: CoverKind,
) -> Result<bool, ErrorCode> {
    let state = app.state::<LibraryState>();
    let rev = state.lib.lock().game(&id)?.cover_rev.wrapping_add(1);
    let user_file = match kind {
        CoverKind::User => {
            let Some(source) = pick_image(&app) else {
                return Ok(false);
            };
            Some(import_cover(&state.covers, &id, rev, &source)?)
        }
        CoverKind::Icon | CoverKind::Generated => None,
    };
    let old = {
        let mut lib = state.lib.lock();
        let old = lib.game(&id)?.user_cover.clone();
        lib.set_cover(&id, kind, user_file.clone())?;
        old
    };
    // The previous user image, replaced by the new one.
    if let Some(old) = old.filter(|_| user_file.is_some()) {
        if let Err(e) = std::fs::remove_file(state.covers.join(&old)) {
            log::warn!("old cover {old}: {e}");
        }
    }
    emit_changed(&app);
    Ok(true)
}

/// Favourite / hidden.
#[tauri::command]
pub fn library_set_flags(
    app: AppHandle,
    state: State<'_, LibraryState>,
    id: String,
    favourite: Option<bool>,
    hidden: Option<bool>,
) -> Result<(), ErrorCode> {
    state.lib.lock().set_flags(&id, favourite, hidden)?;
    emit_changed(&app);
    Ok(())
}
