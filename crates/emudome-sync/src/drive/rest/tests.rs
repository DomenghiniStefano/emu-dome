//! The request shapes of [`super::RestDrive`] against a local scripted server.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde_json::json;

use super::{RestDrive, StaticToken, CHUNK_UNIT};
use crate::drive::{Content, DriveApi, Properties, TokenSource};
use crate::oauth::Secret;
use crate::test_http::{MockServer, Recorded, Reply, TestDir};
use crate::SyncError;

fn drive(server: &MockServer) -> RestDrive<StaticToken> {
    let mut d = RestDrive::with_base(
        crate::http::client().unwrap(),
        StaticToken(Secret::new("tok".into())),
        &server.url("/drive/v3"),
        &server.url("/upload/drive/v3"),
    );
    d.chunk = CHUNK_UNIT;
    d.multipart_max = CHUNK_UNIT;
    d.retry_pause = Duration::from_millis(1);
    d
}

fn file_json(id: &str, name: &str, md5: &str, size: u64) -> serde_json::Value {
    json!({ "id": id, "name": name, "parents": ["p"], "mimeType": "application/octet-stream",
            "md5Checksum": md5, "size": size.to_string(),
            "modifiedTime": "2026-11-21T10:00:00.000Z", "appProperties": { "game": "gba:1a" } })
}

#[tokio::test]
async fn list_pages_through_and_reads_every_field() {
    let server = MockServer::start(|r| match r.query_value("pageToken").as_deref() {
        None => Reply::json(
            200,
            json!({ "nextPageToken": "p2", "files": [file_json("f1", "game.sav", "ABCDEF", 131072)] }),
        ),
        Some("p2") => Reply::json(
            200,
            json!({ "files": [{ "id": "d1", "name": "gba",
                                "mimeType": "application/vnd.google-apps.folder" }] }),
        ),
        Some(_) => Reply::bytes(400, b""),
    })
    .await;
    let files = drive(&server).list("parent'1").await.unwrap();
    assert_eq!(files.len(), 2);
    let f = &files[0];
    assert_eq!(f.id, "f1");
    assert_eq!(f.md5.as_deref(), Some("abcdef"));
    assert_eq!(f.size, Some(131_072));
    assert_eq!(f.modified, 1_795_255_200_000);
    assert_eq!(f.app_properties["game"], "gba:1a");
    assert!(!f.folder && files[1].folder);
    let reqs = server.requests();
    assert_eq!(reqs[0].path(), "/drive/v3/files");
    assert_eq!(reqs[0].header("authorization"), Some("Bearer tok"));
    assert_eq!(
        reqs[0].query_value("q").as_deref(),
        Some(r"'parent\'1' in parents and trashed = false")
    );
    assert_eq!(
        reqs[0].query_value("orderBy").as_deref(),
        Some("createdTime")
    );
    assert_eq!(reqs[1].query_value("pageToken").as_deref(), Some("p2"));
}

#[tokio::test]
async fn a_small_file_goes_up_in_one_multipart_request() {
    let server =
        MockServer::start(|_| Reply::json(200, file_json("n1", "game.sav", "aa", 3))).await;
    let mut props = Properties::new();
    props.insert("device".into(), "dev-1".into());
    let made = drive(&server)
        .create(
            "folder-1",
            "game.sav",
            &props,
            Content::Bytes(b"abc".to_vec()),
        )
        .await
        .unwrap();
    assert_eq!(made.id, "n1");
    let r = &server.requests()[0];
    assert_eq!(r.method, "POST");
    assert_eq!(r.path(), "/upload/drive/v3/files");
    assert_eq!(r.query_value("uploadType").as_deref(), Some("multipart"));
    let ct = r.header("content-type").unwrap();
    let boundary = ct.strip_prefix("multipart/related; boundary=").unwrap();
    let body = String::from_utf8(r.body.clone()).unwrap();
    let parts: Vec<&str> = body.split(&format!("--{boundary}")).collect();
    assert_eq!(parts.len(), 4, "{body}");
    let meta = parts[1].split("\r\n\r\n").nth(1).unwrap().trim();
    let meta: serde_json::Value = serde_json::from_str(meta).unwrap();
    assert_eq!(
        meta,
        json!({ "name": "game.sav", "parents": ["folder-1"], "appProperties": { "device": "dev-1" } })
    );
    assert_eq!(parts[2].split("\r\n\r\n").nth(1).unwrap(), "abc\r\n");
    assert_eq!(parts[3], "--\r\n");

    // New content for an existing file: PATCH on its id.
    drive(&server)
        .update("n1", &Properties::new(), Content::Bytes(b"x".to_vec()))
        .await
        .unwrap();
    let r = &server.requests()[1];
    assert_eq!(r.method, "PATCH");
    assert_eq!(r.path(), "/upload/drive/v3/files/n1");
}

/// A resumable upload server keeping what it received; `fail` decides, per chunk request, to
/// answer 503 without keeping the bytes.
struct Upload {
    received: Mutex<Vec<u8>>,
    puts: AtomicUsize,
}

fn resumable_server(
    up: Arc<Upload>,
    total: usize,
    fail: impl Fn(usize) -> bool + Send + Sync + 'static,
) -> impl Fn(&Recorded) -> Reply + Send + Sync + 'static {
    move |r: &Recorded| {
        if r.query_value("uploadType").as_deref() == Some("resumable") {
            let host = r.header("host").unwrap().to_owned();
            return Reply::bytes(200, b"")
                .with_header("location", &format!("http://{host}/session/1"));
        }
        assert_eq!(r.path(), "/session/1");
        let range = r.header("content-range").unwrap().to_owned();
        let mut received = up.received.lock();
        let held = |n: usize| {
            if n == 0 {
                Reply::bytes(308, b"")
            } else {
                Reply::bytes(308, b"").with_header("range", &format!("bytes=0-{}", n - 1))
            }
        };
        if range.starts_with("bytes */") {
            return held(received.len());
        }
        let n = up.puts.fetch_add(1, Ordering::SeqCst);
        if fail(n) {
            return Reply::bytes(503, b"");
        }
        let start: usize = range[6..].split('-').next().unwrap().parse().unwrap();
        assert_eq!(
            start,
            received.len(),
            "chunks arrive in order, from where it stands"
        );
        received.extend_from_slice(&r.body);
        if received.len() == total {
            Reply::json(200, file_json("big", "auto.state", "ff", total as u64))
        } else {
            held(received.len())
        }
    }
}

#[tokio::test]
async fn a_big_file_goes_up_in_chunks_and_resumes_after_an_interruption() {
    let total = (2 * CHUNK_UNIT + 1000) as usize;
    let content: Vec<u8> = (0..total).map(|i| (i % 251) as u8).collect();
    let up = Arc::new(Upload {
        received: Mutex::new(Vec::new()),
        puts: AtomicUsize::new(0),
    });
    // The second chunk is lost twice.
    let server = MockServer::start(resumable_server(up.clone(), total, |n| n == 1 || n == 2)).await;
    let dir = TestDir::new("resumable");
    dir.write("auto.state", &content);
    let made = drive(&server)
        .create(
            "folder-1",
            "auto.state",
            &Properties::new(),
            Content::File {
                path: dir.path("auto.state"),
                size: total as u64,
            },
        )
        .await
        .unwrap();
    assert_eq!(made.id, "big");
    assert_eq!(*up.received.lock(), content);
    let reqs = server.requests();
    let open = &reqs[0];
    assert_eq!(open.method, "POST");
    assert_eq!(
        open.header("x-upload-content-length"),
        Some(total.to_string().as_str())
    );
    assert_eq!(
        open.json(),
        json!({ "name": "auto.state", "parents": ["folder-1"], "appProperties": {} })
    );
    let statuses = reqs
        .iter()
        .filter(|r| {
            r.header("content-range")
                .is_some_and(|c| c.starts_with("bytes */"))
        })
        .count();
    assert_eq!(statuses, 2, "asked where the session stood after each loss");
    // The chunk requests carry no bearer token: the session URL is the credential.
    assert!(reqs[1..]
        .iter()
        .all(|r| r.header("authorization").is_none()));
}

#[tokio::test]
async fn an_upload_that_never_gets_through_is_offline_and_changes_nothing() {
    let total = (CHUNK_UNIT + 10) as usize;
    let up = Arc::new(Upload {
        received: Mutex::new(Vec::new()),
        puts: AtomicUsize::new(0),
    });
    let server = MockServer::start(resumable_server(up.clone(), total, |_| true)).await;
    let e = drive(&server)
        .update("id", &Properties::new(), Content::Bytes(vec![1; total]))
        .await
        .unwrap_err();
    assert_eq!(e, SyncError::Offline("upload interrupted".into()));
    assert!(up.received.lock().is_empty());
}

#[tokio::test]
async fn errors_are_told_apart_and_transient_ones_retried() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let server = MockServer::start(move |r| {
        let err = |status: u16, reason: &str| {
            Reply::json(
                status,
                json!({ "error": { "code": status, "message": "m", "errors": [{ "reason": reason }] } }),
            )
        };
        match r.path() {
            "/drive/v3/files/gone" => err(404, "notFound"),
            "/drive/v3/files/full/copy" => err(403, "storageQuotaExceeded"),
            "/drive/v3/files/denied" => err(403, "insufficientFilePermissions"),
            "/drive/v3/files/busy" => {
                if seen.fetch_add(1, Ordering::SeqCst) < 2 {
                    err(403, "userRateLimitExceeded")
                } else {
                    Reply::json(200, file_json("busy", "x", "00", 1))
                }
            }
            _ => err(500, "backendError"),
        }
    })
    .await;
    let d = drive(&server);
    assert!(matches!(d.get("gone").await, Err(SyncError::NotFound(_))));
    assert_eq!(
        d.copy("full", "p", "n").await.unwrap_err(),
        SyncError::QuotaFull {
            needed: 0,
            available: 0
        }
    );
    assert_eq!(
        d.get("denied").await.unwrap_err(),
        SyncError::Request("HTTP 403 insufficientFilePermissions: m".into())
    );
    assert_eq!(d.get("busy").await.unwrap().id, "busy");
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        d.get("down").await.unwrap_err(),
        SyncError::Request("HTTP 500 backendError: m".into())
    );
    let tries = server
        .requests()
        .iter()
        .filter(|r| r.path() == "/drive/v3/files/down")
        .count();
    assert_eq!(tries, 5);
}

/// A token source counting refreshes: the first token is stale.
struct Rotating {
    refreshed: AtomicUsize,
}

impl TokenSource for Rotating {
    async fn token(&self) -> Result<Secret, SyncError> {
        Ok(Secret::new(format!(
            "t{}",
            self.refreshed.load(Ordering::SeqCst)
        )))
    }

    async fn invalidate(&self) {
        self.refreshed.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn a_401_gets_a_new_token_once() {
    let server = MockServer::start(|r| match r.header("authorization") {
        Some("Bearer t1") => Reply::json(200, file_json("f", "x", "00", 1)),
        _ => Reply::bytes(401, b""),
    })
    .await;
    let tokens = Arc::new(Rotating {
        refreshed: AtomicUsize::new(0),
    });
    let d = RestDrive::with_base(
        crate::http::client().unwrap(),
        tokens.clone(),
        &server.url("/drive/v3"),
        &server.url("/upload/drive/v3"),
    );
    assert_eq!(d.get("f").await.unwrap().id, "f");
    assert_eq!(tokens.refreshed.load(Ordering::SeqCst), 1);
    // Refused again with the new token: sign in again.
    tokens.refreshed.store(5, Ordering::SeqCst);
    assert_eq!(d.get("f").await.unwrap_err(), SyncError::AccessExpired);
}

#[tokio::test]
async fn folders_properties_copies_downloads_quota_and_changes() {
    let server = MockServer::start(|r| match (r.method.as_str(), r.path()) {
        ("POST", "/drive/v3/files") => Reply::json(
            200,
            json!({ "id": "d", "name": "EmuDome", "mimeType": "application/vnd.google-apps.folder" }),
        ),
        ("PATCH", "/drive/v3/files/d") => Reply::json(200, json!({ "id": "d" })),
        ("POST", "/drive/v3/files/f/copy") => Reply::json(200, file_json("c", "copy", "aa", 1)),
        ("GET", "/drive/v3/files/f") => Reply::bytes(200, b"content"),
        ("GET", "/drive/v3/about") => Reply::json(
            200,
            json!({ "storageQuota": { "limit": "16106127360", "usage": "1000" },
                    "user": { "displayName": "N", "emailAddress": "n@example.com" } }),
        ),
        ("GET", "/drive/v3/changes/startPageToken") => {
            Reply::json(200, json!({ "startPageToken": "100" }))
        }
        ("GET", "/drive/v3/changes") => Reply::json(
            200,
            json!({ "newStartPageToken": "101", "changes": [
                { "fileId": "f", "removed": false, "file": file_json("f", "game.sav", "aa", 1) },
                { "fileId": "g", "removed": true } ] }),
        ),
        _ => Reply::bytes(404, b""),
    })
    .await;
    let d = drive(&server);
    let mut props = Properties::new();
    props.insert("kind".into(), "root".into());
    let folder = d.create_folder("root", "EmuDome", &props).await.unwrap();
    assert!(folder.folder);
    assert_eq!(
        server.requests()[0].json(),
        json!({ "name": "EmuDome", "mimeType": "application/vnd.google-apps.folder",
                "parents": ["root"], "appProperties": { "kind": "root" } })
    );
    let mut set = BTreeMap::new();
    set.insert("inUseDevice".to_owned(), Some("dev".to_owned()));
    set.insert("old".to_owned(), None);
    d.set_properties("d", &set).await.unwrap();
    assert_eq!(
        server.requests()[1].json(),
        json!({ "appProperties": { "inUseDevice": "dev", "old": null } })
    );
    assert_eq!(d.copy("f", "p", "copy").await.unwrap().id, "c");
    assert_eq!(
        server.requests()[2].json(),
        json!({ "name": "copy", "parents": ["p"] })
    );
    assert_eq!(d.download("f").await.unwrap(), b"content");
    assert_eq!(
        server.requests()[3].query_value("alt").as_deref(),
        Some("media")
    );
    let q = d.quota().await.unwrap();
    assert_eq!((q.limit, q.usage), (Some(16_106_127_360), 1000));
    let account = d.account().await.unwrap();
    assert_eq!(account.email.as_deref(), Some("n@example.com"));
    assert_eq!(d.changes_start().await.unwrap(), "100");
    let page = d.changes("100").await.unwrap();
    assert_eq!(page.new_start.as_deref(), Some("101"));
    assert_eq!(page.changes.len(), 2);
    assert!(page.changes[1].removed && page.changes[1].file.is_none());
    let last = server.requests().pop().unwrap();
    assert_eq!(last.query_value("pageToken").as_deref(), Some("100"));
    assert_eq!(last.query_value("includeRemoved").as_deref(), Some("true"));
}
