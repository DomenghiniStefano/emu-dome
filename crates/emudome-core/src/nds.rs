//! Nintendo DS through melonDS DS (Phase 2, Trello #45).
//!
//! EmuDome composes the two screens itself (`docs/decisions.md`, option B), so the core is held
//! to one arrangement the shell knows: the top screen above the bottom one, no gap, full size,
//! one layout only (no cycling inside the core, which would move the screens under the shell's
//! feet). [`SPLIT`] is that arrangement as [`crate::screens`] cuts it.
//!
//! Touch reaches the core as `RETRO_DEVICE_POINTER` (melonDS DS's «Pointer» touch mode), mapped
//! by [`crate::screens::pointer_at`]; the core draws no cursor of its own.
//!
//! BIOS and firmware: the core's built-in replacements only (FreeBIOS, generated firmware);
//! nothing is read from the system folder. TODO(bios.*): user-supplied `bios.bds7`, `bios.bds9`,
//! `bios.bdsfw` (and `bios.bdsi` for DSi mode) once the settings' BIOS files are wired for any
//! console (`bios.bgba` is not wired either); then `melonds_sysfile_mode` becomes `native` when
//! they are set, and DSi mode becomes possible.

use crate::screens::Split;

/// The core's buildbot name (`melondsds_libretro.*`).
pub const CORE: &str = "melondsds";

/// How the frame holds the screens with [`FORCED_OPTIONS`]: 256 × 384, top then bottom.
pub const SPLIT: Split = Split::DsStacked { gap: 0 };

/// Core options the shell always sets for melonDS DS. Keys and values as melonDS DS 1.4.0
/// declares them (`src/libretro/config/constants.hpp`).
pub const FORCED_OPTIONS: &[(&str, &str)] = &[
    // The frame [`SPLIT`] expects.
    ("melonds_number_of_screen_layouts", "1"),
    ("melonds_screen_layout1", "top-bottom"),
    ("melonds_screen_gap", "0"),
    ("melonds_secondary_screen_scale", "100"),
    // Touch from the shell's pointer; no cursor drawn into the game's picture.
    ("melonds_touch_mode", "touch"),
    ("melonds_show_cursor", "disabled"),
    // Built-in BIOS and firmware only (see the module's TODO).
    ("melonds_sysfile_mode", "builtin"),
];

#[cfg(test)]
mod tests {
    use super::{FORCED_OPTIONS, SPLIT};

    #[test]
    fn the_forced_layout_is_the_split() {
        let get = |k: &str| {
            FORCED_OPTIONS
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| *v)
        };
        assert_eq!(get("melonds_screen_layout1"), Some("top-bottom"));
        assert_eq!(get("melonds_number_of_screen_layouts"), Some("1"));
        let gap = get("melonds_screen_gap").and_then(|g| g.parse::<u32>().ok());
        assert_eq!(
            Some(SPLIT),
            gap.map(|gap| crate::screens::Split::DsStacked { gap })
        );
    }
}
