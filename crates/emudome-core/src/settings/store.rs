//! Settings files on disk.
//!
//! Layout (two roots, given by the shell):
//! - synced root (`<app data>/settings`, the one Drive sync will carry):
//!   `global.json`, `console/<console>.json`, `game/<console>-<hex>.json` (the game's key,
//!   [`GameKey::file_stem`]);
//! - device file (`<app local data>/local-settings/device.json`, never synced): the values of the
//!   per-device entries, at every level: `{ "global": {..}, "console": { "gba": {..} },
//!   "game": { "<console>-<hex>": {..} } }`.
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
//!
//! Every file says its layout's version under [`SCHEMA_KEY`] (`"$schema": 2`, written on each
//! save; a file without it is version 0). A file read with an older version goes through
//! [`migrate`] in memory before use (0 → 1 changes nothing: version 1 only adds the number;
//! 1 → 2 renames the touch controls' styles); a file from a newer version is used as it is, its
//! unknown keys kept and its number never lowered.
//!
//! What is in memory follows the disk: before each use, a file whose modification time or size
//! changed since it was read (another process, sync, the user) is read again, so a change made
//! elsewhere is neither missed nor written over by the next save. Before a write the file is
//! read again in full whatever its stamp: a file made unreadable since (a permission taken away,
//! a lock held by another program) keeps its stamp, and is refused as above, never written over.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

use super::{ConsoleId, GameKey};
use crate::{ErrorCode, IoCause};

/// One level's values as stored: `key → JSON value`, unknown keys included.
pub type Layer = Map<String, Json>;

/// The reserved key of a layer that holds `key → Unix ms of its last change`.
pub const CHANGED_KEY: &str = "$changed";

/// The reserved top-level key of every settings file: its layout's version.
pub const SCHEMA_KEY: &str = "$schema";

/// The layout this version writes.
pub const SCHEMA_VERSION: u64 = 2;

/// The version a file's top-level object says (0 when it says none).
fn schema_of(top: &Layer) -> u64 {
    top.get(SCHEMA_KEY).and_then(Json::as_u64).unwrap_or(0)
}

/// The steps from each layout to the next: `MIGRATIONS[n]` turns layout `n` into `n + 1`. A
/// layout change adds its step here and raises [`SCHEMA_VERSION`].
const MIGRATIONS: [fn(&mut Layer); SCHEMA_VERSION as usize] = [layout_0_to_1, layout_1_to_2];

/// 0 → 1: the same layout; version 1 only adds `$schema`, written on the next save.
fn layout_0_to_1(_top: &mut Layer) {}

/// 1 → 2: the touch controls' styles (`touch.tskin`, owner 2026-11-22) became «Fumé» (`smoke`)
/// and «Contorno» (`outline`). «Solo contorni» (`outlines`) is read as `outline`; «Vetro» and
/// «Contrasto alto» (`glass`, `contrast`) go, so the default (`smoke`) shows. Any other value is
/// left as it is (the engine ignores it as invalid, a newer one included).
fn layout_1_to_2(top: &mut Layer) {
    const SKIN: &str = "touch.tskin";
    match top.get(SKIN).and_then(Json::as_str) {
        Some("outlines") => {
            top.insert(SKIN.to_owned(), Json::from("outline"));
        }
        Some("glass" | "contrast") => {
            top.remove(SKIN);
        }
        Some(_) | None => {}
    }
}

/// Brings a file's top-level object read from an older layout to [`SCHEMA_VERSION`], in memory
/// (the file itself changes on its next save). A newer file is left as it is.
fn migrate(top: &mut Layer) {
    let from = usize::try_from(schema_of(top)).unwrap_or(usize::MAX);
    for step in MIGRATIONS.iter().skip(from) {
        step(top);
    }
}

/// Before a save: the file says the layout it is in (a newer file keeps its number).
fn stamp_schema(top: &mut Layer) {
    let version = schema_of(top).max(SCHEMA_VERSION);
    top.insert(SCHEMA_KEY.to_owned(), Json::from(version));
}

/// What a file looked like when it was read: a change of either means someone else wrote it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
}

/// The file's stamp now; `None` when there is no file (or it cannot be looked at).
fn stamp(path: &Path) -> Option<Stamp> {
    let meta = fs::metadata(path).ok()?;
    Some(Stamp {
        modified: meta.modified().ok(),
        len: meta.len(),
    })
}

/// A file's content as read, and its stamp then.
#[derive(Clone, Debug)]
struct Cached<T> {
    value: T,
    stamp: Option<Stamp>,
}

/// Which file (or which part of the device file) a level lives in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileId {
    Global,
    Console(ConsoleId),
    Game(GameKey),
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
            FileId::Game(g) => self.game.get(&g.file_stem()),
        }
    }

    fn layer_mut(&mut self, file: &FileId) -> &mut Layer {
        match file {
            FileId::Global => &mut self.global,
            FileId::Console(c) => self.console.entry(c.id().to_owned()).or_default(),
            FileId::Game(g) => self.game.entry(g.file_stem()).or_default(),
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
    /// Layers read (or found missing), with their file's stamp then; a file that could not be
    /// read is never here.
    synced: HashMap<FileId, Cached<Layer>>,
    device: Option<Cached<DeviceLayers>>,
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
                .join(format!("{}.json", g.file_stem())),
        }
    }

    /// Drops what was read: the next use reads the files again (after something outside the
    /// store moved them, the one-time game key migration).
    pub fn forget(&mut self) {
        self.synced.clear();
        self.device = None;
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
        // Both files are read before either is written: a refusal writes nothing at all. Read
        // again in full, whatever the cached copy's stamp says: a file that can no longer be read
        // (a permission taken away, a lock held elsewhere) keeps its modification time and size,
        // and must not be written over from memory. The files are small.
        if !synced_changes.is_empty() {
            self.synced.remove(file);
        }
        if !device_changes.is_empty() {
            self.device = None;
        }
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
            stamp_schema(&mut layer);
            let path = self.synced_path(file);
            write_json(&path, &layer)?;
            let stamp = stamp(&path);
            self.synced.insert(
                file.clone(),
                Cached {
                    value: layer,
                    stamp,
                },
            );
        }
        if let Some(mut layers) = device {
            apply_to(layers.layer_mut(file), &device_changes);
            stamp_schema(&mut layers.other);
            write_json(&self.device_file, &layers)?;
            self.device = Some(Cached {
                value: layers,
                stamp: stamp(&self.device_file),
            });
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
    pub fn game_ids(&mut self) -> Vec<GameKey> {
        let mut ids: Vec<GameKey> = Vec::new();
        let dir = self.synced_dir.join("game");
        match fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Some(id) = GameKey::from_file_stem(stem) {
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
            if let Some(id) = GameKey::from_file_stem(&key) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        ids.sort();
        ids
    }

    /// The layer of `file`, read on first use and again whenever the file changed on disk since.
    /// A file that cannot be read is not cached: the next call reads it again.
    fn synced_layer(&mut self, file: &FileId) -> Result<&Layer, ErrorCode> {
        let path = self.synced_path(file);
        // Taken before reading: a change while it is read shows at the next use.
        let now = stamp(&path);
        if self.synced.get(file).is_none_or(|c| c.stamp != now) {
            let layer = match self.read::<Layer>(&path) {
                Ok(mut layer) => {
                    migrate(&mut layer);
                    layer
                }
                Err(e) => {
                    self.synced.remove(file);
                    return Err(e);
                }
            };
            self.synced.insert(
                file.clone(),
                Cached {
                    value: layer,
                    stamp: now,
                },
            );
        }
        Ok(&self
            .synced
            .entry(file.clone())
            .or_insert_with(|| Cached {
                value: Layer::new(),
                stamp: None,
            })
            .value)
    }

    /// The per-device layers, read on first use and again when the file changed; not cached when
    /// unreadable, as above.
    fn device_layers(&mut self) -> Result<&DeviceLayers, ErrorCode> {
        let path = self.device_file.clone();
        let now = stamp(&path);
        if self.device.as_ref().is_none_or(|c| c.stamp != now) {
            let layers = match self.read::<DeviceLayers>(&path) {
                Ok(mut layers) => {
                    migrate(&mut layers.other);
                    layers
                }
                Err(e) => {
                    self.device = None;
                    return Err(e);
                }
            };
            self.device = Some(Cached {
                value: layers,
                stamp: now,
            });
        }
        Ok(&self
            .device
            .get_or_insert_with(|| Cached {
                value: DeviceLayers::default(),
                stamp: None,
            })
            .value)
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
            Read::Failed { cause, detail } => {
                if self.unreadable.insert(path.to_path_buf()) {
                    log::warn!(
                        "settingsReadFailed: {}: {detail}; writes to it are refused until it reads",
                        path.display()
                    );
                }
                Err(ErrorCode::SettingsReadFailed { cause })
            }
        }
    }
}

/// What reading a settings file gave.
enum Read<T> {
    Found(T),
    /// No file (or a file that was not JSON, now moved aside).
    Missing,
    /// The file is there but could not be read: never to be written over. `detail` (the OS's
    /// text) for the log.
    Failed {
        cause: IoCause,
        detail: String,
    },
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
    let text = match crate::atomic::read_text(path) {
        Ok(text) => text,
        // A file in place of one of its folders: no file can be there either.
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Read::Missing
        }
        Err(e) => {
            return Read::Failed {
                cause: IoCause::of(&e),
                detail: e.to_string(),
            }
        }
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
                Err(not_moved) => Read::Failed {
                    cause: IoCause::of(&not_moved),
                    detail: format!("not valid JSON ({e}) and not moved aside ({not_moved})"),
                },
            }
        }
    }
}

/// Writes `value` as pretty JSON, atomically.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ErrorCode> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| write_failed(IoCause::Other, &format!("{}: {e}", path.display())))?;
    bytes.push(b'\n');
    crate::write_atomic(path, &bytes)
        .map_err(|e| write_failed(IoCause::of(&e), &format!("{}: {e}", path.display())))
}

/// `settingsWriteFailed` for `cause`; the technical `detail` goes to the log only.
pub(crate) fn write_failed(cause: IoCause, detail: &dyn std::fmt::Display) -> ErrorCode {
    log::warn!("settingsWriteFailed: {detail}");
    ErrorCode::SettingsWriteFailed { cause }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Change, FileId, Store, SCHEMA_VERSION};
    use crate::settings::test_dir::TestDir;
    use crate::settings::{ConsoleId, GameKey};

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
        let game = FileId::Game(GameKey::parse("gba:7e57").unwrap());
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
            json!({ "$schema": SCHEMA_VERSION, "future.thing": [1, 2], "video.scale": "fill" })
        );
        let device: serde_json::Value =
            serde_json::from_str(&dir.read("local/device.json")).unwrap();
        assert_eq!(device["global"]["future.local"], json!(true));
        assert_eq!(device["global"]["video.vsync"], json!(false));
        assert_eq!(device["version"], json!(9));
        assert_eq!(device["$schema"], json!(SCHEMA_VERSION));
    }

    fn json_of(dir: &TestDir, rel: &str) -> serde_json::Value {
        serde_json::from_str(&dir.read(rel)).unwrap()
    }

    #[test]
    fn every_save_says_its_layout_and_a_file_without_one_reads_as_layout_0() {
        let dir = TestDir::new("store-schema");
        // Written before #81: no `$schema`.
        dir.write("synced/global.json", r#"{ "video.scale": "fit" }"#);
        let mut s = store(&dir);
        assert_eq!(
            s.get(&FileId::Global, false, "video.scale"),
            Some(json!("fit"))
        );
        // Reading changes nothing on disk...
        assert_eq!(json_of(&dir, "synced/global.json").get("$schema"), None);
        // ...the next save writes the layout.
        s.apply(&FileId::Global, &[set("audio.vol", false, json!(50))])
            .unwrap();
        assert_eq!(
            json_of(&dir, "synced/global.json")["$schema"],
            json!(SCHEMA_VERSION)
        );
        assert_eq!(
            json_of(&dir, "synced/global.json")["video.scale"],
            json!("fit")
        );
    }

    #[test]
    fn a_newer_layout_keeps_its_number_and_its_keys() {
        let dir = TestDir::new("store-schema-newer");
        dir.write(
            "synced/global.json",
            r#"{ "$schema": 7, "future.thing": { "deep": true }, "video.scale": "fit" }"#,
        );
        let mut s = store(&dir);
        s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .unwrap();
        assert_eq!(
            json_of(&dir, "synced/global.json"),
            json!({ "$schema": 7, "future.thing": { "deep": true }, "video.scale": "fill" })
        );
    }

    #[test]
    fn layout_2_reads_the_old_touch_styles_as_the_new_ones() {
        // «Solo contorni» becomes «Contorno»; «Vetro» and «Contrasto alto» go (the default,
        // «Fumé», shows); a value of layout 2 or one nobody knows is left alone.
        for (old, new) in [
            (json!("outlines"), Some(json!("outline"))),
            (json!("glass"), None),
            (json!("contrast"), None),
            (json!("outline"), Some(json!("outline"))),
            (json!("smoke"), Some(json!("smoke"))),
            (json!("future"), Some(json!("future"))),
        ] {
            for schema in [None, Some(1)] {
                let dir = TestDir::new("store-touch-skin");
                let mut file = json!({ "touch.tskin": old, "audio.vol": 40 });
                if let Some(n) = schema {
                    file["$schema"] = json!(n);
                }
                dir.write("synced/global.json", &file.to_string());
                let mut s = store(&dir);
                assert_eq!(
                    s.get(&FileId::Global, false, "touch.tskin"),
                    new,
                    "{old} at {schema:?}"
                );
                assert_eq!(s.get(&FileId::Global, false, "audio.vol"), Some(json!(40)));
                // Reading changes nothing on disk; the next save writes the new value.
                assert_eq!(json_of(&dir, "synced/global.json")["touch.tskin"], old);
                s.apply(&FileId::Global, &[set("audio.vol", false, json!(50))])
                    .unwrap();
                let saved = json_of(&dir, "synced/global.json");
                assert_eq!(saved.get("touch.tskin").cloned(), new);
                assert_eq!(saved["$schema"], json!(SCHEMA_VERSION));
            }
        }
    }

    #[test]
    fn a_layout_2_file_is_not_migrated_again() {
        // An older version wrote «Solo contorni» into a file already at layout 2 (it keeps the
        // newer number): the value stays as it is, and the engine ignores it as invalid.
        let dir = TestDir::new("store-touch-skin-2");
        dir.write(
            "synced/global.json",
            r#"{ "$schema": 2, "touch.tskin": "outlines" }"#,
        );
        let mut s = store(&dir);
        assert_eq!(
            s.get(&FileId::Global, false, "touch.tskin"),
            Some(json!("outlines"))
        );
    }

    #[test]
    fn a_file_changed_on_disk_is_read_again_and_never_written_over() {
        let dir = TestDir::new("store-mtime");
        let mut s = store(&dir);
        s.apply(
            &FileId::Global,
            &[Change {
                key: "video.scale",
                per_device: false,
                value: Some(json!("fit")),
                at: Some(1_000),
            }],
        )
        .unwrap();
        // Another process (sync, another window) writes the file: a new key, a new value, its
        // own `$changed` record.
        dir.write(
            "synced/global.json",
            r#"{ "$schema": 1, "video.scale": "stretch", "audio.vol": 30,
                 "$changed": { "video.scale": 2000, "audio.vol": 2000 } }"#,
        );
        assert_eq!(
            s.get(&FileId::Global, false, "video.scale"),
            Some(json!("stretch"))
        );
        // A save now starts from what is on disk: the other process's values stay.
        s.apply(
            &FileId::Global,
            &[Change {
                key: "video.vsync",
                per_device: false,
                value: Some(json!(false)),
                at: Some(3_000),
            }],
        )
        .unwrap();
        let file = json_of(&dir, "synced/global.json");
        assert_eq!(file["video.scale"], json!("stretch"));
        assert_eq!(file["audio.vol"], json!(30));
        assert_eq!(file["video.vsync"], json!(false));
        assert_eq!(
            file["$changed"],
            json!({ "video.scale": 2000, "audio.vol": 2000, "video.vsync": 3000 })
        );
    }

    #[test]
    fn the_device_file_changed_on_disk_is_read_again_too() {
        let dir = TestDir::new("store-mtime-device");
        let mut s = store(&dir);
        s.apply(&FileId::Global, &[set("video.vsync", true, json!(false))])
            .unwrap();
        assert_eq!(
            s.get(&FileId::Global, true, "video.vsync"),
            Some(json!(false))
        );
        dir.write(
            "local/device.json",
            r#"{ "$schema": 1, "global": { "video.vsync": true, "video.api": "vulkan" } }"#,
        );
        assert_eq!(
            s.get(&FileId::Global, true, "video.vsync"),
            Some(json!(true))
        );
        assert_eq!(
            s.get(&FileId::Global, true, "video.api"),
            Some(json!("vulkan"))
        );
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
    fn a_file_made_unreadable_after_it_was_read_is_never_written_over() {
        // Trello #178: `chmod 000` (or a lock held by another program) changes neither the
        // modification time nor the size, so the cached copy's stamp still matches.
        let dir = TestDir::new("store-unreadable-later");
        dir.write(
            "synced/global.json",
            r#"{ "audio.vol": 40, "future.thing": true }"#,
        );
        let mut s = store(&dir);
        assert_eq!(s.get(&FileId::Global, false, "audio.vol"), Some(json!(40)));
        let before = dir.read("synced/global.json");
        let path = dir.path("synced/global.json");
        {
            let _denied = crate::atomic::deny_reads(&path);
            for _ in 0..2 {
                let result = s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))]);
                assert!(
                    matches!(
                        result,
                        Err(crate::ErrorCode::SettingsReadFailed {
                            cause: crate::IoCause::PermissionDenied
                        })
                    ),
                    "{result:?}"
                );
            }
            assert_eq!(dir.read("synced/global.json"), before);
            // While it cannot be read, nothing from it is shown either.
            assert_eq!(s.get(&FileId::Global, false, "audio.vol"), None);
        }
        // Readable again: the next change is saved over what the file holds.
        s.apply(&FileId::Global, &[set("video.scale", false, json!("fill"))])
            .unwrap();
        assert_eq!(
            json_of(&dir, "synced/global.json"),
            json!({ "$schema": SCHEMA_VERSION, "audio.vol": 40, "future.thing": true, "video.scale": "fill" })
        );
    }

    #[test]
    fn a_device_file_made_unreadable_after_it_was_read_refuses_the_whole_change() {
        let dir = TestDir::new("store-device-unreadable-later");
        let mut s = store(&dir);
        s.apply(&FileId::Global, &[set("video.vsync", true, json!(false))])
            .unwrap();
        let before = dir.read("local/device.json");
        let _denied = crate::atomic::deny_reads(&dir.path("local/device.json"));
        let result = s.apply(
            &FileId::Global,
            &[
                set("video.scale", false, json!("fill")),
                set("video.vsync", true, json!(true)),
            ],
        );
        assert!(
            matches!(result, Err(crate::ErrorCode::SettingsReadFailed { .. })),
            "{result:?}"
        );
        // Neither file was written.
        assert!(!dir.path("synced/global.json").exists());
        assert_eq!(dir.read("local/device.json"), before);
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
            json!({ "$schema": SCHEMA_VERSION, "audio.vol": 40, "future.thing": true, "video.scale": "fill" })
        );
    }
}
