//! EmuDome's Tauri shell (Phase 0).
//!
//! One native window: wgpu draws the game straight onto the window,
//! the Vite interface sits on top in a transparent webview ("ui").
//! The interface never touches the core: it calls the commands below.
//!
//! Errors reach the interface as [`ErrorCode`] (`{ code, params }`), never as sentences.

mod audio;
mod bench;
mod bios;
mod closing;
mod core_settings;
mod dirs;
mod effects;
mod first_run;
// SDL3 is built for the desktop only: controllers on Android come later (Android input events).
#[cfg(desktop)]
mod gamepad;
mod library;
mod migration;
mod pads;
mod saves;
mod settings;
mod theme;
mod video;
mod window;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use emudome_core::input::{Button, Source};
use emudome_core::screens::{Area, Layout, Split};
use emudome_core::settings::ConsoleId;
use emudome_core::tuning::{Tuning, VideoPath};
use emudome_core::{
    Closed, Command, ErrorCode, Event, GameKey, Session, SessionOptions, Shared, Speed,
    CLOSE_TIMEOUT,
};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::webview::Color;
use tauri::{Emitter, Manager, WindowEvent};

/// The active session, read by renderer, audio and controllers.
pub type CurrentShared = Arc<Mutex<Option<Arc<Shared>>>>;

/// Where the game is drawn: native surface under the webview, or canvas in the webview.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum VideoMode {
    Native,
    Webview,
}

impl VideoMode {
    /// WebView2 background (`DefaultBackgroundColor`): transparent only when the native game is underneath.
    /// In webview mode the page must never let alpha through to the window.
    fn webview_background(self) -> Color {
        match self {
            VideoMode::Native => Color(0, 0, 0, 0),
            // The design system's `bg`, #0e0f13.
            VideoMode::Webview => Color(0x0e, 0x0f, 0x13, 0xff),
        }
    }
}

struct AppState {
    session: Mutex<Option<Session>>,
    current: CurrentShared,
    /// `None` where the native renderer cannot sit under the webview (Linux) or is off.
    renderer: Option<video::Renderer>,
    /// Why the native renderer could not start, when it was wanted.
    native_error: Option<String>,
    video_mode: Mutex<VideoMode>,
    /// Whether the quick menu is open, set by the interface: while it is, the controllers
    /// navigate the menu instead of playing (see `gamepad.rs`).
    menu_open: Arc<AtomicBool>,
    data_dir: PathBuf,
    resource_dir: Option<PathBuf>,
    /// The key of the open game: its saves, slots and thumbnails are named by it.
    game: Mutex<Option<GameKey>>,
    /// The core of the open game (`mgba`, ...), named in `coreNeedsRestart`.
    core_name: Mutex<Option<&'static str>>,
    /// Held for the whole of closing a game: a second close (the window closing while a game
    /// closes) waits for the first, so the app never quits before the exit writes are done.
    /// The emulation thread never takes it.
    closing: Mutex<()>,
    /// The close flow (Trello #86): the window's X, quitting, closing or switching the game
    /// wait for the exit writes off the event loop, with progress on `emu://closing`.
    closer: closing::Closer,
    /// Held while a game opens: opens run off the event loop (they may wait for a close), one
    /// at a time.
    opening: Mutex<()>,
    audio: audio::AudioOutput,
    /// Bench mode (`EMUDOME_BENCH`), off otherwise.
    bench: Option<Arc<bench::Bench>>,
}

impl AppState {
    fn save_dir(&self) -> PathBuf {
        self.data_dir.join("saves")
    }
}

/// One quick save slot for the quick menu (G07). Mirror: `SlotInfo` in app/src/emu.ts.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SlotDto {
    slot: u8,
    /// Unix ms of the last save; null = empty.
    saved_at: Option<u64>,
    /// `emu://slot/<n>?v=<ms>`, the frame on screen when it was saved.
    thumb_url: Option<String>,
}

fn unix_ms(t: std::time::SystemTime) -> Option<u64> {
    t.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
}

/// The auto quick save (slot 0) and slots `1..=count` of the open game.
#[tauri::command]
fn list_slots(state: tauri::State<'_, AppState>, count: u8) -> Vec<SlotDto> {
    let Some(game) = state.game.lock().clone() else {
        return Vec::new();
    };
    emudome_core::slot_infos(&state.save_dir(), &game, count)
        .into_iter()
        .map(|s| {
            let saved_at = s.saved_at.and_then(unix_ms);
            SlotDto {
                slot: s.slot,
                saved_at,
                thumb_url: s.thumb.map(|_| {
                    emudome_platform::protocol_url(
                        "emu",
                        &format!("slot/{}?v={}", s.slot, saved_at.unwrap_or(0)),
                    )
                }),
            }
        })
        .collect()
}

/// `emu://slot/<n>`: the thumbnail of a slot of the open game.
fn slot_thumb_response(app: &tauri::AppHandle, slot: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let bytes = slot.parse::<u8>().ok().and_then(|slot| {
        let state = app.try_state::<AppState>()?;
        let game = state.game.lock().clone()?;
        let info = emudome_core::slot_infos(&state.save_dir(), &game, slot)
            .into_iter()
            .find(|s| s.slot == slot)?;
        std::fs::read(info.thumb?).ok()
    });
    let builder = Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    match bytes {
        // The URL carries the save time (`?v=`): a new save is a new URL.
        Some(b) => builder
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(b),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    }
    .unwrap_or_else(response_error)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameInfo {
    /// The game's one identity (its saves, its «Solo questo gioco» settings).
    id: GameKey,
    title: String,
    console: String,
    console_id: ConsoleId,
    core: String,
    /// `saves.resume` is «ask» and an auto quick save exists: the interface asks.
    resumable: bool,
    /// `saves.autoexit`: closing writes the auto quick save (G14 lists the step).
    autosave_on_exit: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsDto {
    running: bool,
    emu_fps: f64,
    target_fps: f64,
    speed: Option<Speed>,
    paused: bool,
    /// Frames emulated since the game started (the interface's hang watchdog, E11).
    frames: u64,
    /// Unix ms of the last auto quick save in this session.
    auto_saved_at: Option<u64>,
}

/// Which core to use for a file, and its console, by its extension.
fn core_for(path: &Path) -> Result<(&'static str, ConsoleId), ErrorCode> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "gba" => Ok(("mgba", ConsoleId::Gba)),
        "gb" => Ok(("mgba", ConsoleId::Gb)),
        "gbc" => Ok(("mgba", ConsoleId::Gbc)),
        "nds" => Ok((emudome_core::nds::CORE, ConsoleId::Nds)),
        "3ds" | "cci" | "cxi" | "cia" => Err(ErrorCode::ConsoleNotYet {
            console: "3ds".into(),
            phase: 3,
        }),
        _ => Err(ErrorCode::UnsupportedFormat { ext }),
    }
}

/// The console's short name in `GameInfo` (a .gb file whose header asks for a Game Boy Color
/// is «GBC»).
fn console_label(console: ConsoleId) -> &'static str {
    match console {
        ConsoleId::Gb => "GB",
        ConsoleId::Gbc => "GBC",
        ConsoleId::Gba => "GBA",
        ConsoleId::Nds => "NDS",
        ConsoleId::N3ds => "3DS",
    }
}

/// Opens a game file. Every game is its content's key, never its path: a file of the library
/// opens as that library game; any other file is identified first (from the library's hash
/// cache when it knows the file unchanged, else by hashing it, holding no lock), and when its
/// content is a library game's (a copy elsewhere) it opens as that game too.
/// Off the event loop (`async`): it may wait for the open game's close.
#[tauri::command(async)]
fn open_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<GameInfo, ErrorCode> {
    if let Some((key, title)) = library::game_at(&app, &path) {
        return open_library_game(&app, &state, &key, PathBuf::from(path), title);
    }
    let game_path = PathBuf::from(&path);
    // An extension EmuDome does not run is refused before any hashing.
    core_for(&game_path)?;
    let identity = library::identify_path(&app, &game_path)?;
    if let Some(title) = library::title_of(&app, &identity.id) {
        return open_library_game(&app, &state, &identity.id, game_path, title);
    }
    open_path(&app, &state, game_path, identity.id, Some(identity.title))
}

/// Opens a library game: its play time counts until it closes.
fn open_library_game(
    app: &tauri::AppHandle,
    state: &AppState,
    key: &GameKey,
    game_path: PathBuf,
    title: String,
) -> Result<GameInfo, ErrorCode> {
    let info = open_path(app, state, game_path, key.clone(), Some(title))?;
    library::begin_play(app, key);
    Ok(info)
}

/// Core options the shell always sets, per core (`GET_VARIABLE` key and value).
/// Azahar: the software renderer, the only one that does not ask for `SET_HW_RENDER`, which the
/// host does not implement yet (Trello #16); its default `auto` fails to load.
/// melonDS DS: the screens where the shell cuts them, touch from the shell's pointer
/// (`emudome_core::nds`).
/// The settings' core options are `emudome_core::settings::core_values` (one table per core).
const FORCED_CORE_OPTIONS: &[(&str, &[(&str, &str)])] = &[
    ("azahar", &[("citra_graphics_api", "Software")]),
    (emudome_core::nds::CORE, emudome_core::nds::FORCED_OPTIONS),
];

fn forced_core_options(core_name: &str) -> Vec<(String, String)> {
    FORCED_CORE_OPTIONS
        .iter()
        .filter(|(core, _)| *core == core_name)
        .flat_map(|(_, opts)| opts.iter())
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// The options the core starts with: the settings' (console options), then the forced ones,
/// then the system files' (`emudome_core::bios`: what the files in place allow), each winning
/// over the ones before.
fn core_options(core_name: &str, flow: &settings::GameFlow) -> Vec<(String, String)> {
    let mut options = emudome_core::settings::core_values::pairs(&flow.core_options);
    options.extend(forced_core_options(core_name));
    options.extend(
        flow.bios
            .options
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    options
}

/// How the console's core lays its screens out in the frame (with the forced options above).
fn screen_split(console: ConsoleId) -> Split {
    match console {
        ConsoleId::Nds => emudome_core::nds::SPLIT,
        // TODO(Phase 3): Azahar's two screens.
        ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba | ConsoleId::N3ds => Split::Whole,
    }
}

/// Closes the open game, then opens `game_path` as the game `game` (its console, saves and
/// settings). `title` comes from the library or the ROM (else the file name).
fn open_path(
    app: &tauri::AppHandle,
    state: &AppState,
    game_path: PathBuf,
    game: GameKey,
    title: Option<String>,
) -> Result<GameInfo, ErrorCode> {
    // The one-time move of old save names ends before any game reads its saves.
    migration::wait(app);
    let (core_name, _) = core_for(&game_path)?;
    let core_path = emudome_platform::find_core(core_name, state.resource_dir.as_deref())
        .ok_or_else(|| ErrorCode::CoreNotFound {
            file: emudome_platform::core_file_name(core_name),
        })?;
    // A game on this core was let go (E11, or a close that timed out): its thread may still be
    // inside the core's module. Checked before closing the open game, which stays open.
    if emudome_core::core_needs_restart(&core_path) {
        return Err(ErrorCode::CoreNeedsRestart {
            core: core_name.into(),
        });
    }

    // One open at a time; the previous game closes first, through the close flow (its
    // progress on `emu://closing`, the user's choice if it takes too long).
    let _opening = state.opening.lock();
    closing::close_game(app, state)?;

    // The content decides (a `.gb` file can be a Game Boy Color cartridge).
    let console_id = game.console();
    let flow = settings::game_flow(app, core_name, console_id, &game);
    let save_dir = state.save_dir();
    // E04 (a broken in-game save is set aside and the open refused) and the launch backup.
    saves::before_open(app, &save_dir, &game)?;
    let resumable = flow.resume == "ask" && emudome_core::has_auto_save(&save_dir, &game);

    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let session = Session::start(
        SessionOptions {
            core_path,
            game_path: game_path.clone(),
            system_dir: bios::system_dir(app, core_name, state.data_dir.join("system")),
            save_dir,
            game: game.clone(),
            autosave_every: flow.autosave_every,
            autosave_on_exit: flow.autosave_on_exit,
            resume_auto: flow.resume == "always",
            app_version: app.package_info().version.to_string(),
            core_options: core_options(core_name, &flow),
            rewind_seconds: flow.rewind_seconds,
            split: screen_split(console_id),
            username: flow.username.clone(),
            fast_audio: flow.fast_audio,
        },
        tx,
    )
    .map_err(|e| ErrorCode::OpenFailed {
        detail: format!("{e:#}"),
    })?;

    // The core's events go to the interface.
    let app_events = app.clone();
    std::thread::spawn(move || {
        for ev in rx {
            let _ = app_events.emit("emu://event", ev);
        }
    });

    let shared = session.shared().clone();
    if let Some(tone) = state.bench.as_ref().and_then(|b| b.tone()) {
        shared.audio.lock().set_test_tone(Some(tone));
    }
    *state.current.lock() = Some(shared.clone());
    state.native_source(Some(shared));
    *state.session.lock() = Some(session);
    *state.game.lock() = Some(game.clone());
    *state.core_name.lock() = Some(core_name);
    bios::emit_fallbacks(app, &flow.bios.fallbacks);
    core_settings::started(app, core_name, console_id, flow.core_options);
    settings::begin_session(app, console_id, game.clone());

    Ok(GameInfo {
        id: game,
        title: title.unwrap_or_else(|| {
            game_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        }),
        console: console_label(console_id).into(),
        console_id,
        core: core_name.into(),
        resumable,
        autosave_on_exit: flow.autosave_on_exit,
    })
}

/// How a game ends.
#[derive(Clone, Copy)]
enum Ending {
    /// Closed: the thread stops and writes the in-game save and, with `saves.autoexit`, the auto
    /// quick save; waited for up to `CLOSE_TIMEOUT`, then let go.
    Close,
    /// E11, the core stopped answering: let go at once.
    Abandon,
}

/// Ends the open game in place (`Ending::Close`), bounded by `CLOSE_TIMEOUT`: only for
/// `RunEvent::Exit`, which cannot wait (`closing::close_now`). Every other way out goes through
/// the close flow (`closing.rs`): closing the game, opening another, the window's X, quitting.
fn close_current(app: &tauri::AppHandle, state: &AppState) {
    end_game(app, state, Ending::Close);
}

/// Takes the open game out of the app: play time, the settings session, video; returns its
/// session (not yet closed) and its core's name. The caller closes the session, then clears
/// `game_path`.
fn release_game(app: &tauri::AppHandle, state: &AppState) -> (Option<Session>, &'static str) {
    library::end_play(app);
    core_settings::stopped(app);
    settings::end_session(app);
    state.native_source(None);
    *state.current.lock() = None;
    // Taken out first: the session lock is not held while the thread is waited for.
    let session = state.session.lock().take();
    let core = state.core_name.lock().take().unwrap_or_default();
    (session, core)
}

/// A game on `core` was let go: its exit writes may be missing and the core is refused until
/// EmuDome restarts. Tells the interface.
fn say_core_needs_restart(app: &tauri::AppHandle, core: &str) {
    let error = Event::Error {
        error: ErrorCode::CoreNeedsRestart { core: core.into() },
    };
    if let Err(e) = app.emit("emu://event", error) {
        log::warn!("coreNeedsRestart not sent to the interface: {e}");
    }
}

fn end_game(app: &tauri::AppHandle, state: &AppState, ending: Ending) {
    let _closing = state.closing.lock();
    let (session, core) = release_game(app, state);
    if let Some(session) = session {
        let closed = match ending {
            Ending::Close => session.close(CLOSE_TIMEOUT),
            Ending::Abandon => {
                session.abandon();
                Closed::Abandoned
            }
        };
        if let (Ending::Close, Closed::Abandoned) = (ending, closed) {
            log::warn!("the game did not close within {CLOSE_TIMEOUT:?}: let go");
            say_core_needs_restart(app, core);
        }
    }
    // Cleared after the thread ended: the Saves screen refuses this game's files until then.
    state.game.lock().take();
}

/// Closes the open game (back to the library) through the close flow; returns once it is
/// closed. Off the event loop (`async`): it waits for the exit writes and the user's choice.
#[tauri::command(async)]
fn close_game(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    // `AppClosing`: a quit arrived meanwhile; the game is closed all the same.
    if closing::close_game(&app, &state).is_err() {
        log::info!("the game closed while the app is exiting");
    }
}

/// E11: the core stopped answering. Lets go of its thread without waiting for it (joining a
/// core stuck in `retro_run` never returns). The session is dead (it writes nothing more) and its
/// core is refused until EmuDome restarts (`coreNeedsRestart`).
#[tauri::command]
fn abandon_game(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    end_game(&app, &state, Ending::Abandon);
}

/// What the event loop does for the open game: when the window closes or the app quits, the
/// game closes first through the close flow (the event is prevented, the app exits once the
/// exit writes are done); `Exit`, which cannot be prevented, closes what is left in place. When
/// the app goes to the background its saves are flushed (Android: the process may then be
/// killed without a word).
fn on_run_event(app: &tauri::AppHandle, event: &tauri::RunEvent) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
        if !closing::request_quit(app, &state, *code) {
            api.prevent_exit();
        }
        return;
    }
    if let tauri::RunEvent::WindowEvent {
        event: WindowEvent::CloseRequested { api, .. },
        ..
    } = event
    {
        if !closing::request_quit(app, &state, None) {
            api.prevent_close();
        }
        return;
    }
    if let tauri::RunEvent::Exit = event {
        closing::close_now(app, &state);
        return;
    }
    if let tauri::RunEvent::WindowEvent { event, .. } = event {
        if emudome_platform::is_suspend(event) {
            // The game keeps its state; NoGame (nothing open) is fine.
            if state.send(Command::Flush).is_ok() {
                log::info!("in background: the open game's saves are flushed");
            }
        }
    }
}

impl AppState {
    /// Hands the game to the native renderer only when it is the active mode.
    fn native_source(&self, shared: Option<Arc<Shared>>) {
        if let Some(r) = &self.renderer {
            let native = *self.video_mode.lock() == VideoMode::Native;
            r.set_source(if native { shared } else { None });
        }
    }
}

#[tauri::command]
fn video_mode(state: tauri::State<'_, AppState>) -> (VideoMode, bool) {
    (*state.video_mode.lock(), state.renderer.is_some())
}

/// Game given on the command line (`emudome game.gba`), opened at startup.
#[tauri::command]
fn startup_game() -> Option<String> {
    emudome_platform::startup_game()
}

/// Switches mode live (only where the native renderer exists), to compare the two.
#[tauri::command]
fn set_video_mode(
    webview: tauri::Webview,
    state: tauri::State<'_, AppState>,
    mode: VideoMode,
) -> Result<(), ErrorCode> {
    if mode == VideoMode::Native && state.renderer.is_none() {
        return Err(ErrorCode::NativeVideoUnavailable);
    }
    webview
        .set_background_color(Some(mode.webview_background()))
        .map_err(|e| ErrorCode::WebviewBackgroundFailed {
            detail: e.to_string(),
        })?;
    *state.video_mode.lock() = mode;
    let current = state.current.lock().clone();
    state.native_source(current);
    Ok(())
}

/// Quick menu G09: the bottom `fraction` of the window is the menu's bar, so the native game
/// shrinks above it (0 = the whole window again). The webview canvas does the same by itself.
#[tauri::command]
fn set_video_inset(state: tauri::State<'_, AppState>, fraction: f32) {
    if let Some(r) = &state.renderer {
        r.set_inset(fraction);
    }
}

/// `emu://` protocol: the latest frame as raw RGBA, for the webview's canvas.
/// `?since=N` answers 204 when there is no frame newer than N.
fn frame_protocol(
    app: &tauri::AppHandle,
    req: &tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let since: Option<u64> = req
        .uri()
        .query()
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("since=")))
        .and_then(|v| v.parse().ok());
    let builder = Response::builder()
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            "x-width, x-height, x-seq",
        )
        .header(header::CACHE_CONTROL, "no-store");
    let shared = app.state::<AppState>().current.lock().clone();
    let Some(shared) = shared else {
        return builder
            .status(StatusCode::NO_CONTENT)
            .body(Vec::new())
            .unwrap_or_else(response_error);
    };
    let frame = shared.frame.lock();
    if frame.rgba.is_empty() || since == Some(frame.seq) {
        return builder
            .status(StatusCode::NO_CONTENT)
            .body(Vec::new())
            .unwrap_or_else(response_error);
    }
    builder
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-width", frame.width)
        .header("x-height", frame.height)
        .header("x-seq", frame.seq)
        .body(frame.rgba.clone())
        .unwrap_or_else(response_error)
}

/// A response the builder refused (an invalid header value): logged, and answered with an
/// empty 500 so the canvas skips the frame instead of the protocol handler panicking.
fn response_error(e: tauri::http::Error) -> tauri::http::Response<Vec<u8>> {
    log::error!("emu://frame response: {e}");
    let mut res = tauri::http::Response::new(Vec::new());
    *res.status_mut() = tauri::http::StatusCode::INTERNAL_SERVER_ERROR;
    res
}

fn send(state: &tauri::State<'_, AppState>, cmd: Command) -> Result<(), ErrorCode> {
    state.send(cmd)
}

impl AppState {
    fn send(&self, cmd: Command) -> Result<(), ErrorCode> {
        match self.session.lock().as_ref() {
            Some(s) => {
                s.send(cmd);
                Ok(())
            }
            None => Err(ErrorCode::NoGame),
        }
    }
}

/// A command to the running game from outside a Tauri command (the controller combos, setting
/// changes). False when there is no game, or before the state exists.
fn send_command(app: &tauri::AppHandle, cmd: Command) -> bool {
    app.try_state::<AppState>()
        .is_some_and(|state| state.send(cmd).is_ok())
}

/// F11 and the quick menu's «A schermo intero»: fullscreen on / off. Answers whether the
/// window is now fullscreen. No controller shortcut (docs/combo-tasti.md).
#[tauri::command]
fn toggle_fullscreen(app: tauri::AppHandle) -> bool {
    let Some(window) = app.get_window("main") else {
        return false;
    };
    let on = window.is_fullscreen().unwrap_or(false);
    if let Err(e) = emudome_platform::set_fullscreen(&window, !on) {
        log::warn!("fullscreen {}: {e}", !on);
        return on;
    }
    !on
}

/// Whether the window is fullscreen (the quick menu's tile shows it pressed).
#[tauri::command]
fn is_fullscreen(app: tauri::AppHandle) -> bool {
    app.get_window("main")
        .and_then(|w| w.is_fullscreen().ok())
        .unwrap_or(false)
}

/// The interface tells when the quick menu opens and closes, so the controllers
/// navigate it instead of reaching the game.

#[tauri::command]
fn set_menu_open(state: tauri::State<'_, AppState>, open: bool) {
    state.menu_open.store(open, Ordering::Relaxed);
}

#[tauri::command]
fn set_speed(state: tauri::State<'_, AppState>, speed: Speed) -> Result<(), ErrorCode> {
    send(&state, Command::SetSpeed(speed))
}

#[tauri::command]
fn set_paused(state: tauri::State<'_, AppState>, paused: bool) -> Result<(), ErrorCode> {
    send(&state, Command::SetPaused(paused))
}

#[tauri::command]
fn save_state(state: tauri::State<'_, AppState>, slot: u8) -> Result<(), ErrorCode> {
    send(&state, Command::SaveState(slot))
}

#[tauri::command]
fn load_state(state: tauri::State<'_, AppState>, slot: u8) -> Result<(), ErrorCode> {
    send(&state, Command::LoadState(slot))
}

/// «Annulla il caricamento» (`saves.undo`): back to before the last load.
#[tauri::command]
fn undo_load(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    send(&state, Command::UndoLoad)
}

/// «Salvato nello slot N · Annulla»: the slot of the last save gets back what it held.
#[tauri::command]
fn undo_save(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    send(&state, Command::UndoSave)
}

/// «Muto» (quick menu): silence for the open game only; the next game starts with sound.
#[tauri::command]
fn set_muted(state: tauri::State<'_, AppState>, muted: bool) -> Result<(), ErrorCode> {
    let current = state.current.lock().clone();
    let shared = current.ok_or(ErrorCode::NoGame)?;
    shared.muted.store(muted, Ordering::Relaxed);
    Ok(())
}

/// Backspace (held, or toggled with `a11y.toggle`): rewind on / off.
#[tauri::command]
fn rewind(state: tauri::State<'_, AppState>, on: bool) -> Result<(), ErrorCode> {
    send(&state, Command::Rewind(on))
}

/// F12, L3 + B, quick menu › Strumenti: a PNG of the game next to the saves
/// (`screenshotSaved` event, or `screenshotFailed`).
#[tauri::command]
fn screenshot(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    send(&state, Command::Screenshot)
}

#[tauri::command]
fn reset_game(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    send(&state, Command::Reset)
}

/// Keyboard: for now it goes through the webview (which has the focus).
#[tauri::command]
fn input(state: tauri::State<'_, AppState>, button: Button, pressed: bool) {
    if let Some(shared) = state.current.lock().as_ref() {
        shared.input.set(Source::Keyboard, button, pressed);
    }
}

/// Where a pointer position is measured (`pointer`): the window in the renderer's unit (physical
/// pixels for the native video, CSS pixels for the webview canvas) and the bottom share (0..0.9)
/// kept clear for the quick menu. Mirror: `PointerArea` in app/src/emu.ts.
#[derive(serde::Deserialize)]
struct PointerArea {
    width: f32,
    height: f32,
    inset: f32,
}

/// Touch screen (NDS): the mouse or a finger at (`x`, `y`) in `area`, pressed or not. Rust maps it
/// through the composed bottom screen (`screens::pointer_at`); off the touch screen it is a
/// release, as on the console. One-screen consoles ignore it.
/// TODO(#28): the layout from the `screens.*` settings; today the default (stacked).
#[tauri::command]
fn pointer(state: tauri::State<'_, AppState>, x: f32, y: f32, pressed: bool, area: PointerArea) {
    let Some(shared) = state.current.lock().clone() else {
        return;
    };
    let at = if pressed {
        let (frame, split) = {
            let f = shared.frame.lock();
            ((f.width, f.height), f.split)
        };
        let area = Area {
            width: area.width,
            height: area.height,
            inset: area.inset,
        };
        emudome_core::screens::pointer_at(&Layout::default(), split, frame, area, x, y)
    } else {
        None
    };
    shared.input.set_pointer(at);
}

#[tauri::command]
fn stats(state: tauri::State<'_, AppState>) -> StatsDto {
    match state.current.lock().as_ref() {
        Some(shared) => {
            let s = shared.stats.lock();
            StatsDto {
                running: true,
                emu_fps: s.emu_fps,
                target_fps: s.av_info.timing.fps,
                speed: s.speed,
                paused: s.paused,
                frames: s.frames,
                auto_saved_at: s.auto_saved_at.and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .and_then(|d| u64::try_from(d.as_millis()).ok())
                }),
            }
        }
        None => StatsDto {
            running: false,
            emu_fps: 0.0,
            target_fps: 0.0,
            speed: None,
            paused: false,
            frames: 0,
            auto_saved_at: None,
        },
    }
}

/// Hands the system's surface (Android's `SurfaceView` under the webview) to the renderer.
fn attach_surface(ui: &tauri::Webview, renderer: video::Renderer) {
    emudome_platform::attach_external_surface(ui, move |event| match event {
        emudome_platform::SurfaceEvent::Ready(surface) => {
            let (width, height) = surface.size();
            renderer.attach(wgpu::SurfaceTarget::from(surface), width, height);
        }
        emudome_platform::SurfaceEvent::Resized { width, height } => renderer.resize(width, height),
        emudome_platform::SurfaceEvent::Lost => renderer.detach(),
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let main_start = Instant::now();
    let main_wall = SystemTime::now();
    emudome_platform::init_logging();
    log::info!("EmuDome on {}", emudome_platform::OS_NAME);
    let tuning = Tuning::from_env();
    dirs::set_override(tuning.data_dir.clone());
    for ignored in &tuning.warnings {
        log::warn!("ignored, the default stays: {ignored}");
    }
    let bench = bench::Bench::new(&tuning, main_start, main_wall);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("emu", |ctx, req| {
            // `emu://cover/<id>`: covers; `emu://background/<name>`: the user's background
            // (video.surroundImg); `emu://slot/<n>`: slot thumbnails of the open game;
            // `emu://savethumb/<game>/<n>`: those of any game (Saves section); else the frame.
            let path = req.uri().path();
            if let Some(id) = path.strip_prefix("/cover/") {
                library::cover_response(ctx.app_handle(), id)
            } else if let Some(name) = path.strip_prefix("/background/") {
                settings::background_response(ctx.app_handle(), name)
            } else if let Some(rest) = path.strip_prefix("/savethumb/") {
                saves::thumb_response(ctx.app_handle(), rest)
            } else if let Some(slot) = path.strip_prefix("/slot/") {
                slot_thumb_response(ctx.app_handle(), slot)
            } else {
                frame_protocol(ctx.app_handle(), &req)
            }
        })
        .setup(move |app| {
            let data_dir = dirs::data(app)?;
            log::info!("data in {}", data_dir.display());
            let (window, ui) = window::create(
                app,
                &window::Options {
                    // A bench runs while the PC is in use: it never takes the focus.
                    focused: bench.is_none(),
                    webview_data_dir: tuning.data_dir.as_ref().map(|d| d.join("webview")),
                },
            )?;
            let size = window.inner_size()?;

            let current: CurrentShared = Arc::new(Mutex::new(None));
            // The native renderer is created only where it can sit under the webview:
            // where it covers the interface (Linux/GTK) the game goes into the webview's canvas.
            let native = tuning
                .video
                .path
                .map_or_else(emudome_platform::native_video_default, |p| {
                    p == VideoPath::Native
                });
            let surface = video::SurfaceSettings::from_tuning(&tuning.video);
            let mut native_error = None;
            let renderer = if native {
                let spawned = if emudome_platform::EXTERNAL_SURFACE {
                    video::Renderer::spawn_detached(surface)
                        .inspect(|r| attach_surface(&ui, r.clone()))
                } else {
                    video::Renderer::spawn(window.clone(), size.width, size.height, surface)
                };
                match spawned {
                    Ok(r) => Some(r),
                    Err(e) => {
                        log::error!("native video unavailable, using the webview: {e:#}");
                        native_error = Some(format!("{e:#}"));
                        None
                    }
                }
            } else {
                log::info!("video in the webview (canvas)");
                None
            };
            let mode = if renderer.is_some() {
                VideoMode::Native
            } else {
                VideoMode::Webview
            };
            ui.set_background_color(Some(mode.webview_background()))?;
            let audio = audio::AudioOutput::start(
                current.clone(),
                tuning.audio.clone(),
                bench.as_ref().and_then(|b| b.tone()),
            );
            let menu_open = Arc::new(AtomicBool::new(false));
            let pads = pads::PadsState::default();
            app.manage(pads.clone());
            #[cfg(desktop)]
            gamepad::spawn(
                app.handle().clone(),
                current.clone(),
                menu_open.clone(),
                pads,
            );

            // An external surface reports its own size.
            if let Some(resize) = renderer
                .clone()
                .filter(|_| !emudome_platform::EXTERNAL_SURFACE)
            {
                window.on_window_event(move |ev| {
                    if let WindowEvent::Resized(s) = ev {
                        resize.resize(s.width, s.height);
                    }
                });
            }

            app.manage(settings::open(app)?);
            app.manage(library::open(app)?);
            app.manage(AppState {
                session: Mutex::new(None),
                current,
                renderer,
                native_error,
                video_mode: Mutex::new(mode),
                menu_open,
                data_dir,
                resource_dir: app.path().resource_dir().ok(),
                game: Mutex::new(None),
                core_name: Mutex::new(None),
                closing: Mutex::new(()),
                closer: closing::Closer::default(),
                opening: Mutex::new(()),
                audio,
                bench: bench.clone(),
            });
            app.manage(effects::open(app));
            effects::apply(app.handle());
            // After the library is read, before any game opens (opening waits for it).
            migration::start(app);
            library::startup(app.handle());
            if let Some(bench) = bench.clone() {
                bench::spawn(app.handle().clone(), bench);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            close_game,
            closing::close_wait,
            closing::close_anyway,
            abandon_game,
            list_slots,
            set_speed,
            set_paused,
            save_state,
            load_state,
            undo_load,
            undo_save,
            set_muted,
            reset_game,
            rewind,
            screenshot,
            input,
            pointer,
            stats,
            video_mode,
            set_video_mode,
            set_video_inset,
            startup_game,
            set_menu_open,
            bench::bench_active,
            bench::bench_frames,
            toggle_fullscreen,
            is_fullscreen,
            theme::theme,
            settings::settings_schema,
            settings::settings_page,
            settings::settings_recent,
            settings::settings_game_scope,
            settings::settings_game_overrides,
            settings::settings_set,
            settings::settings_reset,
            settings::settings_session_commit,
            settings::settings_session_discard,
            effects::effects_choose_file,
            bios::bios_choose,
            bios::bios_status,
            settings::settings_choose_background,
            pads::gamepad_list,
            pads::gamepad_watch,
            pads::gamepad_capture,
            pads::current_slot,
            pads::step_slot,
            pads::controls_bindings,
            library::library_get,
            library::library_choose_folder,
            library::library_remove_folder,
            library::library_add_paths,
            library::library_scan,
            library::library_open,
            library::library_set_cover,
            library::library_set_flags,
            library::library_set_name,
            saves::saves_summary,
            saves::saves_game,
            saves::saves_restore,
            saves::saves_restore_latest,
            saves::saves_delete_quick,
            saves::saves_export,
            saves::saves_import_pick,
            saves::saves_import,
            saves::saves_import_folder,
            saves::saves_import_many,
            migration::saves_migration_status,
            first_run::first_run_show,
            first_run::first_run_steps,
            first_run::first_run_done,
        ])
        .build(tauri::generate_context!())
        .expect("EmuDome failed to start")
        .run(|app, event| on_run_event(app, &event));
}
