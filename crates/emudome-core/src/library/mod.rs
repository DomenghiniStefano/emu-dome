//! The game library: which games are on this device, what they are, how long they were played.
//!
//! - `identity.rs`: console, identity from the content (`<console>:<hex>`), title and icon of a
//!   ROM file (docs/feasibility.md, "Game identity").
//! - `scan.rs`: walks the library folders, identifying files (cached by path + size + mtime).
//! - `store.rs`: the database (`library.json`), the merge of a scan, flags, covers, play time.
//!
//! Pure logic and file I/O only: the Tauri commands are in `app/src-tauri/src/library.rs`.

pub mod identity;
pub mod scan;
mod store;

pub use identity::{identify, read_icon, Icon, Identity, NotAGame, EXTENSIONS};
pub use scan::{scan, Found, ScanInput, ScanOutcome};
pub use store::{cover_colour, CacheEntry, CoverKind, GameRecord, HashCache, Library, ScanReport};

use crate::settings::GameId;

/// The settings identity of a library game (`gba:1a2b…` → `gba-1a2b…`): its «Solo questo
/// gioco» file is named after the content, not the path.
pub fn settings_game_id(id: &str) -> Option<GameId> {
    GameId::try_from(id.replace(':', "-")).ok()
}

#[cfg(test)]
mod tests;
