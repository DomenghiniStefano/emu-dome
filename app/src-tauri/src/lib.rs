//! EmuDome's Tauri shell (Phase 0).
//!
//! One native window: wgpu draws the game straight onto the window,
//! the Vite interface sits on top in a transparent webview ("ui").
//! The interface never touches the core: it calls the commands below.
//!
//! Errors reach the interface as [`ErrorCode`] (`{ code, params }`), never as sentences.

mod audio;
// SDL3 is built for the desktop only: controllers on Android come later (Android input events).
#[cfg(desktop)]
mod gamepad;
mod library;
mod pads;
mod saves;
mod settings;
mod theme;
mod video;
mod window;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use emudome_core::input::{Button, Source};
use emudome_core::settings::{ConsoleId, GameId};
use emudome_core::{Command, ErrorCode, Event, Session, SessionOptions, Shared, Speed};
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
    video_mode: Mutex<VideoMode>,
    /// Whether the quick menu is open, set by the interface: while it is, the controllers
    /// navigate the menu instead of playing (see `gamepad.rs`).
    menu_open: Arc<AtomicBool>,
    data_dir: PathBuf,
    resource_dir: Option<PathBuf>,
    /// The file of the open game (its slots and thumbnails sit beside its saves).
    game_path: Mutex<Option<PathBuf>>,
    _audio: audio::AudioOutput,
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
    let Some(game) = state.game_path.lock().clone() else {
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
        let game = state.game_path.lock().clone()?;
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

/// Which core to use for a file, by its extension, the console shown and its settings level.
fn core_for(path: &Path) -> Result<(&'static str, &'static str, ConsoleId), ErrorCode> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "gba" => Ok(("mgba", "GBA", ConsoleId::Gba)),
        "gb" => Ok(("mgba", "GB", ConsoleId::Gb)),
        "gbc" => Ok(("mgba", "GBC", ConsoleId::Gbc)),
        "nds" => Err(ErrorCode::ConsoleNotYet {
            console: "nds".into(),
            phase: 2,
        }),
        "3ds" | "cci" | "cxi" | "cia" => Err(ErrorCode::ConsoleNotYet {
            console: "3ds".into(),
            phase: 3,
        }),
        _ => Err(ErrorCode::UnsupportedFormat { ext }),
    }
}

/// Opens a game file. A file that is in the library opens as that library game (its
/// identity, title and play time); any other file is identified by its path for now.
#[tauri::command]
fn open_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<GameInfo, ErrorCode> {
    if let Some((id, title, console)) = library::game_at(&app, &path) {
        return open_library_game(&app, &state, &id, PathBuf::from(path), title, console);
    }
    let game_path = PathBuf::from(&path);
    let game = GameId::from_rom_path(&game_path);
    open_path(&app, &state, game_path, game, None, None)
}

/// Opens a library game: its settings are those of its content identity, and its play time
/// counts until it closes.
fn open_library_game(
    app: &tauri::AppHandle,
    state: &AppState,
    id: &str,
    game_path: PathBuf,
    title: String,
    console: ConsoleId,
) -> Result<GameInfo, ErrorCode> {
    // TODO(game identity, Trello #21): settings files saved under the path hash before the
    // library existed are not moved to the content identity.
    let game = emudome_core::library::settings_game_id(id)
        .unwrap_or_else(|| GameId::from_rom_path(&game_path));
    let info = open_path(app, state, game_path, game, Some(title), Some(console))?;
    library::begin_play(app, id);
    Ok(info)
}

/// Core options the shell always sets, per core (`GET_VARIABLE` key and value).
/// Azahar: the software renderer, the only one that does not ask for `SET_HW_RENDER`, which the
/// host does not implement yet (Trello #16); its default `auto` fails to load.
/// TODO(settings engine): the `cores.*` entries will be mapped to core options here.
const FORCED_CORE_OPTIONS: &[(&str, &[(&str, &str)])] =
    &[("azahar", &[("citra_graphics_api", "Software")])];

fn forced_core_options(core_name: &str) -> Vec<(String, String)> {
    FORCED_CORE_OPTIONS
        .iter()
        .filter(|(core, _)| *core == core_name)
        .flat_map(|(_, opts)| opts.iter())
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// Closes the open game, then opens `game_path` with `game` as its settings identity.
/// `title` and `console` come from the library when known (else the file name and the extension).
fn open_path(
    app: &tauri::AppHandle,
    state: &AppState,
    game_path: PathBuf,
    game: GameId,
    title: Option<String>,
    console: Option<ConsoleId>,
) -> Result<GameInfo, ErrorCode> {
    let (core_name, console_label, ext_console) = core_for(&game_path)?;
    let core_path = emudome_platform::find_core(core_name, state.resource_dir.as_deref())
        .ok_or_else(|| ErrorCode::CoreNotFound {
            file: emudome_platform::core_file_name(core_name),
        })?;

    // Close the previous game before opening another one.
    close_current(app, state);

    let console_id = console.unwrap_or(ext_console);
    let flow = settings::game_flow(app, console_id, &game);
    let save_dir = state.save_dir();
    // E04 (a broken in-game save is set aside and the open refused) and the launch backup.
    saves::before_open(app, &save_dir, &game_path)?;
    let resumable = flow.resume == "ask" && emudome_core::has_auto_save(&save_dir, &game_path);

    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let session = Session::start(
        SessionOptions {
            core_path,
            game_path: game_path.clone(),
            system_dir: state.data_dir.join("system"),
            save_dir,
            autosave_every: flow.autosave_every,
            autosave_on_exit: flow.autosave_on_exit,
            resume_auto: flow.resume == "always",
            core_options: forced_core_options(core_name),
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
    *state.current.lock() = Some(shared.clone());
    state.native_source(Some(shared));
    *state.session.lock() = Some(session);
    *state.game_path.lock() = Some(game_path.clone());
    settings::begin_session(app, console_id, game);

    Ok(GameInfo {
        title: title.unwrap_or_else(|| {
            game_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        }),
        console: console_label.into(),
        console_id,
        core: core_name.into(),
        resumable,
        autosave_on_exit: flow.autosave_on_exit,
    })
}

/// Ends the open game: play time, the settings session, video, the emulation thread.
fn close_current(app: &tauri::AppHandle, state: &AppState) {
    library::end_play(app);
    settings::end_session(app);
    state.native_source(None);
    *state.current.lock() = None;
    state.session.lock().take();
    state.game_path.lock().take();
}

#[tauri::command]
fn close_game(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    close_current(&app, &state);
}

/// E11: the core stopped answering. Lets go of its thread without waiting for it (joining a
/// core stuck in `retro_run` never returns); the interface then offers to reopen the game.
#[tauri::command]
fn abandon_game(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    library::end_play(&app);
    settings::end_session(&app);
    state.native_source(None);
    *state.current.lock() = None;
    state.game_path.lock().take();
    let session = state.session.lock().take();
    if let Some(session) = session {
        session.abandon();
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

/// A command to the running game from outside a Tauri command (the controller combos).
/// False when there is no game, or before the state exists.
#[cfg(desktop)]
fn send_command(app: &tauri::AppHandle, cmd: Command) -> bool {
    app.try_state::<AppState>()
        .is_some_and(|state| state.send(cmd).is_ok())
}

/// The interface tells when the quick menu opens and closes, so the controllers
/// navigate it instead of reaching the game.
/// F11: fullscreen on / off. Keyboard only: no controller shortcut (docs/combo-tasti.md).
#[tauri::command]
fn toggle_fullscreen(app: tauri::AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let on = window.is_fullscreen().unwrap_or(false);
    if let Err(e) = window.set_fullscreen(!on) {
        log::warn!("fullscreen {}: {e}", !on);
    }
}

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
    emudome_platform::init_logging();
    log::info!("EmuDome on {}", emudome_platform::OS_NAME);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("emu", |ctx, req| {
            // `emu://cover/<id>`: covers; `emu://slot/<n>`: slot thumbnails of the open game;
            // `emu://savethumb/<game>/<n>`: those of any game (Saves section); else the frame.
            let path = req.uri().path();
            if let Some(id) = path.strip_prefix("/cover/") {
                library::cover_response(ctx.app_handle(), id)
            } else if let Some(rest) = path.strip_prefix("/savethumb/") {
                saves::thumb_response(ctx.app_handle(), rest)
            } else if let Some(slot) = path.strip_prefix("/slot/") {
                slot_thumb_response(ctx.app_handle(), slot)
            } else {
                frame_protocol(ctx.app_handle(), &req)
            }
        })
        .setup(|app| {
            let (window, ui) = window::create(app)?;
            let size = window.inner_size()?;

            let current: CurrentShared = Arc::new(Mutex::new(None));
            // The native renderer is created only where it can sit under the webview:
            // where it covers the interface (Linux/GTK) the game goes into the webview's canvas.
            let native = emudome_platform::native_video_default();
            let renderer = if native {
                let spawned = if emudome_platform::EXTERNAL_SURFACE {
                    video::Renderer::spawn_detached().inspect(|r| attach_surface(&ui, r.clone()))
                } else {
                    video::Renderer::spawn(window.clone(), size.width, size.height)
                };
                match spawned {
                    Ok(r) => Some(r),
                    Err(e) => {
                        log::error!("native video unavailable, using the webview: {e:#}");
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
            let audio = audio::AudioOutput::start(current.clone());
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
            let data_dir = app.path().app_data_dir()?;
            log::info!("data in {}", data_dir.display());
            app.manage(AppState {
                session: Mutex::new(None),
                current,
                renderer,
                video_mode: Mutex::new(mode),
                menu_open,
                data_dir,
                resource_dir: app.path().resource_dir().ok(),
                game_path: Mutex::new(None),
                _audio: audio,
            });
            library::startup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            close_game,
            abandon_game,
            list_slots,
            set_speed,
            set_paused,
            save_state,
            load_state,
            undo_load,
            reset_game,
            input,
            stats,
            video_mode,
            set_video_mode,
            startup_game,
            set_menu_open,
            toggle_fullscreen,
            theme::theme,
            settings::settings_schema,
            settings::settings_page,
            settings::settings_recent,
            settings::settings_game_scope,
            settings::settings_set,
            settings::settings_reset,
            settings::settings_session_commit,
            settings::settings_session_discard,
            pads::gamepad_list,
            pads::gamepad_watch,
            pads::gamepad_capture,
            pads::controls_bindings,
            library::library_get,
            library::library_choose_folder,
            library::library_remove_folder,
            library::library_add_paths,
            library::library_scan,
            library::library_open,
            library::library_set_cover,
            library::library_set_flags,
            saves::saves_summary,
            saves::saves_game,
            saves::saves_restore,
            saves::saves_restore_latest,
            saves::saves_delete_quick,
            saves::saves_export,
            saves::saves_import_pick,
            saves::saves_import,
        ])
        .build(tauri::generate_context!())
        .expect("EmuDome failed to start")
        .run(|app, event| {
            // Quitting with a game open: its play time still counts.
            if let tauri::RunEvent::Exit = event {
                library::end_play(app);
            }
        });
}
