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
//! device) survive. Writes are atomic: a temporary file next to the target, flushed, then renamed
//! over it. A file that cannot be parsed is moved aside to `<name>.unreadable` and the level
//! starts empty, so the next write does not destroy what was there.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

use super::{ConsoleId, GameId};
use crate::ErrorCode;

/// One level's values as stored: `key → JSON value`, unknown keys included.
pub type Layer = Map<String, Json>;

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

/// A change to apply: `value: None` removes the key.
pub struct Change<'a> {
    pub key: &'a str,
    pub per_device: bool,
    pub value: Option<Json>,
}

/// The settings files, loaded on first use and kept in memory.
pub struct Store {
    synced_dir: PathBuf,
    device_file: PathBuf,
    synced: HashMap<FileId, Layer>,
    device: Option<DeviceLayers>,
}

impl Store {
    pub fn new(synced_dir: PathBuf, device_file: PathBuf) -> Self {
        Self {
            synced_dir,
            device_file,
            synced: HashMap::new(),
            device: None,
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
            self.device_layers().layer(file)?.get(key).cloned()
        } else {
            self.synced_layer(file).get(key).cloned()
        }
    }

    /// Applies the changes to `file` and writes the touched files (synced and/or device).
    /// On a write error nothing changes in memory.
    pub fn apply(&mut self, file: &FileId, changes: &[Change<'_>]) -> Result<(), ErrorCode> {
        let (device_changes, synced_changes): (Vec<&Change<'_>>, Vec<&Change<'_>>) =
            changes.iter().partition(|c| c.per_device);
        if !synced_changes.is_empty() {
            let mut layer = self.synced_layer(file).clone();
            apply_to(&mut layer, &synced_changes);
            write_json(&self.synced_path(file), &layer)?;
            self.synced.insert(file.clone(), layer);
        }
        if !device_changes.is_empty() {
            let mut layers = self.device_layers().clone();
            apply_to(layers.layer_mut(file), &device_changes);
            write_json(&self.device_file, &layers)?;
            self.device = Some(layers);
        }
        Ok(())
    }

    fn synced_layer(&mut self, file: &FileId) -> &Layer {
        if !self.synced.contains_key(file) {
            let layer = read_json::<Layer>(&self.synced_path(file)).unwrap_or_default();
            self.synced.insert(file.clone(), layer);
        }
        self.synced.entry(file.clone()).or_default()
    }

    fn device_layers(&mut self) -> &DeviceLayers {
        if self.device.is_none() {
            self.device = Some(read_json(&self.device_file).unwrap_or_default());
        }
        self.device.get_or_insert_with(DeviceLayers::default)
    }
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
    }
}

/// Reads a JSON file. Missing → `None` silently; unreadable → logged, moved aside, `None`.
fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("settingsReadFailed: {}: {e}", path.display());
            return None;
        }
    };
    match serde_json::from_str(&text) {
        Ok(value) => Some(value),
        Err(e) => {
            let aside = path.with_extension("json.unreadable");
            log::warn!(
                "settingsFileUnreadable: {}: {e}; moved to {}, starting empty",
                path.display(),
                aside.display()
            );
            if let Err(e) = fs::rename(path, &aside) {
                log::warn!("settingsReadFailed: could not move {}: {e}", path.display());
            }
            None
        }
    }
}

/// Writes `value` as pretty JSON, atomically.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ErrorCode> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(write_failed)?;
    bytes.push(b'\n');
    write_atomic(path, &bytes).map_err(write_failed)
}

fn write_failed(e: impl std::fmt::Display) -> ErrorCode {
    ErrorCode::SettingsWriteFailed {
        detail: e.to_string(),
    }
}

/// Writes a temporary file next to `path`, flushes it to disk, then renames it over `path`
/// (a replace on every OS): a crash leaves either the old file or the new one, never half.
pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{write_atomic, Change, FileId, Store};
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
    fn atomic_write_replaces_and_leaves_no_temporary_file() {
        let dir = TestDir::new("store-atomic");
        let path = dir.path("a/b/file.json");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(dir.read("a/b/file.json"), "second");
        assert!(!dir.path("a/b/file.json.tmp").exists());
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
}
