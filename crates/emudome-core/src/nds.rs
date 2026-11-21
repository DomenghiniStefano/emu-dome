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
//! BIOS and firmware: the user's own files when every DS one is in place, else the core's built-in
//! replacements (FreeBIOS, generated firmware); decided by `crate::bios` (`bios/melondsds.rs`),
//! whose options are added after [`FORCED_OPTIONS`].

use crate::screens::Split;
use crate::settings::keys::{self, Reader};
use crate::settings::Value;

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
    // The DS user name comes from the frontend (`GET_USERNAME`, [`username`]); unanswered, the
    // core takes the OS user, which is what an empty `consoles.dsname` means.
    ("melonds_firmware_username", "guess_username"),
];

/// The DS user name for `GET_USERNAME` (`consoles.dsname`): `None` when empty (the core then
/// uses the OS user name). Not a core option: melonDS DS asks the frontend for it. The DS keeps
/// 10 characters; melonDS DS cuts the rest.
pub fn username(value: impl Fn(&str) -> Option<Value>) -> Option<String> {
    let name = Reader::new(value).text(keys::CONSOLES_DS_NAME);
    Some(name.trim().to_owned()).filter(|n| !n.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{username, Value, FORCED_OPTIONS, SPLIT};

    #[test]
    fn the_user_name_is_answered_only_when_set() {
        let name = |v: &str| {
            let v = v.to_owned();
            username(move |key| (key == "consoles.dsname").then(|| Value::Text(v.clone())))
        };
        assert_eq!(name(""), None);
        assert_eq!(name("   "), None);
        assert_eq!(name(" Stefano "), Some("Stefano".into()));
        assert_eq!(username(|_| None), None);
    }

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
