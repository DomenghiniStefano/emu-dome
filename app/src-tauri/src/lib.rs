//! EmuDome's Tauri shell: the wiring.
//!
//! One native window: wgpu draws the game straight onto the window,
//! the Vite interface sits on top in a transparent webview ("ui").
//! The interface never touches the core: it calls the commands (`commands.rs`, `game.rs` and one
//! module per feature), which reach the open game through its `GameSession`
//! (`game_session.rs`). The `emu://` scheme is `protocol.rs`.
//!
//! Errors reach the interface as [`ErrorCode`] (`{ code, params }`), never as sentences.

mod audio;
// Bench mode only in builds with the `bench` feature (off by default, never packaged).
#[cfg(feature = "bench")]
mod bench;
#[cfg(not(feature = "bench"))]
#[path = "bench_off.rs"]
mod bench;
mod bios;
mod closing;
mod commands;
#[cfg(test)]
mod contract;
mod core_settings;
mod cors;
mod dirs;
mod effects;
mod events;
mod first_run;
mod game;
mod game_session;
// SDL3 is built for the desktop only: controllers on Android come later (Android input events).
#[cfg(desktop)]
mod gamepad;
mod gpu_interop;
mod health;
mod hotkeys;
mod hw_render;
mod library;
mod migration;
mod pads;
mod protocol;
mod saves;
mod settings;
mod theme;
mod tuning;
mod video;
mod window;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use emudome_core::input::Source;
use emudome_core::{Command, ErrorCode, Event, Shared};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::webview::Color;
use tauri::{Emitter, Manager, WindowEvent};
use ts_rs::TS;

pub(crate) use crate::commands::unix_ms;
pub(crate) use crate::game::{open_library_game, GameInfo};
use crate::game_session::{GameSession, Hooks};
/// The app's GPU and a Vulkan session's video with no window, for the headless examples
/// (`examples/headless_gpu.rs`).
pub use crate::hw_render::headless_setup;
pub(crate) use crate::protocol::response_error;
use crate::tuning::{Tuning, VideoPath};

/// The active session, read by renderer, audio and controllers: an `ArcSwapOption`, so the audio
/// callback and the controller thread read it without a lock (Trello #79).
pub type CurrentShared = Arc<arc_swap::ArcSwapOption<Shared>>;

/// Where the game is drawn: native surface under the webview, or canvas in the webview.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "app/")]
pub(crate) enum VideoMode {
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
    /// The open game and everything that lives as long as it does.
    games: GameSession,
    /// `None` where the native renderer cannot sit under the webview (Linux) or is off.
    renderer: Option<video::Renderer>,
    /// The GPU for the whole run, the renderer's and the 3DS games' (`hw_render.rs`).
    gpu: Arc<video::GpuService>,
    /// Why the native renderer could not start, when it was wanted.
    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    native_error: Option<String>,
    video_mode: Mutex<VideoMode>,
    /// Whether the quick menu is open, set by the interface: while it is, the controllers
    /// navigate the menu instead of playing (see `gamepad.rs`).
    menu_open: Arc<AtomicBool>,
    /// The shortcuts of the controllers and of the keyboard, carried out the same way.
    hotkeys: Arc<hotkeys::HotkeyRunner>,
    data_dir: PathBuf,
    resource_dir: Option<PathBuf>,
    /// The close flow (Trello #86): the window's X, quitting, closing or switching the game
    /// wait for the exit writes off the event loop, with progress on `emu://closing`.
    closer: closing::Closer,
    /// Held while a game opens: opens run off the event loop (they may wait for a close), one
    /// at a time.
    opening: Mutex<()>,
    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    audio: audio::AudioOutput,
    /// Bench mode (`EMUDOME_BENCH`), off otherwise.
    bench: Option<Arc<bench::Bench>>,
}

impl AppState {
    fn save_dir(&self) -> PathBuf {
        self.data_dir.join("saves")
    }

    /// Lets go of every key (menu opened, focus lost): the open game's keyboard buttons, and the
    /// keyboard's fast forward and held rewind.
    fn release_keyboard(&self) {
        if let Some(shared) = self.games.current().load().as_ref() {
            shared.input.release(Source::Keyboard);
        }
        self.hotkeys.release_keys();
    }

    /// Hands the game to the native renderer only when it is the active mode.
    fn native_source(&self, shared: Option<Arc<Shared>>) {
        if let Some(r) = &self.renderer {
            let native = *self.video_mode.lock() == VideoMode::Native;
            r.set_source(if native { shared } else { None });
        }
    }
}

/// What lives as long as a game is open, in the order it starts (it stops in reverse): the
/// native renderer's source, the settings session («in prova»), the core options that follow
/// the settings, the library's play time.
fn game_hooks(app: &tauri::AppHandle) -> Vec<Hooks> {
    let (a, b) = (app.clone(), app.clone());
    let renderer = Hooks::new(
        move |g| {
            if let Some(state) = a.try_state::<AppState>() {
                state.native_source(Some(g.shared.clone()));
            }
        },
        move || {
            if let Some(state) = b.try_state::<AppState>() {
                state.native_source(None);
            }
        },
    );
    let (a, b) = (app.clone(), app.clone());
    let settings_session = Hooks::new(
        move |g| settings::begin_session(&a, g.console, g.key.clone()),
        move || settings::end_session(&b),
    );
    let (a, b) = (app.clone(), app.clone());
    let core_options = Hooks::new(
        move |g| core_settings::started(&a, g.core, g.console, g.core_options.clone()),
        move || core_settings::stopped(&b),
    );
    let (a, b) = (app.clone(), app.clone());
    let play_time = Hooks::new(
        move |g| {
            if g.library {
                library::begin_play(&a, &g.key);
            }
        },
        move || library::end_play(&b),
    );
    vec![renderer, settings_session, core_options, play_time]
}

/// Runs `f` on a blocking thread of the async runtime and waits for it there: a command whose
/// work is heavy (reading a ROM, loading a core, waiting for a game to close, copying saves)
/// never holds the event loop nor an async worker. A panic in `f` (a build that unwinds)
/// answers `threadStopped`.
pub(crate) async fn blocking<R: Send + 'static>(
    app: tauri::AppHandle,
    f: impl FnOnce(&tauri::AppHandle) -> R + Send + 'static,
) -> Result<R, ErrorCode> {
    tauri::async_runtime::spawn_blocking(move || f(&app))
        .await
        .map_err(|e| {
            log::error!("a command stopped: {e}");
            ErrorCode::ThreadStopped {
                thread: "commands".into(),
            }
        })
}

/// A game on `core` was let go: its exit writes may be missing and the core is refused until
/// EmuDome restarts. Tells the interface.
fn say_core_needs_restart(app: &tauri::AppHandle, core: &str) {
    let error = Event::Error {
        error: ErrorCode::CoreNeedsRestart { core: core.into() },
    };
    if let Err(e) = app.emit(events::SESSION, error) {
        log::warn!("coreNeedsRestart not sent to the interface: {e}");
    }
}

/// A command to the running game from outside a Tauri command (the controller combos, setting
/// changes). False when there is no game, or before the state exists.
fn send_command(app: &tauri::AppHandle, cmd: Command) -> bool {
    app.try_state::<AppState>()
        .is_some_and(|state| state.games.send(cmd).is_ok())
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
        // The window lost the focus: keys let go elsewhere never reach the page (#74). With the
        // webview holding the focus this may not fire; the page's `blur` calls `release_keys`.
        if matches!(event, WindowEvent::Focused(false)) {
            state.release_keyboard();
        }
        if emudome_platform::is_suspend(event) {
            // The game keeps its state; NoGame (nothing open) is fine.
            if state.games.send(Command::Flush).is_ok() {
                log::info!("in background: the open game's saves are flushed");
            }
        }
    }
}

/// Hands the system's surface (Android's `SurfaceView` under the webview) to the renderer.
/// A surface the renderer cannot use is reported by the renderer (`videoSurfaceFailed`).
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
        .register_uri_scheme_protocol("emu", |ctx, req| protocol::handle(ctx.app_handle(), &req))
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
            // What is not working, for the interface (`emu://health`, `diagnostics`).
            let health = health::SharedHealth::default();
            health.attach(app.handle().clone());
            app.manage(health.clone());

            let current: CurrentShared = Arc::default();
            // The native renderer is created only where it can sit under the webview:
            // where it covers the interface (Linux/GTK) the game goes into the webview's canvas.
            let native = tuning
                .video
                .path
                .map_or_else(emudome_platform::native_video_default, |p| {
                    p == VideoPath::Native
                });
            let surface = video::SurfaceSettings::from_tuning(&tuning.video);
            let gpu = video::GpuService::new();
            let mut native_error = None;
            let renderer = if native {
                let spawned = if emudome_platform::EXTERNAL_SURFACE {
                    video::Renderer::spawn_detached(gpu.clone(), surface, health.clone())
                        .inspect(|r| attach_surface(&ui, r.clone()))
                } else {
                    video::Renderer::spawn(
                        gpu.clone(),
                        window.clone(),
                        size.width,
                        size.height,
                        surface,
                        health.clone(),
                    )
                };
                match spawned {
                    Ok(r) => Some(r),
                    Err(e) => {
                        log::error!("native video unavailable, using the webview: {e:#}");
                        let detail = format!("{e:#}");
                        health.set(
                            health::Part::NativeVideo,
                            Some(ErrorCode::NativeVideoFallback {
                                detail: detail.clone(),
                            }),
                        );
                        native_error = Some(detail);
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
                bench.is_some(),
                health.clone(),
            );
            let menu_open = Arc::new(AtomicBool::new(false));
            let pads = pads::PadsState::default();
            app.manage(pads.clone());
            let hotkeys =
                hotkeys::HotkeyRunner::new(app.handle().clone(), current.clone(), pads.clone());
            #[cfg(desktop)]
            gamepad::spawn(
                app.handle().clone(),
                current.clone(),
                menu_open.clone(),
                pads,
                hotkeys.clone(),
                health.clone(),
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
            app.manage(core_settings::CoreSettings::default());
            app.manage(AppState {
                games: GameSession::new(current, game_hooks(app.handle())),
                renderer,
                gpu,
                native_error,
                video_mode: Mutex::new(mode),
                menu_open,
                hotkeys,
                data_dir,
                resource_dir: app.path().resource_dir().ok(),
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
            game::open_game,
            game::close_game,
            closing::close_wait,
            closing::close_anyway,
            game::abandon_game,
            commands::list_slots,
            commands::set_speed,
            commands::set_paused,
            commands::save_state,
            commands::load_state,
            commands::undo_load,
            commands::undo_save,
            commands::set_muted,
            commands::reset_game,
            commands::rewind,
            commands::screenshot,
            commands::input,
            commands::key,
            commands::pointer,
            commands::stats,
            commands::video_mode,
            commands::set_video_mode,
            commands::set_video_inset,
            commands::startup_game,
            game::open_extensions,
            commands::set_menu_open,
            commands::release_keys,
            bench::bench_active,
            #[cfg(feature = "bench")]
            bench::bench_frames,
            commands::toggle_fullscreen,
            commands::is_fullscreen,
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
            health::diagnostics,
        ])
        .build(tauri::generate_context!())
        .expect("EmuDome failed to start")
        .run(|app, event| on_run_event(app, &event));
}
