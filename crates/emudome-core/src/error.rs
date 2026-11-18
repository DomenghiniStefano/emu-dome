//! Errors shown to the user, as codes with parameters and never as sentences.
//!
//! Serialized as `{ "code": "slotEmpty", "params": { "slot": 2 } }` (`params` is absent
//! for codes without parameters). The interface turns each code into text through i18n
//! (`errors.<code>` in `app/src/i18n/*.json`); the TypeScript mirror is `ErrorCode` in
//! `app/src/emu.ts`. `detail` parameters carry technical text (English, from the OS or a
//! library) that the interface may show as is, never a sentence meant for the user.

use serde::Serialize;

use crate::settings::Level;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", content = "params", rename_all = "camelCase")]
pub enum ErrorCode {
    /// The file extension belongs to no supported console.
    UnsupportedFormat { ext: String },
    /// The console is planned for a later phase of the roadmap.
    ConsoleNotYet { console: String, phase: u8 },
    /// The libretro core file was not found in any search directory.
    CoreNotFound { file: String },
    /// The core or the game failed to load.
    OpenFailed { detail: String },
    /// A command that needs a running game arrived with none open.
    NoGame,
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
    /// A library folder cannot be read now (missing, disconnected drive, no permission).
    LibraryFolderUnreachable { path: String },
    /// No game in the library has this id.
    LibraryGameUnknown { id: String },
    /// The game is in the library but its file cannot be reached now.
    LibraryGameUnavailable { id: String },
    /// The library database could not be written.
    LibraryWriteFailed { detail: String },
    /// The chosen cover cannot be used (not an image, too large, or no icon in the ROM).
    CoverInvalid { detail: String },
    /// «Annulla il caricamento» with no load to undo in this session.
    NothingToUndo,
    /// The core refused the state from before the last load.
    UndoFailed { detail: String },
    /// E04: the in-game save cannot be used (unreadable, empty, too large). It was moved aside
    /// as `aside`; `backup_at` is the Unix ms of the newest backup, if there is one.
    #[serde(rename_all = "camelCase")]
    SaveUnreadable {
        aside: String,
        backup_at: Option<u64>,
    },
    /// The game is open: its saves cannot change under the running session.
    SaveGameOpen,
    /// The chosen file is not a save EmuDome can import.
    SaveImportInvalid { detail: String },
    /// A save file could not be read, written, copied or deleted.
    SaveFileFailed { detail: String },
    /// The screenshot could not be encoded or written (no frame yet, folder not writable).
    ScreenshotFailed { detail: String },
    /// The screen effect `name` (a built-in one or the user's `.slangp` file name) could not
    /// be loaded or drawn; the game shows without it.
    EffectNotLoaded { name: String, detail: String },
}

#[cfg(test)]
mod tests {
    use super::ErrorCode;

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
            aside: "Game.broken-1.sav".into(),
            backup_at: Some(5),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "code": "saveUnreadable",
                "params": { "aside": "Game.broken-1.sav", "backupAt": 5 } })
        );
    }
}
