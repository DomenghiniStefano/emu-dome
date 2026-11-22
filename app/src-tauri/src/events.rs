//! The names of every event the shell sends to the interface, in one place. The listeners are in
//! `app/src/emu.ts`, the only file that names them there.
//!
//! Naming: `<domain>://<event>`, the event in kebab case saying what happened. `emu://` is the
//! emulator itself (the open game, its input, video, controllers and health); `library://`,
//! `settings://` and `saves://` are those features' own. (`emu://` is also the scheme of the
//! resources `protocol.rs` serves: frames, covers and thumbnails are fetched, never emitted.)

/// The open game's session events (`emudome_core::Event`). Was `emu://event` before #81.
pub const SESSION: &str = "emu://session";
/// Closing a game, the window or the app: `close_flow::Progress`.
pub const CLOSING: &str = "emu://closing";
/// Open / close the quick menu (Guide, L3 + Start).
pub const MENU: &str = "emu://menu";
/// One focus step in the interface from a controller (`hotkeys::NavEvent`).
pub const NAV: &str = "emu://nav";
/// What a shortcut changed (`hotkeys::HotkeyEvent`).
pub const HOTKEY: &str = "emu://hotkey";
/// The open game's screens changed: layout, swap, lid (`emudome_core::screens::ScreenStatus`).
pub const SCREENS: &str = "emu://screens";
/// Settings waiting for the game to be opened again (`core_settings::CoreRestart`).
pub const CORE_RESTART: &str = "emu://core-restart";
/// A game opened without the system files a setting asked for (an `ErrorCode`).
pub const BIOS_FALLBACK: &str = "emu://bios-fallback";
/// A screen effect could not be used (an `ErrorCode`).
pub const EFFECT_ERROR: &str = "emu://effect-error";
/// What is not working (the whole list of `ErrorCode`s).
pub const HEALTH: &str = "emu://health";
/// A controller connected / disconnected / its battery changed (`pads::PadInfo`).
#[allow(dead_code, reason = "sent by gamepad.rs, desktop only (SDL3)")]
pub const GAMEPAD_CONNECTED: &str = "emu://gamepad-connected";
#[allow(dead_code, reason = "sent by gamepad.rs, desktop only (SDL3)")]
pub const GAMEPAD_DISCONNECTED: &str = "emu://gamepad-disconnected";
#[allow(dead_code, reason = "sent by gamepad.rs, desktop only (SDL3)")]
pub const GAMEPAD_CHANGED: &str = "emu://gamepad-changed";
/// The controllers' live state while the interface watches (`pads::PadStates`). Was
/// `emu://pad-state` before #81.
#[allow(dead_code, reason = "sent by gamepad.rs, desktop only (SDL3)")]
pub const GAMEPAD_STATE: &str = "emu://gamepad-state";
/// A setting changed (`settings::Changed`).
pub const SETTINGS_CHANGED: &str = "settings://changed";
/// What changed in the library (`library::LibraryChanged`).
pub const LIBRARY_CHANGED: &str = "library://changed";
/// A library scan's progress (`library::ScanEvent`).
pub const LIBRARY_SCAN: &str = "library://scan";
/// The one-time move of old save names ended (`migration::Status`).
pub const SAVES_MIGRATION: &str = "saves://migration";
/// The notifications changed (X03): the whole list, newest first (`notifications::Notification`).
pub const NOTIFICATIONS_CHANGED: &str = "notifications://changed";
