//! [`DriveApi`] in memory, for the tests of the sync logic (and of the app later): files with
//! content, `appProperties`, md5, a modification clock, a quota, the changes log, and failure
//! injection (offline from the n-th call on). A failed call changes nothing, as on Drive.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::Mutex;

use super::{
    md5_hex, Account, Change, ChangesPage, Content, Downloaded, DriveApi, Properties, Quota,
    RemoteFile, ROOT,
};
use crate::SyncError;

/// Drive in memory. A clone is the same Drive (two devices of one account).
#[derive(Clone, Default)]
pub struct FakeDrive {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    files: BTreeMap<String, Entry>,
    next_id: u64,
    /// Unix ms of the next write; every write moves it on by a second.
    clock: u64,
    /// File ids in the order they changed (`changes.list`); a token is an index into it.
    log: Vec<(String, bool)>,
    /// Calls still answered before going offline; `None`: always online.
    calls_left: Option<usize>,
    limit: Option<u64>,
    /// What [`DriveApi::trash`] moved to the trash, oldest first.
    trashed: Vec<RemoteFile>,
}

struct Entry {
    file: RemoteFile,
    content: Vec<u8>,
    created: u64,
}

impl FakeDrive {
    pub fn new() -> Self {
        let drive = Self::default();
        drive.state.lock().clock = 1_700_000_000_000;
        drive
    }

    /// Answers `n` more calls, then every call fails with [`SyncError::Offline`]
    /// (`None`: online again).
    pub fn offline_after(&self, n: Option<usize>) {
        self.state.lock().calls_left = n;
    }

    /// The account's space limit in bytes (`None`: unlimited, the default).
    pub fn set_limit(&self, limit: Option<u64>) {
        self.state.lock().limit = limit;
    }

    /// The content of the file `id`.
    pub fn content(&self, id: &str) -> Option<Vec<u8>> {
        self.state.lock().files.get(id).map(|e| e.content.clone())
    }

    /// The file at `path` (names from «My Drive»), if there is one.
    pub fn at(&self, path: &[&str]) -> Option<RemoteFile> {
        let state = self.state.lock();
        let mut parent = ROOT.to_owned();
        let mut found = None;
        for name in path {
            let next = state.child(&parent, name)?;
            parent = next.file.id.clone();
            found = Some(next.file.clone());
        }
        found
    }

    /// Every file and folder, as `(path, file)`, paths `/`-joined from «My Drive».
    pub fn tree(&self) -> Vec<(String, RemoteFile)> {
        let state = self.state.lock();
        let mut out: Vec<(String, RemoteFile)> = state
            .files
            .values()
            .map(|e| (state.path_of(&e.file), e.file.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Changes a file as another program would (the user on drive.google.com): new content,
    /// same id.
    pub fn overwrite(&self, id: &str, content: &[u8]) {
        let mut state = self.state.lock();
        let now = state.tick();
        if let Some(e) = state.files.get_mut(id) {
            e.content = content.to_vec();
            e.file.md5 = Some(md5_hex(content));
            e.file.size = Some(content.len() as u64);
            e.file.modified = now;
        }
        state.log.push((id.to_owned(), false));
    }

    /// The files moved to the trash, oldest first.
    pub fn trashed(&self) -> Vec<RemoteFile> {
        self.state.lock().trashed.clone()
    }

    /// Removes a file as another program would.
    pub fn remove(&self, id: &str) {
        let mut state = self.state.lock();
        state.files.remove(id);
        state.log.push((id.to_owned(), true));
    }

    /// One call: refused once offline.
    fn call(&self) -> Result<parking_lot::MutexGuard<'_, State>, SyncError> {
        let mut state = self.state.lock();
        match state.calls_left {
            Some(0) => Err(SyncError::Offline("fake: offline".into())),
            Some(n) => {
                state.calls_left = Some(n - 1);
                Ok(state)
            }
            None => Ok(state),
        }
    }
}

impl State {
    fn tick(&mut self) -> u64 {
        self.clock += 1000;
        self.clock
    }

    fn child(&self, parent: &str, name: &str) -> Option<&Entry> {
        self.children(parent)
            .into_iter()
            .find(|e| e.file.name == name)
    }

    /// Oldest first, as the REST client asks Drive.
    fn children(&self, parent: &str) -> Vec<&Entry> {
        let mut out: Vec<&Entry> = self
            .files
            .values()
            .filter(|e| e.file.parents.iter().any(|p| p == parent))
            .collect();
        out.sort_by_key(|e| (e.created, e.file.id.clone()));
        out
    }

    fn path_of(&self, file: &RemoteFile) -> String {
        let mut names = vec![file.name.clone()];
        let mut parent = file.parents.first().cloned();
        while let Some(p) = parent {
            match self.files.get(&p) {
                Some(e) => {
                    names.push(e.file.name.clone());
                    parent = e.file.parents.first().cloned();
                }
                None => break,
            }
        }
        names.reverse();
        names.join("/")
    }

    fn usage(&self) -> u64 {
        self.files.values().map(|e| e.content.len() as u64).sum()
    }

    fn check_room(&self, adding: u64, replacing: u64) -> Result<(), SyncError> {
        let Some(limit) = self.limit else {
            return Ok(());
        };
        let after = self.usage() - replacing + adding;
        if after > limit {
            return Err(SyncError::QuotaFull {
                needed: adding,
                available: limit.saturating_sub(self.usage()),
            });
        }
        Ok(())
    }

    fn parent_exists(&self, parent: &str) -> Result<(), SyncError> {
        if parent == ROOT || self.files.get(parent).is_some_and(|e| e.file.folder) {
            Ok(())
        } else {
            Err(SyncError::NotFound(format!("folder {parent}")))
        }
    }

    fn insert(&mut self, mut file: RemoteFile, content: Vec<u8>) -> RemoteFile {
        self.next_id += 1;
        file.id = format!("id{}", self.next_id);
        file.modified = self.tick();
        if !file.folder {
            file.md5 = Some(md5_hex(&content));
            file.size = Some(content.len() as u64);
        }
        let created = file.modified;
        self.log.push((file.id.clone(), false));
        self.files.insert(
            file.id.clone(),
            Entry {
                file: file.clone(),
                content,
                created,
            },
        );
        file
    }

    fn entry_mut(&mut self, id: &str) -> Result<&mut Entry, SyncError> {
        self.files
            .get_mut(id)
            .ok_or_else(|| SyncError::NotFound(format!("file {id}")))
    }
}

async fn bytes_of(content: Content) -> Result<Vec<u8>, SyncError> {
    match content {
        Content::Bytes(b) => Ok(b),
        Content::File { path, .. } => tokio::fs::read(&path)
            .await
            .map_err(|e| SyncError::Local(format!("{}: {e}", path.display()))),
    }
}

impl DriveApi for FakeDrive {
    async fn list(&self, parent: &str) -> Result<Vec<RemoteFile>, SyncError> {
        let state = self.call()?;
        Ok(state
            .children(parent)
            .into_iter()
            .map(|e| e.file.clone())
            .collect())
    }

    async fn find(&self, parent: &str, name: &str) -> Result<Option<RemoteFile>, SyncError> {
        let state = self.call()?;
        Ok(state.child(parent, name).map(|e| e.file.clone()))
    }

    async fn get(&self, id: &str) -> Result<RemoteFile, SyncError> {
        let state = self.call()?;
        state
            .files
            .get(id)
            .map(|e| e.file.clone())
            .ok_or_else(|| SyncError::NotFound(format!("file {id}")))
    }

    async fn create_folder(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
    ) -> Result<RemoteFile, SyncError> {
        let mut state = self.call()?;
        state.parent_exists(parent)?;
        Ok(state.insert(
            RemoteFile {
                name: name.to_owned(),
                parents: vec![parent.to_owned()],
                folder: true,
                app_properties: props.clone(),
                ..RemoteFile::default()
            },
            Vec::new(),
        ))
    }

    async fn create(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
        content: Content,
    ) -> Result<RemoteFile, SyncError> {
        let bytes = bytes_of(content).await?;
        let mut state = self.call()?;
        state.parent_exists(parent)?;
        state.check_room(bytes.len() as u64, 0)?;
        Ok(state.insert(
            RemoteFile {
                name: name.to_owned(),
                parents: vec![parent.to_owned()],
                app_properties: props.clone(),
                ..RemoteFile::default()
            },
            bytes,
        ))
    }

    async fn update(
        &self,
        id: &str,
        props: &Properties,
        content: Content,
    ) -> Result<RemoteFile, SyncError> {
        let bytes = bytes_of(content).await?;
        let mut state = self.call()?;
        let old = state.entry_mut(id)?.content.len() as u64;
        state.check_room(bytes.len() as u64, old)?;
        let now = state.tick();
        let e = state.entry_mut(id)?;
        e.file.md5 = Some(md5_hex(&bytes));
        e.file.size = Some(bytes.len() as u64);
        e.file.modified = now;
        e.file
            .app_properties
            .extend(props.iter().map(|(k, v)| (k.clone(), v.clone())));
        e.content = bytes;
        let file = e.file.clone();
        state.log.push((id.to_owned(), false));
        Ok(file)
    }

    async fn set_properties(
        &self,
        id: &str,
        props: &BTreeMap<String, Option<String>>,
    ) -> Result<RemoteFile, SyncError> {
        let mut state = self.call()?;
        let e = state.entry_mut(id)?;
        for (k, v) in props {
            match v {
                Some(v) => {
                    e.file.app_properties.insert(k.clone(), v.clone());
                }
                None => {
                    e.file.app_properties.remove(k);
                }
            }
        }
        Ok(e.file.clone())
    }

    async fn copy(&self, id: &str, parent: &str, name: &str) -> Result<RemoteFile, SyncError> {
        let mut state = self.call()?;
        state.parent_exists(parent)?;
        let (file, content) = {
            let e = state
                .files
                .get(id)
                .ok_or_else(|| SyncError::NotFound(format!("file {id}")))?;
            (e.file.clone(), e.content.clone())
        };
        state.check_room(content.len() as u64, 0)?;
        Ok(state.insert(
            RemoteFile {
                name: name.to_owned(),
                parents: vec![parent.to_owned()],
                app_properties: file.app_properties,
                ..RemoteFile::default()
            },
            content,
        ))
    }

    async fn download(&self, id: &str) -> Result<Vec<u8>, SyncError> {
        let state = self.call()?;
        state
            .files
            .get(id)
            .map(|e| e.content.clone())
            .ok_or_else(|| SyncError::NotFound(format!("file {id}")))
    }

    async fn download_to(
        &self,
        id: &str,
        to: &Path,
        progress: &(dyn Fn(u64) + Send + Sync),
    ) -> Result<Downloaded, SyncError> {
        let bytes = self.download(id).await?;
        tokio::fs::write(to, &bytes)
            .await
            .map_err(|e| SyncError::Local(format!("{}: {e}", to.display())))?;
        progress(bytes.len() as u64);
        Ok(Downloaded {
            md5: md5_hex(&bytes),
            size: bytes.len() as u64,
        })
    }

    async fn list_all(&self) -> Result<Vec<RemoteFile>, SyncError> {
        let state = self.call()?;
        let mut out: Vec<&Entry> = state.files.values().collect();
        out.sort_by_key(|e| (e.created, e.file.id.clone()));
        Ok(out.into_iter().map(|e| e.file.clone()).collect())
    }

    async fn trash(&self, id: &str) -> Result<(), SyncError> {
        let mut state = self.call()?;
        let entry = state
            .files
            .remove(id)
            .ok_or_else(|| SyncError::NotFound(format!("file {id}")))?;
        state.trashed.push(entry.file);
        state.log.push((id.to_owned(), true));
        Ok(())
    }

    async fn quota(&self) -> Result<Quota, SyncError> {
        let state = self.call()?;
        Ok(Quota {
            limit: state.limit,
            usage: state.usage(),
        })
    }

    async fn account(&self) -> Result<Account, SyncError> {
        let _state = self.call()?;
        Ok(Account {
            name: Some("Fake".into()),
            email: Some("fake@example.com".into()),
        })
    }

    async fn changes_start(&self) -> Result<String, SyncError> {
        let state = self.call()?;
        Ok(state.log.len().to_string())
    }

    async fn changes(&self, token: &str) -> Result<ChangesPage, SyncError> {
        let state = self.call()?;
        let from: usize = token
            .parse()
            .map_err(|_| SyncError::Request(format!("bad page token {token:?}")))?;
        let mut changes: Vec<Change> = Vec::new();
        for (id, removed) in state.log.iter().skip(from) {
            changes.retain(|c| &c.file_id != id);
            changes.push(Change {
                file_id: id.clone(),
                removed: *removed,
                file: state.files.get(id).map(|e| e.file.clone()),
            });
        }
        Ok(ChangesPage {
            changes,
            next_page: None,
            new_start: Some(state.log.len().to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FakeDrive;
    use crate::drive::{md5_hex, Content, DriveApi, Properties, ROOT};
    use crate::SyncError;

    #[tokio::test]
    async fn behaves_like_drive_for_what_the_sync_uses() {
        let d = FakeDrive::new();
        let start = d.changes_start().await.unwrap();
        let root = d
            .create_folder(ROOT, "EmuDome", &Properties::new())
            .await
            .unwrap();
        let f = d
            .create(
                &root.id,
                "game.sav",
                &Properties::new(),
                Content::Bytes(b"one".to_vec()),
            )
            .await
            .unwrap();
        assert_eq!(f.md5.as_deref(), Some(md5_hex(b"one").as_str()));
        assert_eq!(d.find(&root.id, "game.sav").await.unwrap(), Some(f.clone()));
        assert_eq!(d.at(&["EmuDome", "game.sav"]), Some(f.clone()));
        let mut props = Properties::new();
        props.insert("device".into(), "a".into());
        let g = d
            .update(&f.id, &props, Content::Bytes(b"two".to_vec()))
            .await
            .unwrap();
        assert!(g.modified > f.modified);
        assert_eq!(g.app_properties["device"], "a");
        assert_eq!(d.download(&f.id).await.unwrap(), b"two");
        let c = d.copy(&f.id, &root.id, "game (copy).sav").await.unwrap();
        assert_eq!(d.content(&c.id).unwrap(), b"two");
        assert_eq!(d.list(&root.id).await.unwrap().len(), 2);
        assert_eq!(d.quota().await.unwrap().usage, 6);
        let page = d.changes(&start).await.unwrap();
        assert_eq!(page.changes.len(), 3, "the folder, the file, its copy");
        let page = d.changes(page.new_start.as_deref().unwrap()).await.unwrap();
        assert!(page.changes.is_empty());
        assert!(matches!(
            d.create("nope", "x", &Properties::new(), Content::Bytes(vec![]))
                .await,
            Err(SyncError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn quota_and_offline_refuse_without_changing_anything() {
        let d = FakeDrive::new();
        d.set_limit(Some(4));
        let f = d
            .create(
                ROOT,
                "a",
                &Properties::new(),
                Content::Bytes(b"abc".to_vec()),
            )
            .await
            .unwrap();
        assert_eq!(
            d.update(&f.id, &Properties::new(), Content::Bytes(b"abcde".to_vec()))
                .await
                .unwrap_err(),
            SyncError::QuotaFull {
                needed: 5,
                available: 1
            }
        );
        assert_eq!(d.content(&f.id).unwrap(), b"abc");
        d.offline_after(Some(1));
        d.get(&f.id).await.unwrap();
        assert!(matches!(d.get(&f.id).await, Err(SyncError::Offline(_))));
        d.offline_after(None);
        d.get(&f.id).await.unwrap();
    }
}
