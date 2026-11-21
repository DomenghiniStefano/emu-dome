//! Nintendo 3DS through Azahar (Phase 3, Trello #46). The model is [`crate::nds`]: the core is
//! held to one arrangement the shell knows, and EmuDome composes the screens itself
//! (`docs/decisions.md`, option B).
//!
//! - Renderer: Vulkan on the shell's device ([`crate::hw_render::vulkan`]), else Azahar's software
//!   renderer; chosen by [`crate::core_profile::CoreProfile::hw_render`], not forced here.
//! - Screens: Azahar's `default` layout, not swapped: the top screen above the bottom one,
//!   centred, no gap ([`SPLIT`]). The swap is not left to the core (its L3 swap key never
//!   reaches it: L3 is EmuDome's modifier), the other layouts wait for the screen settings (#28).
//! - Touch: `RETRO_DEVICE_POINTER` from the shell's pointer path ([`crate::screens::pointer_at`]).
//!   Azahar maps the pointer over its whole frame and back to the bottom screen
//!   (`input/mouse_tracker.cpp`), as melonDS DS does. Its cursor is drawn only after the right
//!   stick moved it, and that stick is the C-stick only here, so no cursor shows.
//! - Saves: Azahar keeps the emulated SD card and NAND under the frontend's save directory (the
//!   game's own folder, `GameFiles`). When it cannot reach that folder it falls back to its own
//!   user folder, shared by every game: the session refuses a game folder it cannot create, and
//!   the shell watches Azahar's own folders for what slips through ([`fallback`], Trello #97).
//! - Encrypted games are never added to the library and are refused before the core sees them
//!   ([`rom`], E10, `gameEncrypted`): identification only, nothing is ever decrypted.
//! - System files: EmuDome supplies none (no firmware, no keys, no system archives).
//! - States: on Vulkan, Azahar 2126.2 crashes loading a state of content whose title ID is 0
//!   (every `.3dsx` homebrew): such loads, and rewind, are refused
//!   ([`crate::core_profile::state_load_block`], [`title_id`], Trello #93).

pub mod fallback;
pub mod rom;

use crate::screens::Split;
use crate::settings::ConsoleId;
use crate::GameKey;

/// How the frame holds the screens with [`FORCED_OPTIONS`]: 400 × 480 times the internal
/// resolution, top then bottom.
pub const SPLIT: Split = Split::N3dsStacked;

/// Options the shell always sets for Azahar (the renderer's is the profile's). Keys and values as
/// Azahar 2126.2 declares them (`headless --list-options`, `src/citra_libretro/core_settings.cpp`).
pub const FORCED_OPTIONS: &[(&str, &str)] = &[
    // The frame [`SPLIT`] expects.
    ("citra_layout_option", "default"),
    ("citra_swap_screen", "Top"),
    // Stereoscopic modes draw two pictures in one frame: not the split.
    ("citra_render_3d", "off"),
    // Touch from the shell's pointer (RETRO_DEVICE_POINTER), not from the mouse.
    ("citra_enable_touch_touchscreen", "enabled"),
    ("citra_enable_mouse_touchscreen", "disabled"),
    // The right stick is the C-stick only: no stick-driven pointer, so no cursor in the picture.
    ("citra_analog_function", "c_stick"),
    ("citra_enable_touch_pointer_timeout", "enabled"),
    // The emulated SD card and NAND (the game's saves) in the game's own folder.
    ("citra_use_libretro_save_path", "LibRetro Default"),
];

/// The deepest folder and file Azahar creates under the game's folder while it loads a game
/// (Trello #94): its emulated NAND, written in `retro_load_game`. Past the system's path limit
/// Azahar aborts the whole process there, so the shell checks first ([`crate::save_path`]).
///
/// Measured 2026-11-21 on Azahar 2126.2 (commit 219d743) with Craftus Reloaded 0.3 (`.3dsx`),
/// software renderer: the tree the headless example left under `saves/3ds/<hex>/`. The same
/// tree for every game: the NAND's system data, not the game's. A game's own saves on the
/// emulated SD card (`Azahar/sdmc/...`) are named by the game and created while it runs; they
/// are not covered.
pub const SAVE_DEPTH: crate::save_path::SaveDepth = crate::save_path::SaveDepth {
    deepest_dir:
        "Azahar/nand/data/00000000000000000000000000000000/extdata/00048000/F000000B/boss",
    deepest_file:
        "Azahar/nand/data/00000000000000000000000000000000/extdata/00048000/F000000B/user/gamecoin.dat",
};

/// The title ID Azahar runs `game` with, read from its key ([`crate::library::identity`]): a 3DS
/// title's program ID, the key's first 16 hex digits (the last 4 are its version). 0 for
/// homebrew (`.3dsx`, keyed by the SHA-1 of the file: it has no title ID, and Azahar runs it with
/// 0) and for any key that is not a 3DS title's, so an unknown game counts as homebrew.
pub fn title_id(game: &GameKey) -> u64 {
    let hex = game.hex();
    if game.console() != ConsoleId::N3ds || hex.len() != 20 {
        return 0;
    }
    u64::from_str_radix(&hex[..16], 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{title_id, FORCED_OPTIONS, SPLIT};
    use crate::screens::{ScreenId, Split};
    use crate::GameKey;

    #[test]
    fn the_title_id_is_the_program_id_and_homebrew_has_none() {
        let key = |text: &str| GameKey::parse(text).unwrap();
        // A retail title: program ID 0004000000030800, version 0000.
        assert_eq!(
            title_id(&key("3ds:00040000000308000000")),
            0x0004_0000_0003_0800
        );
        // An NCCH whose program ID is 0.
        assert_eq!(title_id(&key("3ds:00000000000000000010")), 0);
        // `.3dsx` homebrew: the SHA-1 of the file.
        assert_eq!(
            title_id(&key("3ds:da39a3ee5e6b4b0d3255bfef95601890afd80709")),
            0
        );
        // Not a 3DS game.
        assert_eq!(title_id(&key("gba:00040000000308000000")), 0);
    }

    fn get(key: &str) -> Option<&'static str> {
        FORCED_OPTIONS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }

    #[test]
    fn the_forced_layout_is_the_split() {
        assert_eq!(get("citra_layout_option"), Some("default"));
        assert_eq!(get("citra_swap_screen"), Some("Top"));
        assert_eq!(get("citra_render_3d"), Some("off"));
        assert_eq!(SPLIT, Split::N3dsStacked);
        let screens = SPLIT.screens(400, 480);
        assert_eq!(screens[0].id, ScreenId::Top);
        assert_eq!(screens[1].id, ScreenId::Bottom);
    }

    #[test]
    fn every_option_is_set_once_and_the_renderer_is_not_among_them() {
        let mut keys: Vec<_> = FORCED_OPTIONS.iter().map(|(k, _)| *k).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), FORCED_OPTIONS.len());
        assert_eq!(get("citra_graphics_api"), None);
    }
}
