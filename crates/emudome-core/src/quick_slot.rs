//! The quick slot of each game (Trello #110): the slot the quick save and quick load shortcuts
//! (L3 + R1 / L1, F1 / F4) and the phone quick menu's «Salva rapido» / «Carica rapido» use,
//! chosen by L3 + d-pad left / right, F2 / F3 and «Rendi rapido».
//!
//! Kept in the game's own saves folder ([`GameFiles::quick_slot`], `quick-slot.json`,
//! `{ "$schema": 1, "slot": n }`), so Drive sync carries it with the saves. The shell reads it
//! when the game opens, off the emulation thread ([`restore`]); the session writes it through
//! its I/O worker at every change the user makes ([`write`], `Command::SetQuickSlot`). A file
//! that cannot be read gives the first slot and is left as it is: only a change the user makes
//! writes it again (engineering rule 1).

use std::io;

use serde::{Deserialize, Serialize};

use crate::core::GameFiles;
use crate::hotkeys::{FIRST_SLOT, LAST_SLOT};
use crate::write_atomic;

/// The layout of `quick-slot.json` this version writes and reads.
const SCHEMA: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Stored {
    #[serde(rename = "$schema")]
    schema: u32,
    slot: u8,
}

/// `slot` within the quick slots ([`FIRST_SLOT`]..=[`LAST_SLOT`]): below them the first,
/// above them the last.
pub fn clamp(slot: u8) -> u8 {
    slot.clamp(FIRST_SLOT, LAST_SLOT)
}

/// What a game's `quick-slot.json` says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// No file: the game never had its quick slot moved.
    Missing,
    /// The slot, clamped ([`clamp`]).
    Slot(u8),
    /// There but unreadable (an I/O error, not JSON, a newer layout): why.
    Unreadable(String),
}

/// Reads a game's quick slot file.
pub fn read(files: &GameFiles) -> Found {
    let path = files.quick_slot();
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Found::Missing,
        Err(e) => return Found::Unreadable(e.to_string()),
    };
    match serde_json::from_slice::<Stored>(&bytes) {
        Ok(s) if s.schema <= SCHEMA => Found::Slot(clamp(s.slot)),
        Ok(s) => Found::Unreadable(format!("layout {} is newer than {SCHEMA}", s.schema)),
        Err(e) => Found::Unreadable(e.to_string()),
    }
}

/// The slot a game opens on: its own, else the first. An unreadable file is logged and left as
/// it is.
pub fn restore(files: &GameFiles) -> u8 {
    match read(files) {
        Found::Slot(slot) => slot,
        Found::Missing => FIRST_SLOT,
        Found::Unreadable(why) => {
            log::warn!(
                "quick slot of {} unreadable, slot {FIRST_SLOT} used, file left as it is: {why}",
                files.dir().display()
            );
            FIRST_SLOT
        }
    }
}

/// Records `slot` (clamped) as the game's quick slot, atomically.
pub fn write(files: &GameFiles, slot: u8) -> io::Result<()> {
    let bytes = serde_json::to_vec(&Stored {
        schema: SCHEMA,
        slot: clamp(slot),
    })
    .map_err(io::Error::other)?;
    write_atomic(&files.quick_slot(), &bytes)
}

#[cfg(test)]
mod tests {
    use super::{clamp, read, restore, write, Found};
    use crate::core::GameFiles;
    use crate::hotkeys::{FIRST_SLOT, LAST_SLOT};
    use crate::settings::test_dir::TestDir;
    use crate::GameKey;

    fn files(dir: &TestDir) -> GameFiles {
        GameFiles::new(&dir.path("saves"), &GameKey::parse("gba:1a").unwrap())
    }

    #[test]
    fn a_slot_out_of_range_is_clamped() {
        assert_eq!(clamp(0), FIRST_SLOT);
        assert_eq!(clamp(4), 4);
        assert_eq!(clamp(200), LAST_SLOT);
    }

    #[test]
    fn a_written_slot_is_read_back_and_a_game_without_one_opens_on_the_first() {
        let dir = TestDir::new("quick-slot-roundtrip");
        let files = files(&dir);
        assert_eq!(read(&files), Found::Missing);
        assert_eq!(restore(&files), FIRST_SLOT);
        write(&files, 5).unwrap();
        assert_eq!(read(&files), Found::Slot(5));
        assert_eq!(restore(&files), 5);
        assert_eq!(
            dir.read("saves/gba/1a/quick-slot.json"),
            r#"{"$schema":1,"slot":5}"#
        );
        // Written out of range: clamped on the way in; read out of range: clamped on the way out.
        write(&files, 42).unwrap();
        assert_eq!(read(&files), Found::Slot(LAST_SLOT));
        dir.write("saves/gba/1a/quick-slot.json", r#"{"$schema":1,"slot":0}"#);
        assert_eq!(restore(&files), FIRST_SLOT);
    }

    #[test]
    fn an_unreadable_file_gives_the_first_slot_and_is_left_as_it_is() {
        let dir = TestDir::new("quick-slot-unreadable");
        let files = files(&dir);
        for text in ["not json", r#"{"$schema":2,"slot":3}"#, r#"{"slot":"x"}"#] {
            dir.write("saves/gba/1a/quick-slot.json", text);
            assert!(matches!(read(&files), Found::Unreadable(_)), "{text}");
            assert_eq!(restore(&files), FIRST_SLOT);
            assert_eq!(dir.read("saves/gba/1a/quick-slot.json"), text);
        }
    }
}
