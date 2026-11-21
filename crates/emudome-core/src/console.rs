//! The consoles EmuDome runs: a game's console comes from its content (`library::identity`), and
//! each console is a settings level (`settings::Scope::Console`) and a folder of saves.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// Also reachable as `settings::ConsoleId`, where it was first declared; its doc line below is the
// generated TypeScript's, kept as it was.
/// A console, as a settings level. TypeScript: `ConsoleId` (generated).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
pub enum ConsoleId {
    #[serde(rename = "gb")]
    Gb,
    #[serde(rename = "gbc")]
    Gbc,
    #[serde(rename = "gba")]
    Gba,
    #[serde(rename = "nds")]
    Nds,
    #[serde(rename = "3ds")]
    N3ds,
}

impl ConsoleId {
    /// Every console, in the order the interface lists them.
    pub const ALL: [ConsoleId; 5] = [
        ConsoleId::Gb,
        ConsoleId::Gbc,
        ConsoleId::Gba,
        ConsoleId::Nds,
        ConsoleId::N3ds,
    ];

    /// The id in file names and in the interface.
    pub fn id(self) -> &'static str {
        match self {
            ConsoleId::Gb => "gb",
            ConsoleId::Gbc => "gbc",
            ConsoleId::Gba => "gba",
            ConsoleId::Nds => "nds",
            ConsoleId::N3ds => "3ds",
        }
    }

    /// The console of an id ([`ConsoleId::id`]).
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.id() == id)
    }

    /// The console's short name as the interface shows it beside a game (`GameInfo.console`): a
    /// .gb file whose header asks for a Game Boy Color is «GBC».
    pub fn short_name(self) -> &'static str {
        match self {
            ConsoleId::Gb => "GB",
            ConsoleId::Gbc => "GBC",
            ConsoleId::Gba => "GBA",
            ConsoleId::Nds => "NDS",
            ConsoleId::N3ds => "3DS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleId;

    #[test]
    fn every_console_goes_back_from_its_id() {
        for console in ConsoleId::ALL {
            assert_eq!(ConsoleId::from_id(console.id()), Some(console));
            let json = serde_json::to_string(&console).unwrap();
            assert_eq!(json, format!("\"{}\"", console.id()));
        }
        assert_eq!(ConsoleId::from_id("n64"), None);
    }
}
