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
mod video;
mod window;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use emudome_core::input::{Button, Source};
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
    data_dir: PathBuf,
    resource_dir: Option<PathBuf>,
    _audio: audio::AudioOutput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameInfo {
    title: String,
    console: String,
    core: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsDto {
    running: bool,
    emu_fps: f64,
    target_fps: f64,
    speed: Option<Speed>,
    paused: bool,
}

/// Which core to use for a file, by its extension.
fn core_for(path: &Path) -> Result<(&'static str, &'static str), ErrorCode> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "gba" | "gb" | "gbc" => Ok(("mgba", "GBA")),
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

#[tauri::command]
fn open_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<GameInfo, ErrorCode> {
    let game_path = PathBuf::from(&path);
    let (core_name, console) = core_for(&game_path)?;
    let core_path = emudome_platform::find_core(core_name, state.resource_dir.as_deref())
        .ok_or_else(|| ErrorCode::CoreNotFound {
            file: emudome_platform::core_file_name(core_name),
        })?;

    // Close the previous game before opening another one.
    state.native_source(None);
    *state.current.lock() = None;
    state.session.lock().take();

    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let session = Session::start(
        SessionOptions {
            core_path,
            game_path: game_path.clone(),
            system_dir: state.data_dir.join("system"),
            save_dir: state.data_dir.join("saves"),
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

    Ok(GameInfo {
        title: game_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        console: console.into(),
        core: core_name.into(),
    })
}

#[tauri::command]
fn close_game(state: tauri::State<'_, AppState>) {
    state.native_source(None);
    *state.current.lock() = None;
    state.session.lock().take();
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
    match state.session.lock().as_ref() {
        Some(s) => {
            s.send(cmd);
            Ok(())
        }
        None => Err(ErrorCode::NoGame),
    }
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
            }
        }
        None => StatsDto {
            running: false,
            emu_fps: 0.0,
            target_fps: 0.0,
            speed: None,
            paused: false,
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
        .register_uri_scheme_protocol("emu", |ctx, req| frame_protocol(ctx.app_handle(), &req))
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
            #[cfg(desktop)]
            gamepad::spawn(app.handle().clone(), current.clone());

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

            let data_dir = app.path().app_data_dir()?;
            log::info!("data in {}", data_dir.display());
            app.manage(AppState {
                session: Mutex::new(None),
                current,
                renderer,
                video_mode: Mutex::new(mode),
                data_dir,
                resource_dir: app.path().resource_dir().ok(),
                _audio: audio,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            close_game,
            set_speed,
            set_paused,
            save_state,
            load_state,
            reset_game,
            input,
            stats,
            video_mode,
            set_video_mode,
            startup_game,
        ])
        .run(tauri::generate_context!())
        .expect("EmuDome failed to start");
}
