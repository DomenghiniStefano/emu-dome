//! Errors shown to the user, as codes with parameters and never as sentences.
//!
//! Serialized as `{ "code": "slotEmpty", "params": { "slot": 2 } }` (`params` is absent
//! for codes without parameters). The interface turns each code into text through i18n
//! (`errors.<code>` in `app/src/i18n/*.json`); the TypeScript type is generated from this enum
//! into `app/src/generated/ErrorCode.ts`, with the list of codes in `error-codes.json`, which a
//! Vitest test holds against both message files (`crate::contract`). `detail` parameters carry technical text (English, from the OS or a
//! library) that the interface may show as is, never a sentence meant for the user. A file that
//! could not be read or written is told by its name or path and an [`IoCause`], each a value of
//! its own, never by the OS's sentence (which has the path inside it, Trello #128).

use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use crate::bios::Slot;
use crate::settings::Level;
use crate::GameKey;

/// Why a file or folder could not be read or created, as the interface needs it to say what to
/// do: the kind of an `io::Error`, never its text (that stays in the log). The interface has one
/// text per code and cause (`errorCauses.<code>.<cause>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum IoCause {
    /// It is not there (moved, deleted, a disconnected drive).
    NotFound,
    /// The system refused EmuDome access (permissions; on Android, a folder's permission given
    /// back).
    PermissionDenied,
    /// Any other failure (a read error, a damaged disk, a name the system cannot take).
    Other,
}

impl IoCause {
    /// The cause of `e`.
    pub fn of(e: &std::io::Error) -> Self {
        Self::from(e.kind())
    }
}

impl From<std::io::ErrorKind> for IoCause {
    // `ErrorKind` is non-exhaustive and grows: compared, not matched, so a new kind is `Other`.
    fn from(kind: std::io::ErrorKind) -> Self {
        if kind == std::io::ErrorKind::NotFound {
            IoCause::NotFound
        } else if kind == std::io::ErrorKind::PermissionDenied {
            IoCause::PermissionDenied
        } else {
            IoCause::Other
        }
    }
}

impl ErrorCode {
    /// The game file at `path` cannot be read (`cause`): its name and its path as values.
    pub fn rom_unreadable(path: &Path, cause: IoCause) -> Self {
        ErrorCode::RomUnreadable {
            file: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            path: path.display().to_string(),
            cause,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "code", content = "params", rename_all = "camelCase")]
pub enum ErrorCode {
    /// The file extension belongs to no supported console.
    UnsupportedFormat { ext: String },
    /// The console is planned for a later phase of the roadmap.
    ConsoleNotYet { console: String, phase: u8 },
    /// The libretro core file was not found in any search directory.
    CoreNotFound { file: String },
    /// This build does not ship the core of `console` (its id, e.g. `3ds`) for this platform
    /// (`cores.lock`): the console is not available on this device yet. Not a broken install.
    CoreUnavailable { console: String },
    /// The core file found is not the one this build ships (its sha256 differs from
    /// `cores.lock`'s): it is not loaded. The installation is damaged or was tampered with.
    CoreDamaged { file: String },
    /// The core or the game failed to load, for a reason none of the codes below names.
    OpenFailed { detail: String },
    /// The core file cannot be loaded or is not a libretro core EmuDome can drive (damaged,
    /// another architecture, a function missing): reinstalling EmuDome puts the right one back.
    CoreIncompatible { detail: String },
    /// The game file cannot be read: `file` is its name (a document's display name), `path`
    /// where it is (a document's URI), `cause` why. The OS's own text goes to the log only.
    RomUnreadable {
        file: String,
        path: String,
        cause: IoCause,
    },
    /// The game came as a document (Android) that is not a file on this device (a cloud
    /// provider's stream, `file` its name): it cannot be read again from its start, as cores and
    /// EmuDome do. Copying it to the device's storage helps.
    DocumentNotLocal { file: String },
    /// The core read the game and refused it: a bad or incomplete dump, an encrypted 3DS game,
    /// a format this core does not run.
    CoreRefusedGame,
    /// The game came as a document (Android) the system lets EmuDome read only through the
    /// handle it gave, and its core (`core`: Azahar) opens the game file by itself, by its path.
    DocumentNeedsPath { core: String },
    /// The file opened has the extension of a console but its content is not a game of it (its
    /// header does not match): it has no identity, so it has no place for its saves.
    GameUnidentified { detail: String },
    /// E10: the 3DS game is encrypted; EmuDome plays decrypted games only (it never decrypts,
    /// and never adds an encrypted game to the library).
    GameEncrypted,
    /// The game's save folder (`folder`) is too deep for the files its core creates under it on
    /// this system: the deepest would be `length` characters, over the `limit` the system allows
    /// (Windows without long paths: 259). Refused before the core loads, which would otherwise
    /// abort EmuDome (Azahar, Trello #94).
    SavePathTooLong {
        folder: String,
        length: usize,
        limit: usize,
    },
    /// The game's save folder (`folder`) could not be created (no permission, a file in its
    /// place, a disconnected drive): the game does not start, since its core would write its
    /// saves elsewhere (Azahar: its own folder, shared by every game; Trello #97).
    SaveFolderUnavailable { folder: String, cause: IoCause },
    /// A game on this core stopped answering and was let go (E11): its thread may still hold
    /// the core, so the core cannot open another game until EmuDome is restarted. `core` is the
    /// core's name (e.g. `mgba`).
    CoreNeedsRestart { core: String },
    /// A command that needs a running game arrived with none open.
    NoGame,
    /// EmuDome is closing (a quit arrived while a game closed): no game opens any more.
    AppClosing,
    /// The native video path does not exist on this system.
    NativeVideoUnavailable,
    /// The webview background could not be changed when switching video mode.
    WebviewBackgroundFailed { detail: String },
    /// The slot has no saved state yet.
    SlotEmpty { slot: u8 },
    /// The state could not be saved into the slot.
    StateSaveFailed { slot: u8, detail: String },
    /// The slot file exists but could not be read.
    StateLoadFailed { slot: u8, detail: String },
    /// The core refused the state (another game or another core version).
    StateInvalid { slot: u8 },
    /// Loading states (a quick load, the auto quick save, «Annulla il caricamento», rewind) is
    /// off for the open game: its core would crash on it (`reason`, Trello #93). Saving works.
    StateLoadUnsupported {
        reason: crate::core_profile::StateLoadBlock,
    },
    /// No setting has this key.
    SettingUnknown { key: String },
    /// The setting cannot be stored at `level` (e.g. a global-only one for a single game);
    /// `levels` are the ones it can be stored at, weakest first.
    SettingLevelRefused {
        key: String,
        level: Level,
        levels: Vec<Level>,
    },
    /// The value does not fit the setting (wrong type, unknown option, out of range), or the
    /// setting holds no value (an action).
    SettingInvalidValue { key: String },
    /// A settings file could not be written.
    SettingsWriteFailed { detail: String },
    /// A settings file is there but could not be read (permission, I/O error): it is not
    /// written over, so the change was not saved. Writes work again once it reads.
    SettingsReadFailed { detail: String },
    /// A library folder cannot be read now (missing, disconnected drive, no permission).
    LibraryFolderUnreachable { path: String },
    /// No game in the library has this id.
    LibraryGameUnknown { id: String },
    /// The game is in the library but its file cannot be reached now.
    LibraryGameUnavailable { id: String },
    /// The library database could not be written.
    LibraryWriteFailed { detail: String },
    /// The library database is there but could not be read: it is not written over, so the
    /// change was not saved. Writes work again once it reads.
    LibraryReadFailed { detail: String },
    /// The notifications file (`<app local data>/notifications.json`, X03) could not be written.
    NotificationsWriteFailed { detail: String },
    /// The notifications file is there but could not be read: it is not written over, so the
    /// change was not kept. Writes work again once it reads.
    NotificationsReadFailed { detail: String },
    /// The chosen cover cannot be used (not an image, too large, or no icon in the ROM).
    CoverInvalid { detail: String },
    /// «Annulla il caricamento» with no load to undo in this session.
    NothingToUndo,
    /// The core refused the state from before the last load.
    UndoFailed { detail: String },
    /// E04: the in-game save of `game` cannot be used (unreadable, empty, too large). It was
    /// moved aside as `aside`; `backup_at` is the Unix ms of the newest backup, if there is one.
    #[serde(rename_all = "camelCase")]
    SaveUnreadable {
        game: GameKey,
        aside: String,
        backup_at: Option<u64>,
    },
    /// The game is open: its saves cannot change under the running session.
    SaveGameOpen,
    /// The chosen file is not a save EmuDome can import.
    SaveImportInvalid { detail: String },
    /// A save file could not be read, written, copied or deleted.
    SaveFileFailed { detail: String },
    /// The one-time move of the saves and settings named the old way (by ROM file name, by
    /// path hash) to the game key could not run or stopped on an error. Nothing was lost: what
    /// was not moved is where it was, and it is tried again at the next start.
    SavesMigrationFailed { detail: String },
    /// The one-time move ended, but `left` old files could not be given to one game for sure
    /// (no library game, several with that file name, or already a file at the new place): they
    /// stay where they were, untouched, listed in the report file `report`.
    SavesMigrationIncomplete { left: usize, report: String },
    /// Health: the one-time copy of the data to the folders of the new app identifier (Trello
    /// #90) did not finish. Nothing was lost: this run uses the old folder `old`, untouched, and
    /// the copy is tried again at the next start.
    DataMoveFailed { old: String, detail: String },
    /// Health: the folders of the new app identifier (`new`) already held data, so the data of
    /// the old ones (`old`) was not copied over it. The old folders stay as they are; the new
    /// ones are used. Shown at the start that found it; `report` lists the folders.
    DataMoveConflict {
        old: String,
        new: String,
        report: String,
    },
    /// The screenshot could not be encoded or written (no frame yet, folder not writable).
    ScreenshotFailed { detail: String },
    /// A game's screenshots folder is there but could not be listed (L12 «Da uno screenshot»).
    ScreenshotsUnreadable { detail: String },
    /// The open game's core has no way to close the lid (`core_profile::Lid::Unsupported`:
    /// Azahar, a one-screen console). The interface shows the lid disabled before this.
    LidUnsupported,
    /// The screen layout `layout` (a `ScreenLayoutId`) cannot be shown for the open game: a
    /// custom layout that is not saved, or a one-screen game.
    ScreenLayoutUnavailable { layout: String },
    /// The screen effect `name` (a built-in one or the user's `.slangp` file name) could not
    /// be loaded or drawn; the game shows without it.
    EffectNotLoaded { name: String, detail: String },
    /// The chosen BIOS / firmware file cannot be read (`cause`).
    BiosUnreadable { cause: IoCause },
    /// The chosen file has a size the row's file never has (`size` in bytes).
    BiosWrongSize { size: u64 },
    /// The chosen file is the one of another BIOS / firmware row (`slot`).
    BiosOtherSlot { slot: Slot },
    /// EmuDome's copy of a BIOS / firmware file could not be written or deleted.
    BiosFileFailed { detail: String },
    /// DSi mode was asked for without the DS and DSi system files: the game runs as a DS.
    BiosDsiFilesMissing,
    /// Booting through the DS menu was asked for without the DS system files: the game starts
    /// directly.
    BiosDsMenuFilesMissing,
    /// Drive sync does not exist in this build (it was compiled without the Google OAuth client
    /// id, a build-time value: `emudome-sync`).
    CloudNotConfigured,
    /// No Google account is signed in on this device.
    CloudNotSignedIn,
    /// The Google sign-in did not finish (refused, the browser closed, time ran out, Google
    /// refused the code); `detail` says which.
    CloudSignInFailed { detail: String },
    /// Signing in to Google is not possible on this system yet (Android: Phase 4).
    CloudSignInUnsupported,
    /// The Google sign-in was revoked or expired: only signing in again helps.
    CloudAccessExpired,
    /// Google Drive could not be reached (no network, a timeout): nothing changed, the sync is
    /// tried again later.
    CloudOffline { detail: String },
    /// Google Drive has no room for `needed` bytes (`available` left, 0 when Drive only said it
    /// is full).
    CloudQuotaFull { needed: u64, available: u64 },
    /// Google Drive answered with an error none of the other codes names.
    CloudRequestFailed { detail: String },
    /// The sync index (`<app data>/sync/index.json`) is there but cannot be read: sync stops
    /// and the file is not written over until it reads again.
    CloudIndexReadFailed { detail: String },
    /// The system's credential store refused (Linux without a Secret Service, a locked
    /// keyring): the Google sign-in cannot be kept on this device.
    CloudCredentialsUnavailable { detail: String },
    /// Health (`emu://health`, `diagnostics`): the native video could not start, the game is
    /// drawn in the interface's canvas instead.
    NativeVideoFallback { detail: String },
    /// Health: a game whose core draws on the GPU (3DS) found no Vulkan device to render on
    /// (another graphics API, no Vulkan driver): it runs with the core's software renderer.
    HwRenderFallback { detail: String },
    /// Health: the graphics API chosen in `video.api` (`api`: the option id, `vulkan`, `d3d12`,
    /// `opengl`) has no GPU on this system: EmuDome chose one by itself for this run.
    GraphicsApiUnavailable { api: String },
    /// Health: the surface the system handed over (Android's) cannot be drawn on; nothing is
    /// shown until the next one.
    VideoSurfaceFailed { detail: String },
    /// Health: no audio output (no device, the device refused the stream, it went away);
    /// EmuDome keeps trying and clears this once a stream plays.
    AudioUnavailable { detail: String },
    /// Health: the controllers cannot be used in this run (SDL did not start).
    ControllersUnavailable { detail: String },
    /// Health: the last game on `core` (`azahar`) could not reach its save folder and wrote into
    /// the core's own user folder (`folder`) instead, shared by every game of that core: the
    /// progress of that session is there, not with the game's saves (Trello #97). Seen by
    /// comparing that folder before and after the session; EmuDome never writes or deletes there.
    CoreSavedOutsideGame { core: String, folder: String },
    /// Health: a part of EmuDome (`thread`: `video`, `audio`, `controllers`) stopped after an
    /// internal error (a panic caught in a build that unwinds; a release build aborts instead).
    ThreadStopped { thread: String },
    /// This build of EmuDome cannot update itself: it was built without the update key (a local
    /// or development build), or it runs where updates come from the store (Android, Google
    /// Play). Trello #44.
    UpdateUnavailable,
    /// Looking for a new version failed (`detail`: offline, the release server's answer).
    UpdateCheckFailed { detail: String },
    /// Downloading the new version failed, or its signature did not match (`detail`): nothing
    /// was installed.
    UpdateDownloadFailed { detail: String },
    /// The installer of the new version could not be started (`detail`): EmuDome stays as it is.
    UpdateInstallFailed { detail: String },
    /// «Riavvia e aggiorna» with no new version found yet.
    UpdateNotReady,
    /// «Riavvia e aggiorna» while a game is open: the game is closed first (its saves), never
    /// cut by the restart.
    UpdateGameOpen,
    /// No log file to export: logging to a file could not start in this run (the data folder).
    LogUnavailable,
    /// The log could not be written where the user chose (`detail`).
    LogExportFailed { detail: String },
    /// A licence text shipped with EmuDome could not be read (`detail`): the installation is
    /// incomplete, or this is a development build without the packaged files.
    LicenceUnreadable { detail: String },
    /// «Togli dal telefono» on a game EmuDome may not remove: its file is not in EmuDome's own
    /// games folder (a file of the user's), or not on Google Drive (it could not come back).
    StorageGameNotRemovable,
    /// A deletion of «Gestisci lo spazio» stopped (`detail`); what went before it is gone, the
    /// rest is as it was.
    StorageDeleteFailed { detail: String },
    /// «Apri la cartella»: the game's folder could not be shown (`detail`).
    StorageFolderUnavailable { detail: String },
}

impl crate::StartError {
    /// The code the interface shows for a game that did not start on `core` (its name, e.g.
    /// `mgba`).
    pub fn into_code(self, core: &str) -> ErrorCode {
        match self {
            crate::StartError::CoreNeedsRestart => ErrorCode::CoreNeedsRestart {
                core: core.to_owned(),
            },
            crate::StartError::CoreUnloadable { detail }
            | crate::StartError::CoreIncompatible { detail } => {
                ErrorCode::CoreIncompatible { detail }
            }
            crate::StartError::RomUnreadable { path, cause, .. } => {
                ErrorCode::rom_unreadable(&path, cause)
            }
            crate::StartError::CoreRefusedGame => ErrorCode::CoreRefusedGame,
            crate::StartError::GameNeedsPath => ErrorCode::DocumentNeedsPath {
                core: core.to_owned(),
            },
            crate::StartError::ThreadFailed { detail } => ErrorCode::OpenFailed { detail },
            crate::StartError::SaveFolderUnavailable { folder, cause, .. } => {
                ErrorCode::SaveFolderUnavailable {
                    folder: folder.display().to_string(),
                    cause,
                }
            }
        }
    }
}

impl From<crate::save_path::SavePathTooLong> for ErrorCode {
    fn from(e: crate::save_path::SavePathTooLong) -> Self {
        ErrorCode::SavePathTooLong {
            folder: e.save_dir.display().to_string(),
            length: e.length,
            limit: e.limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, IoCause};

    #[test]
    fn a_save_folder_too_deep_says_where_and_by_how_much() {
        let e = crate::save_path::SavePathTooLong {
            save_dir: "C:/deep/saves/3ds/00".into(),
            path: "C:/deep/saves/3ds/00/Azahar/x".into(),
            length: 260,
            limit: 259,
        };
        assert_eq!(
            serde_json::to_value(ErrorCode::from(e)).unwrap(),
            serde_json::json!({ "code": "savePathTooLong",
                "params": { "folder": "C:/deep/saves/3ds/00", "length": 260, "limit": 259 } })
        );
    }

    #[test]
    fn a_start_error_becomes_a_code_the_interface_can_explain() {
        use crate::StartError;
        let code = |e: StartError| e.into_code("mgba");
        assert_eq!(
            code(StartError::CoreNeedsRestart),
            ErrorCode::CoreNeedsRestart {
                core: "mgba".into()
            }
        );
        assert_eq!(
            code(StartError::CoreUnloadable { detail: "x".into() }),
            ErrorCode::CoreIncompatible { detail: "x".into() }
        );
        // The file and its path as values of their own, the OS's text left out (Trello #128).
        assert_eq!(
            serde_json::to_value(code(StartError::RomUnreadable {
                path: "D:/games/Kirby (USA).gba".into(),
                cause: IoCause::NotFound,
                detail: "The system cannot find the file specified. (os error 2)".into(),
            }))
            .unwrap(),
            serde_json::json!({ "code": "romUnreadable", "params": {
                "file": "Kirby (USA).gba", "path": "D:/games/Kirby (USA).gba", "cause": "notFound" } })
        );
        assert_eq!(
            code(StartError::CoreRefusedGame),
            ErrorCode::CoreRefusedGame
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::CoreRefusedGame).unwrap(),
            serde_json::json!({ "code": "coreRefusedGame" })
        );
        assert_eq!(
            serde_json::to_value(code(StartError::SaveFolderUnavailable {
                folder: "saves/3ds/00".into(),
                cause: IoCause::PermissionDenied,
                detail: "denied".into(),
            }))
            .unwrap(),
            serde_json::json!({ "code": "saveFolderUnavailable",
                "params": { "folder": "saves/3ds/00", "cause": "permissionDenied" } })
        );
    }

    #[test]
    fn an_io_error_becomes_its_cause_and_a_new_kind_is_other() {
        use std::io::{Error, ErrorKind};
        assert_eq!(
            IoCause::of(&Error::from(ErrorKind::NotFound)),
            IoCause::NotFound
        );
        assert_eq!(
            IoCause::of(&Error::from(ErrorKind::PermissionDenied)),
            IoCause::PermissionDenied
        );
        assert_eq!(
            IoCause::of(&Error::from(ErrorKind::UnexpectedEof)),
            IoCause::Other
        );
        assert_eq!(IoCause::of(&Error::other("x")), IoCause::Other);
        // A path with no file name keeps the whole path as its name.
        assert_eq!(
            ErrorCode::rom_unreadable(std::path::Path::new("/"), IoCause::Other),
            ErrorCode::RomUnreadable {
                file: "/".into(),
                path: "/".into(),
                cause: IoCause::Other
            }
        );
    }

    #[test]
    fn serializes_as_code_and_params() {
        let json = serde_json::to_value(ErrorCode::SlotEmpty { slot: 2 }).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "code": "slotEmpty", "params": { "slot": 2 } })
        );
        let json = serde_json::to_value(ErrorCode::NoGame).unwrap();
        assert_eq!(json, serde_json::json!({ "code": "noGame" }));
        let json = serde_json::to_value(ErrorCode::SaveUnreadable {
            game: crate::GameKey::parse("gba:1a").unwrap(),
            aside: "broken-1.sav".into(),
            backup_at: Some(5),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "code": "saveUnreadable",
                "params": { "game": "gba:1a", "aside": "broken-1.sav", "backupAt": 5 } })
        );
    }
}
