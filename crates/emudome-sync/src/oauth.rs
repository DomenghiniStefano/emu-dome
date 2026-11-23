//! Google's OAuth 2.0 for installed apps, the pieces without the OS: PKCE (S256), the
//! authorization URL, the code → token exchange, the refresh and the revocation.
//!
//! The flow (docs/feasibility.md, «Google Drive sync»): the system browser opens the
//! authorization URL ([`authorization_url`]), Google redirects to a loopback address on a random
//! port ([`crate::loopback`]) with a code, the code and the PKCE verifier are exchanged for an
//! access token and a refresh token ([`exchange_code`]). Embedded webviews are refused by Google,
//! so the sign-in never runs in Tauri's webview. Google's desktop clients still want the client
//! secret in the exchange even with PKCE: it is sent when the build has one. Both values are
//! build-time values ([`OAuthClient::from_build`]), never in the repo (licence rule 6).
//!
//! Nothing here logs or formats a token, a code, the verifier or the secret: their types print
//! as `<redacted>`.

use std::fmt;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use emudome_core::SignInProblem;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::SyncError;

/// The one scope EmuDome asks for: the files it creates (and the ones the user opens with it).
pub const DRIVE_FILE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

/// Google's endpoints. Tests point them at a local server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints {
    pub authorize: String,
    pub token: String,
    pub revoke: String,
}

impl Endpoints {
    pub fn google() -> Self {
        Self {
            authorize: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            revoke: "https://oauth2.googleapis.com/revoke".into(),
        }
    }
}

/// The app's OAuth client (the Desktop client of the `emudome` Cloud project).
#[derive(Clone, PartialEq, Eq)]
pub struct OAuthClient {
    id: String,
    secret: Option<String>,
}

impl OAuthClient {
    /// The client this build was compiled with: `EMUDOME_GOOGLE_CLIENT_ID` and
    /// `EMUDOME_GOOGLE_CLIENT_SECRET` read at compile time. `None` when the id was not set (or
    /// empty): Drive sync does not exist in this build ([`SyncError::NotConfigured`]).
    pub fn from_build() -> Option<Self> {
        Self::from_values(
            option_env!("EMUDOME_GOOGLE_CLIENT_ID"),
            option_env!("EMUDOME_GOOGLE_CLIENT_SECRET"),
        )
    }

    fn from_values(id: Option<&str>, secret: Option<&str>) -> Option<Self> {
        let id = id.map(str::trim).filter(|id| !id.is_empty())?;
        Some(Self::new(
            id,
            secret.map(str::trim).filter(|s| !s.is_empty()),
        ))
    }

    pub fn new(id: &str, secret: Option<&str>) -> Self {
        Self {
            id: id.to_owned(),
            secret: secret.map(str::to_owned),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Debug for OAuthClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuthClient(<redacted>)")
    }
}

/// A secret string (a token, a code, the PKCE verifier): kept out of every `Debug` output.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The value, to send it where it belongs (a request, the credential store).
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// `n` bytes from the system's random source, as base64url without padding.
pub fn random_token(n: usize) -> Result<String, SyncError> {
    let mut bytes = vec![0u8; n];
    getrandom::fill(&mut bytes).map_err(|e| SyncError::sign_in(format!("random: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// PKCE (RFC 7636): the verifier stays here, its S256 challenge goes in the URL.
#[derive(Clone, Debug)]
pub struct Pkce {
    verifier: Secret,
    challenge: String,
}

impl Pkce {
    /// A new verifier: 48 random bytes, 64 base64url characters (RFC 7636 wants 43-128 of
    /// `[A-Za-z0-9-._~]`).
    pub fn new() -> Result<Self, SyncError> {
        Ok(Self::from_verifier(random_token(48)?))
    }

    pub fn from_verifier(verifier: String) -> Self {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier: Secret(verifier),
            challenge,
        }
    }

    pub fn challenge(&self) -> &str {
        &self.challenge
    }

    pub fn verifier(&self) -> &Secret {
        &self.verifier
    }
}

/// The URL the browser opens: code flow, PKCE S256, offline access (a refresh token), and the
/// consent screen every time (so a new sign-in always gets a refresh token, also after a
/// sign-out on this device).
pub fn authorization_url(
    endpoints: &Endpoints,
    client: &OAuthClient,
    redirect_uri: &str,
    pkce: &Pkce,
    state: &str,
) -> Result<url::Url, SyncError> {
    url::Url::parse_with_params(
        &endpoints.authorize,
        [
            ("client_id", client.id.as_str()),
            ("redirect_uri", redirect_uri),
            ("response_type", "code"),
            ("scope", DRIVE_FILE_SCOPE),
            ("code_challenge", pkce.challenge()),
            ("code_challenge_method", "S256"),
            ("state", state),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ],
    )
    .map_err(|e| SyncError::sign_in(format!("authorization URL: {e}")))
}

/// What the token endpoint gave.
#[derive(Clone, Debug)]
pub struct Tokens {
    pub access: AccessToken,
    /// Only on the first exchange (and every exchange with `prompt=consent`).
    pub refresh: Option<Secret>,
    /// The scopes granted, space-separated: the user may untick Drive on the consent screen.
    pub scope: String,
}

/// An access token and when it stops working (Unix ms).
#[derive(Clone, Debug)]
pub struct AccessToken {
    pub token: Secret,
    pub expires_at: u64,
}

impl AccessToken {
    /// Still usable at `now` (Unix ms), with a minute to spare for the request in flight.
    pub fn fresh_at(&self, now: u64) -> bool {
        now.saturating_add(60_000) < self.expires_at
    }
}

#[derive(Deserialize)]
struct TokenReply {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    #[serde(default)]
    scope: String,
}

#[derive(Deserialize)]
struct ErrorReply {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

/// Exchanges the authorization `code` (with the PKCE verifier and the same redirect URI) for
/// tokens. Refused when the user did not grant the Drive scope.
pub async fn exchange_code(
    http: &reqwest::Client,
    endpoints: &Endpoints,
    client: &OAuthClient,
    code: &Secret,
    pkce: &Pkce,
    redirect_uri: &str,
    now: u64,
) -> Result<Tokens, SyncError> {
    let mut form = vec![
        ("grant_type", "authorization_code"),
        ("code", code.expose()),
        ("code_verifier", pkce.verifier.expose()),
        ("redirect_uri", redirect_uri),
        ("client_id", client.id.as_str()),
    ];
    if let Some(secret) = &client.secret {
        form.push(("client_secret", secret));
    }
    let reply = post_form(http, &endpoints.token, &form)
        .await
        .required()
        .map_err(|e| match e {
            TokenError::Refused(reason) => SyncError::sign_in(reason),
            TokenError::Other(e) => e,
        })?;
    let tokens = tokens_from(reply, now);
    if !tokens.scope.split(' ').any(|s| s == DRIVE_FILE_SCOPE) {
        // The consent page lets the user leave Drive's box unticked.
        return Err(SyncError::SignInFailed(
            SignInProblem::NotGranted,
            "Drive access not granted".into(),
        ));
    }
    Ok(tokens)
}

/// A new access token from the refresh token. [`SyncError::AccessExpired`] when Google refuses
/// it (revoked by the user, expired, the password changed): only a new sign-in helps then.
pub async fn refresh(
    http: &reqwest::Client,
    endpoints: &Endpoints,
    client: &OAuthClient,
    refresh_token: &Secret,
    now: u64,
) -> Result<Tokens, SyncError> {
    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.expose()),
        ("client_id", client.id.as_str()),
    ];
    if let Some(secret) = &client.secret {
        form.push(("client_secret", secret));
    }
    let reply = post_form(http, &endpoints.token, &form)
        .await
        .required()
        .map_err(|e| match e {
            TokenError::Refused(reason) if reason.starts_with("invalid_grant") => {
                SyncError::AccessExpired
            }
            TokenError::Refused(reason) => SyncError::Request(format!("token refresh: {reason}")),
            TokenError::Other(e) => e,
        })?;
    Ok(tokens_from(reply, now))
}

/// Revokes `token` (a refresh token revokes the whole grant) at sign-out. Best effort: the
/// caller forgets the token whatever this returns.
pub async fn revoke(
    http: &reqwest::Client,
    endpoints: &Endpoints,
    token: &Secret,
) -> Result<(), SyncError> {
    post_form(http, &endpoints.revoke, &[("token", token.expose())])
        .await
        .map(|_| ())
        .map_err(|e| match e {
            TokenError::Refused(reason) => SyncError::Request(format!("revoke: {reason}")),
            TokenError::Other(e) => e,
        })
}

fn tokens_from(reply: TokenReply, now: u64) -> Tokens {
    Tokens {
        access: AccessToken {
            token: Secret(reply.access_token),
            expires_at: now.saturating_add(reply.expires_in.saturating_mul(1000)),
        },
        refresh: reply.refresh_token.map(Secret),
        scope: reply.scope,
    }
}

enum TokenError {
    /// The endpoint answered with an OAuth error (`error`: `invalid_grant`, `access_denied`...).
    Refused(String),
    Other(SyncError),
}

async fn post_form(
    http: &reqwest::Client,
    url: &str,
    form: &[(&str, &str)],
) -> Result<Option<TokenReply>, TokenError> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form)
        .finish();
    let response = http
        .post(url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .map_err(|e| TokenError::Other(e.into()))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| TokenError::Other(e.into()))?;
    if status.is_success() {
        if bytes.is_empty() {
            return Ok(None);
        }
        return serde_json::from_slice(&bytes).map(Some).map_err(|e| {
            TokenError::Other(SyncError::Request(format!(
                "token reply not understood: {e}"
            )))
        });
    }
    match serde_json::from_slice::<ErrorReply>(&bytes) {
        Ok(e) if status.is_client_error() => Err(TokenError::Refused(match e.error_description {
            Some(d) => format!("{}: {d}", e.error),
            None => e.error,
        })),
        Ok(_) | Err(_) => Err(TokenError::Other(SyncError::Request(format!(
            "token endpoint: HTTP {}",
            status.as_u16()
        )))),
    }
}

/// `exchange_code` and `refresh` read a reply: an empty success is an error there.
trait RequireReply {
    fn required(self) -> Result<TokenReply, TokenError>;
}

impl RequireReply for Result<Option<TokenReply>, TokenError> {
    fn required(self) -> Result<TokenReply, TokenError> {
        self.and_then(|r| {
            r.ok_or_else(|| TokenError::Other(SyncError::Request("empty token reply".into())))
        })
    }
}

#[cfg(test)]
mod tests {
    use emudome_core::SignInProblem;
    use serde_json::json;

    use super::{
        authorization_url, exchange_code, random_token, refresh, Endpoints, OAuthClient, Pkce,
        Secret, DRIVE_FILE_SCOPE,
    };
    use crate::test_http::{MockServer, Reply};
    use crate::SyncError;

    #[test]
    fn the_challenge_is_rfc_7636s_s256() {
        // RFC 7636, appendix B.
        let pkce = Pkce::from_verifier("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".into());
        assert_eq!(
            pkce.challenge(),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        let fresh = Pkce::new().unwrap();
        let v = fresh.verifier().expose();
        assert_eq!(v.len(), 64);
        assert!(v
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        assert_ne!(Pkce::new().unwrap().verifier(), fresh.verifier());
        assert_ne!(random_token(16).unwrap(), random_token(16).unwrap());
    }

    #[test]
    fn secrets_never_reach_a_debug_output() {
        let client = OAuthClient::new("the-id", Some("the-secret"));
        let pkce = Pkce::from_verifier("the-verifier".into());
        let text = format!("{client:?} {pkce:?} {:?}", Secret::new("the-token".into()));
        for secret in ["the-id", "the-secret", "the-verifier", "the-token"] {
            assert!(!text.contains(secret), "{text}");
        }
    }

    #[test]
    fn a_build_without_a_client_id_has_no_client() {
        assert_eq!(OAuthClient::from_values(None, Some("s")), None);
        assert_eq!(OAuthClient::from_values(Some("  "), None), None);
        assert_eq!(
            OAuthClient::from_values(Some("id"), Some("")),
            Some(OAuthClient::new("id", None))
        );
        assert_eq!(
            OAuthClient::from_values(Some("id"), Some("s")),
            Some(OAuthClient::new("id", Some("s")))
        );
    }

    #[test]
    fn the_authorization_url_asks_for_drive_file_with_pkce_and_offline_access() {
        let pkce = Pkce::from_verifier("v".repeat(43));
        let url = authorization_url(
            &Endpoints::google(),
            &OAuthClient::new("cid.apps.googleusercontent.com", Some("never-in-url")),
            "http://127.0.0.1:51234/",
            &pkce,
            "st&ate",
        )
        .unwrap();
        assert_eq!(url.host_str(), Some("accounts.google.com"));
        assert!(!url.as_str().contains("never-in-url"));
        let q: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        let get = |n: &str| q.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
        assert_eq!(get("client_id"), Some("cid.apps.googleusercontent.com"));
        assert_eq!(get("redirect_uri"), Some("http://127.0.0.1:51234/"));
        assert_eq!(get("response_type"), Some("code"));
        assert_eq!(get("scope"), Some(DRIVE_FILE_SCOPE));
        assert_eq!(get("code_challenge"), Some(pkce.challenge()));
        assert_eq!(get("code_challenge_method"), Some("S256"));
        assert_eq!(get("state"), Some("st&ate"));
        assert_eq!(get("access_type"), Some("offline"));
        assert_eq!(get("prompt"), Some("consent"));
    }

    fn endpoints(server: &MockServer) -> Endpoints {
        Endpoints {
            authorize: server.url("/auth"),
            token: server.url("/token"),
            revoke: server.url("/revoke"),
        }
    }

    #[tokio::test]
    async fn the_exchange_sends_the_verifier_and_reads_the_tokens() {
        let server = MockServer::start(|_| {
            Reply::json(
                200,
                json!({ "access_token": "at", "expires_in": 3599, "refresh_token": "rt",
                        "scope": format!("{DRIVE_FILE_SCOPE} openid"), "token_type": "Bearer" }),
            )
        })
        .await;
        let http = crate::http::client().unwrap();
        let pkce = Pkce::from_verifier("x".repeat(64));
        let tokens = exchange_code(
            &http,
            &endpoints(&server),
            &OAuthClient::new("cid", Some("sec")),
            &Secret::new("4/code".into()),
            &pkce,
            "http://127.0.0.1:5000/",
            1_000,
        )
        .await
        .unwrap();
        assert_eq!(tokens.access.token.expose(), "at");
        assert_eq!(tokens.access.expires_at, 1_000 + 3_599_000);
        assert!(tokens.access.fresh_at(1_000));
        assert!(!tokens.access.fresh_at(1_000 + 3_599_000 - 59_000));
        assert_eq!(tokens.refresh.unwrap().expose(), "rt");
        let r = &server.requests()[0];
        assert_eq!(r.method, "POST");
        assert_eq!(r.path(), "/token");
        assert_eq!(
            r.header("content-type"),
            Some("application/x-www-form-urlencoded")
        );
        let form = r.form();
        let get = |n: &str| form.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
        assert_eq!(get("grant_type"), Some("authorization_code"));
        assert_eq!(get("code"), Some("4/code"));
        assert_eq!(get("code_verifier"), Some("x".repeat(64).as_str()));
        assert_eq!(get("redirect_uri"), Some("http://127.0.0.1:5000/"));
        assert_eq!(get("client_id"), Some("cid"));
        assert_eq!(get("client_secret"), Some("sec"));
    }

    #[tokio::test]
    async fn an_exchange_without_the_drive_scope_or_refused_fails_the_sign_in() {
        let server = MockServer::start(|r| match r.form_value("code").as_deref() {
            Some("no-drive") => Reply::json(
                200,
                json!({ "access_token": "at", "expires_in": 3599, "refresh_token": "rt", "scope": "openid" }),
            ),
            _ => Reply::json(400, json!({ "error": "invalid_grant", "error_description": "Bad Request" })),
        })
        .await;
        let http = crate::http::client().unwrap();
        let pkce = Pkce::from_verifier("x".repeat(64));
        let client = OAuthClient::new("cid", None);
        for (code, want) in [
            (
                "no-drive",
                SyncError::SignInFailed(
                    SignInProblem::NotGranted,
                    "Drive access not granted".into(),
                ),
            ),
            (
                "stale",
                SyncError::SignInFailed(SignInProblem::Other, "invalid_grant: Bad Request".into()),
            ),
        ] {
            let e = exchange_code(
                &http,
                &endpoints(&server),
                &client,
                &Secret::new(code.into()),
                &pkce,
                "http://127.0.0.1:1/",
                0,
            )
            .await
            .unwrap_err();
            assert_eq!(e, want);
        }
    }

    #[tokio::test]
    async fn refresh_errors_are_told_apart() {
        let server = MockServer::start(|r| match r.form_value("refresh_token").as_deref() {
            Some("revoked") => Reply::json(400, json!({ "error": "invalid_grant" })),
            Some("bad-client") => Reply::json(401, json!({ "error": "invalid_client" })),
            _ => Reply::bytes(503, b"down"),
        })
        .await;
        let http = crate::http::client().unwrap();
        let client = OAuthClient::new("cid", None);
        let run = |t: &'static str| {
            let (http, client, ep) = (http.clone(), client.clone(), endpoints(&server));
            async move { refresh(&http, &ep, &client, &Secret::new(t.into()), 0).await }
        };
        assert_eq!(run("revoked").await.unwrap_err(), SyncError::AccessExpired);
        assert_eq!(
            run("bad-client").await.unwrap_err(),
            SyncError::Request("token refresh: invalid_client".into())
        );
        assert_eq!(
            run("other").await.unwrap_err(),
            SyncError::Request("token endpoint: HTTP 503".into())
        );
    }
}
