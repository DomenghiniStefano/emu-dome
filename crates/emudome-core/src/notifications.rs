//! The notification centre (X03, Trello #32): what happened while the user was elsewhere (a
//! controller connected, later a sync done or a cover not found), kept until read. In game only the
//! short toasts show (G13); the notification is recorded and found in the panel later.
//!
//! Per device, never synced: `<app local data>/notifications.json`, written through
//! [`crate::write_atomic`]. The file says its layout (`"$schema": 1`); one that is there but cannot
//! be read (an I/O error, broken JSON, a newer layout) is never written over: [`Store`] refuses
//! every change with [`ErrorCode::NotificationsReadFailed`] and reads it again at the next call.
//! An item of a kind this build does not know (written by a newer one) is kept as it is and not
//! listed. At most [`MAX_KEPT`] items are kept, the oldest dropped first.
//!
//! Items carry codes and values, never sentences: the interface writes the text (i18n).

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use ts_rs::TS;

use crate::ErrorCode;

/// The layout written today.
pub const SCHEMA: u64 = 1;
/// How many notifications are kept, read or not.
pub const MAX_KEPT: usize = 100;

/// What a notification says. TypeScript: `Notice` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Notice {
    /// A controller was connected while EmuDome ran (`name`: the device's own name, as SDL
    /// reports it; empty when it has none).
    ControllerConnected { name: String },
}

/// One notification. TypeScript: `Notification` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: u64,
    /// When it happened, in ms since the Unix epoch.
    pub at: u64,
    pub read: bool,
    pub notice: Notice,
}

/// An item of the file: one this build reads, or one it keeps as it is (a newer kind).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Item {
    Known(Notification),
    Other(Json),
}

impl Item {
    fn id(&self) -> Option<u64> {
        match self {
            Item::Known(n) => Some(n.id),
            Item::Other(v) => v.get("id").and_then(Json::as_u64),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
    #[serde(rename = "$schema", default)]
    schema: u64,
    #[serde(rename = "nextId", default)]
    next_id: u64,
    #[serde(default)]
    items: Vec<Item>,
}

/// The notifications in memory, newest first.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Notifications {
    items: Vec<Item>,
    next_id: u64,
}

impl Notifications {
    /// Reads a file's bytes. A newer layout is refused: this build cannot know what it means.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let file: File = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if file.schema > SCHEMA {
            return Err(format!("layout {} is newer than {SCHEMA}", file.schema));
        }
        let highest = file.items.iter().filter_map(Item::id).max();
        Ok(Self {
            next_id: file.next_id.max(highest.map_or(0, |h| h + 1)),
            items: file.items,
        })
    }

    /// The file's bytes, in today's layout.
    pub fn to_bytes(&self) -> Vec<u8> {
        let file = File {
            schema: SCHEMA,
            next_id: self.next_id,
            items: self.items.clone(),
        };
        let mut bytes = serde_json::to_vec_pretty(&file).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    }

    /// Adds a notification, unread, as the newest; the oldest beyond [`MAX_KEPT`] go.
    pub fn push(&mut self, notice: Notice, at: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.insert(
            0,
            Item::Known(Notification {
                id,
                at,
                read: false,
                notice,
            }),
        );
        self.items.truncate(MAX_KEPT);
        id
    }

    /// Marks one read; false when it was already read or is not there.
    pub fn mark_read(&mut self, id: u64) -> bool {
        let mut changed = false;
        for item in &mut self.items {
            if let Item::Known(n) = item {
                if n.id == id && !n.read {
                    n.read = true;
                    changed = true;
                }
            }
        }
        changed
    }

    /// Marks every one read; false when none was unread.
    pub fn mark_all_read(&mut self) -> bool {
        let mut changed = false;
        for item in &mut self.items {
            if let Item::Known(n) = item {
                changed |= !n.read;
                n.read = true;
            }
        }
        changed
    }

    /// The ones this build reads, newest first.
    pub fn list(&self) -> Vec<Notification> {
        self.items
            .iter()
            .filter_map(|i| match i {
                Item::Known(n) => Some(n.clone()),
                Item::Other(_) => None,
            })
            .collect()
    }
}

/// The notifications file and what was read from it.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    /// `Err`: the file is there but could not be read; it is read again at the next call.
    state: Result<Notifications, String>,
}

impl Store {
    /// Reads `path`; a missing file is an empty list.
    pub fn open(path: PathBuf) -> Self {
        let state = read(&path);
        if let Err(e) = &state {
            log::warn!("notifications not read ({}): {e}", path.display());
        }
        Self { path, state }
    }

    /// What was read, reading the file again if it could not be read before.
    fn ready(&mut self) -> Result<&mut Notifications, ErrorCode> {
        if self.state.is_err() {
            self.state = read(&self.path);
        }
        self.state
            .as_mut()
            .map_err(|detail| ErrorCode::NotificationsReadFailed {
                detail: detail.clone(),
            })
    }

    pub fn list(&mut self) -> Result<Vec<Notification>, ErrorCode> {
        Ok(self.ready()?.list())
    }

    /// Records a notification and writes the file.
    pub fn record(&mut self, notice: Notice, at: u64) -> Result<u64, ErrorCode> {
        let id = self.ready()?.push(notice, at);
        self.save()?;
        Ok(id)
    }

    /// Marks one read; true when it changed (the file is written then).
    pub fn mark_read(&mut self, id: u64) -> Result<bool, ErrorCode> {
        let changed = self.ready()?.mark_read(id);
        if changed {
            self.save()?;
        }
        Ok(changed)
    }

    /// Marks every one read; true when one changed (the file is written then).
    pub fn mark_all_read(&mut self) -> Result<bool, ErrorCode> {
        let changed = self.ready()?.mark_all_read();
        if changed {
            self.save()?;
        }
        Ok(changed)
    }

    fn save(&mut self) -> Result<(), ErrorCode> {
        let bytes = self.ready()?.to_bytes();
        write(&self.path, &bytes).map_err(|e| ErrorCode::NotificationsWriteFailed {
            detail: e.to_string(),
        })
    }
}

fn read(path: &Path) -> Result<Notifications, String> {
    match std::fs::read(path) {
        Ok(bytes) => Notifications::parse(&bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Notifications::default()),
        Err(e) => Err(e.to_string()),
    }
}

fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    crate::write_atomic(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::{Notice, Notifications, Store, MAX_KEPT};
    use crate::settings::test_dir::TestDir;
    use crate::ErrorCode;

    fn pad(name: &str) -> Notice {
        Notice::ControllerConnected { name: name.into() }
    }

    fn scratch() -> TestDir {
        TestDir::new("notifications")
    }

    #[test]
    fn a_missing_file_is_an_empty_list() {
        let dir = scratch();
        let mut store = Store::open(dir.path("notifications.json"));
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn what_is_recorded_survives_a_restart_newest_first() {
        let dir = scratch();
        let path = dir.path("deep/notifications.json");
        let mut store = Store::open(path.clone());
        let first = store.record(pad("8BitDo"), 10).unwrap();
        let second = store.record(pad("Xbox"), 20).unwrap();
        assert!(store.mark_read(first).unwrap());
        drop(store);

        let mut again = Store::open(path.clone());
        let list = again.list().unwrap();
        assert_eq!(
            list.iter().map(|n| (n.id, n.read)).collect::<Vec<_>>(),
            vec![(second, false), (first, true)]
        );
        assert_eq!(list[0].notice, pad("Xbox"));
        // New ids never reuse an old one.
        assert!(again.record(pad("Steam"), 30).unwrap() > second);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"$schema\": 1"), "{text}");
    }

    #[test]
    fn an_unreadable_file_is_never_written_over_and_is_read_again_later() {
        let dir = scratch();
        let path = dir.path("notifications.json");
        std::fs::write(&path, b"{ broken").unwrap();
        let mut store = Store::open(path.clone());
        assert!(matches!(
            store.record(pad("Xbox"), 1),
            Err(ErrorCode::NotificationsReadFailed { .. })
        ));
        assert!(matches!(
            store.mark_all_read(),
            Err(ErrorCode::NotificationsReadFailed { .. })
        ));
        assert!(store.list().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{ broken");
        // Fixed on disk: the next call reads it and works again.
        std::fs::write(&path, br#"{ "$schema": 1, "items": [] }"#).unwrap();
        store.record(pad("Xbox"), 2).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn a_newer_layout_is_unreadable() {
        let dir = scratch();
        let path = dir.path("notifications.json");
        let newer = br#"{ "$schema": 2, "items": [] }"#;
        std::fs::write(&path, newer).unwrap();
        let mut store = Store::open(path.clone());
        assert!(store.record(pad("Xbox"), 1).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), newer);
    }

    #[test]
    fn a_kind_this_build_does_not_know_is_kept_and_not_listed() {
        let bytes = br#"{ "$schema": 1, "nextId": 3, "items": [
            { "id": 7, "at": 5, "read": false, "notice": { "kind": "syncDone", "games": 3 } },
            { "id": 2, "at": 4, "read": true, "notice": { "kind": "controllerConnected", "name": "Pad" } }
        ] }"#;
        let mut n = Notifications::parse(bytes).unwrap();
        assert_eq!(n.list().len(), 1);
        // The next id is past every id in the file, known or not.
        assert_eq!(n.push(pad("Xbox"), 9), 8);
        let written = String::from_utf8(n.to_bytes()).unwrap();
        assert!(written.contains("syncDone"), "{written}");
        assert!(n.mark_all_read());
        assert!(!n.mark_all_read());
        assert!(String::from_utf8(n.to_bytes())
            .unwrap()
            .contains("syncDone"));
    }

    #[test]
    fn at_most_max_kept_stay_the_oldest_go_first() {
        let mut n = Notifications::default();
        for at in 0..(MAX_KEPT as u64 + 5) {
            n.push(pad("Pad"), at);
        }
        let list = n.list();
        assert_eq!(list.len(), MAX_KEPT);
        assert_eq!(list[0].at, MAX_KEPT as u64 + 4);
        assert_eq!(list[MAX_KEPT - 1].at, 5);
    }

    #[test]
    fn marking_read_tells_whether_anything_changed() {
        let mut n = Notifications::default();
        let id = n.push(pad("Pad"), 1);
        assert!(n.mark_read(id));
        assert!(!n.mark_read(id));
        assert!(!n.mark_read(99));
        assert!(!n.mark_all_read());
    }

    #[test]
    fn serializes_as_the_interface_reads_it() {
        let mut n = Notifications::default();
        n.push(pad("Xbox"), 42);
        assert_eq!(
            serde_json::to_value(n.list()).unwrap(),
            serde_json::json!([{ "id": 0, "at": 42, "read": false,
                "notice": { "kind": "controllerConnected", "name": "Xbox" } }])
        );
    }
}
