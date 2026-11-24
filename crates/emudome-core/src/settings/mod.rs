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

pub(crate) mod catalog;
pub mod core_values;
pub mod keys;
pub mod schema;
mod store;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use schema::{
    entry, page_entries, schema, synonyms, ActionStyle, ChoiceStyle, DateTime, DefaultRule,
    DefaultValue, Device, Entry, Kind, Level, Levels, Platform, SchemaDto, Unit, Value,
};
use store::{Change, FileId, Store};

/// A console, as a settings level (its own module since #81; re-exported here, where the code
/// has always named it).
pub use crate::console::ConsoleId;
pub use crate::game_key::GameKey;
use crate::ErrorCode;

/// What the interface is looking at or changing. TypeScript: `SettingsScope` (generated).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "level", rename_all = "camelCase")]
#[ts(rename = "SettingsScope")]
pub enum Scope {
    Global,
    Console {
        console: ConsoleId,
    },
    Game {
        console: ConsoleId,
        game: GameKey,
    },
    /// The open game with its changes «in prova».
    Session,
}

/// Where the quick menu's changes are saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
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

/// One row of a settings page. TypeScript: `SettingValueRow` (generated).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "SettingRow")]
pub struct PageValue {
    #[ts(as = "crate::contract::SettingKey")]
    pub key: &'static str,
    pub value: Option<Value>,
    /// The level the value comes from: when weaker than the page's scope, the row shows
    /// «Come in "…"»; when equal, it can be reset.
    pub source: Level,
    /// The value differs from the schema's default.
    pub changed: bool,
}

/// A value stored for one game (Game details L06, «Impostazioni del gioco»), with what every game
/// of its console gets instead. TypeScript: `GameOverride` (generated).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GameOverride {
    #[ts(as = "crate::contract::SettingKey")]
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
    game: GameKey,
    values: BTreeMap<&'static str, Value>,
}

/// A value changed recently, at one level («Modificate di recente», T01).
/// TypeScript: `SettingRecent` (generated).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "SettingRecent")]
pub struct RecentChange {
    #[ts(as = "crate::contract::SettingKey")]
    pub key: &'static str,
    /// Global, Console or Game.
    pub level: Level,
    /// The console of a Console level.
    pub console: Option<ConsoleId>,
    /// The game of a Game level.
    pub game: Option<GameKey>,
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

    /// Reads the files again on next use: the one-time game key migration
    /// ([`crate::migration`]) moved some of them.
    pub fn forget_files(&mut self) {
        self.store.forget();
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
                    changed: resolved.value != self.default_of(entry, scope),
                    value: resolved.value,
                    source: resolved.source,
                })
            })
            .collect()
    }

    /// Stores `value` for `key` at `scope`'s level (in memory only for the session).
    /// Returns the level written, for the `settings://changed` event.
    pub fn set(&mut self, key: &str, scope: &Scope, value: Value) -> Result<Level, ErrorCode> {
        self.set_many(scope, &[(key, value)])
    }

    /// Stores several values at `scope`'s level in one write (in memory only for the session).
    /// All or nothing: every key and value is checked first, and the file is written once, so a
    /// refusal or a failed write leaves every key as it was.
    pub fn set_many(
        &mut self,
        scope: &Scope,
        values: &[(&str, Value)],
    ) -> Result<Level, ErrorCode> {
        let checked = values
            .iter()
            .map(|(key, value)| {
                let entry = known(key)?;
                let level = accepted_level(entry, scope)?;
                if !entry.has_value() || !entry.accepts_value_on(value, self.device) {
                    return Err(ErrorCode::SettingInvalidValue { key: (*key).into() });
                }
                Ok((entry, level, value))
            })
            .collect::<Result<Vec<_>, ErrorCode>>()?;
        let level = checked
            .first()
            .map(|(_, level, _)| *level)
            .ok_or(ErrorCode::NoGame)?;
        match file_of(scope) {
            Some(file) => {
                let at = (self.clock)();
                let mut changes = Vec::with_capacity(checked.len());
                for (entry, _, value) in &checked {
                    changes.push(self.change_to_store(entry, scope, value, at)?);
                }
                self.store.apply(&file, &changes)?;
            }
            None => {
                let session = self.session_mut()?;
                checked.iter().for_each(|(entry, _, value)| {
                    session.values.insert(entry.key, (*value).clone());
                });
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
    pub fn begin_session(&mut self, console: ConsoleId, game: GameKey) {
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
        let scope = match target {
            CommitTarget::Game => Scope::Game {
                console: session.console,
                game: session.game.clone(),
            },
            CommitTarget::Console => Scope::Console {
                console: session.console,
            },
        };
        let at = (self.clock)();
        let values = session
            .values
            .iter()
            .map(|(key, value)| {
                let entry = known(key)?;
                if !entry.levels.accepts(level) {
                    return Err(level_refused(entry, level));
                }
                Ok((entry, value.clone()))
            })
            .collect::<Result<Vec<_>, ErrorCode>>()?;
        let mut changes: Vec<Change<'static>> = Vec::with_capacity(values.len());
        for (entry, value) in &values {
            changes.push(self.change_to_store(entry, &scope, value, at)?);
        }
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
        let mut files: Vec<FileId> = vec![FileId::Global];
        files.extend(ConsoleId::ALL.into_iter().map(FileId::Console));
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
    pub fn game_overrides(&mut self, console: ConsoleId, game: &GameKey) -> Vec<GameOverride> {
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

    /// The change that stores `value` for `entry` at `scope`'s level (a file level). A value equal
    /// to what the weaker levels give there removes the key instead: the level then has nothing
    /// of its own, so the row is not «Modificato» and the file keeps only real differences. A
    /// value equal to the default but different from a weaker level (a game back to the default
    /// while its console has another value) is stored.
    fn change_to_store(
        &mut self,
        entry: &'static Entry,
        scope: &Scope,
        value: &Value,
        at: u64,
    ) -> Result<Change<'static>, ErrorCode> {
        let below = self.resolve_below(entry, scope)?;
        let stored = if below.as_ref() == Some(value) {
            None
        } else {
            Some(
                serde_json::to_value(value)
                    .map_err(|e| store::write_failed(crate::IoCause::Other, &e))?,
            )
        };
        Ok(Change {
            key: entry.key,
            per_device: entry.per_device,
            value: stored,
            at: Some(at),
        })
    }

    /// The value `entry` would have at `scope` with nothing stored at `scope`'s own level: the
    /// weaker levels, then the Default as seen from `scope` (its device and console rules).
    fn resolve_below(&mut self, entry: &Entry, scope: &Scope) -> Result<Option<Value>, ErrorCode> {
        let own = level_of(scope);
        let chain = self.chain(scope)?;
        let found = chain
            .into_iter()
            .filter(|(level, _)| *level != own && entry.levels.accepts(*level))
            .find_map(|(level, file)| self.stored(entry, level, file.as_ref()));
        Ok(found.or_else(|| self.default_of(entry, scope)))
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
            value: self.default_of(entry, scope),
            source: Level::Default,
        }))
    }

    /// The Default level of `entry` seen from `scope`: on this device, for its console.
    fn default_of(&self, entry: &Entry, scope: &Scope) -> Option<Value> {
        entry.default_for(self.device, self.console_of(scope))
    }

    /// The console `scope` looks at (`None` at the Global level, or without an open game).
    fn console_of(&self, scope: &Scope) -> Option<ConsoleId> {
        match scope {
            Scope::Global => None,
            Scope::Console { console } | Scope::Game { console, .. } => Some(*console),
            Scope::Session => self.session.as_ref().map(|s| s.console),
        }
    }

    /// The levels to look at for `scope`, strongest first (Default excluded).
    fn chain(&self, scope: &Scope) -> Result<Vec<(Level, Option<FileId>)>, ErrorCode> {
        let game_chain = |console: ConsoleId, game: &GameKey| {
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
