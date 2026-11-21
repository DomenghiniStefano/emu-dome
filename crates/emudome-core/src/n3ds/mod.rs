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
//!   game's own folder, `GameFiles`), never its own default location.
//! - Encrypted games are never added to the library and are refused before the core sees them
//!   ([`rom`], E10, `gameEncrypted`): identification only, nothing is ever decrypted.
//! - System files: EmuDome supplies none (no firmware, no keys, no system archives).

pub mod rom;

use crate::screens::Split;

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

#[cfg(test)]
mod tests {
    use super::{FORCED_OPTIONS, SPLIT};
    use crate::screens::{ScreenId, Split};

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
