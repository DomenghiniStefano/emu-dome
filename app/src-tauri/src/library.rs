//! Tauri side of the game library (`emudome_core::library`): the database and covers on disk,
//! the commands the interface calls (through `app/src/emu.ts` only), background scans with
//! `library://scan` progress, `library://changed` after every change, play time, covers served on
//! `emu://cover/<id>?v=<rev>`, and folder watching (`library.watch`, desktop only).
//!
//! Android (Trello #47): a library folder is a document tree picked through the system
//! (`emudome_platform::pick_document_tree`), stored in `library.folders` as its `content://` tree
//! URI (never turned into a path), walked through the platform's listing
//! (`emudome_platform::list_document_tree`, one call per folder) and its games read through
//! their descriptors. Not watched: rescanned at start (`library.scanstart`) and on demand.
//!
//! Files, all in `<app local data>` because paths are per device: `library/library.json` and
//! `covers/` (ROM icons as PNG; the user's covers, `<console>-<hex>.user-<rev>.jpg`, each the
//! normalized 600 × 900 copy `emudome_core::cover` makes from the user's image or a screenshot,
//! Trello #85).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use emudome_core::cover::{self, CoverView};
use emudome_core::library::{
    self, core_of, cover_colour, region, CoverKind, DocEntry, Documents, Family, GameRecord,
    Identity, Library, NotAGame, OpenedDocument, Region, ScanInput, ScanReport, Skipped,
};
use emudome_core::settings::{keys, ConsoleId, Scope, Value};
use emudome_core::{ErrorCode, GameKey, ImageProblem, IoCause};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use ts_rs::TS;

use crate::settings::{self as app_settings, SettingsState};
use crate::{AppState, GameInfo};

/// Largest image accepted as a cover source (and as a background, `settings.rs`).
pub(crate) const MAX_COVER_BYTES: u64 = cover::MAX_SOURCE_BYTES;

/// The image the user picked for «Adatta la copertina»: its bytes kept in memory (never copied
/// anywhere) until a cover is made from it or another one is picked.
struct PickedImage {
    token: u32,
    bytes: Vec<u8>,
    mime: &'static str,
}

pub struct LibraryState {
    lib: Mutex<Library>,
    covers: PathBuf,
    /// The last image picked for a cover, served on `emu://picked/<token>`.
    picked: Mutex<Option<PickedImage>>,
    next_token: AtomicU32,
    scanning: AtomicBool,
    /// A scan was asked for while one was running: run another when it ends.
    rescan: AtomicBool,
    /// Files dropped on the window, for the next scan.
    pending: Mutex<Vec<PathBuf>>,
    /// The library game being played, and since when.
    playing: Mutex<Option<(GameKey, Instant)>>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
    /// The picked document trees of `library.folders` (Android) as last seen: their display
    /// name and whether the last scan could list them, and why not. In memory: filled by the
    /// picker and by every scan.
    trees: Mutex<HashMap<String, TreeSeen>>,
}

/// A picked document tree as last seen ([`LibraryState::trees`]).
#[derive(Clone, Debug, Default)]
struct TreeSeen {
    name: Option<String>,
    access: FolderAccess,
}

/// Whether a library folder can be read now (E01), and why not.
/// TypeScript: `FolderAccess` (generated).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub(crate) enum FolderAccess {
    #[default]
    Reachable,
    /// Missing, on a disconnected drive, or unreadable for a reason EmuDome cannot tell.
    Unreachable,
    /// A picked Android folder whose read permission the app no longer holds (Android takes
    /// it back when the app is uninstalled, even with its data kept): picking it again is
    /// enough, its games and saves are keyed by their content.
    PermissionLost,
}

impl FolderAccess {
    /// A picked tree after a scan: `listed` whether it could be listed, `permitted` whether
    /// the app still holds its read permission (`None` when the system cannot tell).
    fn of_tree(listed: bool, permitted: Option<bool>) -> Self {
        match (listed, permitted) {
            (true, _) => Self::Reachable,
            (false, Some(false)) => Self::PermissionLost,
            (false, Some(true) | None) => Self::Unreachable,
        }
    }
}

/// Opens the library database and the covers folder.
pub fn open(app: &tauri::App) -> tauri::Result<LibraryState> {
    let local = crate::dirs::local(app)?;
    let dir = local.join("library");
    log::info!("library in {}", dir.display());
    Ok(LibraryState {
        lib: Mutex::new(Library::open(&dir)),
        covers: local.join("covers"),
        picked: Mutex::new(None),
        next_token: AtomicU32::new(0),
        scanning: AtomicBool::new(false),
        rescan: AtomicBool::new(false),
        pending: Mutex::new(Vec::new()),
        playing: Mutex::new(None),
        watcher: Mutex::new(None),
        trees: Mutex::new(HashMap::new()),
    })
}

/// Whether the library is set up: folders chosen or games in it (first run).
pub fn is_set_up(app: &AppHandle) -> bool {
    !folders(app).is_empty()
        || app
            .try_state::<LibraryState>()
            .is_some_and(|state| state.lib.lock().games().next().is_some())
}

/// `general.start` = «last game» (`startup.rs`): the row of the game played last, when its file
/// is still there (a `content://` document is not looked at: opening it says).
pub(crate) fn last_played_game(app: &AppHandle) -> Option<GameDto> {
    let state = app.try_state::<LibraryState>()?;
    let lib = state.lib.lock();
    let game = crate::startup::last_played(lib.games())?;
    let there = emudome_platform::is_content_uri(&game.path) || Path::new(&game.path).is_file();
    there.then(|| GameDto::new(game))
}

/// At startup: a scan if `library.scanstart`, and the folder watcher if `library.watch`.
pub fn startup(app: &AppHandle) {
    if settings(app).bool(keys::LIBRARY_SCAN_AT_START) {
        start_scan(app);
    }
    refresh_watch(app);
}

// ---- DTOs (TypeScript generated into app/src/generated/app/ by src/contract.rs) ----

/// TypeScript: `LibraryGame` (generated).
#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "LibraryGame")]
pub(crate) struct GameDto {
    /// The game's one identity (`GameId` in app/src/emu.ts): its saves and settings too.
    id: GameKey,
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
    /// «Nome mostrato» is the user's (L10).
    renamed: bool,
    /// The ROM's own title, what «Nome mostrato» goes back to.
    rom_title: String,
    /// Game code from the header (L10 «Codice del gioco»).
    code: Option<String>,
    /// ROM version from the header.
    version: Option<u16>,
    region: Option<Region>,
    /// The core that runs it (`mgba`, `melondsds`, `azahar`).
    core: &'static str,
}

impl GameDto {
    fn new(g: &GameRecord) -> Self {
        let cover_url = match g.cover {
            CoverKind::Generated => None,
            CoverKind::Icon | CoverKind::User => Some(emudome_platform::protocol_url(
                "emu",
                &format!("cover/{}?v={}", g.id.file_stem(), g.cover_rev),
            )),
        };
        Self {
            id: g.id.clone(),
            console: g.console,
            title: g.shown_title().to_owned(),
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
            colour: cover_colour(g.id.as_str()),
            favourite: g.favourite,
            hidden: g.hidden,
            // TODO(save state thumbnails): states have no thumbnail yet.
            last_state_thumb_url: None,
            renamed: g.name.is_some(),
            rom_title: g.title.clone(),
            code: g.code.clone(),
            version: g.version,
            region: region(g.code.as_deref()),
            core: core_of(g.console),
        }
    }
}

/// TypeScript: `LibraryFolder` (generated).
#[derive(Clone, Serialize, TS)]
#[ts(export_to = "app/", rename = "LibraryFolder")]
pub(crate) struct FolderDto {
    /// What `library.folders` holds: a path, or a picked document tree's `content://` URI
    /// (Android), never shown as it is.
    path: String,
    /// What to show: the path, or the tree's display name (null while it is not known).
    name: Option<String>,
    access: FolderAccess,
}

/// TypeScript: `Library` (generated).
#[derive(Clone, Serialize, TS)]
#[ts(export_to = "app/", rename = "Library")]
pub struct LibraryDto {
    games: Vec<GameDto>,
    folders: Vec<FolderDto>,
    scanning: bool,
    /// The regional versions grouped (#102): one tile per family, its `shown` version.
    families: Vec<Family>,
}

/// Payload of `library://changed`: what changed, for the interface to update in place instead of
/// asking for the whole library again. TypeScript: `LibraryChanged` (generated).
#[derive(Clone, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "LibraryChanged")]
pub(crate) enum LibraryChanged {
    /// These games' rows changed (play time, last played, cover, flags, name, availability):
    /// each replaces the row with its id.
    Games { games: Vec<GameDto> },
    /// Anything may have changed (a scan, a folder removed): the whole library.
    All { library: LibraryDto },
    /// The families changed (a pin, a game launched, hidden or shown, `library.versions`): the
    /// whole list.
    Families { families: Vec<Family> },
}

/// Payload of `library://scan`. TypeScript: `LibraryScanEvent` (generated).
#[derive(Clone, Serialize, TS)]
#[serde(tag = "phase", rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "LibraryScanEvent")]
pub(crate) enum ScanEvent {
    Started,
    Progress {
        checked: usize,
        found: usize,
    },
    Done {
        found: usize,
        added: usize,
        unreachable: Vec<String>,
        /// Files with a game's extension not taken (#127): encrypted 3DS games (E10), and files
        /// not recognised as a game.
        #[serde(rename = "skippedEncrypted")]
        skipped_encrypted: usize,
        #[serde(rename = "skippedUnrecognised")]
        skipped_unrecognised: usize,
    },
}

/// The image picked for a cover (`library_pick_cover_image`), for «Adatta la copertina»: its size
/// as shown and where the interface reads it. TypeScript: `PickedImage` (generated).
#[derive(Clone, Serialize, TS)]
#[ts(export_to = "app/", rename = "PickedImage")]
pub(crate) struct PickedDto {
    token: u32,
    url: String,
    width: u32,
    height: u32,
}

/// What a cover is made from (`library_set_cover_image`). TypeScript: `CoverSource` (generated).
#[derive(Clone, Debug, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CoverSource")]
pub(crate) enum CoverSource {
    /// The image of `library_pick_cover_image` with this token.
    Picked { token: u32 },
    /// The game's own screenshot with this file name (`library_screenshots`).
    Screenshot { name: String },
}

/// One screenshot of a game (`library_screenshots`), newest first. TypeScript:
/// `GameScreenshot` (generated).
#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "GameScreenshot")]
pub(crate) struct ScreenshotDto {
    /// Its file name in the game's screenshots folder: what `CoverSource::Screenshot` names.
    name: String,
    /// `emu://screenshot/<console>-<hex>/<name>`.
    url: String,
    /// When it was taken, Unix ms.
    taken_at: u64,
    /// Its size from the PNG header (`null` if unreadable).
    width: Option<u32>,
    height: Option<u32>,
}

// ---- Settings ----

/// The library's settings at the Global level (the catalogue's default for a value that cannot
/// be read).
fn settings(app: &AppHandle) -> keys::Reader<impl Fn(&str) -> Option<Value> + '_> {
    keys::Reader::new(move |key: &str| {
        let state = app.try_state::<SettingsState>()?;
        let resolved = state.0.lock().resolve(key, &Scope::Global);
        match resolved {
            Ok(r) => r.value,
            Err(e) => {
                log::warn!("{key} unavailable: {e:?}");
                None
            }
        }
    })
}

/// `library.folders` of this device.
fn folders(app: &AppHandle) -> Vec<String> {
    settings(app).list(keys::LIBRARY_FOLDERS)
}

/// Stores `library.folders` (Global level; the entry is per device, so it goes to this
/// device's file) and tells the interface.
fn set_folders(app: &AppHandle, list: Vec<String>) -> Result<(), ErrorCode> {
    let state = app.state::<SettingsState>();
    let level = state.0.lock().set(
        keys::LIBRARY_FOLDERS.as_str(),
        &Scope::Global,
        Value::List(list),
    )?;
    app_settings::emit_changed(app, &[keys::LIBRARY_FOLDERS.as_str()], level);
    Ok(())
}

/// A library setting changed from the settings pages.
pub fn on_setting_changed(app: &AppHandle, key: &str) {
    if key == keys::LIBRARY_FOLDERS.as_str() {
        refresh_watch(app);
        start_scan(app);
    } else if key == keys::LIBRARY_WATCH.as_str() {
        refresh_watch(app);
    } else if key == keys::LIBRARY_VERSIONS.as_str() {
        if let Some(state) = app.try_state::<LibraryState>() {
            emit_families(app, &state);
        }
    }
}

// ---- Events, time ----

fn emit_changed(app: &AppHandle, change: LibraryChanged) {
    if let Err(e) = app.emit(crate::events::LIBRARY_CHANGED, change) {
        log::warn!("library://changed: {e}");
    }
}

/// The rows of `ids` changed.
fn emit_games(app: &AppHandle, state: &LibraryState, ids: &[&GameKey]) {
    let games = {
        let lib = state.lib.lock();
        ids.iter()
            .filter_map(|id| lib.game(id).ok().map(GameDto::new))
            .collect()
    };
    emit_changed(app, LibraryChanged::Games { games });
}

/// Anything may have changed: the whole library.
fn emit_all(app: &AppHandle, state: &LibraryState) {
    let library = library_dto(app, state);
    emit_changed(app, LibraryChanged::All { library });
}

// ---- Families of regional versions (#102) ----

/// `library.versions`: one tile per family (read before the library's lock is taken).
fn grouped(app: &AppHandle) -> bool {
    settings(app).bool(keys::LIBRARY_VERSIONS)
}

/// The families the library shows: none while `library.versions` is off (every dump its own
/// tile). Grouped once per change, never per render.
fn families(grouped: bool, lib: &Library) -> Vec<Family> {
    if grouped {
        lib.families()
    } else {
        Vec::new()
    }
}

/// The families changed: the whole list.
fn emit_families(app: &AppHandle, state: &LibraryState) {
    let grouped = grouped(app);
    let families = families(grouped, &state.lib.lock());
    emit_changed(app, LibraryChanged::Families { families });
}

fn emit_scan(app: &AppHandle, event: ScanEvent) {
    if let Err(e) = app.emit(crate::events::LIBRARY_SCAN, event) {
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
                // A scan that panics (a malformed file the identity code did not expect) must not
                // leave the interface waiting for its end: it is logged and reported as done.
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_scan(&app, &state)))
                    .is_err()
                {
                    log::error!("library scan stopped: a file could not be read (panic above)");
                    emit_scan(
                        &app,
                        ScanEvent::Done {
                            found: 0,
                            added: 0,
                            unreachable: Vec::new(),
                            skipped_encrypted: 0,
                            skipped_unrecognised: 0,
                        },
                    );
                }
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
    let (paths, trees): (Vec<String>, Vec<String>) = folders(app)
        .into_iter()
        .partition(|f| !emudome_platform::is_content_uri(f));
    // The trees' names, read again (renamed, or the first scan of this run).
    for tree in &trees {
        let name = emudome_platform::document_tree_name(tree);
        state.trees.lock().entry(tree.clone()).or_default().name = name;
    }
    let input = {
        let lib = state.lib.lock();
        let mut files = lib.single_files();
        files.append(&mut state.pending.lock());
        ScanInput {
            folders: paths.into_iter().map(PathBuf::from).collect(),
            trees: trees.clone(),
            files,
            cache: lib.cache().clone(),
        }
    };
    let started = Instant::now();
    let mut last: Option<Instant> = None;
    let documents = PlatformDocuments::new(app);
    let outcome = library::scan_with_documents(input, &documents, &mut |checked, found| {
        // About ten progress events a second.
        if last.is_none_or(|t| t.elapsed() >= Duration::from_millis(100)) {
            last = Some(Instant::now());
            emit_scan(app, ScanEvent::Progress { checked, found });
        }
    });
    // Asked of the system before the lock (the interface reads it on the event loop), and only
    // of a tree that could not be listed: the one case the answer changes what is shown.
    let accesses: Vec<(&String, FolderAccess)> = trees
        .iter()
        .map(|tree| {
            let listed = !outcome.unreachable.contains(tree);
            let permitted = if listed {
                None
            } else {
                emudome_platform::document_tree_permitted(tree)
            };
            (tree, FolderAccess::of_tree(listed, permitted))
        })
        .collect();
    {
        let mut seen = state.trees.lock();
        for (tree, access) in accesses {
            if access == FolderAccess::PermissionLost {
                log::warn!("library scan: tree {tree}: read permission no longer held");
            }
            seen.entry(tree.clone()).or_default().access = access;
        }
    }
    let report = state.lib.lock().apply_scan(outcome, now_ms());
    let report = report.unwrap_or_else(|e| {
        log::error!("library scan not saved: {e:?}");
        ScanReport {
            found: 0,
            added: 0,
            unreachable: Vec::new(),
            skipped: Skipped::default(),
        }
    });
    log::info!(
        "library scan: {} games, {} added, {} folders unreachable, {} encrypted and {} \
         unrecognised files skipped, {:?}",
        report.found,
        report.added,
        report.unreachable.len(),
        report.skipped.encrypted,
        report.skipped.unrecognised,
        started.elapsed()
    );
    cache_icons(app, state);
    emit_scan(
        app,
        ScanEvent::Done {
            found: report.found,
            added: report.added,
            unreachable: report.unreachable,
            skipped_encrypted: report.skipped.encrypted,
            skipped_unrecognised: report.skipped.unrecognised,
        },
    );
    emit_all(app, state);
}

/// The picked document trees as the scan reads them: listed and opened through the platform
/// (Android's Storage Access Framework). The desktop has none.
struct PlatformDocuments {
    /// The app's private cache, where an opened document's named link goes.
    cache: Option<PathBuf>,
}

impl PlatformDocuments {
    fn new(app: &AppHandle) -> Self {
        let cache = app
            .path()
            .app_cache_dir()
            .inspect_err(|e| log::warn!("no cache folder for documents: {e}"))
            .ok();
        Self { cache }
    }
}

impl Documents for PlatformDocuments {
    fn children(&self, tree: &str, dir: Option<&str>) -> Result<Vec<DocEntry>, String> {
        let entries = emudome_platform::list_document_tree(tree, dir)
            .map_err(|e| format!("{tree}: {e:?}"))?;
        Ok(entries
            .into_iter()
            .map(|e| DocEntry {
                uri: e.uri,
                name: e.name,
                size: e.size,
                mtime: e.mtime,
                dir: e.dir,
            })
            .collect())
    }

    fn open(&self, uri: &str) -> Result<OpenedDocument, String> {
        let cache = self
            .cache
            .as_deref()
            .ok_or_else(|| format!("{uri}: no cache folder"))?;
        let doc =
            emudome_platform::open_content(uri, cache).map_err(|e| format!("{uri}: {e:?}"))?;
        let file = doc.reader().map_err(|e| format!("{uri}: {e}"))?;
        Ok(OpenedDocument {
            file,
            hold: Box::new(doc),
        })
    }
}

// ---- Covers ----

fn icon_file(covers: &Path, id: &GameKey) -> PathBuf {
    covers.join(format!("{}.icon.png", id.file_stem()))
}

/// The icon inside a game's file: a path, or a document (read through its descriptor).
fn read_game_icon(app: &AppHandle, game: &GameRecord) -> Result<Option<library::Icon>, String> {
    if !emudome_platform::is_content_uri(&game.path) {
        return library::read_icon(Path::new(&game.path)).map_err(|e| e.to_string());
    }
    let file = crate::game::GameFile::open(app, &game.path).map_err(|e| format!("{e:?}"))?;
    let document = file
        .document()
        .ok_or_else(|| format!("{}: not a document", game.path))?;
    let mut reader = document.reader().map_err(|e| e.to_string())?;
    library::read_icon_file(file.path(), &mut reader).map_err(|e| e.to_string())
}

/// Extracts the icon of a game into the covers folder, if it is not there yet.
fn cache_icon(app: &AppHandle, covers: &Path, game: &GameRecord) -> Option<PathBuf> {
    let file = icon_file(covers, &game.id);
    if file.is_file() {
        return Some(file);
    }
    let icon = match read_game_icon(app, game) {
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
    let written = emudome_core::write_atomic(&file, &png);
    match written {
        Ok(()) => Some(file),
        Err(e) => {
            log::warn!("icon of {} not cached: {e}", game.id);
            None
        }
    }
}

/// After a scan: the icons of the reachable games, so they show even when the ROM is offline.
fn cache_icons(app: &AppHandle, state: &LibraryState) {
    let games: Vec<GameRecord> = state
        .lib
        .lock()
        .games()
        .filter(|g| g.has_icon && g.available)
        .cloned()
        .collect();
    for game in &games {
        cache_icon(app, &state.covers, game);
    }
}

/// `emu://cover/<console>-<hex>`: the game's icon or user image.
pub fn cover_response(app: &AppHandle, path: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let found = app.try_state::<LibraryState>().and_then(|state| {
        let id = GameKey::from_file_stem(path)?;
        let game = state.lib.lock().game(&id).ok().cloned()?;
        let file = match game.cover {
            CoverKind::Icon => cache_icon(app, &state.covers, &game)?,
            CoverKind::User => user_cover_file(&state.covers, game.user_cover.as_deref())?,
            CoverKind::Generated => return None,
        };
        let bytes = std::fs::read(&file)
            .inspect_err(|e| log::warn!("cover {}: {e}", file.display()))
            .ok()?;
        Some((bytes, image_type(&file)))
    });
    let builder = Response::builder();
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

/// The game's user image in the covers folder. `library.json` syncs, so its `user_cover` is
/// untrusted: only a plain file name of the covers folder is accepted, never a path.
fn user_cover_file(covers: &Path, user_cover: Option<&str>) -> Option<PathBuf> {
    let name = user_cover?;
    let file = crate::dirs::file_in(covers, name);
    if file.is_none() {
        log::warn!("user cover {name:?} is not a file name of the covers folder, ignored");
    }
    file
}

pub(crate) fn image_type(file: &Path) -> &'static str {
    let ext = file
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    ext_type(&ext)
}

/// The media type of an image file extension (lower case); PNG for anything else.
fn ext_type(ext: &str) -> &'static str {
    match ext {
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

/// The bytes of an image file the user picked (a cover's source, a background), at most
/// [`MAX_COVER_BYTES`], PNG / JPEG / WebP by its signature, with its extension. Refused with
/// `coverInvalid` and the reason; the file's own error in the log.
pub(crate) fn read_image(source: &Path) -> Result<(Vec<u8>, &'static str), ErrorCode> {
    let unreadable = |e: std::io::Error| {
        cover::invalid(ImageProblem::Missing, format!("{}: {e}", source.display()))
    };
    let size = std::fs::metadata(source).map_err(unreadable)?.len();
    if size > MAX_COVER_BYTES {
        return Err(cover::invalid(
            ImageProblem::TooLarge,
            format!("{size} bytes, limit {MAX_COVER_BYTES}"),
        ));
    }
    let bytes = std::fs::read(source).map_err(unreadable)?;
    let ext = image_ext(&bytes).ok_or_else(|| {
        cover::invalid(
            ImageProblem::Format,
            format!("{}: not a PNG, JPEG or WebP image", source.display()),
        )
    })?;
    Ok((bytes, ext))
}

/// The bytes of a cover source on disk ([`read_image`]) with their media type.
fn read_source(source: &Path) -> Result<(Vec<u8>, &'static str), ErrorCode> {
    read_image(source).map(|(bytes, ext)| (bytes, ext_type(ext)))
}

/// The user cover file of revision `rev`: always the normalized JPEG.
fn user_cover_name(id: &GameKey, rev: u32) -> String {
    format!("{}.user-{rev}.jpg", id.file_stem())
}

/// Makes the normalized cover of `bytes` showing `view` and writes it into the covers folder
/// (`write_atomic`); returns its file name there.
fn write_cover(
    covers: &Path,
    id: &GameKey,
    rev: u32,
    bytes: &[u8],
    view: CoverView,
) -> Result<String, ErrorCode> {
    let jpeg = cover::make(bytes, view)?;
    let name = user_cover_name(id, rev);
    emudome_core::write_atomic(&covers.join(&name), &jpeg).map_err(|e| {
        log::warn!("cover {name} not written: {e}");
        ErrorCode::LibraryWriteFailed {
            cause: IoCause::of(&e),
        }
    })?;
    Ok(name)
}

/// `emu://picked/<token>`: the image picked for a cover, while it is the last one picked.
pub fn picked_response(app: &AppHandle, token: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let found = app.try_state::<LibraryState>().and_then(|state| {
        let token: u32 = token.parse().ok()?;
        let picked = state.picked.lock();
        picked
            .as_ref()
            .filter(|p| p.token == token)
            .map(|p| (p.bytes.clone(), p.mime))
    });
    let builder = Response::builder();
    let response = match found {
        // Tokens start again at every run: never cached.
        Some((bytes, mime)) => builder
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "no-store")
            .body(bytes),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.unwrap_or_else(crate::response_error)
}

// ---- Screenshots of a game (Trello #85): emu://screenshot/<console>-<hex>/<name> ----

fn screenshot_url(id: &GameKey, name: &str) -> String {
    emudome_platform::protocol_url("emu", &format!("screenshot/{}/{name}", id.file_stem()))
}

/// The screenshot file a `<console>-<hex>/<name>` path names, when it is one: a game key and a
/// plain `.png` name of that game's folder, nothing else (no separator, no `..`).
fn screenshot_file(saves_root: &Path, rest: &str) -> Option<PathBuf> {
    let (stem, name) = rest.split_once('/')?;
    let id = GameKey::from_file_stem(stem)?;
    emudome_core::screenshot::file(&emudome_core::screenshot::game_dir(saves_root, &id), name)
}

/// `emu://screenshot/<console>-<hex>/<name>`: one of a game's screenshots.
pub fn screenshot_response(app: &AppHandle, rest: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let found = app.try_state::<AppState>().and_then(|state| {
        let file = screenshot_file(&state.save_dir(), rest)?;
        std::fs::read(&file)
            .inspect_err(|e| log::warn!("screenshot {}: {e}", file.display()))
            .ok()
    });
    let builder = Response::builder();
    let response = match found {
        Some(bytes) => builder
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(bytes),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.unwrap_or_else(crate::response_error)
}

// ---- Native pickers (desktop; Android picks documents, `emudome_platform::pick_document*`) ----

#[cfg(desktop)]
pub(crate) fn pick_folder(app: &AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("picked folder: {e}"))
        .ok()
}

#[cfg(desktop)]
pub(crate) fn pick_image(app: &AppHandle) -> Option<PathBuf> {
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

/// Any file (a BIOS dump has no fixed extension).
#[cfg(desktop)]
pub(crate) fn pick_any_file(app: &AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_file()?
        .into_path()
        .inspect_err(|e| log::warn!("picked file: {e}"))
        .ok()
}

#[cfg(mobile)]
pub(crate) fn pick_any_file(_app: &AppHandle) -> Option<PathBuf> {
    log::warn!("file picker not available on this platform yet");
    None
}

#[cfg(mobile)]
pub(crate) fn pick_folder(_app: &AppHandle) -> Option<PathBuf> {
    log::warn!("folder picker not available on this platform yet");
    None
}

#[cfg(mobile)]
pub(crate) fn pick_image(_app: &AppHandle) -> Option<PathBuf> {
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
    // Not on the phone: Android's library folders are document trees, which no file watcher
    // sees; they are rescanned at start (`library.scanstart`) and on demand.
    if emudome_platform::PHONE || !settings(app).bool(keys::LIBRARY_WATCH) {
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
        if emudome_platform::is_content_uri(&folder) {
            continue;
        }
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
pub fn begin_play(app: &AppHandle, id: &GameKey) {
    let Some(state) = app.try_state::<LibraryState>() else {
        return;
    };
    *state.playing.lock() = Some((id.clone(), Instant::now()));
    if let Err(e) = state.lib.lock().mark_played(id, now_ms()) {
        log::warn!("last played of {id}: {e:?}");
    }
    emit_games(app, &state, &[id]);
    // The version launched last is its family's tile (#102).
    emit_families(app, &state);
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
    emit_games(app, &state, &[&id]);
}

/// The library game whose file is `path`: its key and title.
pub fn game_at(app: &AppHandle, path: &str) -> Option<(GameKey, String)> {
    let state = app.try_state::<LibraryState>()?;
    let lib = state.lib.lock();
    lib.game_at(path)
        .map(|g| (g.id.clone(), g.shown_title().to_owned()))
}

/// The title of the library game `id`, when it is one.
pub fn title_of(app: &AppHandle, id: &GameKey) -> Option<String> {
    let state = app.try_state::<LibraryState>()?;
    let lib = state.lib.lock();
    lib.game(id).ok().map(|g| g.shown_title().to_owned())
}

/// The file of the library game `id`, when it is one.
pub fn path_of(app: &AppHandle, id: &GameKey) -> Option<PathBuf> {
    let state = app.try_state::<LibraryState>()?;
    let lib = state.lib.lock();
    lib.game(id).ok().map(|g| PathBuf::from(&g.path))
}

/// Every library game as stored, copied out of the lock (the cloud sync works on the copy).
pub fn records(app: &AppHandle) -> Vec<library::GameRecord> {
    app.try_state::<LibraryState>()
        .map(|state| state.lib.lock().games().cloned().collect())
        .unwrap_or_default()
}

/// The name a library folder shows («In <cartella>», «Gestisci lo spazio»): a picked document
/// tree's display name as the last scan saw it (Android), else the folder's last part.
pub fn folder_name(app: &AppHandle, folder: &str) -> Option<String> {
    if emudome_platform::is_content_uri(folder) {
        let state = app.try_state::<LibraryState>()?;
        let trees = state.trees.lock();
        return trees.get(folder).and_then(|t| t.name.clone());
    }
    Path::new(folder)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .or_else(|| Some(folder.to_owned()))
}

/// The game file of `id` was removed from this device («Togli dal telefono»): the game stays in
/// the library, unavailable (faded), its saves and play time kept; `library://changed` tells.
pub fn file_removed(app: &AppHandle, id: &GameKey) -> Result<(), ErrorCode> {
    let state = app
        .try_state::<LibraryState>()
        .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.to_string() })?;
    state.lib.lock().set_unavailable(id)?;
    emit_games(app, &state, &[id]);
    Ok(())
}

/// A game file that arrived outside the library folders (downloaded from Google Drive) joins
/// the library at the next scan, started now; `library://changed` tells when it is in.
pub fn add_file(app: &AppHandle, path: PathBuf) {
    if let Some(state) = app.try_state::<LibraryState>() {
        state.pending.lock().push(path);
        start_scan(app);
    }
}

/// What a file opened by its path is. The library's hash cache entry for that path is copied
/// out first, so hashing a large ROM never holds the library's lock (nor any lock the audio or
/// the renderer takes).
pub fn identify_path(app: &AppHandle, path: &Path) -> Result<Identity, ErrorCode> {
    let cached = app.try_state::<LibraryState>().and_then(|state| {
        let lib = state.lib.lock();
        lib.cache().get(path.to_string_lossy().as_ref()).cloned()
    });
    library::identify_cached(path, cached.as_ref()).map_err(|e| {
        not_a_game(e, |io| {
            log::warn!("{}: cannot be read: {io}", path.display());
            ErrorCode::rom_unreadable(path, emudome_core::IoCause::of(io))
        })
    })
}

/// What a game file opened outside the library is: [`identify_path`] for a path; a document is
/// read through its descriptor, its cache entry keyed by its URI (as the scan of a picked folder
/// keys it).
pub fn identify_game_file(
    app: &AppHandle,
    file: &crate::game::GameFile,
) -> Result<Identity, ErrorCode> {
    let Some(document) = file.document() else {
        return identify_path(app, file.path());
    };
    let cached = app
        .try_state::<LibraryState>()
        .and_then(|state| state.lib.lock().cache().get(file.given()).cloned());
    let mut reader = document.reader().map_err(|e| file.unreadable(&e))?;
    library::identify_open_file(file.path(), &mut reader, cached.as_ref())
        .map_err(|e| not_a_game(e, |io| file.unreadable(io)))
}

/// A file that is not a game, for the interface; `unreadable` tells a read that failed.
fn not_a_game(e: NotAGame, unreadable: impl FnOnce(&std::io::Error) -> ErrorCode) -> ErrorCode {
    match e {
        NotAGame::Io(io) => unreadable(&io),
        NotAGame::Extension | NotAGame::Header(_) => {
            log::warn!("gameUnidentified: {e}");
            ErrorCode::GameUnidentified
        }
        NotAGame::Encrypted => ErrorCode::GameEncrypted,
    }
}

/// Every file the library knows the key of, for the one-time game key migration;
/// `libraryReadFailed` while `library.json` cannot be read (nothing may be decided then).
pub fn known_files(app: &AppHandle) -> Result<Vec<(String, GameKey)>, ErrorCode> {
    let state = app.try_state::<LibraryState>().ok_or_else(|| {
        log::warn!("library not open");
        ErrorCode::LibraryReadFailed {
            cause: IoCause::Other,
        }
    })?;
    let mut lib = state.lib.lock();
    lib.readable()?;
    Ok(lib.known_files())
}

// ---- Commands ----

/// The whole library for the interface.
#[tauri::command]
pub fn library_get(app: AppHandle, state: State<'_, LibraryState>) -> LibraryDto {
    library_dto(&app, &state)
}

fn library_dto(app: &AppHandle, state: &LibraryState) -> LibraryDto {
    let grouped = grouped(app);
    let (games, families) = {
        let lib = state.lib.lock();
        let games: Vec<GameDto> = lib.games().map(GameDto::new).collect();
        (games, families(grouped, &lib))
    };
    let trees = state.trees.lock().clone();
    LibraryDto {
        games,
        families,
        folders: folders(app)
            .into_iter()
            .map(|path| folder_dto(path, &trees))
            .collect(),
        scanning: state.scanning.load(Ordering::SeqCst),
    }
}

/// A folder of `library.folders` for the interface. A picked document tree (Android) shows its
/// display name and is reachable as the last scan found it (until a scan, as picked; never
/// looked at here: this runs on the event loop).
fn folder_dto(path: String, trees: &HashMap<String, TreeSeen>) -> FolderDto {
    if emudome_platform::is_content_uri(&path) {
        let seen = trees.get(&path);
        return FolderDto {
            name: seen.and_then(|t| t.name.clone()),
            access: seen.map(|t| t.access).unwrap_or_default(),
            path,
        };
    }
    FolderDto {
        name: Some(path.clone()),
        access: if Path::new(&path).is_dir() {
            FolderAccess::Reachable
        } else {
            FolderAccess::Unreachable
        },
        path,
    }
}

/// Native folder picker: the folder joins `library.folders` and is scanned. `None` = cancelled.
/// Android: the system's folder picker, the folder kept as its tree URI with its read permission
/// persisted. Off the event loop: it waits for the user.
#[tauri::command]
pub async fn library_choose_folder(app: AppHandle) -> Result<Option<String>, ErrorCode> {
    crate::blocking(app, choose_folder).await?
}

fn choose_folder(app: &AppHandle) -> Result<Option<String>, ErrorCode> {
    let folder = if emudome_platform::DOCUMENT_PICKERS {
        let picked = emudome_platform::pick_document_tree().unwrap_or_else(|e| {
            log::warn!("folder picker: {e:?}");
            None
        });
        let Some(picked) = picked else {
            return Ok(None);
        };
        if let Some(state) = app.try_state::<LibraryState>() {
            state.trees.lock().insert(
                picked.uri.clone(),
                TreeSeen {
                    name: picked.name,
                    access: FolderAccess::Reachable,
                },
            );
        }
        picked.uri
    } else {
        let Some(folder) = pick_folder(app) else {
            return Ok(None);
        };
        folder.to_string_lossy().into_owned()
    };
    add_folders(app, std::slice::from_ref(&folder))?;
    start_scan(app);
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
    // A picked document tree (Android): its persisted read permission is given back.
    if emudome_platform::is_content_uri(&path) {
        emudome_platform::release_document_tree(&path);
        state.trees.lock().remove(&path);
    }
    refresh_watch(&app);
    emit_all(&app, &state);
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

/// Opens a library game and counts its play time until it closes. Off the event loop (a
/// blocking thread): it loads the core and the ROM, and may wait for the open game to close
/// (`closing::close_game`).
#[tauri::command]
pub async fn library_open(app: AppHandle, id: GameKey) -> Result<GameInfo, ErrorCode> {
    crate::blocking(app, move |app| open_now(app, id)).await?
}

fn open_now(app: &AppHandle, id: GameKey) -> Result<GameInfo, ErrorCode> {
    let app_state = app.state::<AppState>();
    let state = app.state::<LibraryState>();
    let (path, title) = {
        let lib = state.lib.lock();
        let game = lib.game(&id)?;
        (game.path.clone(), game.shown_title().to_owned())
    };
    // A document (`content://`, Android) is checked by opening it: its provider says.
    if !emudome_platform::is_content_uri(&path) && !Path::new(&path).is_file() {
        state.lib.lock().set_unavailable(&id)?;
        emit_games(app, &state, &[&id]);
        return Err(ErrorCode::LibraryGameUnavailable { id: id.to_string() });
    }
    let file = crate::game::GameFile::open(app, &path)?;
    crate::open_library_game(app, &app_state, &id, file, title)
}

/// Changes the cover (L12) to the generated one, the ROM's icon, or the user cover the game
/// already has (a new one comes from `library_set_cover_image`).
#[tauri::command]
pub fn library_set_cover(
    app: AppHandle,
    state: State<'_, LibraryState>,
    id: GameKey,
    kind: CoverKind,
) -> Result<(), ErrorCode> {
    {
        let mut lib = state.lib.lock();
        let user_file = match kind {
            CoverKind::User => lib.game(&id)?.user_cover.clone().filter(|name| {
                user_cover_file(&state.covers, Some(name)).is_some_and(|f| f.is_file())
            }),
            CoverKind::Icon | CoverKind::Generated => None,
        };
        lib.set_cover(&id, kind, user_file)?;
    }
    emit_games(&app, &state, &[&id]);
    Ok(())
}

/// The game's screenshots, newest first (L12 «Da uno screenshot»). A game with no folder yet
/// has none.
#[tauri::command]
pub fn library_screenshots(
    state: State<'_, AppState>,
    id: GameKey,
) -> Result<Vec<ScreenshotDto>, ErrorCode> {
    let dir = emudome_core::screenshot::game_dir(&state.save_dir(), &id);
    let shots = emudome_core::screenshot::list(&dir).map_err(|e| {
        log::warn!("screenshots of {} not listed: {e}", dir.display());
        ErrorCode::ScreenshotsUnreadable {
            cause: IoCause::of(&e),
        }
    })?;
    Ok(shots
        .into_iter()
        .map(|s| ScreenshotDto {
            url: screenshot_url(&id, &s.name),
            name: s.name,
            taken_at: s.taken_ms,
            width: s.width,
            height: s.height,
        })
        .collect())
}

/// Native image picker for a new user cover (L12 «Immagine tua»): the image is checked (at most
/// [`MAX_COVER_BYTES`], PNG / JPEG / WebP, at most `cover::MAX_SIDE` a side) and kept in memory
/// for «Adatta la copertina». `None` = cancelled. Off the event loop (it decodes the image).
#[tauri::command]
pub async fn library_pick_cover_image(app: AppHandle) -> Result<Option<PickedDto>, ErrorCode> {
    crate::blocking(app, pick_cover_image).await?
}

fn pick_cover_image(app: &AppHandle) -> Result<Option<PickedDto>, ErrorCode> {
    let Some(source) = pick_image(app) else {
        return Ok(None);
    };
    let (bytes, mime) = read_source(&source)?;
    let (width, height) = cover::dimensions(&bytes)?;
    let state = app.state::<LibraryState>();
    let token = state
        .next_token
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);
    *state.picked.lock() = Some(PickedImage { token, bytes, mime });
    Ok(Some(PickedDto {
        token,
        url: emudome_platform::protocol_url("emu", &format!("picked/{token}")),
        width,
        height,
    }))
}

/// «Usa questa copertina» (Trello #85): the cover made from `source` showing `view` (decoded,
/// cropped, resized to 600 × 900, written as a JPEG with `write_atomic`) becomes the game's user
/// cover. Off the event loop (decoding and resizing take a while).
#[tauri::command]
pub async fn library_set_cover_image(
    app: AppHandle,
    id: GameKey,
    source: CoverSource,
    view: CoverView,
) -> Result<(), ErrorCode> {
    crate::blocking(app, move |app| set_cover_image(app, &id, &source, view)).await?
}

fn set_cover_image(
    app: &AppHandle,
    id: &GameKey,
    source: &CoverSource,
    view: CoverView,
) -> Result<(), ErrorCode> {
    let state = app.state::<LibraryState>();
    let bytes = match source {
        CoverSource::Picked { token } => {
            let picked = state.picked.lock();
            match picked.as_ref() {
                Some(p) if p.token == *token => p.bytes.clone(),
                Some(_) | None => {
                    return Err(cover::invalid(
                        ImageProblem::Missing,
                        format!("picked image {token} is no longer there"),
                    ))
                }
            }
        }
        CoverSource::Screenshot { name } => {
            let dir = emudome_core::screenshot::game_dir(&app.state::<AppState>().save_dir(), id);
            let file = emudome_core::screenshot::file(&dir, name).ok_or_else(|| {
                cover::invalid(
                    ImageProblem::Missing,
                    format!("{id} has no screenshot {name:?}"),
                )
            })?;
            read_source(&file)?.0
        }
    };
    let rev = state.lib.lock().game(id)?.cover_rev.wrapping_add(1);
    let name = write_cover(&state.covers, id, rev, &bytes, view)?;
    let old = {
        let mut lib = state.lib.lock();
        let old = lib.game(id)?.user_cover.clone();
        lib.set_cover(id, CoverKind::User, Some(name.clone()))?;
        old
    };
    // The previous user cover, replaced by the new one (never the one just written, should two
    // requests have made the same name). Only a plain name of the covers folder: a tampered
    // `library.json` must not delete any file.
    let old_file = old
        .filter(|old| *old != name)
        .and_then(|old| crate::dirs::file_in(&state.covers, &old));
    if let Some(old) = old_file {
        if let Err(e) = std::fs::remove_file(&old) {
            log::warn!("old cover {}: {e}", old.display());
        }
    }
    if let CoverSource::Picked { token } = source {
        let mut picked = state.picked.lock();
        if picked.as_ref().is_some_and(|p| p.token == *token) {
            *picked = None;
        }
    }
    emit_games(app, &state, &[id]);
    Ok(())
}

/// Favourite / hidden.
#[tauri::command]
pub fn library_set_flags(
    app: AppHandle,
    state: State<'_, LibraryState>,
    id: GameKey,
    favourite: Option<bool>,
    hidden: Option<bool>,
) -> Result<(), ErrorCode> {
    state.lib.lock().set_flags(&id, favourite, hidden)?;
    emit_games(&app, &state, &[&id]);
    // A hidden game leaves its family, a shown one comes back to it (#102).
    if hidden.is_some() {
        emit_families(&app, &state);
    }
    Ok(())
}

/// «Mostra questa in libreria» in a game's «Versioni» (#102): `pin` makes the library show `id`
/// for its family; not `pin` gives the family back to the version launched last.
#[tauri::command]
pub fn library_pin_version(
    app: AppHandle,
    state: State<'_, LibraryState>,
    id: GameKey,
    pin: bool,
) -> Result<(), ErrorCode> {
    state.lib.lock().pin_version(&id, pin)?;
    emit_families(&app, &state);
    Ok(())
}

/// «Nome mostrato» (L10); blank goes back to the ROM's title.
#[tauri::command]
pub fn library_set_name(
    app: AppHandle,
    state: State<'_, LibraryState>,
    id: GameKey,
    name: String,
) -> Result<(), ErrorCode> {
    state.lib.lock().set_name(&id, &name)?;
    emit_games(&app, &state, &[&id]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{screenshot_file, user_cover_file, FolderAccess};
    use std::path::Path;

    #[test]
    fn a_tree_without_its_permission_says_so_and_only_then() {
        // Listed: reachable whatever the permission says.
        for permitted in [Some(true), Some(false), None] {
            assert_eq!(
                FolderAccess::of_tree(true, permitted),
                FolderAccess::Reachable
            );
        }
        // Not listed: the permission given back is told apart; anything else is unreachable.
        assert_eq!(
            FolderAccess::of_tree(false, Some(false)),
            FolderAccess::PermissionLost
        );
        assert_eq!(
            FolderAccess::of_tree(false, Some(true)),
            FolderAccess::Unreachable
        );
        assert_eq!(
            FolderAccess::of_tree(false, None),
            FolderAccess::Unreachable
        );
    }

    #[test]
    fn a_screenshot_url_reaches_only_a_screenshot_of_that_game() {
        let dir = std::env::temp_dir().join(format!("emudome-shot-url-{}", std::process::id()));
        let game = dir.join("screenshots").join("gba").join("0a1b");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("2026-11-22_10-11-12.png"), b"png").unwrap();
        std::fs::write(dir.join("secret.png"), b"no").unwrap();
        assert_eq!(
            screenshot_file(&dir, "gba-0a1b/2026-11-22_10-11-12.png"),
            Some(game.join("2026-11-22_10-11-12.png"))
        );
        for bad in [
            "gba-0a1b/missing.png",
            "gba-0a1b/../../../secret.png",
            "gba-0a1b/..%2F..%2Fsecret.png",
            "gba-0a1b",
            "gba-0A1B/2026-11-22_10-11-12.png",
            "gba:0a1b/2026-11-22_10-11-12.png",
            "../gba-0a1b/2026-11-22_10-11-12.png",
        ] {
            assert_eq!(screenshot_file(&dir, bad), None, "{bad}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_tampered_user_cover_never_leaves_the_covers_folder() {
        let covers = Path::new("covers");
        assert_eq!(
            user_cover_file(covers, Some("gba-0a1b.user-2.png")),
            Some(covers.join("gba-0a1b.user-2.png"))
        );
        assert_eq!(user_cover_file(covers, None), None);
        for tampered in [
            r"C:\Users\me\.ssh\id_rsa",
            "/home/me/.ssh/id_rsa",
            "../library.json",
            r"..\settings\global.json",
        ] {
            assert_eq!(user_cover_file(covers, Some(tampered)), None, "{tampered}");
        }
    }
}
