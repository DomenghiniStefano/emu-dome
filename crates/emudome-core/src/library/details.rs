//! What Game details › «File e informazioni» (L10) derives from a library game: the region its
//! game code names, and the core that runs its console.

use serde::Serialize;
use ts_rs::TS;

use crate::settings::ConsoleId;

/// Where a game was sold, from its game code. TypeScript: `GameRegion` (generated); the
/// interface names it with `details.region.<id>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "GameRegion")]
pub enum Region {
    Japan,
    Usa,
    Europe,
    Germany,
    France,
    Italy,
    Spain,
    Netherlands,
    Australia,
    Korea,
    China,
    World,
}

/// The region a game code names (GBA and NDS 4-letter codes, 3DS product codes such as
/// `CTR-P-ABCP`): its last letter, as Nintendo assigns them. `None` without a code (Game Boy,
/// homebrew) or for a letter with no single region.
pub fn region(code: Option<&str>) -> Option<Region> {
    let last = code?.trim().chars().last()?.to_ascii_uppercase();
    Some(match last {
        'J' => Region::Japan,
        'E' => Region::Usa,
        'P' | 'X' | 'Y' | 'Z' | 'V' => Region::Europe,
        'D' => Region::Germany,
        'F' => Region::France,
        'I' => Region::Italy,
        'S' => Region::Spain,
        'H' => Region::Netherlands,
        'U' => Region::Australia,
        'K' => Region::Korea,
        'C' => Region::China,
        'A' | 'O' => Region::World,
        _ => return None,
    })
}

/// The core that runs a console's games (its libretro file name without the suffix):
/// `mgba`, `melondsds`, `azahar`. The interface names it with `details.files.core.<id>`.
pub fn core_of(console: ConsoleId) -> &'static str {
    crate::core_profile::CoreId::for_console(console).name()
}

/// The roadmap phase a console's games open in, when that is not now (`consoleNotYet`); `None`:
/// they open today.
pub fn console_phase_pending(console: ConsoleId) -> Option<u8> {
    match console {
        // 3DS opens since #46 (Azahar on Vulkan, software where there is none).
        ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba | ConsoleId::Nds | ConsoleId::N3ds => None,
    }
}

/// Whether a file with extension `ext` opens today: its console's games open, and its core runs
/// that kind of file (`CoreProfile::extensions`: Azahar's libretro core runs no `.cia`).
pub fn opens(ext: &str) -> bool {
    super::identity::console_of_extension(ext).is_some_and(|c| {
        console_phase_pending(c).is_none()
            && crate::core_profile::CoreId::for_console(c)
                .profile()
                .extensions
                .contains(&ext.to_lowercase().as_str())
    })
}

/// The file extensions «Apri un file» offers: those of [`super::EXTENSIONS`] that open today
/// ([`opens`]). The one list the file picker uses (`open_extensions`).
pub fn open_extensions() -> Vec<&'static str> {
    super::EXTENSIONS
        .iter()
        .copied()
        .filter(|ext| opens(ext))
        .collect()
}
