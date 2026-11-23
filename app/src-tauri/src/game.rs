//! Opening and ending a game: which core runs a file, the session started with the game's
//! settings and handed to the `GameSession` (`game_session.rs`), `close_game` / `abandon_game`.

use std::path::{Path, PathBuf};

use emudome_core::core_profile::{state_load_block, CoreId, Renderer, StateLoadBlock};
use emudome_core::n3ds::fallback::FallbackWatch;
use emudome_core::save_path::{PathLimits, PathUnit};
use emudome_core::settings::ConsoleId;
use emudome_core::{
    Command, ErrorCode, Event, GameKey, GameReader, Session, SessionOptions, Speed, VideoSetup,
};
use emudome_platform::CoreLookupError;
use serde::Serialize;
use tauri::{Emitter, Manager};
use ts_rs::TS;

use crate::game_session::{AfterEnd, Opened};
use crate::health::{Part, SharedHealth};
use crate::{bios, closing, cloud, library, migration, saves, settings, AppState};

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
    /// Why loading states is off for this game on the renderer it runs with (Trello #93), else
    /// null: the interface shows every load (Carica, the resume question's «Riprendi») disabled
    /// with this reason, and the session refuses them, rewind included.
    state_load_block: Option<StateLoadBlock>,
    /// The speed it starts at (`speed.spstart`).
    speed: Speed,
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
/// was replaced since it was added). Identification only: one header flag is read (a document
/// through its descriptor).
fn refuse_encrypted(
    console: ConsoleId,
    game_path: &Path,
    document: Option<&emudome_platform::ContentFile>,
) -> Result<(), ErrorCode> {
    use emudome_core::n3ds::rom::{check_file, check_open_file, Encryption};
    let encryption = || match document {
        None => check_file(game_path),
        Some(doc) => doc.reader().map_or(Encryption::Unknown, |mut f| {
            check_open_file(game_path, &mut f)
        }),
    };
    match console {
        ConsoleId::N3ds if encryption() == Encryption::Encrypted => Err(ErrorCode::GameEncrypted),
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

/// Trello #97: a core that cannot reach the game's folder may write into its own user folder,
/// shared by every game (Azahar). Its folders are looked at now, before the session, and again
/// once its thread ended ([`AfterEnd`]); a write there is reported once on `emu://health`
/// (`coreSavedOutsideGame`). Read only: nothing is created, written or deleted there.
fn watch_core_user_folder(app: &tauri::AppHandle, core: CoreId) -> Option<AfterEnd> {
    let folders = match core {
        CoreId::Azahar => emudome_platform::azahar_user_dirs(),
        // Neither writes outside the folders it is given.
        CoreId::Mgba | CoreId::MelonDsDs => return None,
    };
    let watch = FallbackWatch::before(folders);
    let app = app.clone();
    Some(Box::new(move || {
        if let Some(folder) = watch.written() {
            log::warn!(
                "{} wrote into its own folder {} instead of the game's",
                core.name(),
                folder.display()
            );
            app.state::<SharedHealth>().set(
                Part::CoreUserFolder,
                Some(ErrorCode::CoreSavedOutsideGame {
                    core: core.name().into(),
                    folder: folder.display().to_string(),
                }),
            );
        }
    }))
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

/// «Apri un file»: the game file the user picks; `filter` names the extensions EmuDome opens in
/// the desktop's dialog. Android: the system's document picker (`ACTION_OPEN_DOCUMENT`, its read
/// permission persisted), which offers every file (ROMs have no registered MIME types): the
/// extension of the document's name is checked here as opening checks it (`unsupportedFormat`,
/// `consoleNotYet`). `None` = cancelled. Off the event loop: it waits for the user.
#[tauri::command]
pub(crate) async fn pick_game_file(
    app: tauri::AppHandle,
    filter: String,
) -> Result<Option<String>, ErrorCode> {
    crate::blocking(app, move |app| pick_game_file_now(app, &filter)).await?
}

fn pick_game_file_now(app: &tauri::AppHandle, filter: &str) -> Result<Option<String>, ErrorCode> {
    if !emudome_platform::DOCUMENT_PICKERS {
        let extensions = emudome_core::library::open_extensions();
        return Ok(emudome_platform::pick_file(app, filter, &extensions)
            .map(|p| p.to_string_lossy().into_owned()));
    }
    let picked = emudome_platform::pick_document().unwrap_or_else(|e| {
        log::warn!("file picker: {e:?}");
        None
    });
    let Some(picked) = picked else {
        return Ok(None);
    };
    // A document without a name is checked when it opens (its link is named `game`).
    if let Some(name) = &picked.name {
        core_for(Path::new(name)).inspect_err(|e| {
            log::info!("picked file refused: {name}: {e:?}");
        })?;
    }
    Ok(Some(picked.uri))
}

/// The title of a game file outside the library, before it opens (G01, G02): the file name
/// without its extension, for a document (Android) its display name as the provider tells it,
/// never its URI. `None` when a document's name cannot be read.
#[tauri::command]
pub(crate) async fn game_file_title(
    app: tauri::AppHandle,
    path: String,
) -> Result<Option<String>, ErrorCode> {
    crate::blocking(app, move |_| {
        let name = if emudome_platform::is_content_uri(&path) {
            let name = emudome_platform::document_name(&path);
            log::info!("document {path}: display name {name:?}");
            name?
        } else {
            path
        };
        file_title(&name)
    })
    .await
}

/// A file name without folder and extension; `None` when nothing is left.
fn file_title(name: &str) -> Option<String> {
    let stem = Path::new(name)
        .file_stem()?
        .to_string_lossy()
        .trim()
        .to_owned();
    (!stem.is_empty()).then_some(stem)
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
        return open_library_game(app, &state, &key, GameFile::open(app, &path)?, title);
    }
    let file = GameFile::open(app, &path)?;
    // An extension EmuDome does not run is refused before any hashing.
    core_for(file.path())?;
    // The content decides, whether the file came as a path or as a document (Trello #14): the
    // same game has the same key both ways.
    let identity = library::identify_game_file(app, &file)?;
    if let Some(title) = library::title_of(app, &identity.id) {
        return open_library_game(app, &state, &identity.id, file, title);
    }
    open_path(app, &state, file, identity.id, Some(identity.title), false)
}

/// Opens a library game: its play time counts until it closes.
pub(crate) fn open_library_game(
    app: &tauri::AppHandle,
    state: &AppState,
    key: &GameKey,
    file: GameFile,
    title: String,
) -> Result<GameInfo, ErrorCode> {
    open_path(app, state, file, key.clone(), Some(title), true)
}

/// A game file as the core reads it: a filesystem path, or a `content://` document (Android's
/// picker, «Open with») opened by the platform without copying it, read through a path that
/// stays valid while this value lives (`emudome_platform::open_content`). The session keeps it
/// until its thread ended ([`keep_until_end`]).
pub(crate) struct GameFile {
    /// As the interface or the library has it: a path, or a `content://` URI.
    given: String,
    /// The path the core is given; for a document, named after it (it may not open: read the
    /// document through `document`).
    path: PathBuf,
    document: Option<emudome_platform::ContentFile>,
}

impl GameFile {
    /// `given` as the interface or the library has it: a path, or a `content://` URI.
    pub(crate) fn open(app: &tauri::AppHandle, given: &str) -> Result<Self, ErrorCode> {
        if !emudome_platform::is_content_uri(given) {
            return Ok(Self {
                given: given.to_owned(),
                path: PathBuf::from(given),
                document: None,
            });
        }
        let cache = app
            .path()
            .app_cache_dir()
            .map_err(|e| ErrorCode::RomUnreadable {
                detail: format!("{given}: no cache folder: {e}"),
            })?;
        let document =
            emudome_platform::open_content(given, &cache).map_err(|e| content_error(given, e))?;
        log::info!("{given}: document {:?}", document.name());
        Ok(Self {
            given: given.to_owned(),
            path: document.path().to_path_buf(),
            document: Some(document),
        })
    }

    /// Named after the game (its extension decides the console); opens it only for a path.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The path or URI it was opened from (a document's cache key in the library).
    pub(crate) fn given(&self) -> &str {
        &self.given
    }

    /// The document, when the game came as one.
    pub(crate) fn document(&self) -> Option<&emudome_platform::ContentFile> {
        self.document.as_ref()
    }
}

/// A document that could not be opened, for the interface: the game file cannot be read, and
/// the detail says why (no new code: the interface already tells it with what to check).
fn content_error(uri: &str, e: emudome_platform::ContentError) -> ErrorCode {
    use emudome_platform::ContentError;
    let detail = match e {
        ContentError::Unreadable(detail) => detail,
        ContentError::NotLocal => format!(
            "{uri} is not a file on this device (a cloud document?): copy the game to the \
             device's storage and open it from there"
        ),
        ContentError::Unsupported => format!("{uri}: documents open on Android only"),
    };
    ErrorCode::RomUnreadable { detail }
}

/// The session's end-of-thread check, extended to keep `document` open until then: a core may
/// read its game again while it runs (a 3DS game is streamed from its file).
fn keep_until_end(
    after_end: Option<AfterEnd>,
    document: Option<emudome_platform::ContentFile>,
) -> Option<AfterEnd> {
    let Some(document) = document else {
        return after_end;
    };
    Some(Box::new(move || {
        if let Some(check) = after_end {
            check();
        }
        drop(document);
    }))
}

/// `first`, then `second`, once the session's thread ended.
fn then(first: Option<AfterEnd>, second: Option<AfterEnd>) -> Option<AfterEnd> {
    match (first, second) {
        (Some(a), Some(b)) => Some(Box::new(move || {
            a();
            b();
        })),
        (a, None) => a,
        (None, b) => b,
    }
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

/// Closes the open game, then opens `file` as the game `game` (its console, saves and
/// settings). `title` comes from the library or the ROM (else the file name); `library`: a
/// library game, whose play time counts. A document stays open until the session's thread ended.
fn open_path(
    app: &tauri::AppHandle,
    state: &AppState,
    file: GameFile,
    game: GameKey,
    title: Option<String>,
    library: bool,
) -> Result<GameInfo, ErrorCode> {
    let GameFile {
        path: game_path,
        document,
        ..
    } = file;
    // The one-time move of old save names ends before any game reads its saves.
    migration::wait(app);
    let (core, console) = core_for(&game_path)?;
    refuse_encrypted(console, &game_path, document.as_ref())?;
    // A document is read through its descriptor (Android 17 may refuse its path).
    let game_file = match &document {
        None => None,
        Some(doc) => Some(GameReader {
            file: doc.reader().map_err(|e| ErrorCode::RomUnreadable {
                detail: format!("{}: {e}", game_path.display()),
            })?,
            path_opens: doc.reopenable(),
        }),
    };
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
    // Drive (#31): the game's newer saves come down first (G03), before anything reads them.
    cloud::before_launch(app, &game);
    let flow = settings::game_flow(app, core, console_id, &game);
    let save_dir = state.save_dir();
    // E04 (a broken in-game save is set aside and the open refused) and the launch backup.
    saves::before_open(app, &save_dir, &game)?;
    let resumable = flow.resume == "ask" && emudome_core::has_auto_save(&save_dir, &game);
    let (renderer, video) = game_video(app, state, core);
    // Trello #93: on this renderer the core would crash loading a state of this game.
    let state_load_block = state_load_block(core, renderer, &game);
    // After the previous game closed: its writes are in the "before".
    // Then, once its exit writes are done, its push to Drive (#31), in the background.
    let after_end = then(
        keep_until_end(watch_core_user_folder(app, core), document),
        cloud::after_exit(app, &game),
    );

    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let session = Session::start(
        SessionOptions {
            core_path,
            game_path: game_path.clone(),
            game_file,
            system_dir: bios::system_dir(app, core, state.data_dir.join("system")),
            save_dir,
            game: game.clone(),
            autosave_every: flow.autosave_every,
            autosave_on_exit: flow.autosave_on_exit,
            resume_auto: flow.resume == "always",
            app_version: app.package_info().version.to_string(),
            core_options: core_options(core, renderer, &flow),
            rewind_seconds: flow.rewind_seconds,
            state_load_block,
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

    // `speed.spstart`: queued before the first frame runs.
    if flow.start_speed != NORMAL_SPEED {
        session.send(Command::SetSpeed(flow.start_speed));
    }

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
        after_end,
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
        state_load_block,
        speed: flow.start_speed,
    })
}

/// The speed a game runs at unless `speed.spstart` says otherwise.
const NORMAL_SPEED: Speed = Speed::Factor(1.0);

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

#[cfg(test)]
mod tests {
    use super::file_title;

    #[test]
    fn a_file_title_is_its_name_without_folder_and_extension() {
        assert_eq!(
            file_title("Pokemon Mystery Dungeon (Europe) (En,Fr).nds").as_deref(),
            Some("Pokemon Mystery Dungeon (Europe) (En,Fr)")
        );
        assert_eq!(
            file_title("/storage/emulated/0/ROMs/tiny.blocks.gb").as_deref(),
            Some("tiny.blocks")
        );
        assert_eq!(file_title("rom").as_deref(), Some("rom"));
        assert_eq!(file_title("").as_deref(), None);
    }
}
