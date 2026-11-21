//! Errors shown to the user, as codes with parameters and never as sentences.
//!
//! Serialized as `{ "code": "slotEmpty", "params": { "slot": 2 } }` (`params` is absent
//! for codes without parameters). The interface turns each code into text through i18n
//! (`errors.<code>` in `app/src/i18n/*.json`); the TypeScript type is generated from this enum
//! into `app/src/generated/ErrorCode.ts`, with the list of codes in `error-codes.json`, which a
//! Vitest test holds against both message files (`crate::contract`). `detail` parameters carry technical text (English, from the OS or a
//! library) that the interface may show as is, never a sentence meant for the user.

use serde::Serialize;
use ts_rs::TS;

use crate::bios::Slot;
use crate::settings::Level;
use crate::GameKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "code", content = "params", rename_all = "camelCase")]
pub enum ErrorCode {
    /// The file extension belongs to no supported console.
    UnsupportedFormat { ext: String },
    /// The console is planned for a later phase of the roadmap.
    ConsoleNotYet { console: String, phase: u8 },
    /// The libretro core file was not found in any search directory.
    CoreNotFound { file: String },
    /// The core file found is not the one this build ships (its sha256 differs from
    /// `cores.lock`'s): it is not loaded. The installation is damaged or was tampered with.
    CoreDamaged { file: String },
    /// The core or the game failed to load, for a reason none of the codes below names.
    OpenFailed { detail: String },
    /// The core file cannot be loaded or is not a libretro core EmuDome can drive (damaged,
    /// another architecture, a function missing): reinstalling EmuDome puts the right one back.
    CoreIncompatible { detail: String },
    /// The game file cannot be read (moved, deleted, a disconnected drive, no permission).
    RomUnreadable { detail: String },
    /// The core read the game and refused it: a bad or incomplete dump, an encrypted 3DS game,
    /// a format this core does not run.
    CoreRefusedGame,
    /// The file opened has the extension of a console but its content is not a game of it (its
    /// header does not match): it has no identity, so it has no place for its saves.
    GameUnidentified { detail: String },
    /// E10: the 3DS game is encrypted; EmuDome plays decrypted games only (it never decrypts,
    /// and never adds an encrypted game to the library).
    GameEncrypted,
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
    /// The screenshot could not be encoded or written (no frame yet, folder not writable).
    ScreenshotFailed { detail: String },
    /// The screen effect `name` (a built-in one or the user's `.slangp` file name) could not
    /// be loaded or drawn; the game shows without it.
    EffectNotLoaded { name: String, detail: String },
    /// The chosen BIOS / firmware file cannot be read.
    BiosUnreadable { detail: String },
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
    /// Health (`emu://health`, `diagnostics`): the native video could not start, the game is
    /// drawn in the interface's canvas instead.
    NativeVideoFallback { detail: String },
    /// Health: a game whose core draws on the GPU (3DS) found no Vulkan device to render on
    /// (another graphics API, no Vulkan driver): it runs with the core's software renderer.
    HwRenderFallback { detail: String },
    /// Health: the surface the system handed over (Android's) cannot be drawn on; nothing is
    /// shown until the next one.
    VideoSurfaceFailed { detail: String },
    /// Health: no audio output (no device, the device refused the stream, it went away);
    /// EmuDome keeps trying and clears this once a stream plays.
    AudioUnavailable { detail: String },
    /// Health: the controllers cannot be used in this run (SDL did not start).
    ControllersUnavailable { detail: String },
    /// Health: a part of EmuDome (`thread`: `video`, `audio`, `controllers`) stopped after an
    /// internal error (a panic caught in a build that unwinds; a release build aborts instead).
    ThreadStopped { thread: String },
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
            crate::StartError::RomUnreadable { detail } => ErrorCode::RomUnreadable { detail },
            crate::StartError::CoreRefusedGame => ErrorCode::CoreRefusedGame,
            crate::StartError::ThreadFailed { detail } => ErrorCode::OpenFailed { detail },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorCode;

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
        assert_eq!(
            code(StartError::RomUnreadable {
                detail: "gone".into()
            }),
            ErrorCode::RomUnreadable {
                detail: "gone".into()
            }
        );
        assert_eq!(
            code(StartError::CoreRefusedGame),
            ErrorCode::CoreRefusedGame
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::CoreRefusedGame).unwrap(),
            serde_json::json!({ "code": "coreRefusedGame" })
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
