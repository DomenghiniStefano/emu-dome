//! What Game details › «File e informazioni» (L10) derives from a library game: the region its
//! game code names, and the core that runs its console.

use serde::Serialize;

use crate::settings::ConsoleId;

/// Where a game was sold, from its game code. Mirror: `GameRegion` in app/src/emu.ts; the
/// interface names it with `details.region.<id>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
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
