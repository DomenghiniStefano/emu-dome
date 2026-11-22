//! The game library: which games are on this device, what they are, how long they were played.
//!
//! - `identity.rs`: console, identity from the content (the game's [`crate::GameKey`]), title
//!   and icon of a ROM file (docs/feasibility.md, "Game identity").
//! - `scan.rs`: walks the library folders, identifying files (cached by path + size + mtime), and
//!   the picked document trees of Android (cached by document URI + size + mtime).
//! - `store.rs`: the database (`library.json`), the merge of a scan, flags, covers, play time,
//!   the user's name for a game.
//! - `details.rs`: what Game details › File e informazioni (L10) derives: region, core.
//! - `group.rs`: the regional versions of one game as a family, the version shown (#102).
//!
//! Pure logic and file I/O only: the Tauri commands are in `app/src-tauri/src/library.rs`.

mod details;
pub mod group;
pub mod identity;
pub mod scan;
mod store;

pub use details::{console_phase_pending, core_of, open_extensions, opens, region, Region};
pub use group::Family;
pub use identity::{
    console_of_extension, identify, identify_file, read_icon, read_icon_file, Icon, Identity,
    NotAGame, EXTENSIONS,
};
pub use scan::{
    identify_cached, identify_open_file, scan, scan_with_documents, DocEntry, Documents, Found,
    OpenedDocument, ScanInput, ScanOutcome,
};
pub use store::{
    cover_colour, CacheEntry, CoverKind, GameRecord, HashCache, Library, ScanReport, NAME_MAX,
};

#[cfg(test)]
mod tests;
