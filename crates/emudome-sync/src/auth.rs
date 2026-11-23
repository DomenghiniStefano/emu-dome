//! The signed-in account of this device: where its refresh token is kept ([`SecretStore`]: the
//! system's credential store in the app, [`MemoryStore`] in tests), the access token in memory,
//! refreshed when it runs out, the sign-in and the sign-out.
//!
//! The refresh token is the only thing kept, and only in the credential store: never in a file,
//! a log or the sync index. Access tokens live in memory for their hour.

use std::sync::Arc;
use std::time::Duration;

use emudome_core::SignInProblem;
use parking_lot::Mutex;

use crate::loopback::{Loopback, Pages};
use crate::oauth::{self, AccessToken, Endpoints, OAuthClient, Pkce, Secret};
use crate::{time, SyncError};

/// Where the refresh token is kept. Calls may block (the system's credential store): [`Auth`]
/// makes them on a blocking thread.
pub trait SecretStore: Send + Sync {
    fn load(&self) -> Result<Option<Secret>, SyncError>;
    fn save(&self, token: &Secret) -> Result<(), SyncError>;
    fn delete(&self) -> Result<(), SyncError>;
}

/// A store in memory: tests, and a run that must not touch the system's store.
#[derive(Default)]
pub struct MemoryStore {
    value: Mutex<Option<Secret>>,
}

impl SecretStore for MemoryStore {
    fn load(&self) -> Result<Option<Secret>, SyncError> {
        Ok(self.value.lock().clone())
    }

    fn save(&self, token: &Secret) -> Result<(), SyncError> {
        *self.value.lock() = Some(token.clone());
        Ok(())
    }

    fn delete(&self) -> Result<(), SyncError> {
        *self.value.lock() = None;
        Ok(())
    }
}

/// How long the user has to finish the sign-in in the browser.
pub const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The account name the refresh token is kept under in the system's credential store (service
/// `it.domehub.emudome`, `emudome_platform::cloud::CREDENTIAL_SERVICE`).
pub const REFRESH_TOKEN_ACCOUNT: &str = "google-drive";

/// The account of this device.
pub struct Auth {
    http: reqwest::Client,
    endpoints: Endpoints,
    client: Option<OAuthClient>,
    store: Arc<dyn SecretStore>,
    /// The current access token; the async lock makes concurrent callers share one refresh.
    access: tokio::sync::Mutex<Option<AccessToken>>,
}

impl Auth {
    /// `client` is `None` in a build without one ([`OAuthClient::from_build`]): everything then
    /// answers [`SyncError::NotConfigured`].
    pub fn new(
        http: reqwest::Client,
        endpoints: Endpoints,
        client: Option<OAuthClient>,
        store: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            http,
            endpoints,
            client,
            store,
            access: tokio::sync::Mutex::new(None),
        }
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Whether this build has an OAuth client.
    pub fn configured(&self) -> bool {
        self.client.is_some()
    }

    fn client(&self) -> Result<&OAuthClient, SyncError> {
        self.client.as_ref().ok_or(SyncError::NotConfigured)
    }

    /// Whether a refresh token is kept on this device (the account may still have revoked it:
    /// the next request says so with [`SyncError::AccessExpired`]).
    pub async fn signed_in(&self) -> Result<bool, SyncError> {
        self.client()?;
        Ok(self.load_refresh().await?.is_some())
    }

    async fn load_refresh(&self) -> Result<Option<Secret>, SyncError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.load())
            .await
            .map_err(|e| SyncError::Credentials(e.to_string()))?
    }

    async fn save_refresh(&self, token: Secret) -> Result<(), SyncError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.save(&token))
            .await
            .map_err(|e| SyncError::Credentials(e.to_string()))?
    }

    /// A usable access token: the one in memory, else a new one from the refresh token.
    pub async fn access_token(&self) -> Result<Secret, SyncError> {
        let client = self.client()?;
        let mut access = self.access.lock().await;
        if let Some(token) = access.as_ref().filter(|t| t.fresh_at(time::now_ms())) {
            return Ok(token.token.clone());
        }
        let refresh = self.load_refresh().await?.ok_or(SyncError::NotSignedIn)?;
        let tokens = oauth::refresh(
            &self.http,
            &self.endpoints,
            client,
            &refresh,
            time::now_ms(),
        )
        .await?;
        if let Some(new_refresh) = tokens.refresh {
            // Google rotates it rarely; the old one stops working once it does.
            self.save_refresh(new_refresh).await?;
        }
        let token = tokens.access.token.clone();
        *access = Some(tokens.access);
        Ok(token)
    }

    /// Forgets the access token in memory (Drive answered 401 with it): the next
    /// [`Auth::access_token`] refreshes.
    pub async fn invalidate(&self) {
        *self.access.lock().await = None;
    }

    /// The whole sign-in: listens on the loopback, has `open` show the authorization URL (the
    /// system browser in the app; printed by `drive_probe --print-url`), waits up to `timeout`
    /// for the redirect, exchanges the code and keeps the refresh token.
    pub async fn sign_in(
        &self,
        open: impl FnOnce(&str) -> Result<(), String>,
        pages: &Pages,
        timeout: Duration,
    ) -> Result<(), SyncError> {
        let client = self.client()?;
        let loopback = Loopback::bind().await?;
        let redirect_uri = loopback.redirect_uri().to_owned();
        let pkce = Pkce::new()?;
        let state = oauth::random_token(16)?;
        let url = oauth::authorization_url(&self.endpoints, client, &redirect_uri, &pkce, &state)?;
        open(url.as_str()).map_err(|e| {
            SyncError::SignInFailed(SignInProblem::Browser, format!("browser: {e}"))
        })?;
        let code = tokio::time::timeout(timeout, loopback.wait(&state, pages))
            .await
            .map_err(|_| SyncError::SignInFailed(SignInProblem::TimedOut, "timed out".into()))??;
        let tokens = oauth::exchange_code(
            &self.http,
            &self.endpoints,
            client,
            &code,
            &pkce,
            &redirect_uri,
            time::now_ms(),
        )
        .await?;
        let refresh = tokens
            .refresh
            .ok_or_else(|| SyncError::sign_in("no refresh token in the reply"))?;
        self.save_refresh(refresh).await?;
        *self.access.lock().await = Some(tokens.access);
        Ok(())
    }

    /// Signs this device out: the grant is revoked at Google (best effort: offline, the token
    /// is forgotten all the same) and the refresh token leaves the credential store.
    pub async fn sign_out(&self) -> Result<(), SyncError> {
        *self.access.lock().await = None;
        let Some(refresh) = self.load_refresh().await? else {
            return Ok(());
        };
        if let Err(e) = oauth::revoke(&self.http, &self.endpoints, &refresh).await {
            log::warn!("cloud: sign-out: the grant was not revoked at Google ({e})");
        }
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.delete())
            .await
            .map_err(|e| SyncError::Credentials(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use emudome_core::SignInProblem;
    use serde_json::json;

    use super::{Auth, MemoryStore, SecretStore};
    use crate::loopback::Pages;
    use crate::oauth::{Endpoints, OAuthClient, Secret, DRIVE_FILE_SCOPE};
    use crate::test_http::{MockServer, Reply};
    use crate::SyncError;

    fn endpoints(server: &MockServer) -> Endpoints {
        Endpoints {
            authorize: server.url("/authorize"),
            token: server.url("/token"),
            revoke: server.url("/revoke"),
        }
    }

    fn auth(server: &MockServer, store: Arc<MemoryStore>, secret: Option<&str>) -> Auth {
        Auth::new(
            crate::http::client().unwrap(),
            endpoints(server),
            Some(OAuthClient::new("client-1", secret)),
            store,
        )
    }

    #[tokio::test]
    async fn the_whole_sign_in_through_the_loopback_keeps_the_refresh_token() {
        let server = MockServer::start(|r| match r.path() {
            "/token" => Reply::json(
                200,
                json!({ "access_token": "at-1", "expires_in": 3599,
                        "refresh_token": "rt-1", "scope": DRIVE_FILE_SCOPE,
                        "token_type": "Bearer" }),
            ),
            _ => Reply::bytes(404, b""),
        })
        .await;
        let store = Arc::new(MemoryStore::default());
        let auth = auth(&server, store.clone(), Some("shh"));
        // The "browser": reads the URL, follows the redirect like Google would.
        let browser = |url: &str| {
            let url = url::Url::parse(url).unwrap();
            let q: Vec<(String, String)> = url.query_pairs().into_owned().collect();
            let get = |n: &str| q.iter().find(|(k, _)| k == n).unwrap().1.clone();
            assert_eq!(get("client_id"), "client-1");
            assert_eq!(get("code_challenge_method"), "S256");
            assert_eq!(get("scope"), DRIVE_FILE_SCOPE);
            let redirect = get("redirect_uri");
            let state = get("state");
            tokio::spawn(async move {
                let target = format!("{redirect}?state={state}&code=code-1");
                let addr = redirect
                    .trim_start_matches("http://")
                    .trim_end_matches('/')
                    .to_owned();
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
                let path = target.split_once("/?").unwrap().1;
                s.write_all(format!("GET /?{path} HTTP/1.1\r\n\r\n").as_bytes())
                    .await
                    .unwrap();
                let mut out = Vec::new();
                s.read_to_end(&mut out).await.unwrap();
            });
            Ok(())
        };
        auth.sign_in(browser, &Pages::plain(), Duration::from_secs(10))
            .await
            .unwrap();
        assert_eq!(store.load().unwrap().unwrap().expose(), "rt-1");
        assert_eq!(auth.access_token().await.unwrap().expose(), "at-1");
        let reqs = server.requests();
        assert_eq!(reqs.len(), 1, "the cached access token is reused");
        let exchange = &reqs[0];
        assert_eq!(exchange.method, "POST");
        assert_eq!(
            exchange.form_value("grant_type").as_deref(),
            Some("authorization_code")
        );
        assert_eq!(exchange.form_value("code").as_deref(), Some("code-1"));
        assert_eq!(exchange.form_value("client_secret").as_deref(), Some("shh"));
        let verifier = exchange.form_value("code_verifier").unwrap();
        assert_eq!(verifier.len(), 64);
        assert!(exchange
            .form_value("redirect_uri")
            .unwrap()
            .starts_with("http://127.0.0.1:"));
    }

    #[tokio::test]
    async fn a_timed_out_sign_in_keeps_nothing() {
        let server = MockServer::start(|_| Reply::bytes(500, b"")).await;
        let store = Arc::new(MemoryStore::default());
        let auth = auth(&server, store.clone(), None);
        let e = auth
            .sign_in(|_| Ok(()), &Pages::plain(), Duration::from_millis(50))
            .await
            .unwrap_err();
        assert_eq!(
            e,
            SyncError::SignInFailed(SignInProblem::TimedOut, "timed out".into())
        );
        assert!(store.load().unwrap().is_none());
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn the_access_token_is_refreshed_and_a_revoked_grant_says_sign_in_again() {
        let server = MockServer::start(|r| {
            match r.form_value("refresh_token").as_deref() {
                Some("good") => Reply::json(
                    200,
                    json!({ "access_token": "fresh", "expires_in": 3600, "scope": DRIVE_FILE_SCOPE }),
                ),
                _ => Reply::json(
                    400,
                    json!({ "error": "invalid_grant", "error_description": "Token has been expired or revoked." }),
                ),
            }
        })
        .await;
        let store = Arc::new(MemoryStore::default());
        let auth = auth(&server, store.clone(), None);
        assert_eq!(
            auth.access_token().await.unwrap_err(),
            SyncError::NotSignedIn
        );
        store.save(&Secret::new("good".into())).unwrap();
        assert!(auth.signed_in().await.unwrap());
        assert_eq!(auth.access_token().await.unwrap().expose(), "fresh");
        let refresh = &server.requests()[0];
        assert_eq!(
            refresh.form_value("grant_type").as_deref(),
            Some("refresh_token")
        );
        assert_eq!(refresh.form_value("client_id").as_deref(), Some("client-1"));
        assert_eq!(
            refresh.form_value("client_secret"),
            None,
            "no secret in this build"
        );
        auth.invalidate().await;
        store.save(&Secret::new("revoked".into())).unwrap();
        assert_eq!(
            auth.access_token().await.unwrap_err(),
            SyncError::AccessExpired
        );
    }

    #[tokio::test]
    async fn sign_out_revokes_and_forgets_even_offline() {
        let server = MockServer::start(|_| Reply::bytes(200, b"")).await;
        let store = Arc::new(MemoryStore::default());
        store.save(&Secret::new("rt".into())).unwrap();
        let auth = auth(&server, store.clone(), None);
        auth.sign_out().await.unwrap();
        assert!(store.load().unwrap().is_none());
        let revoke = &server.requests()[0];
        assert_eq!(revoke.path(), "/revoke");
        assert_eq!(revoke.form_value("token").as_deref(), Some("rt"));

        // Offline: the revocation fails, the token goes all the same.
        store.save(&Secret::new("rt".into())).unwrap();
        // A port that was free a moment ago: nothing answers there.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let offline = Auth::new(
            crate::http::client().unwrap(),
            Endpoints {
                authorize: format!("http://127.0.0.1:{port}/a"),
                token: format!("http://127.0.0.1:{port}/t"),
                revoke: format!("http://127.0.0.1:{port}/r"),
            },
            Some(OAuthClient::new("c", None)),
            store.clone(),
        );
        offline.sign_out().await.unwrap();
        assert!(store.load().unwrap().is_none());
    }

    #[tokio::test]
    async fn a_build_without_a_client_is_not_configured() {
        let auth = Auth::new(
            crate::http::client().unwrap(),
            Endpoints::google(),
            None,
            Arc::new(MemoryStore::default()),
        );
        assert!(!auth.configured());
        assert_eq!(
            auth.signed_in().await.unwrap_err(),
            SyncError::NotConfigured
        );
        assert_eq!(
            auth.access_token().await.unwrap_err(),
            SyncError::NotConfigured
        );
    }
}
