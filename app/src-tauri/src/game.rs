//! Opening and ending a game: which core runs a file, the session started with the game's
//! settings and handed to the `GameSession` (`game_session.rs`), `close_game` / `abandon_game`.

use std::path::{Path, PathBuf};

use emudome_core::core_profile::{CoreId, Renderer};
use emudome_core::save_path::{PathLimits, PathUnit};
use emudome_core::settings::ConsoleId;
use emudome_core::{ErrorCode, Event, GameKey, Session, SessionOptions, VideoSetup};
use emudome_platform::CoreLookupError;
use serde::Serialize;
use tauri::{Emitter, Manager};
use ts_rs::TS;

use crate::game_session::Opened;
use crate::health::{Part, SharedHealth};
use crate::{bios, closing, library, migration, saves, settings, AppState};

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub(crate) struct GameInfo {
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

/// Which core to use for a file, and its console, by its extension.
fn core_for(path: &Path) -> Result<(CoreId, ConsoleId), ErrorCode> {
    use emudome_core::library::{console_of_extension, console_phase_pending};
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let Some(console) = console_of_extension(&ext) else {
        return Err(ErrorCode::UnsupportedFormat { ext });
    };
    if let Some(phase) = console_phase_pending(console) {
        return Err(ErrorCode::ConsoleNotYet {
            console: console.id().into(),
            phase,
        });
    }
    // A kind of file the console's core does not run (a 3DS `.cia`).
    if !emudome_core::library::opens(&ext) {
        return Err(ErrorCode::UnsupportedFormat { ext });
    }
    Ok((CoreId::for_console(console), console))
}

/// E10: an encrypted 3DS game is refused before the core sees it (also a library game whose file
/// was replaced since it was added). Identification only: one header flag is read.
fn refuse_encrypted(console: ConsoleId, game_path: &Path) -> Result<(), ErrorCode> {
    use emudome_core::n3ds::rom::{check_file, Encryption};
    match console {
        ConsoleId::N3ds if check_file(game_path) == Encryption::Encrypted => {
            Err(ErrorCode::GameEncrypted)
        }
        ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba | ConsoleId::Nds | ConsoleId::N3ds => {
            Ok(())
        }
    }
}

/// Trello #94: a game whose save folder is too deep for the files its core creates there
/// (Azahar's emulated NAND) is refused before the core loads, which would abort EmuDome. Checked
/// before the open game closes, so a refused open leaves it running.
fn refuse_deep_save_dir(core: CoreId, saves_root: &Path, game: &GameKey) -> Result<(), ErrorCode> {
    let files = emudome_core::GameFiles::new(saves_root, game);
    core.profile()
        .check_save_dir(files.dir(), path_limits())
        .map_err(|e| {
            log::warn!(
                "{}: save folder too deep: {e} (long paths: policy {:?}, this process {})",
                core.name(),
                emudome_platform::long_paths_policy(),
                emudome_platform::long_paths_enabled()
            );
            ErrorCode::from(e)
        })
}

/// This process's path limits (`emudome_platform`), in the core's terms.
fn path_limits() -> PathLimits {
    let l = emudome_platform::path_limits();
    PathLimits {
        dir: l.dir,
        file: l.file,
        unit: if l.utf16 {
            PathUnit::Utf16
        } else {
            PathUnit::Byte
        },
    }
}

/// The extensions «Apri un file» offers: every console whose games open today.
#[tauri::command]
pub(crate) fn open_extensions() -> Vec<&'static str> {
    emudome_core::library::open_extensions()
}

/// Opens a game file. Every game is its content's key, never its path: a file of the library
/// opens as that library game; any other file is identified first (from the library's hash
/// cache when it knows the file unchanged, else by hashing it, holding no lock), and when its
/// content is a library game's (a copy elsewhere) it opens as that game too.
/// Off the event loop: it reads and hashes the ROM, loads the core, may wait for the open game's
/// close.
#[tauri::command]
pub(crate) async fn open_game(app: tauri::AppHandle, path: String) -> Result<GameInfo, ErrorCode> {
    crate::blocking(app, move |app| open_game_now(app, path)).await?
}

fn open_game_now(app: &tauri::AppHandle, path: String) -> Result<GameInfo, ErrorCode> {
    let state = app.state::<AppState>();
    if let Some((key, title)) = library::game_at(app, &path) {
        return open_library_game(app, &state, &key, PathBuf::from(path), title);
    }
    let game_path = PathBuf::from(&path);
    // An extension EmuDome does not run is refused before any hashing.
    core_for(&game_path)?;
    let identity = library::identify_path(app, &game_path)?;
    if let Some(title) = library::title_of(app, &identity.id) {
        return open_library_game(app, &state, &identity.id, game_path, title);
    }
    open_path(
        app,
        &state,
        game_path,
        identity.id,
        Some(identity.title),
        false,
    )
}

/// Opens a library game: its play time counts until it closes.
pub(crate) fn open_library_game(
    app: &tauri::AppHandle,
    state: &AppState,
    key: &GameKey,
    game_path: PathBuf,
    title: String,
) -> Result<GameInfo, ErrorCode> {
    open_path(app, state, game_path, key.clone(), Some(title), true)
}

/// The options the core starts with: the settings' (console options), then the ones the shell
/// always forces (`CoreProfile::forced_options`, the renderer's among them), then the system
/// files' (`emudome_core::bios`: what the files in place allow), each winning over the ones
/// before.
fn core_options(
    core: CoreId,
    renderer: Renderer,
    flow: &settings::GameFlow,
) -> Vec<(String, String)> {
    let mut options = emudome_core::settings::core_values::pairs(&flow.core_options);
    options.extend(core.profile().forced_pairs(renderer));
    options.extend(
        flow.bios
            .options
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    options
}

/// The renderer a game runs with, and the session's video: Vulkan on the app's device for a
/// core that can draw with it (3DS), else software. A core that would draw on the GPU and found
/// no Vulkan device is reported (`hwRenderFallback`) and runs in software.
fn game_video(app: &tauri::AppHandle, state: &AppState, core: CoreId) -> (Renderer, VideoSetup) {
    let hw = core.profile().hw_render;
    if hw.renderer(true) != Renderer::Vulkan {
        return (Renderer::Software, VideoSetup::default());
    }
    let health = app.state::<SharedHealth>();
    match state.gpu.vulkan_lease() {
        Ok(lease) => {
            health.set(Part::HwRender, None);
            (Renderer::Vulkan, crate::hw_render::video_setup(lease))
        }
        Err(e) => {
            log::warn!(
                "{}: no Vulkan device ({e:#}), software renderer",
                core.name()
            );
            health.set(
                Part::HwRender,
                Some(ErrorCode::HwRenderFallback {
                    detail: format!("{e:#}"),
                }),
            );
            (Renderer::Software, VideoSetup::default())
        }
    }
}

/// Closes the open game, then opens `game_path` as the game `game` (its console, saves and
/// settings). `title` comes from the library or the ROM (else the file name); `library`: a
/// library game, whose play time counts.
fn open_path(
    app: &tauri::AppHandle,
    state: &AppState,
    game_path: PathBuf,
    game: GameKey,
    title: Option<String>,
    library: bool,
) -> Result<GameInfo, ErrorCode> {
    // The one-time move of old save names ends before any game reads its saves.
    migration::wait(app);
    let (core, console) = core_for(&game_path)?;
    refuse_encrypted(console, &game_path)?;
    let file = emudome_platform::core_file_name(core.name());
    let core_path = emudome_platform::find_core(core.name(), state.resource_dir.as_deref())
        .map_err(|e| match e {
            CoreLookupError::NotFound => ErrorCode::CoreNotFound { file },
            CoreLookupError::Mismatch { .. } => ErrorCode::CoreDamaged { file },
        })?;
    // A game on this core was let go (E11, or a close that timed out): its thread may still be
    // inside the core's module. Checked before closing the open game, which stays open.
    if emudome_core::core_needs_restart(&core_path) {
        return Err(ErrorCode::CoreNeedsRestart {
            core: core.name().into(),
        });
    }
    refuse_deep_save_dir(core, &state.save_dir(), &game)?;

    // One open at a time; the previous game closes first, through the close flow (its
    // progress on `emu://closing`, the user's choice if it takes too long).
    let _opening = state.opening.lock();
    closing::close_game(app, state)?;

    // The content decides (a `.gb` file can be a Game Boy Color cartridge).
    let console_id = game.console();
    let flow = settings::game_flow(app, core, console_id, &game);
    let save_dir = state.save_dir();
    // E04 (a broken in-game save is set aside and the open refused) and the launch backup.
    saves::before_open(app, &save_dir, &game)?;
    let resumable = flow.resume == "ask" && emudome_core::has_auto_save(&save_dir, &game);
    let (renderer, video) = game_video(app, state, core);

    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let session = Session::start(
        SessionOptions {
            core_path,
            game_path: game_path.clone(),
            system_dir: bios::system_dir(app, core, state.data_dir.join("system")),
            save_dir,
            game: game.clone(),
            autosave_every: flow.autosave_every,
            autosave_on_exit: flow.autosave_on_exit,
            resume_auto: flow.resume == "always",
            app_version: app.package_info().version.to_string(),
            core_options: core_options(core, renderer, &flow),
            rewind_seconds: flow.rewind_seconds,
            // The content decides the console, the core its frame's screens.
            split: core.profile().split,
            username: flow.username.clone(),
            fast_audio: flow.fast_audio,
            video,
        },
        tx,
    )
    .map_err(|e| {
        log::warn!("{} did not start: {e}", core.name());
        e.into_code(core.name())
    })?;

    // The core's events go to the interface.
    let app_events = app.clone();
    std::thread::spawn(move || {
        for ev in rx {
            let _ = app_events.emit(crate::events::SESSION, ev);
        }
    });

    let shared = session.shared().clone();
    if let Some(tone) = state.bench.as_ref().and_then(|b| b.tone()) {
        shared.audio.set_test_tone(Some(tone));
    }
    state.games.open(
        session,
        &Opened {
            key: game.clone(),
            core,
            console: console_id,
            library,
            core_options: flow.core_options,
            shared,
        },
    );
    bios::emit_fallbacks(app, &flow.bios.fallbacks);

    Ok(GameInfo {
        id: game,
        title: title.unwrap_or_else(|| {
            game_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        }),
        console: console_id.short_name().into(),
        console_id,
        core: core.name().into(),
        resumable,
        autosave_on_exit: flow.autosave_on_exit,
    })
}

/// Closes the open game (back to the library) through the close flow; returns once it is
/// closed. Off the event loop: it waits for the exit writes and the user's choice.
#[tauri::command]
pub(crate) async fn close_game(app: tauri::AppHandle) {
    let closed = crate::blocking(app, |app| {
        closing::close_game(app, &app.state::<AppState>())
    });
    // `AppClosing`: a quit arrived meanwhile; the game is closed all the same. A command that
    // stopped was logged by `blocking`.
    if let Ok(Err(_)) = closed.await {
        log::info!("the game closed while the app is exiting");
    }
}

/// E11: the core stopped answering. Lets go of its thread without waiting for it (joining a
/// core stuck in `retro_run` never returns). The session is dead (it writes nothing more) and its
/// core is refused until EmuDome restarts (`coreNeedsRestart`).
#[tauri::command]
pub(crate) fn abandon_game(state: tauri::State<'_, AppState>) {
    state.games.abandon();
}
