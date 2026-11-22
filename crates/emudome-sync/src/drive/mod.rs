//! Google Drive as the sync sees it: the [`DriveApi`] trait, its REST v3 implementation
//! ([`rest::RestDrive`]) and an in-memory one for the tests ([`fake::FakeDrive`]), so the sync
//! logic is tested without the network.
//!
//! Only what the `drive.file` scope allows: the files EmuDome created. Files are compared by
//! their `md5Checksum`, never by downloading them; `appProperties` (private to the app) carry
//! the game id, the device, the play time and the core version.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

use crate::oauth::Secret;
use crate::SyncError;

pub mod fake;
pub mod rest;

/// Drive's MIME type of a folder.
pub const FOLDER_MIME: &str = "application/vnd.google-apps.folder";

/// The id of the user's «My Drive» in requests.
pub const ROOT: &str = "root";

/// `appProperties` of a file or folder.
pub type Properties = BTreeMap<String, String>;

/// A file or folder on Drive.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RemoteFile {
    pub id: String,
    pub name: String,
    pub parents: Vec<String>,
    pub folder: bool,
    /// Lowercase hex; files only (Drive gives none for folders).
    pub md5: Option<String>,
    pub size: Option<u64>,
    /// `modifiedTime`, Unix ms.
    pub modified: u64,
    pub app_properties: Properties,
}

/// What a new or updated file holds.
#[derive(Clone, Debug)]
pub enum Content {
    Bytes(Vec<u8>),
    /// Read from the disk in chunks: a file too big to hold in memory (a 3DS state, a game).
    File {
        path: PathBuf,
        size: u64,
    },
}

impl Content {
    pub fn len(&self) -> u64 {
        match self {
            Content::Bytes(b) => b.len() as u64,
            Content::File { size, .. } => *size,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The Drive space of the account (`about.storageQuota`), in bytes. TypeScript: `DriveQuota`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "DriveQuota")]
pub struct Quota {
    /// `None`: unlimited (some Workspace accounts).
    pub limit: Option<u64>,
    /// Everything the account uses (Drive, Gmail, Photos).
    pub usage: u64,
}

impl Quota {
    /// Bytes still free; `None` when unlimited.
    pub fn available(&self) -> Option<u64> {
        self.limit.map(|l| l.saturating_sub(self.usage))
    }

    /// Whether `bytes` more fit: what the interface asks before uploading a game.
    pub fn fits(&self, bytes: u64) -> bool {
        self.available().is_none_or(|a| bytes <= a)
    }
}

/// What [`DriveApi::download_to`] wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Downloaded {
    /// Lowercase hex, as Drive's `md5Checksum`.
    pub md5: String,
    pub size: u64,
}

/// Who is signed in (`about.user`), for the account row.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Account {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// One entry of `changes.list`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub file_id: String,
    /// The file is gone for the app (deleted, or no longer shared with it).
    pub removed: bool,
    pub file: Option<RemoteFile>,
}

/// A page of changes. Keep `new_start` (on the last page) for the next sync.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangesPage {
    pub changes: Vec<Change>,
    pub next_page: Option<String>,
    pub new_start: Option<String>,
}

/// What the sync asks of Drive. Every method may fail with [`SyncError::Offline`],
/// [`SyncError::AccessExpired`], [`SyncError::NotFound`], [`SyncError::QuotaFull`] or
/// [`SyncError::Request`]; a failed write leaves the file on Drive as it was.
pub trait DriveApi: Send + Sync {
    /// The files and folders in `parent` (not trashed), oldest first.
    fn list(&self, parent: &str)
        -> impl Future<Output = Result<Vec<RemoteFile>, SyncError>> + Send;

    /// The oldest child of `parent` named `name` (two devices may have created the same folder
    /// at once: the oldest is the one everybody uses).
    fn find(
        &self,
        parent: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<RemoteFile>, SyncError>> + Send;

    fn get(&self, id: &str) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    fn create_folder(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
    ) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    /// A new file in `parent`.
    fn create(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
        content: Content,
    ) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    /// New content for the file `id`; `props` are merged into its `appProperties`.
    fn update(
        &self,
        id: &str,
        props: &Properties,
        content: Content,
    ) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    /// Changes `appProperties` only: `Some` sets a key, `None` removes it.
    fn set_properties(
        &self,
        id: &str,
        props: &BTreeMap<String, Option<String>>,
    ) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    /// A copy of the file `id` named `name` in `parent`, made on Drive (nothing downloaded).
    fn copy(
        &self,
        id: &str,
        parent: &str,
        name: &str,
    ) -> impl Future<Output = Result<RemoteFile, SyncError>> + Send;

    /// The content of the file `id`.
    fn download(&self, id: &str) -> impl Future<Output = Result<Vec<u8>, SyncError>> + Send;

    /// The content of the file `id` streamed into the new file `to` (made, or emptied), never
    /// held whole in memory (a game of several GB); `progress` gets the bytes written so far.
    /// Returns what was written: its MD5 (to compare with Drive's) and size. The caller owns
    /// `to` (a temporary name, renamed once checked).
    fn download_to(
        &self,
        id: &str,
        to: &Path,
        progress: &(dyn Fn(u64) + Send + Sync),
    ) -> impl Future<Output = Result<Downloaded, SyncError>> + Send;

    /// Every file and folder the app can see, not trashed: with the `drive.file` scope, only
    /// the ones EmuDome made. One request per thousand, instead of one per folder.
    fn list_all(&self) -> impl Future<Output = Result<Vec<RemoteFile>, SyncError>> + Send;

    /// Moves the file `id` to Drive's trash (the user can restore it there for 30 days).
    fn trash(&self, id: &str) -> impl Future<Output = Result<(), SyncError>> + Send;

    fn quota(&self) -> impl Future<Output = Result<Quota, SyncError>> + Send;

    fn account(&self) -> impl Future<Output = Result<Account, SyncError>> + Send;

    /// The page token from which [`DriveApi::changes`] lists what changes next.
    fn changes_start(&self) -> impl Future<Output = Result<String, SyncError>> + Send;

    fn changes(&self, token: &str) -> impl Future<Output = Result<ChangesPage, SyncError>> + Send;
}

/// Where the REST client gets its access token: [`crate::auth::Auth`] in the app.
pub trait TokenSource: Send + Sync {
    fn token(&self) -> impl Future<Output = Result<Secret, SyncError>> + Send;
    /// Drive refused the token (401): forget it, the next [`TokenSource::token`] gets another.
    fn invalidate(&self) -> impl Future<Output = ()> + Send;
}

impl TokenSource for crate::auth::Auth {
    async fn token(&self) -> Result<Secret, SyncError> {
        self.access_token().await
    }

    async fn invalidate(&self) {
        crate::auth::Auth::invalidate(self).await;
    }
}

impl<T: TokenSource> TokenSource for std::sync::Arc<T> {
    async fn token(&self) -> Result<Secret, SyncError> {
        T::token(self).await
    }

    async fn invalidate(&self) {
        T::invalidate(self).await;
    }
}

/// The lowercase hex MD5 of `bytes`, as Drive's `md5Checksum`.
pub fn md5_hex(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    hex(&Md5::digest(bytes))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{md5_hex, Quota};

    #[test]
    fn md5_is_drives_checksum_format() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn quota_says_what_fits() {
        let q = Quota {
            limit: Some(100),
            usage: 60,
        };
        assert_eq!(q.available(), Some(40));
        assert!(q.fits(40) && !q.fits(41));
        let over = Quota {
            limit: Some(100),
            usage: 120,
        };
        assert_eq!(over.available(), Some(0));
        let unlimited = Quota {
            limit: None,
            usage: 1 << 40,
        };
        assert!(unlimited.fits(u64::MAX));
    }
}
