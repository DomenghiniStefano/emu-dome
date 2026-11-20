//! The settings engine: one schema, five levels, files on disk, the quick menu's session.
//!
//! - **Schema** (`schema.rs` types, `catalog.rs` entries): the only description of every setting;
//!   the interface builds its pages from it (`schema()`), never from a hand-written list.
//! - **Levels**, weakest first: Default → Global («Tutti i giochi») → Console («Giochi <console>»)
//!   → Game («Solo questo gioco») → Session (changes «in prova»). Each entry accepts only some of
//!   them; storing at another is refused with [`ErrorCode::SettingLevelRefused`].
//!   [`Settings::resolve`] answers with the value **and the level it comes from**.
//! - **Device axis**: per-device entries are stored in the device file, never in the synced ones
//!   (`store.rs`).
//! - **Session**: bound to the open game; set live, committed to Game or Console, discarded on
//!   close. Never written to disk until committed.
//! - **Recent changes** («Modificate di recente», T01): every write to a level records when, next
//!   to the value (`store.rs`, `$changed`); [`Settings::recent`] lists the newest across levels.
//!   The session records nothing: its changes count when committed.
//!
//! Pure logic and file I/O only: the Tauri commands are in `app/src-tauri/src/settings.rs`.

mod catalog;
pub mod core_values;
pub mod schema;
mod store;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use schema::{
    entry, page_entries, schema, synonyms, ActionStyle, ChoiceStyle, DefaultValue, Device, Entry,
    Kind, Level, Levels, Platform, SchemaDto, Unit, Value,
};
use store::{Change, FileId, Store};

use crate::ErrorCode;

/// A console, as a settings level. Mirror: `ConsoleId` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
}

/// A game's identity for its settings file: lowercase letters, digits and `-` only, so it is
/// always a safe file name (checked when it comes from the interface).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GameId(String);

impl GameId {
    /// For now the identity is a hash of the ROM path.
    /// TODO(game identity, Trello #21): the content hash / title ID once it exists; then move
    /// the files named after the path hash.
    pub fn from_rom_path(path: &Path) -> Self {
        Self(format!(
            "path-{:016x}",
            fnv1a(path.to_string_lossy().as_bytes())
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GameId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let valid = !value.is_empty()
            && value.len() <= 128
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if valid {
            Ok(Self(value))
        } else {
            Err(format!("invalid game id {value:?}"))
        }
    }
}

impl From<GameId> for String {
    fn from(id: GameId) -> Self {
        id.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// FNV-1a, 64 bit: stable across Rust versions and platforms (unlike `DefaultHasher`).
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// What the interface is looking at or changing. Mirror: `SettingsScope` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "camelCase")]
pub enum Scope {
    Global,
    Console {
        console: ConsoleId,
    },
    Game {
        console: ConsoleId,
        game: GameId,
    },
    /// The open game with its changes «in prova».
    Session,
}

/// Where the quick menu's changes are saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommitTarget {
    /// «Salva per questo gioco».
    Game,
    /// «Salva invece per tutti i giochi <console>».
    Console,
}

/// A resolved value and the level it comes from (`value` is `None` for actions).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub value: Option<Value>,
    pub source: Level,
}

/// One row of a settings page. Mirror: `SettingValueRow` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageValue {
    pub key: &'static str,
    pub value: Option<Value>,
    /// The level the value comes from: when weaker than the page's scope, the row shows
    /// «Come in "…"»; when equal, it can be reset.
    pub source: Level,
    /// The value differs from the schema's default.
    pub changed: bool,
}

/// A value stored for one game (Game details L06, «Impostazioni del gioco»), with what every game
/// of its console gets instead. Mirror: `GameOverride` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameOverride {
    pub key: &'static str,
    /// The settings page it is shown on.
    pub page: &'static str,
    pub value: Value,
    /// The value resolved at the console's level: «Per tutti i giochi <console>: …».
    pub base: Option<Value>,
}

/// The quick menu's changes «in prova» for the open game.
struct SessionLayer {
    console: ConsoleId,
    game: GameId,
    values: BTreeMap<&'static str, Value>,
}

/// A value changed recently, at one level («Modificate di recente», T01).
/// Mirror: `SettingRecent` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentChange {
    pub key: &'static str,
    /// Global, Console or Game.
    pub level: Level,
    /// The console of a Console level.
    pub console: Option<ConsoleId>,
    /// The game of a Game level.
    pub game: Option<GameId>,
    /// Unix ms.
    pub at: u64,
}

/// Unix ms now.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// The settings of this device: files plus the open game's session.
pub struct Settings {
    device: Device,
    store: Store,
    session: Option<SessionLayer>,
    /// Unix ms now, for the change records (replaced in the tests).
    clock: fn() -> u64,
}

impl Settings {
    /// `synced_dir`: folder of the files that sync; `device_file`: the per-device file,
    /// outside it.
    pub fn open(synced_dir: PathBuf, device_file: PathBuf, device: Device) -> Self {
        Self {
            device,
            store: Store::new(synced_dir, device_file),
            session: None,
            clock: now_ms,
        }
    }

    pub fn device(&self) -> Device {
        self.device
    }

    /// The schema for this device.
    pub fn schema(&self) -> SchemaDto {
        schema(self.device)
    }

    /// The value of `key` seen from `scope`, with its source level.
    pub fn resolve(&mut self, key: &str, scope: &Scope) -> Result<Resolved, ErrorCode> {
        let entry = known(key)?;
        self.resolve_entry(entry, scope)
    }

    /// Every entry of `page`, seen from `scope`.
    pub fn page(&mut self, page: &str, scope: &Scope) -> Result<Vec<PageValue>, ErrorCode> {
        if !catalog::PAGES.contains(&page) {
            return Err(ErrorCode::SettingUnknown { key: page.into() });
        }
        page_entries(page)
            .map(|entry| {
                let resolved = self.resolve_entry(entry, scope)?;
                Ok(PageValue {
                    key: entry.key,
                    changed: resolved.value != entry.default_value(),
                    value: resolved.value,
                    source: resolved.source,
                })
            })
            .collect()
    }

    /// Stores `value` for `key` at `scope`'s level (in memory only for the session).
    /// Returns the level written, for the `settings://changed` event.
    pub fn set(&mut self, key: &str, scope: &Scope, value: Value) -> Result<Level, ErrorCode> {
        let entry = known(key)?;
        let level = accepted_level(entry, scope)?;
        if !entry.has_value() || !entry.accepts_value_on(&value, self.device) {
            return Err(ErrorCode::SettingInvalidValue { key: key.into() });
        }
        match file_of(scope) {
            Some(file) => {
                let json =
                    serde_json::to_value(&value).map_err(|e| ErrorCode::SettingsWriteFailed {
                        detail: e.to_string(),
                    })?;
                self.store.apply(
                    &file,
                    &[Change {
                        key: entry.key,
                        per_device: entry.per_device,
                        value: Some(json),
                        at: Some((self.clock)()),
                    }],
                )?;
            }
            None => {
                self.session_mut()?.values.insert(entry.key, value);
            }
        }
        Ok(level)
    }

    /// Removes the value of `key` at `scope`'s level, so the weaker levels show through.
    pub fn reset(&mut self, key: &str, scope: &Scope) -> Result<Level, ErrorCode> {
        let entry = known(key)?;
        let level = accepted_level(entry, scope)?;
        match file_of(scope) {
            Some(file) => self.store.apply(
                &file,
                &[Change {
                    key: entry.key,
                    per_device: entry.per_device,
                    value: None,
                    at: Some((self.clock)()),
                }],
            )?,
            None => {
                self.session_mut()?.values.remove(entry.key);
            }
        }
        Ok(level)
    }

    /// A game opened: its session starts empty (any previous one is discarded).
    pub fn begin_session(&mut self, console: ConsoleId, game: GameId) {
        self.session = Some(SessionLayer {
            console,
            game,
            values: BTreeMap::new(),
        });
    }

    /// The game closed: its unsaved changes are dropped. Returns the keys that had one.
    pub fn end_session(&mut self) -> Vec<&'static str> {
        self.session
            .take()
            .map(|s| s.values.into_keys().collect())
            .unwrap_or_default()
    }

    /// The scope of the open game (`None` without one).
    pub fn session_scope(&self) -> Option<Scope> {
        self.session.as_ref().map(|s| Scope::Game {
            console: s.console,
            game: s.game.clone(),
        })
    }

    /// Saves the session's changes to the game or its console, then empties the session.
    /// All or nothing: if one entry does not accept the target, nothing is saved.
    pub fn commit_session(
        &mut self,
        target: CommitTarget,
    ) -> Result<(Level, Vec<&'static str>), ErrorCode> {
        let session = self.session.as_ref().ok_or(ErrorCode::NoGame)?;
        let (level, file) = match target {
            CommitTarget::Game => (Level::Game, FileId::Game(session.game.clone())),
            CommitTarget::Console => (Level::Console, FileId::Console(session.console)),
        };
        let at = (self.clock)();
        let changes = session
            .values
            .iter()
            .map(|(key, value)| {
                let entry = known(key)?;
                if !entry.levels.accepts(level) {
                    return Err(level_refused(entry, level));
                }
                let json =
                    serde_json::to_value(value).map_err(|e| ErrorCode::SettingsWriteFailed {
                        detail: e.to_string(),
                    })?;
                Ok(Change {
                    key: entry.key,
                    per_device: entry.per_device,
                    value: Some(json),
                    at: Some(at),
                })
            })
            .collect::<Result<Vec<Change<'static>>, _>>()?;
        let keys = changes.iter().map(|c| c.key).collect();
        self.store.apply(&file, &changes)?;
        self.session_mut()?.values.clear();
        Ok((level, keys))
    }

    /// Drops the session's changes, keeping the session open. Returns the keys that had one.
    pub fn discard_session(&mut self) -> Result<Vec<&'static str>, ErrorCode> {
        let values = std::mem::take(&mut self.session_mut()?.values);
        Ok(values.into_keys().collect())
    }

    /// The `limit` newest changes across every level (Global, each console, each game), newest
    /// first. Keys this version does not know, and entries without a value, are left out.
    pub fn recent(&mut self, limit: usize) -> Vec<RecentChange> {
        let consoles = [
            ConsoleId::Gb,
            ConsoleId::Gbc,
            ConsoleId::Gba,
            ConsoleId::Nds,
            ConsoleId::N3ds,
        ];
        let mut files: Vec<FileId> = vec![FileId::Global];
        files.extend(consoles.into_iter().map(FileId::Console));
        files.extend(self.store.game_ids().into_iter().map(FileId::Game));
        let mut out: Vec<RecentChange> = Vec::new();
        for file in files {
            for (key, at) in self.store.changed_in(&file) {
                let Some(entry) = entry(&key).filter(|e| e.has_value()) else {
                    continue;
                };
                let (level, console, game) = match &file {
                    FileId::Global => (Level::Global, None, None),
                    FileId::Console(c) => (Level::Console, Some(*c), None),
                    FileId::Game(g) => (Level::Game, None, Some(g.clone())),
                };
                out.push(RecentChange {
                    key: entry.key,
                    level,
                    console,
                    game,
                    at,
                });
            }
        }
        // Newest first; equal times in a stable order (key, then level).
        out.sort_by(|a, b| {
            b.at.cmp(&a.at)
                .then_with(|| a.key.cmp(b.key))
                .then_with(|| a.level.cmp(&b.level))
        });
        out.truncate(limit);
        out
    }

    /// Every value stored at the Game level for `game`, in the catalogue's order, each with the
    /// value its console's games get (L06). Entries without a value are left out.
    pub fn game_overrides(&mut self, console: ConsoleId, game: &GameId) -> Vec<GameOverride> {
        let file = FileId::Game(game.clone());
        let console_scope = Scope::Console { console };
        let mut out = Vec::new();
        for entry in catalog::ENTRIES
            .iter()
            .filter(|e| e.has_value() && e.levels.accepts(Level::Game))
        {
            let Some(value) = self.stored(entry, Level::Game, Some(&file)) else {
                continue;
            };
            let base = match self.resolve_entry(entry, &console_scope) {
                Ok(r) => r.value,
                Err(e) => {
                    log::warn!("{}: no console value ({e:?})", entry.key);
                    None
                }
            };
            out.push(GameOverride {
                key: entry.key,
                page: entry.page,
                value,
                base,
            });
        }
        out
    }

    fn session_mut(&mut self) -> Result<&mut SessionLayer, ErrorCode> {
        self.session.as_mut().ok_or(ErrorCode::NoGame)
    }

    fn resolve_entry(&mut self, entry: &Entry, scope: &Scope) -> Result<Resolved, ErrorCode> {
        if !entry.has_value() {
            return Ok(Resolved {
                value: None,
                source: Level::Default,
            });
        }
        let chain = self.chain(scope)?;
        let found = chain
            .into_iter()
            .filter(|(level, _)| entry.levels.accepts(*level))
            .find_map(|(level, file)| {
                self.stored(entry, level, file.as_ref())
                    .map(|value| Resolved {
                        value: Some(value),
                        source: level,
                    })
            });
        Ok(found.unwrap_or_else(|| Resolved {
            value: entry.default_value(),
            source: Level::Default,
        }))
    }

    /// The levels to look at for `scope`, strongest first (Default excluded).
    fn chain(&self, scope: &Scope) -> Result<Vec<(Level, Option<FileId>)>, ErrorCode> {
        let game_chain = |console: ConsoleId, game: &GameId| {
            vec![
                (Level::Game, Some(FileId::Game(game.clone()))),
                (Level::Console, Some(FileId::Console(console))),
                (Level::Global, Some(FileId::Global)),
            ]
        };
        Ok(match scope {
            Scope::Global => vec![(Level::Global, Some(FileId::Global))],
            Scope::Console { console } => vec![
                (Level::Console, Some(FileId::Console(*console))),
                (Level::Global, Some(FileId::Global)),
            ],
            Scope::Game { console, game } => game_chain(*console, game),
            Scope::Session => {
                let s = self.session.as_ref().ok_or(ErrorCode::NoGame)?;
                std::iter::once((Level::Session, None))
                    .chain(game_chain(s.console, &s.game))
                    .collect()
            }
        })
    }

    /// The valid value stored for `entry` at one level; an invalid one is logged and skipped.
    fn stored(&mut self, entry: &Entry, level: Level, file: Option<&FileId>) -> Option<Value> {
        let Some(file) = file else {
            return self.session.as_ref()?.values.get(entry.key).cloned();
        };
        let json = self.store.get(file, entry.per_device, entry.key)?;
        match serde_json::from_value::<Value>(json.clone()) {
            Ok(value) if entry.accepts_value_on(&value, self.device) => Some(value),
            // Valid, but an option of another device (the synced files are shared): kept in the
            // file, skipped here.
            Ok(value) if entry.accepts_value(&value) => {
                log::debug!(
                    "{} = {json} at {level:?} is not offered on {:?}: using the weaker levels",
                    entry.key,
                    self.device
                );
                None
            }
            Ok(_) | Err(_) => {
                log::warn!(
                    "settingInvalidValue: {} = {json} at {level:?} ({file:?}): ignored, using the weaker levels",
                    entry.key
                );
                None
            }
        }
    }
}

fn known(key: &str) -> Result<&'static Entry, ErrorCode> {
    entry(key).ok_or_else(|| ErrorCode::SettingUnknown { key: key.into() })
}

/// The level `scope` writes to, if `entry` accepts it.
fn accepted_level(entry: &Entry, scope: &Scope) -> Result<Level, ErrorCode> {
    let level = level_of(scope);
    if entry.levels.accepts(level) {
        Ok(level)
    } else {
        Err(level_refused(entry, level))
    }
}

/// `entry` cannot be stored at `level`: the error names the levels it can be stored at.
fn level_refused(entry: &Entry, level: Level) -> ErrorCode {
    ErrorCode::SettingLevelRefused {
        key: entry.key.into(),
        level,
        levels: entry
            .levels
            .list()
            .into_iter()
            .filter(|l| *l != Level::Session)
            .collect(),
    }
}

fn level_of(scope: &Scope) -> Level {
    match scope {
        Scope::Global => Level::Global,
        Scope::Console { .. } => Level::Console,
        Scope::Game { .. } => Level::Game,
        Scope::Session => Level::Session,
    }
}

/// The file `scope` writes to (`None` for the session, which stays in memory).
fn file_of(scope: &Scope) -> Option<FileId> {
    match scope {
        Scope::Global => Some(FileId::Global),
        Scope::Console { console } => Some(FileId::Console(*console)),
        Scope::Game { game, .. } => Some(FileId::Game(game.clone())),
        Scope::Session => None,
    }
}

#[cfg(test)]
pub(crate) mod test_dir {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A fresh folder under the system temp dir, removed when dropped.
    pub struct TestDir(PathBuf);

    impl TestDir {
        pub fn new(name: &str) -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "emudome-test-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        pub fn path(&self, rel: &str) -> PathBuf {
            self.0.join(rel)
        }

        pub fn write(&self, rel: &str, text: &str) {
            let path = self.path(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }

        pub fn read(&self, rel: &str) -> String {
            std::fs::read_to_string(self.path(rel)).unwrap()
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests;
