//! [`DriveApi`] over Drive REST v3 (reqwest).
//!
//! - Small files (up to [`MULTIPART_MAX`]) go up in one multipart request (metadata + content).
//! - Bigger ones use the resumable protocol: a session is opened (`uploadType=resumable`), the
//!   content goes up in chunks of [`RestDrive::chunk`] bytes (a multiple of 256 KiB, as Drive
//!   requires), and after an interruption (a dropped connection, a 5xx) the client asks the
//!   session how much it holds (`Content-Range: bytes */<size>`) and goes on from there; a
//!   session Drive no longer knows (404 / 410) is started again from the first byte.
//! - Every request carries the access token of the [`TokenSource`]; a 401 makes it get a new one
//!   and the request is sent once more. 429, 5xx and Drive's rate-limit 403s are retried with a
//!   growing pause.
//!
//! The session URL of a resumable upload is a credential of its own: it is never logged.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value as Json};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::{
    Account, Change, ChangesPage, Content, DriveApi, Properties, Quota, RemoteFile, TokenSource,
    FOLDER_MIME,
};
use crate::oauth::Secret;
use crate::SyncError;

/// Largest content sent in one multipart request.
pub const MULTIPART_MAX: u64 = 5 * 1024 * 1024;
/// Drive wants resumable chunks in multiples of this.
pub const CHUNK_UNIT: u64 = 256 * 1024;
/// How many times one request is tried in all (transient errors, a lost chunk).
const ATTEMPTS: u32 = 5;
/// The fields of a file every call asks for.
const FILE_FIELDS: &str = "id,name,parents,mimeType,md5Checksum,size,modifiedTime,appProperties";

/// Drive REST v3 with the token of `T`.
pub struct RestDrive<T> {
    http: reqwest::Client,
    tokens: T,
    api: String,
    upload: String,
    /// Bytes per resumable chunk (a multiple of [`CHUNK_UNIT`]).
    pub chunk: u64,
    /// Largest content sent in one multipart request ([`MULTIPART_MAX`]).
    pub multipart_max: u64,
    /// Pause before the first retry; doubled at each one.
    pub retry_pause: Duration,
}

impl<T: TokenSource> RestDrive<T> {
    /// Google's Drive.
    pub fn new(http: reqwest::Client, tokens: T) -> Self {
        Self::with_base(
            http,
            tokens,
            "https://www.googleapis.com/drive/v3",
            "https://www.googleapis.com/upload/drive/v3",
        )
    }

    /// Another base URL (tests: a local server).
    pub fn with_base(http: reqwest::Client, tokens: T, api: &str, upload: &str) -> Self {
        Self {
            http,
            tokens,
            api: api.trim_end_matches('/').to_owned(),
            upload: upload.trim_end_matches('/').to_owned(),
            chunk: 32 * CHUNK_UNIT,
            multipart_max: MULTIPART_MAX,
            retry_pause: Duration::from_millis(500),
        }
    }

    /// Sends the request `build` makes with the current token, retrying as the module says.
    /// Returns the response of the first attempt that is not retried (success or not).
    async fn send(
        &self,
        build: impl Fn(&Secret) -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, SyncError> {
        let mut pause = self.retry_pause;
        let mut refreshed = false;
        let mut attempt = 1;
        loop {
            let token = self.tokens.token().await?;
            let result = build(&token).send().await;
            let retry = match &result {
                Ok(r) if r.status().as_u16() == 401 && !refreshed => {
                    self.tokens.invalidate().await;
                    refreshed = true;
                    continue;
                }
                Ok(r) => transient(r.status().as_u16()),
                Err(_) => true,
            };
            if !retry || attempt >= ATTEMPTS {
                return result.map_err(SyncError::from);
            }
            // A rate-limit 403 is told apart from a refusal by its reason: read it.
            if let Ok(r) = result {
                if r.status().as_u16() == 403 {
                    let error = read_error(r).await;
                    if !error.rate_limited() {
                        return Err(error.into_sync(0));
                    }
                }
            }
            attempt += 1;
            tokio::time::sleep(pause).await;
            pause *= 2;
        }
    }

    async fn json<R: for<'de> Deserialize<'de>>(
        &self,
        build: impl Fn(&Secret) -> reqwest::RequestBuilder,
        needed: u64,
    ) -> Result<R, SyncError> {
        let response = self.send(build).await?;
        if !response.status().is_success() {
            return Err(read_error(response).await.into_sync(needed));
        }
        let bytes = response.bytes().await?;
        serde_json::from_slice(&bytes)
            .map_err(|e| SyncError::Request(format!("Drive reply not understood: {e}")))
    }

    async fn file(
        &self,
        build: impl Fn(&Secret) -> reqwest::RequestBuilder,
        needed: u64,
    ) -> Result<RemoteFile, SyncError> {
        self.json::<FileJson>(build, needed).await?.into_remote()
    }

    fn files_url(&self) -> String {
        format!("{}/files", self.api)
    }

    async fn list_query(&self, q: String) -> Result<Vec<RemoteFile>, SyncError> {
        let mut out = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let fields = format!("nextPageToken,files({FILE_FIELDS})");
            let page_now = page.clone();
            let list: ListJson = self
                .json(
                    |t| {
                        let mut query = vec![
                            ("q", q.as_str()),
                            ("fields", fields.as_str()),
                            ("pageSize", "1000"),
                            ("orderBy", "createdTime"),
                            ("spaces", "drive"),
                        ];
                        if let Some(p) = &page_now {
                            query.push(("pageToken", p.as_str()));
                        }
                        self.http
                            .get(self.files_url())
                            .bearer_auth(t.expose())
                            .query(&query)
                    },
                    0,
                )
                .await?;
            for f in list.files {
                out.push(f.into_remote()?);
            }
            match list.next_page_token {
                Some(p) => page = Some(p),
                None => return Ok(out),
            }
        }
    }

    /// Creates (`id` `None`) or updates a file with `content`, by the protocol its size calls for.
    async fn upload(
        &self,
        id: Option<&str>,
        metadata: Json,
        content: Content,
    ) -> Result<RemoteFile, SyncError> {
        let size = content.len();
        if size <= self.multipart_max {
            let bytes = read_all(&content).await?;
            return self.multipart(id, &metadata, bytes).await;
        }
        let mut restarts = 0;
        loop {
            let session = self.open_session(id, &metadata, size).await?;
            match self.send_chunks(&session, &content, size).await {
                Err(Interrupted::SessionGone) if restarts < 2 => restarts += 1,
                Err(Interrupted::SessionGone) => {
                    return Err(SyncError::Request("upload session lost".into()))
                }
                Err(Interrupted::Failed(e)) => return Err(e),
                Ok(file) => return Ok(file),
            }
        }
    }

    async fn multipart(
        &self,
        id: Option<&str>,
        metadata: &Json,
        bytes: Vec<u8>,
    ) -> Result<RemoteFile, SyncError> {
        let boundary = format!(
            "emudome-{}",
            crate::oauth::random_token(12)?.replace(['-', '_'], "x")
        );
        let mut body = format!(
            "--{boundary}\r\ncontent-type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n\
             --{boundary}\r\ncontent-type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        body.extend_from_slice(&bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let size = bytes.len() as u64;
        let url = self.upload_url(id);
        self.file(
            |t| {
                self.request(id, &url)
                    .bearer_auth(t.expose())
                    .query(&[("uploadType", "multipart"), ("fields", FILE_FIELDS)])
                    .header(
                        reqwest::header::CONTENT_TYPE,
                        format!("multipart/related; boundary={boundary}"),
                    )
                    .body(body.clone())
            },
            size,
        )
        .await
    }

    fn upload_url(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("{}/files/{id}", self.upload),
            None => format!("{}/files", self.upload),
        }
    }

    /// POST for a new file, PATCH for new content of an existing one.
    fn request(&self, id: Option<&str>, url: &str) -> reqwest::RequestBuilder {
        match id {
            Some(_) => self.http.patch(url),
            None => self.http.post(url),
        }
    }

    async fn open_session(
        &self,
        id: Option<&str>,
        metadata: &Json,
        size: u64,
    ) -> Result<Secret, SyncError> {
        let url = self.upload_url(id);
        let response = self
            .send(|t| {
                self.request(id, &url)
                    .bearer_auth(t.expose())
                    .query(&[("uploadType", "resumable"), ("fields", FILE_FIELDS)])
                    .header("X-Upload-Content-Type", "application/octet-stream")
                    .header("X-Upload-Content-Length", size.to_string())
                    .json(metadata)
            })
            .await?;
        if !response.status().is_success() {
            return Err(read_error(response).await.into_sync(size));
        }
        response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(|v| Secret::new(v.to_owned()))
            .ok_or_else(|| SyncError::Request("no upload session URL".into()))
    }

    /// Sends the content from where the session stands, chunk by chunk.
    async fn send_chunks(
        &self,
        session: &Secret,
        content: &Content,
        size: u64,
    ) -> Result<RemoteFile, Interrupted> {
        let mut offset = 0u64;
        let mut failures = 0;
        let mut pause = self.retry_pause;
        loop {
            let end = (offset + self.chunk).min(size);
            let bytes = read_range(content, offset, end - offset)
                .await
                .map_err(Interrupted::Failed)?;
            let sent = self
                .http
                .put(session.expose())
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {offset}-{}/{size}", end - 1),
                )
                .body(bytes)
                .send()
                .await;
            let lost = match sent {
                Ok(r) => match r.status().as_u16() {
                    200 | 201 => return file_from(r).await.map_err(Interrupted::Failed),
                    308 => {
                        offset = next_offset(&r);
                        failures = 0;
                        continue;
                    }
                    404 | 410 => return Err(Interrupted::SessionGone),
                    s if transient(s) => true,
                    _ => {
                        return Err(Interrupted::Failed(read_error(r).await.into_sync(size)));
                    }
                },
                Err(_) => true,
            };
            if lost {
                failures += 1;
                if failures >= ATTEMPTS {
                    return Err(Interrupted::Failed(SyncError::Offline(
                        "upload interrupted".into(),
                    )));
                }
                tokio::time::sleep(pause).await;
                pause *= 2;
                // Where does the session stand now?
                match self.session_status(session, size).await {
                    Ok(Status::Done(file)) => return Ok(*file),
                    Ok(Status::At(at)) => offset = at,
                    Ok(Status::Gone) => return Err(Interrupted::SessionGone),
                    Err(_) => {} // still offline: try the same chunk again
                }
            }
        }
    }

    async fn session_status(&self, session: &Secret, size: u64) -> Result<Status, SyncError> {
        let r = self
            .http
            .put(session.expose())
            .header(reqwest::header::CONTENT_RANGE, format!("bytes */{size}"))
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send()
            .await?;
        match r.status().as_u16() {
            200 | 201 => Ok(Status::Done(Box::new(file_from(r).await?))),
            308 => Ok(Status::At(next_offset(&r))),
            404 | 410 => Ok(Status::Gone),
            _ => Err(read_error(r).await.into_sync(size)),
        }
    }
}

enum Interrupted {
    /// Drive no longer knows the session: start a new one.
    SessionGone,
    Failed(SyncError),
}

enum Status {
    Done(Box<RemoteFile>),
    At(u64),
    Gone,
}

/// The byte after the last one the session holds (`Range: bytes=0-<last>`); none yet: 0.
fn next_offset(r: &reqwest::Response) -> u64 {
    r.headers()
        .get(reqwest::header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit('-').next())
        .and_then(|last| last.trim().parse::<u64>().ok())
        .map_or(0, |last| last + 1)
}

async fn file_from(r: reqwest::Response) -> Result<RemoteFile, SyncError> {
    let bytes = r.bytes().await?;
    serde_json::from_slice::<FileJson>(&bytes)
        .map_err(|e| SyncError::Request(format!("Drive reply not understood: {e}")))?
        .into_remote()
}

/// 429, 5xx and 403 (a 403 is retried only for a rate-limit reason, checked by the caller).
fn transient(status: u16) -> bool {
    status == 429 || status == 403 || (500..600).contains(&status)
}

async fn read_all(content: &Content) -> Result<Vec<u8>, SyncError> {
    match content {
        Content::Bytes(b) => Ok(b.clone()),
        Content::File { path, .. } => tokio::fs::read(path)
            .await
            .map_err(|e| SyncError::Local(format!("{}: {e}", path.display()))),
    }
}

async fn read_range(content: &Content, offset: u64, len: u64) -> Result<Vec<u8>, SyncError> {
    let len = usize::try_from(len).map_err(|e| SyncError::Local(e.to_string()))?;
    match content {
        Content::Bytes(b) => {
            let start = usize::try_from(offset).map_err(|e| SyncError::Local(e.to_string()))?;
            b.get(start..start + len)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| SyncError::Local("range outside the content".into()))
        }
        Content::File { path, .. } => {
            let local = |e: std::io::Error| SyncError::Local(format!("{}: {e}", path.display()));
            let mut file = tokio::fs::File::open(path).await.map_err(local)?;
            file.seek(std::io::SeekFrom::Start(offset))
                .await
                .map_err(local)?;
            let mut buf = vec![0u8; len];
            file.read_exact(&mut buf).await.map_err(local)?;
            Ok(buf)
        }
    }
}

/// Drive's error body: `{ "error": { "code", "message", "errors": [{ "reason" }] } }`.
struct DriveError {
    status: u16,
    reason: String,
    message: String,
}

impl DriveError {
    fn rate_limited(&self) -> bool {
        matches!(
            self.reason.as_str(),
            "rateLimitExceeded" | "userRateLimitExceeded"
        )
    }

    fn into_sync(self, needed: u64) -> SyncError {
        match (self.status, self.reason.as_str()) {
            (401, _) => SyncError::AccessExpired,
            (403, "storageQuotaExceeded") | (403, "quotaExceeded") => SyncError::QuotaFull {
                needed,
                available: 0,
            },
            (404, _) => SyncError::NotFound(self.message),
            _ => SyncError::Request(format!(
                "HTTP {} {}: {}",
                self.status, self.reason, self.message
            )),
        }
    }
}

async fn read_error(r: reqwest::Response) -> DriveError {
    let status = r.status().as_u16();
    let body: Json = r
        .bytes()
        .await
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Json::Null);
    let error = &body["error"];
    DriveError {
        status,
        reason: error["errors"][0]["reason"]
            .as_str()
            .unwrap_or("")
            .to_owned(),
        message: error["message"].as_str().unwrap_or("").to_owned(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileJson {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    parents: Vec<String>,
    #[serde(default)]
    mime_type: String,
    md5_checksum: Option<String>,
    /// int64 as a string.
    size: Option<String>,
    modified_time: Option<String>,
    #[serde(default)]
    app_properties: Properties,
}

impl FileJson {
    fn into_remote(self) -> Result<RemoteFile, SyncError> {
        let size = match self.size {
            Some(s) => Some(
                s.parse()
                    .map_err(|_| SyncError::Request(format!("Drive file size {s:?}")))?,
            ),
            None => None,
        };
        Ok(RemoteFile {
            folder: self.mime_type == FOLDER_MIME,
            id: self.id,
            name: self.name,
            parents: self.parents,
            md5: self.md5_checksum.map(|m| m.to_ascii_lowercase()),
            size,
            modified: self
                .modified_time
                .as_deref()
                .and_then(crate::time::parse_rfc3339)
                .unwrap_or(0),
            app_properties: self.app_properties,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListJson {
    #[serde(default)]
    files: Vec<FileJson>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AboutJson {
    storage_quota: Option<QuotaJson>,
    user: Option<UserJson>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuotaJson {
    limit: Option<String>,
    usage: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserJson {
    display_name: Option<String>,
    email_address: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartJson {
    start_page_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangesJson {
    #[serde(default)]
    changes: Vec<ChangeJson>,
    next_page_token: Option<String>,
    new_start_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeJson {
    file_id: String,
    #[serde(default)]
    removed: bool,
    file: Option<FileJson>,
}

/// A name inside a Drive query string: `\` and `'` escaped.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\\', "\\\\").replace('\'', "\\'"))
}

impl<T: TokenSource> DriveApi for RestDrive<T> {
    async fn list(&self, parent: &str) -> Result<Vec<RemoteFile>, SyncError> {
        self.list_query(format!("{} in parents and trashed = false", quoted(parent)))
            .await
    }

    async fn find(&self, parent: &str, name: &str) -> Result<Option<RemoteFile>, SyncError> {
        let found = self
            .list_query(format!(
                "{} in parents and name = {} and trashed = false",
                quoted(parent),
                quoted(name)
            ))
            .await?;
        Ok(found.into_iter().next())
    }

    async fn get(&self, id: &str) -> Result<RemoteFile, SyncError> {
        let url = format!("{}/{id}", self.files_url());
        self.file(
            |t| {
                self.http
                    .get(&url)
                    .bearer_auth(t.expose())
                    .query(&[("fields", FILE_FIELDS)])
            },
            0,
        )
        .await
    }

    async fn create_folder(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
    ) -> Result<RemoteFile, SyncError> {
        let body = json!({ "name": name, "mimeType": FOLDER_MIME, "parents": [parent],
                           "appProperties": props });
        let url = self.files_url();
        self.file(
            |t| {
                self.http
                    .post(&url)
                    .bearer_auth(t.expose())
                    .query(&[("fields", FILE_FIELDS)])
                    .json(&body)
            },
            0,
        )
        .await
    }

    async fn create(
        &self,
        parent: &str,
        name: &str,
        props: &Properties,
        content: Content,
    ) -> Result<RemoteFile, SyncError> {
        let metadata = json!({ "name": name, "parents": [parent], "appProperties": props });
        self.upload(None, metadata, content).await
    }

    async fn update(
        &self,
        id: &str,
        props: &Properties,
        content: Content,
    ) -> Result<RemoteFile, SyncError> {
        self.upload(Some(id), json!({ "appProperties": props }), content)
            .await
    }

    async fn set_properties(
        &self,
        id: &str,
        props: &BTreeMap<String, Option<String>>,
    ) -> Result<RemoteFile, SyncError> {
        let url = format!("{}/{id}", self.files_url());
        let body = json!({ "appProperties": props });
        self.file(
            |t| {
                self.http
                    .patch(&url)
                    .bearer_auth(t.expose())
                    .query(&[("fields", FILE_FIELDS)])
                    .json(&body)
            },
            0,
        )
        .await
    }

    async fn copy(&self, id: &str, parent: &str, name: &str) -> Result<RemoteFile, SyncError> {
        let url = format!("{}/{id}/copy", self.files_url());
        let body = json!({ "name": name, "parents": [parent] });
        self.file(
            |t| {
                self.http
                    .post(&url)
                    .bearer_auth(t.expose())
                    .query(&[("fields", FILE_FIELDS)])
                    .json(&body)
            },
            0,
        )
        .await
    }

    async fn download(&self, id: &str) -> Result<Vec<u8>, SyncError> {
        let url = format!("{}/{id}", self.files_url());
        let response = self
            .send(|t| {
                self.http
                    .get(&url)
                    .bearer_auth(t.expose())
                    .query(&[("alt", "media")])
            })
            .await?;
        if !response.status().is_success() {
            return Err(read_error(response).await.into_sync(0));
        }
        Ok(response.bytes().await?.to_vec())
    }

    async fn quota(&self) -> Result<Quota, SyncError> {
        let url = format!("{}/about", self.api);
        let about: AboutJson = self
            .json(
                |t| {
                    self.http
                        .get(&url)
                        .bearer_auth(t.expose())
                        .query(&[("fields", "storageQuota(limit,usage)")])
                },
                0,
            )
            .await?;
        let q = about
            .storage_quota
            .ok_or_else(|| SyncError::Request("no storageQuota in the reply".into()))?;
        let number = |s: Option<String>| s.and_then(|s| s.parse::<u64>().ok());
        Ok(Quota {
            limit: number(q.limit),
            usage: number(q.usage).unwrap_or(0),
        })
    }

    async fn account(&self) -> Result<Account, SyncError> {
        let url = format!("{}/about", self.api);
        let about: AboutJson = self
            .json(
                |t| {
                    self.http
                        .get(&url)
                        .bearer_auth(t.expose())
                        .query(&[("fields", "user(displayName,emailAddress)")])
                },
                0,
            )
            .await?;
        let user = about.user.unwrap_or(UserJson {
            display_name: None,
            email_address: None,
        });
        Ok(Account {
            name: user.display_name,
            email: user.email_address,
        })
    }

    async fn changes_start(&self) -> Result<String, SyncError> {
        let url = format!("{}/changes/startPageToken", self.api);
        let start: StartJson = self
            .json(|t| self.http.get(&url).bearer_auth(t.expose()), 0)
            .await?;
        Ok(start.start_page_token)
    }

    async fn changes(&self, token: &str) -> Result<ChangesPage, SyncError> {
        let url = format!("{}/changes", self.api);
        let fields =
            format!("nextPageToken,newStartPageToken,changes(fileId,removed,file({FILE_FIELDS}))");
        let page: ChangesJson = self
            .json(
                |t| {
                    self.http.get(&url).bearer_auth(t.expose()).query(&[
                        ("pageToken", token),
                        ("fields", fields.as_str()),
                        ("spaces", "drive"),
                        ("includeRemoved", "true"),
                        ("pageSize", "1000"),
                    ])
                },
                0,
            )
            .await?;
        let mut changes = Vec::new();
        for c in page.changes {
            changes.push(Change {
                file_id: c.file_id,
                removed: c.removed,
                file: c.file.map(FileJson::into_remote).transpose()?,
            });
        }
        Ok(ChangesPage {
            changes,
            next_page: page.next_page_token,
            new_start: page.new_start_page_token,
        })
    }
}

/// A fixed token: tests, and the probe after its own sign-in.
pub struct StaticToken(pub Secret);

impl TokenSource for StaticToken {
    async fn token(&self) -> Result<Secret, SyncError> {
        Ok(self.0.clone())
    }

    async fn invalidate(&self) {}
}

#[cfg(test)]
mod tests;
