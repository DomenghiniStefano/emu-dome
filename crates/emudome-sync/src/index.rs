//! The sync index: what this device last synced, one JSON file (`<app data>/sync/index.json`).
//!
//! `{ "$schema": 1, "deviceId": "...", "files": { "<path>": { "id", "md5", "modified", "size" } },
//! "changesToken": "..." }` plus any key a newer version wrote (kept). `<path>` is the file's
//! place under Drive's `EmuDome/` folder (`gba/<hex>/game.sav`, `settings/global.json`): the
//! game's key, never a ROM path (rule 2). An entry says: at the last sync, the local file and
//! the Drive file `id` both had `md5`; it is the base of the conflict decision
//! ([`crate::decide`]).
//!
//! Written only through `emudome_core::write_atomic`, after every file synced, so an
//! interruption leaves an index that matches what was done. A file that is not valid JSON is
//! moved aside (`index.json.unreadable`) and the index starts empty, which is safe: with no base,
//! two sides that differ are a conflict the user is asked about, never an overwrite. A file that
//! is there but cannot be read (permission, I/O error) is never written over: every change is
//! refused with [`SyncError::IndexReadFailed`], each one first trying to read it again.
//!
//! The device id (random, made once) lives here too: the index is this device's own and is
//! never uploaded.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use emudome_core::IoCause;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

use crate::SyncError;

/// The layout version written in `$schema`.
pub const SCHEMA_VERSION: u64 = 1;

/// One synced file as it was at its last sync.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// The Drive file id.
    pub id: String,
    /// The MD5 both sides had (lowercase hex).
    pub md5: String,
    /// Drive's `modifiedTime` then, Unix ms.
    pub modified: u64,
    pub size: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    #[serde(rename = "$schema", default)]
    schema: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_id: Option<String>,
    #[serde(default)]
    files: BTreeMap<String, Entry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    changes_token: Option<String>,
    /// Anything else a newer version wrote.
    #[serde(flatten)]
    other: Map<String, Json>,
}

/// The index of this device.
pub struct SyncIndex {
    file: PathBuf,
    data: Data,
    /// Why the file could not be read, while it cannot: no write may replace it.
    read_failed: Option<(IoCause, String)>,
}

enum Read {
    Found(Data),
    Missing,
    Failed(IoCause, String),
}

impl SyncIndex {
    /// Opens `<dir>/index.json` (empty if missing).
    pub fn open(dir: &Path) -> Self {
        let mut index = Self {
            file: dir.join("index.json"),
            data: Data::default(),
            read_failed: None,
        };
        index.read();
        index
    }

    pub fn path(&self) -> &Path {
        &self.file
    }

    fn read(&mut self) {
        match read_data(&self.file) {
            Read::Found(data) => {
                self.data = data;
                self.read_failed = None;
            }
            Read::Missing => self.read_failed = None,
            Read::Failed(cause, detail) => {
                log::warn!(
                    "cloudIndexReadFailed: {}: {detail}; sync refused until it reads",
                    self.file.display()
                );
                self.read_failed = Some((cause, detail));
            }
        }
    }

    /// `Err(IndexReadFailed)` while the file cannot be read (tried again first): nothing may be
    /// decided from an index that is not the one on disk.
    pub fn readable(&mut self) -> Result<(), SyncError> {
        if self.read_failed.is_some() {
            self.read();
        }
        match &self.read_failed {
            Some((cause, detail)) => Err(SyncError::IndexReadFailed(*cause, detail.clone())),
            None => Ok(()),
        }
    }

    /// The entry of `path`.
    pub fn get(&self, path: &str) -> Option<&Entry> {
        self.data.files.get(path)
    }

    /// Every entry under `prefix` (`gba/<hex>/`).
    pub fn under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (&'a str, &'a Entry)> {
        self.data
            .files
            .range(prefix.to_owned()..)
            .take_while(move |(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Records `path` as synced and writes the index.
    pub fn set(&mut self, path: &str, entry: Entry) -> Result<(), SyncError> {
        self.readable()?;
        let before = self.data.files.insert(path.to_owned(), entry);
        self.save().inspect_err(|_| {
            // What is in memory stays what is on disk.
            match before {
                Some(old) => self.data.files.insert(path.to_owned(), old),
                None => self.data.files.remove(path),
            };
        })
    }

    /// Forgets `path` and writes the index.
    pub fn remove(&mut self, path: &str) -> Result<(), SyncError> {
        self.readable()?;
        let Some(before) = self.data.files.remove(path) else {
            return Ok(());
        };
        self.save().inspect_err(|_| {
            self.data.files.insert(path.to_owned(), before);
        })
    }

    /// This device's id: made once (random) and kept.
    pub fn device_id(&mut self) -> Result<String, SyncError> {
        self.readable()?;
        if let Some(id) = &self.data.device_id {
            return Ok(id.clone());
        }
        let id = crate::oauth::random_token(12).map_err(SyncError::local_other)?;
        self.data.device_id = Some(id.clone());
        self.save().inspect_err(|_| self.data.device_id = None)?;
        Ok(id)
    }

    /// The `changes.list` page token of the last sync.
    pub fn changes_token(&self) -> Option<&str> {
        self.data.changes_token.as_deref()
    }

    pub fn set_changes_token(&mut self, token: &str) -> Result<(), SyncError> {
        self.readable()?;
        let before = self.data.changes_token.replace(token.to_owned());
        self.save()
            .inspect_err(|_| self.data.changes_token = before)
    }

    fn save(&mut self) -> Result<(), SyncError> {
        self.data.schema = self.data.schema.max(SCHEMA_VERSION);
        let mut text = serde_json::to_vec_pretty(&self.data)
            .map_err(|e| SyncError::local_other(format!("sync index: {e}")))?;
        text.push(b'\n');
        emudome_core::write_atomic(&self.file, &text).map_err(|e| SyncError::local(&self.file, &e))
    }
}

fn read_data(path: &Path) -> Read {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Read::Missing
        }
        Err(e) => return Read::Failed(IoCause::of(&e), e.to_string()),
    };
    match serde_json::from_str(&text) {
        Ok(data) => Read::Found(data),
        Err(e) => {
            let aside = path.with_extension("json.unreadable");
            log::warn!(
                "cloudIndexUnreadable: {}: {e}; moved to {}, starting empty",
                path.display(),
                aside.display()
            );
            match fs::rename(path, &aside) {
                Ok(()) => Read::Missing,
                Err(not_moved) => Read::Failed(
                    IoCause::of(&not_moved),
                    format!("not valid JSON ({e}) and not moved aside ({not_moved})"),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::{json, Value as Json};

    use super::{Entry, SyncIndex};
    use crate::test_http::TestDir;
    use crate::SyncError;

    fn entry(md5: &str) -> Entry {
        Entry {
            id: "id1".into(),
            md5: md5.into(),
            modified: 5,
            size: 3,
        }
    }

    fn on_disk(dir: &TestDir) -> Json {
        serde_json::from_slice(&fs::read(dir.path("sync/index.json")).unwrap()).unwrap()
    }

    #[test]
    fn keeps_entries_the_device_and_unknown_keys() {
        let dir = TestDir::new("index-keep");
        dir.write(
            "sync/index.json",
            br#"{ "$schema": 1, "future": [1], "files": {} }"#,
        );
        let mut index = SyncIndex::open(&dir.path("sync"));
        index.set("gba/1a/game.sav", entry("aa")).unwrap();
        let device = index.device_id().unwrap();
        assert_eq!(index.device_id().unwrap(), device, "made once");
        index.set_changes_token("42").unwrap();
        let json = on_disk(&dir);
        assert_eq!(json["$schema"], json!(1));
        assert_eq!(json["future"], json!([1]));
        assert_eq!(json["deviceId"], json!(device));
        assert_eq!(json["changesToken"], json!("42"));
        assert_eq!(
            json["files"]["gba/1a/game.sav"],
            json!({ "id": "id1", "md5": "aa", "modified": 5, "size": 3 })
        );
        let again = SyncIndex::open(&dir.path("sync"));
        assert_eq!(again.get("gba/1a/game.sav"), Some(&entry("aa")));
        assert_eq!(again.changes_token(), Some("42"));
        let mut again = again;
        again.set("gba/1a/slot1.state", entry("bb")).unwrap();
        again.set("gba/1b/game.sav", entry("cc")).unwrap();
        let under: Vec<&str> = again.under("gba/1a/").map(|(k, _)| k).collect();
        assert_eq!(under, vec!["gba/1a/game.sav", "gba/1a/slot1.state"]);
        again.remove("gba/1a/game.sav").unwrap();
        assert_eq!(
            SyncIndex::open(&dir.path("sync")).get("gba/1a/game.sav"),
            None
        );
    }

    #[test]
    fn an_index_that_is_not_json_is_moved_aside_and_starts_empty() {
        let dir = TestDir::new("index-garbage");
        dir.write("sync/index.json", b"{ not json");
        let mut index = SyncIndex::open(&dir.path("sync"));
        index.readable().unwrap();
        assert_eq!(index.get("x"), None);
        assert_eq!(
            fs::read(dir.path("sync/index.json.unreadable")).unwrap(),
            b"{ not json"
        );
        index.set("x", entry("aa")).unwrap();
    }

    #[test]
    fn an_index_that_cannot_be_read_is_never_written_over() {
        let dir = TestDir::new("index-unreadable");
        // A folder where the file should be: there, but not readable as a file.
        fs::create_dir_all(dir.path("sync/index.json/inside")).unwrap();
        let mut index = SyncIndex::open(&dir.path("sync"));
        assert!(matches!(
            index.set("x", entry("aa")),
            Err(SyncError::IndexReadFailed(..))
        ));
        assert!(matches!(
            index.device_id(),
            Err(SyncError::IndexReadFailed(..))
        ));
        assert!(dir.path("sync/index.json/inside").is_dir(), "untouched");
        // Once it reads again, writes work.
        fs::remove_dir_all(dir.path("sync/index.json")).unwrap();
        index.set("x", entry("aa")).unwrap();
        assert_eq!(on_disk(&dir)["files"]["x"]["md5"], json!("aa"));
    }

    #[test]
    fn a_failed_write_leaves_memory_as_the_disk() {
        let dir = TestDir::new("index-write-fails");
        let mut index = SyncIndex::open(&dir.path("sync"));
        index.set("a", entry("aa")).unwrap();
        // The folder becomes a file: the next write cannot happen.
        fs::remove_dir_all(dir.path("sync")).unwrap();
        fs::write(dir.path("sync"), b"").unwrap();
        assert!(matches!(
            index.set("b", entry("bb")),
            Err(SyncError::Local(..))
        ));
        assert_eq!(index.get("b"), None);
        assert_eq!(index.get("a"), Some(&entry("aa")));
    }
}
