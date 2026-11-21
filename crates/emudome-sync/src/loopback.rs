//! The loopback redirect of the installed-app flow: a one-shot HTTP listener on
//! `127.0.0.1:<random port>` that waits for Google's redirect, checks its `state` and hands
//! back the authorization code.
//!
//! The port is the system's choice (port 0): never a fixed one, never one taken by force. Only
//! `127.0.0.1` is bound, so nothing outside the device can reach it. Requests that carry
//! neither a code nor an error (the browser's `/favicon.ico`) get a 404 and the wait goes on.
//! The page the browser shows afterwards is the caller's ([`Pages`]): its text belongs to the
//! interface (i18n), not to this crate.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::oauth::Secret;
use crate::SyncError;

/// Longest request head read from the browser.
const MAX_HEAD: usize = 16 * 1024;
/// How long one connection may take to send its request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// The HTML pages the browser shows once the redirect arrived.
#[derive(Clone, Debug)]
pub struct Pages {
    /// The code arrived: the user can close the tab and go back to EmuDome.
    pub done: String,
    /// The user refused, or the redirect was not the one expected.
    pub failed: String,
}

impl Pages {
    /// Plain English pages, for the `drive_probe` example and tests.
    pub fn plain() -> Self {
        Self {
            done: page("Signed in. You can close this tab and go back to EmuDome."),
            failed: page("Sign-in not completed. Close this tab and try again from EmuDome."),
        }
    }
}

/// A minimal HTML page with `text` (escaped).
pub fn page(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>EmuDome</title></head>\
         <body style=\"font-family:sans-serif;margin:3rem\"><p>{escaped}</p></body></html>"
    )
}

/// The listener, bound before the browser opens.
pub struct Loopback {
    listener: TcpListener,
    redirect_uri: String,
}

impl Loopback {
    /// Binds `127.0.0.1:0`.
    pub async fn bind() -> Result<Self, SyncError> {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(|e| SyncError::SignInFailed(format!("loopback listener: {e}")))?;
        let port = listener
            .local_addr()
            .map_err(|e| SyncError::SignInFailed(format!("loopback listener: {e}")))?
            .port();
        Ok(Self {
            listener,
            redirect_uri: format!("http://127.0.0.1:{port}/"),
        })
    }

    /// The `redirect_uri` of the authorization URL and of the exchange (the same string both
    /// times, as Google requires).
    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Waits for the redirect whose `state` is `state` and returns its code. A redirect with
    /// `error` (the user refused) or with another state ends the wait with
    /// [`SyncError::SignInFailed`]; the caller bounds the whole wait (`tokio::time::timeout`).
    pub async fn wait(self, state: &str, pages: &Pages) -> Result<Secret, SyncError> {
        loop {
            let (mut stream, _) = self
                .listener
                .accept()
                .await
                .map_err(|e| SyncError::SignInFailed(format!("loopback accept: {e}")))?;
            let Ok(Some(target)) = tokio::time::timeout(REQUEST_TIMEOUT, read_target(&mut stream))
                .await
                .map_err(|_| ())
                .and_then(|r| r.map_err(|_| ()))
            else {
                continue;
            };
            match redirect_outcome(&target, state) {
                Outcome::Ignore => respond(&mut stream, 404, "").await,
                Outcome::Code(code) => {
                    respond(&mut stream, 200, &pages.done).await;
                    return Ok(code);
                }
                Outcome::Failed(reason) => {
                    respond(&mut stream, 200, &pages.failed).await;
                    return Err(SyncError::SignInFailed(reason));
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Not the redirect (favicon, a probe): answer 404 and keep waiting.
    Ignore,
    Code(Secret),
    Failed(String),
}

/// What a request target (`/?code=...&state=...`) says. The code is accepted only with the
/// expected `state` (a redirect with another one is not this sign-in's: refused, never used).
fn redirect_outcome(target: &str, state: &str) -> Outcome {
    let Some(query) = target.split_once('?').map(|(_, q)| q) else {
        return Outcome::Ignore;
    };
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let get = |name: &str| pairs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
    let state_ok = get("state").is_some_and(|s| s == state);
    match (get("code"), get("error")) {
        (_, Some(error)) => Outcome::Failed(format!("refused by Google: {error}")),
        (Some(_), None) if !state_ok => Outcome::Failed("state mismatch".into()),
        (Some(code), None) if !code.is_empty() => Outcome::Code(Secret::new(code.clone())),
        (Some(_), None) | (None, None) => Outcome::Ignore,
    }
}

/// The request target of the request line, once the whole head arrived.
async fn read_target(stream: &mut TcpStream) -> std::io::Result<Option<String>> {
    let mut head = Vec::new();
    let mut chunk = [0u8; 2048];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        if head.len() > MAX_HEAD {
            return Ok(None);
        }
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(None);
        }
        head.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&head);
    let mut first = text.lines().next().unwrap_or("").split(' ');
    Ok(match (first.next(), first.next()) {
        (Some("GET"), Some(target)) => Some(target.to_owned()),
        _ => None,
    })
}

async fn respond(stream: &mut TcpStream, status: u16, html: &str) {
    let reason = if status == 200 { "OK" } else { "Not Found" };
    let reply = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: text/html; charset=utf-8\r\n\
         content-length: {}\r\ncache-control: no-store\r\nconnection: close\r\n\r\n{html}",
        html.len()
    );
    // The browser's page only: the code is already in hand whatever happens here.
    let _ = stream.write_all(reply.as_bytes()).await;
    let _ = stream.shutdown().await;
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use super::{redirect_outcome, Loopback, Outcome, Pages};
    use crate::oauth::Secret;

    #[test]
    fn the_code_counts_only_with_the_expected_state() {
        assert_eq!(
            redirect_outcome("/?state=s1&code=4%2Fabc&scope=x", "s1"),
            Outcome::Code(Secret::new("4/abc".into()))
        );
        assert_eq!(
            redirect_outcome("/?state=other&code=abc", "s1"),
            Outcome::Failed("state mismatch".into())
        );
        assert_eq!(
            redirect_outcome("/?code=abc", "s1"),
            Outcome::Failed("state mismatch".into())
        );
        assert_eq!(
            redirect_outcome("/?error=access_denied&state=s1", "s1"),
            Outcome::Failed("refused by Google: access_denied".into())
        );
        assert_eq!(redirect_outcome("/favicon.ico", "s1"), Outcome::Ignore);
        assert_eq!(redirect_outcome("/?state=s1", "s1"), Outcome::Ignore);
        assert_eq!(redirect_outcome("/?state=s1&code=", "s1"), Outcome::Ignore);
    }

    async fn get(addr: &str, target: &str) -> String {
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(format!("GET {target} HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes())
            .await
            .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).await.unwrap();
        out
    }

    #[tokio::test]
    async fn waits_past_the_favicon_and_returns_the_code() {
        let lb = Loopback::bind().await.unwrap();
        let uri = lb.redirect_uri().to_owned();
        assert!(uri.starts_with("http://127.0.0.1:") && uri.ends_with('/'));
        let addr = uri
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_owned();
        assert_ne!(addr, "127.0.0.1:0");
        let pages = Pages::plain();
        let waiting = tokio::spawn(async move { lb.wait("st", &pages).await });
        let favicon = get(&addr, "/favicon.ico").await;
        assert!(favicon.starts_with("HTTP/1.1 404"), "{favicon}");
        let done = get(&addr, "/?state=st&code=the-code").await;
        assert!(done.starts_with("HTTP/1.1 200"), "{done}");
        assert!(done.contains("Signed in"), "{done}");
        let code = waiting.await.unwrap().unwrap();
        assert_eq!(code.expose(), "the-code");
    }

    #[tokio::test]
    async fn a_refusal_ends_the_wait_with_an_error() {
        let lb = Loopback::bind().await.unwrap();
        let addr = lb
            .redirect_uri()
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_owned();
        let pages = Pages::plain();
        let waiting = tokio::spawn(async move { lb.wait("st", &pages).await });
        let page = get(&addr, "/?error=access_denied&state=st").await;
        assert!(page.contains("not completed"), "{page}");
        assert!(matches!(
            waiting.await.unwrap(),
            Err(crate::SyncError::SignInFailed(_))
        ));
    }
}
