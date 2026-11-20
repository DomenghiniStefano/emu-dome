//! The one identity of a game (engineering rule 2): the library's content id, written
//! `<console>:<hex>` (docs/decisions.md, «Game identity»; how the hex is made:
//! [`crate::library::identity`]). Everything stored per game is keyed by it, never by a path or
//! a file name: saves and quick saves ([`crate::GameFiles`]), «Solo questo gioco» settings, the
//! library, covers, play time; the interface names games by it too (`GameId` in app/src/emu.ts).
//!
//! The same game has the same key wherever its file is and whatever it is called, so renaming or
//! moving a ROM keeps its saves; two different ROMs with the same file name (`Tetris.gb` and
//! `Tetris.gba`, two hacks) have two keys and never share a file.
//!
//! As text: `gba:1a2b…` (the library, the interface); in file names `gba-1a2b…`
//! ([`GameKey::file_stem`], since `:` is not allowed in Windows file names); as a folder
//! `<console>/<hex>` ([`GameKey::dir`]). Only lowercase hex digits after the console, so every
//! form is a safe file name, also when the key comes from the interface.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::settings::ConsoleId;

/// Longest hex part accepted (SHA-1 is 40, a 3DS title id + version 20).
const MAX_HEX: usize = 64;

/// A game's identity from its content. Mirror: `GameId` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GameKey {
    console: ConsoleId,
    /// `<console>:<hex>`, kept whole so it can be lent as a `&str`.
    text: String,
}

impl GameKey {
    /// The key of `console` with this hex part (lowercase hex digits only).
    pub fn new(console: ConsoleId, hex: &str) -> Result<Self, String> {
        let valid = !hex.is_empty()
            && hex.len() <= MAX_HEX
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if valid {
            Ok(Self {
                console,
                text: format!("{}:{hex}", console.id()),
            })
        } else {
            Err(format!("invalid game key hex {hex:?}"))
        }
    }

    /// Reads `<console>:<hex>`.
    pub fn parse(text: &str) -> Result<Self, String> {
        Self::split(text, ':')
    }

    /// Reads a file name stem `<console>-<hex>` ([`GameKey::file_stem`]); `None` for anything
    /// else (a settings file named after an old path hash, `path-…`, is not a key).
    pub fn from_file_stem(stem: &str) -> Option<Self> {
        Self::split(stem, '-').ok()
    }

    fn split(text: &str, sep: char) -> Result<Self, String> {
        let (console, hex) = text
            .split_once(sep)
            .ok_or_else(|| format!("invalid game key {text:?}"))?;
        let console =
            ConsoleId::from_id(console).ok_or_else(|| format!("invalid game key {text:?}"))?;
        Self::new(console, hex)
    }

    pub fn console(&self) -> ConsoleId {
        self.console
    }

    /// The hex part, after `<console>:`.
    pub fn hex(&self) -> &str {
        &self.text[self.console.id().len() + 1..]
    }

    /// `<console>:<hex>`.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// `<console>-<hex>`: the key in a file name (settings `game/<stem>.json`, covers, URLs).
    pub fn file_stem(&self) -> String {
        format!("{}-{}", self.console.id(), self.hex())
    }

    /// `<console>/<hex>`: the game's own folder under a root (its saves: [`crate::GameFiles`]).
    pub fn dir(&self) -> PathBuf {
        PathBuf::from(self.console.id()).join(self.hex())
    }
}

impl TryFrom<String> for GameKey {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<GameKey> for String {
    fn from(key: GameKey) -> Self {
        key.text
    }
}

impl fmt::Display for GameKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl PartialEq<str> for GameKey {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}

impl PartialEq<&str> for GameKey {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl PartialEq<String> for GameKey {
    fn eq(&self, other: &String) -> bool {
        self.text == *other
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::GameKey;
    use crate::settings::ConsoleId;

    #[test]
    fn reads_and_writes_every_form() {
        let key = GameKey::parse("gba:1a2b").unwrap();
        assert_eq!(key.console(), ConsoleId::Gba);
        assert_eq!(key.hex(), "1a2b");
        assert_eq!(key.as_str(), "gba:1a2b");
        assert_eq!(key.file_stem(), "gba-1a2b");
        assert_eq!(key.dir(), Path::new("gba").join("1a2b"));
        assert_eq!(GameKey::from_file_stem("gba-1a2b"), Some(key.clone()));
        let three = GameKey::parse("3ds:0004000000abcd000002").unwrap();
        assert_eq!(three.console(), ConsoleId::N3ds);
        assert_eq!(three.file_stem(), "3ds-0004000000abcd000002");
        assert_eq!(
            serde_json::to_value(&key).unwrap(),
            serde_json::json!("gba:1a2b")
        );
        assert_eq!(
            serde_json::from_value::<GameKey>(serde_json::json!("gba:1a2b")).unwrap(),
            key
        );
    }

    #[test]
    fn refuses_anything_that_is_not_a_key() {
        for bad in [
            "",
            "gba",
            "gba:",
            "gba:XYZ",
            "gba:1A2B",
            "psx:1a2b",
            "gba:1a2b/../x",
            "gba:1a2b:3",
            "path-0123abcd",
        ] {
            assert!(GameKey::parse(bad).is_err(), "{bad:?}");
        }
        assert!(GameKey::parse(&format!("gba:{}", "a".repeat(65))).is_err());
        // The old settings identity, a hash of the ROM path, is not a key.
        assert_eq!(GameKey::from_file_stem("path-0123456789abcdef"), None);
        assert!(serde_json::from_value::<GameKey>(serde_json::json!("gba:..")).is_err());
    }
}
