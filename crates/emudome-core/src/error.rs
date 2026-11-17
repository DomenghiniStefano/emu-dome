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
    /// The setting cannot be stored at this level (e.g. a global-only one for a single game).
    SettingLevelRefused { key: String, level: Level },
    /// The value does not fit the setting (wrong type, unknown option, out of range), or the
    /// setting holds no value (an action).
    SettingInvalidValue { key: String },
    /// A settings file could not be written.
    SettingsWriteFailed { detail: String },
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
    }
}
