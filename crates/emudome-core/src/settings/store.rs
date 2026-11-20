//! Settings files on disk.
//!
//! Layout (two roots, given by the shell):
//! - synced root (`<app data>/settings`, the one Drive sync will carry):
//!   `global.json`, `console/<console>.json`, `game/<game id>.json`;
//! - device file (`<app local data>/settings/device.json`, never synced): the values of the
//!   per-device entries, at every level: `{ "global": {..}, "console": { "gba": {..} },
//!   "game": { "<game id>": {..} } }`.
//!
//! Each file is a flat JSON object `key → value`. The whole object is kept in memory and
//! written back, so keys this version does not know (from a newer version, or from another
//! device) survive. Writes go through [`crate::write_atomic`] (temporary file, fsync, rename).
//!
//! Reading a file has three outcomes, kept apart so that a file is never lost:
//! - missing: the level is empty, and the first write creates the file;
//! - not valid JSON: the file is renamed to `<name>.json.unreadable` (kept for the user), the
//!   level starts empty and the next write creates a new file; if the rename fails, it is
//!   treated as below;
//! - not readable (permission denied, a folder in its place, an I/O error): nothing is cached,
//!   the level reads as empty (the defaults show) and every write to that file is refused with
//!   `settingsReadFailed` until a later read succeeds. The file is never written over.
//!
//! When each value was last changed («Modificate di recente», T01) lives in the same layer as the
//! value, under the reserved key [`CHANGED_KEY`]: `"$changed": { "<key>": <Unix ms> }`. So the
//! time travels with the value: synced with the synced files, kept on this device for the
//! per-device entries. A reset records its time too (the value went back to the weaker level).
//! Older versions see `$changed` as an unknown key and keep it.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

use super::{ConsoleId, GameId};
use crate::ErrorCode;

/// One level's values as stored: `key → JSON value`, unknown keys included.
pub type Layer = Map<String, Json>;

/// The reserved key of a layer that holds `key → Unix ms of its last change`.
pub const CHANGED_KEY: &str = "$changed";

/// Which file (or which part of the device file) a level lives in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileId {
    Global,
    Console(ConsoleId),
    Game(GameId),
}

/// The per-device values, by level.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DeviceLayers {
    #[serde(default)]
    global: Layer,
    #[serde(default)]
    console: BTreeMap<String, Layer>,
    #[serde(default)]
    game: BTreeMap<String, Layer>,
    /// Anything else a newer version wrote.
    #[serde(flatten)]
    other: Layer,
}

impl DeviceLayers {
    fn layer(&self, file: &FileId) -> Option<&Layer> {
        match file {
            FileId::Global => Some(&self.global),
            FileId::Console(c) => self.console.get(c.id()),
            FileId::Game(g) => self.game.get(g.as_str()),
        }
    }

    fn layer_mut(&mut self, file: &FileId) -> &mut Layer {
        match file {
            FileId::Global => &mut self.global,
            FileId::Console(c) => self.console.entry(c.id().to_owned()).or_default(),
            FileId::Game(g) => self.game.entry(g.as_str().to_owned()).or_default(),
        }
    }
}

/// A change to apply: `value: None` removes the key. `at` (Unix ms) records when, for
/// «Modificate di recente»; `None` leaves the record as it is.
pub struct Change<'a> {
    pub key: &'a str,
    pub per_device: bool,
    pub value: Option<Json>,
    pub at: Option<u64>,
}

/// The settings files, loaded on first use and kept in memory.
pub struct Store {
    synced_dir: PathBuf,
    device_file: PathBuf,
    /// Layers read (or found missing); a file that could not be read is never here.
    synced: HashMap<FileId, Layer>,
    device: Option<DeviceLayers>,
    /// Files whose last read failed: logged once until they read again.
    unreadable: HashSet<PathBuf>,
}

impl Store {
    pub fn new(synced_dir: PathBuf, device_file: PathBuf) -> Self {
        Self {
            synced_dir,
            device_file,
            synced: HashMap::new(),
            device: None,
            unreadable: HashSet::new(),
        }
    }

    fn synced_path(&self, file: &FileId) -> PathBuf {
        match file {
            FileId::Global => self.synced_dir.join("global.json"),
            FileId::Console(c) => self
                .synced_dir
                .join("console")
                .join(format!("{}.json", c.id())),
            FileId::Game(g) => self
                .synced_dir
                .join("game")
                .join(format!("{}.json", g.as_str())),
        }
    }

    /// The stored JSON for `key` at `file`, from the device file when `per_device`.
    pub fn get(&mut self, file: &FileId, per_device: bool, key: &str) -> Option<Json> {
        if per_device {
            self.device_layers().ok()?.layer(file)?.get(key).cloned()
        } else {
            self.synced_layer(file).ok()?.get(key).cloned()
        }
    }

    /// Applies the changes to `file` and writes the touched files (synced and/or device).
    /// On a write error nothing changes in memory. Refused with `settingsReadFailed`, writing
    /// nothing, when a file it would write could not be read.
    pub fn apply(&mut self, file: &FileId, changes: &[Change<'_>]) -> Result<(), ErrorCode> {
        let (device_changes, synced_changes): (Vec<&Change<'_>>, Vec<&Change<'_>>) =
            changes.iter().partition(|c| c.per_device);
        // Both files are read before either is written: a refusal writes nothing at all.
        let synced = if synced_changes.is_empty() {
            None
        } else {
            Some(self.synced_layer(file)?.clone())
        };
        let device = if device_changes.is_empty() {
            None
        } else {
            Some(self.device_layers()?.clone())
        };
        if let Some(mut layer) = synced {
            apply_to(&mut layer, &synced_changes);
            write_json(&self.synced_path(file), &layer)?;
            self.synced.insert(file.clone(), layer);
        }
        if let Some(mut layers) = device {
            apply_to(layers.layer_mut(file), &device_changes);
            write_json(&self.device_file, &layers)?;
            self.device = Some(layers);
        }
        Ok(())
    }

    /// When each key of `file` was last changed (synced and per-device values together).
    pub fn changed_in(&mut self, file: &FileId) -> Vec<(String, u64)> {
        let mut out = self.synced_layer(file).map(changed_of).unwrap_or_default();
        if let Some(layer) = self.device_layers().ok().and_then(|d| d.layer(file)) {
            out.extend(changed_of(layer));
        }
        out
    }

    /// The games that have a settings file, or per-device values on this device.
    pub fn game_ids(&mut self) -> Vec<GameId> {
        let mut ids: Vec<GameId> = Vec::new();
        let dir = self.synced_dir.join("game");
        match fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(id) = GameId::try_from(stem.to_owned()) {
                                ids.push(id);
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => log::warn!("settingsReadFailed: {}: {e}", dir.display()),
        }
        let device_games: Vec<String> = self
            .device_layers()
            .map(|d| d.game.keys().cloned().collect())
            .unwrap_or_default();
        for key in device_games {
            if let Ok(id) = GameId::try_from(key) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        ids.sort();
        ids
    }

    /// The layer of `file`, read on first use. A file that cannot be read is not cached: the
    /// next call reads it again.
    fn synced_layer(&mut self, file: &FileId) -> Result<&Layer, ErrorCode> {
        if !self.synced.contains_key(file) {
            let path = self.synced_path(file);
            let layer = self.read::<Layer>(&path)?;
            self.synced.insert(file.clone(), layer);
        }
        Ok(self.synced.entry(file.clone()).or_default())
    }

    /// The per-device layers, read on first use; not cached when unreadable, as above.
    fn device_layers(&mut self) -> Result<&DeviceLayers, ErrorCode> {
        if self.device.is_none() {
            let path = self.device_file.clone();
            let layers = self.read::<DeviceLayers>(&path)?;
            self.device = Some(layers);
        }
        Ok(self.device.get_or_insert_with(DeviceLayers::default))
    }

    /// Reads a file: missing (or moved aside) gives the default; unreadable gives
    /// `settingsReadFailed`, logged once until the file reads again.
    fn read<T>(&mut self, path: &Path) -> Result<T, ErrorCode>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        match read_json(path) {
            Read::Found(value) => {
                self.unreadable.remove(path);
                Ok(value)
            }
            Read::Missing => {
                self.unreadable.remove(path);
                Ok(T::default())
            }
            Read::Failed(detail) => {
                if self.unreadable.insert(path.to_path_buf()) {
                    log::warn!(
                        "settingsReadFailed: {}: {detail}; writes to it are refused until it reads",
                        path.display()
                    );
                }
                Err(ErrorCode::SettingsReadFailed { detail })
            }
        }
    }
}

/// What reading a settings file gave.
enum Read<T> {
    Found(T),
    /// No file (or a file that was not JSON, now moved aside).
    Missing,
    /// The file is there but could not be read: never to be written over.
    Failed(String),
}

fn apply_to(layer: &mut Layer, changes: &[&Change<'_>]) {
    for change in changes {
        match &change.value {
            Some(v) => {
                layer.insert(change.key.to_owned(), v.clone());
            }
            None => {
                layer.remove(change.key);
            }
        }
        if let Some(at) = change.at {
            let times = layer
                .entry(CHANGED_KEY.to_owned())
                .or_insert_with(|| Json::Object(Map::new()));
            if !times.is_object() {
                // Something else under the reserved key: replaced, it can only be ours.
                *times = Json::Object(Map::new());
            }
            if let Json::Object(map) = times {
                map.insert(change.key.to_owned(), Json::from(at));
            }
        }
    }
}

/// The `$changed` record of a layer; malformed items are skipped.
fn changed_of(layer: &Layer) -> Vec<(String, u64)> {
    match layer.get(CHANGED_KEY) {
        Some(Json::Object(map)) => map
            .iter()
            .filter_map(|(key, at)| at.as_u64().map(|at| (key.clone(), at)))
            .collect(),
        Some(_) | None => Vec::new(),
    }
}

/// Reads a JSON file (the three outcomes of the module comment).
fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Read<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        // A file in place of one of its folders: no file can be there either.
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Read::Missing
        }
        Err(e) => return Read::Failed(e.to_string()),
    };
    match serde_json::from_str(&text) {
        Ok(value) => Read::Found(value),
        Err(e) => {
            let aside = path.with_extension("json.unreadable");
            log::warn!(
                "settingsFileUnreadable: {}: {e}; moved to {}, starting empty",
                path.display(),
                aside.display()
            );
            match fs::rename(path, &aside) {
                Ok(()) => Read::Missing,
                Err(not_moved) => Read::Failed(format!(
                    "not valid JSON ({e}) and not moved aside ({not_moved})"
                )),
            }
        }
    }
}

/// Writes `value` as pretty JSON, atomically.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ErrorCode> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(write_failed)?;
    bytes.push(b'\n');
    crate::write_atomic(path, &bytes).map_err(write_failed)
}

fn write_failed(e: impl std::fmt::Display) -> ErrorCode {
    ErrorCode::SettingsWriteFailed {
        detail: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Change, FileId, Store};
    use crate::settings::test_dir::TestDir;
    use crate::settings::{ConsoleId, GameId};

    fn store(dir: &TestDir) -> Store {
        Store::new(dir.path("synced"), dir.path("local/device.json"))
    }

    fn set(key: &str, per_device: bool, value: serde_json::Value) -> Change<'_> {
        Change {
            key,
            per_device,
            value: Some(value),
            at: None,
        }
    }

    #[test]
    fn round_trip_through_the_files() {
        let dir = TestDir::new("store-round-trip");
        let game = FileId::Game(GameId::from_rom_path("C:/games/test.gba".as_ref()));
        let console = FileId::Console(ConsoleId::Gba);
        {
            let mut s = store(&dir);
            s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
                .unwrap();
            s.apply(&console, &[set("video.vsync", true, json!(false))])
                .unwrap();
            s.apply(&game, &[set("audio.vol", false, json!(55))])
                .unwrap();
        }
        let mut s = store(&dir);
        assert_eq!(
            s.get(&FileId::Global, false, "video.scale"),
            Some(json!("fill"))
        );
        assert_eq!(s.get(&console, true, "video.vsync"), Some(json!(false)));
        assert_eq!(s.get(&console, false, "video.vsync"), None);
        assert_eq!(s.get(&game, false, "audio.vol"), Some(json!(55)));
    }

    #[test]
    fn unknown_keys_survive_a_write() {
        let dir = TestDir::new("store-unknown-keys");
        dir.write(
            "synced/global.json",
            r#"{ "future.thing": [1, 2], "video.scale": "fit" }"#,
        );
        dir.write(
            "local/device.json",
            r#"{ "global": { "future.local": true }, "version": 9 }"#,
        );
        let mut s = store(&dir);
        s.apply(
            &FileId::Global,
            &[
                set("video.scale", false, json!("fill")),
                set("video.vsync", true, json!(false)),
            ],
        )
        .unwrap();
        let synced: serde_json::Value =
            serde_json::from_str(&dir.read("synced/global.json")).unwrap();
        assert_eq!(
            synced,
            json!({ "future.thing": [1, 2], "video.scale": "fill" })
        );
        let device: serde_json::Value =
            serde_json::from_str(&dir.read("local/device.json")).unwrap();
        assert_eq!(device["global"]["future.local"], json!(true));
        assert_eq!(device["global"]["video.vsync"], json!(false));
        assert_eq!(device["version"], json!(9));
    }

    #[test]
    fn removing_a_key_writes_it_out() {
        let dir = TestDir::new("store-remove");
        let mut s = store(&dir);
        s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .unwrap();
        s.apply(
            &FileId::Global,
            &[Change {
                key: "video.scale",
                per_device: false,
                value: None,
                at: None,
            }],
        )
        .unwrap();
        assert_eq!(store(&dir).get(&FileId::Global, false, "video.scale"), None);
    }

    #[test]
    fn an_unreadable_file_is_moved_aside_not_overwritten() {
        let dir = TestDir::new("store-unreadable");
        dir.write("synced/global.json", "{ not json");
        let mut s = store(&dir);
        assert_eq!(s.get(&FileId::Global, false, "video.scale"), None);
        assert_eq!(dir.read("synced/global.json.unreadable"), "{ not json");
        s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .unwrap();
        assert_eq!(dir.read("synced/global.json.unreadable"), "{ not json");
    }

    #[test]
    fn a_failed_write_changes_nothing_in_memory() {
        let dir = TestDir::new("store-failed-write");
        // A file where the synced folder should be: creating global.json inside it fails.
        dir.write("synced", "not a folder");
        let mut s = store(&dir);
        let result = s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))]);
        assert!(matches!(
            result,
            Err(crate::ErrorCode::SettingsWriteFailed { .. })
        ));
        assert_eq!(s.get(&FileId::Global, false, "video.scale"), None);
    }

    /// Puts a folder (with a file inside) where `rel` should be a file: reading it fails on
    /// every OS (Windows: access denied; Linux and Android: is a directory).
    fn make_unreadable(dir: &TestDir, rel: &str) {
        dir.write(&format!("{rel}/inside.txt"), "kept");
    }

    #[test]
    fn an_unreadable_file_is_never_written_and_writes_are_refused() {
        let dir = TestDir::new("store-read-failed");
        make_unreadable(&dir, "synced/global.json");
        let mut s = store(&dir);
        // Reads fall back to the defaults.
        assert_eq!(s.get(&FileId::Global, false, "video.scale"), None);
        for _ in 0..2 {
            let result = s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))]);
            assert!(
                matches!(result, Err(crate::ErrorCode::SettingsReadFailed { .. })),
                "{result:?}"
            );
        }
        assert!(dir.path("synced/global.json").is_dir());
        assert_eq!(dir.read("synced/global.json/inside.txt"), "kept");
        // Other files are not affected.
        s.apply(
            &FileId::Console(ConsoleId::Gba),
            &[set("video.scale", false, json!("fit"))],
        )
        .unwrap();
    }

    #[test]
    fn an_unreadable_device_file_refuses_the_whole_change() {
        let dir = TestDir::new("store-device-read-failed");
        make_unreadable(&dir, "local/device.json");
        let mut s = store(&dir);
        let result = s.apply(
            &FileId::Global,
            &[
                set("video.scale", false, json!("fill")),
                set("video.vsync", true, json!(false)),
            ],
        );
        assert!(matches!(
            result,
            Err(crate::ErrorCode::SettingsReadFailed { .. })
        ));
        // Neither the synced file nor the device file was written.
        assert!(!dir.path("synced/global.json").exists());
        assert_eq!(dir.read("local/device.json/inside.txt"), "kept");
        assert_eq!(s.get(&FileId::Global, false, "video.scale"), None);
    }

    #[test]
    fn writes_resume_once_the_file_reads_again_and_nothing_is_lost() {
        let dir = TestDir::new("store-read-recovers");
        make_unreadable(&dir, "synced/global.json");
        let mut s = store(&dir);
        assert!(s
            .apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .is_err());
        // The drive comes back / the permission is fixed: the real file is there again.
        std::fs::remove_dir_all(dir.path("synced/global.json")).unwrap();
        dir.write(
            "synced/global.json",
            r#"{ "audio.vol": 40, "future.thing": true }"#,
        );
        assert_eq!(s.get(&FileId::Global, false, "audio.vol"), Some(json!(40)));
        s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .unwrap();
        let synced: serde_json::Value =
            serde_json::from_str(&dir.read("synced/global.json")).unwrap();
        assert_eq!(
            synced,
            json!({ "audio.vol": 40, "future.thing": true, "video.scale": "fill" })
        );
    }
}
