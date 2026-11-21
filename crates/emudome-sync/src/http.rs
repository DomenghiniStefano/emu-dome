//! The one HTTP client of the sync: reqwest on tokio, TLS by rustls with the ring provider and
//! the system's root certificates (`rustls-native-certs`: Windows' certificate store, the Linux
//! certificate files).
//!
//! Android (Phase 4): `rustls-native-certs` finds no roots there, so every HTTPS request would
//! fail the certificate check. The plan is to hand rustls Android's own trust store from the
//! Kotlin side (`KeyStore.getInstance("AndroidCAStore")`, read once through
//! `emudome-platform`) into [`client_with_roots`]; until then the Android sign-in is refused
//! before any request ([`crate::SyncError::SignInUnsupported`]).

use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;

use crate::SyncError;

/// The `User-Agent` of every request: the app and its version, nothing about the user.
pub const USER_AGENT: &str = concat!("EmuDome/", env!("CARGO_PKG_VERSION"));

/// How long a connection may take to open.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
/// How long a request may go without receiving anything (an upload chunk, a download).
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// The client, with the system's root certificates. Reads the certificate store: build it once
/// and clone it (a clone shares the connections).
pub fn client() -> Result<reqwest::Client, SyncError> {
    let found = rustls_native_certs::load_native_certs();
    for e in &found.errors {
        log::warn!("cloud: a system certificate could not be read: {e}");
    }
    client_with_roots(found.certs)
}

/// The client trusting `roots` only.
pub fn client_with_roots(
    roots: impl IntoIterator<Item = CertificateDer<'static>>,
) -> Result<reqwest::Client, SyncError> {
    let mut store = RootCertStore::empty();
    let (added, ignored) = store.add_parsable_certificates(roots);
    if ignored > 0 {
        log::debug!("cloud: {ignored} system certificates not usable by rustls");
    }
    if added == 0 {
        log::warn!("cloud: no root certificate found: HTTPS requests will fail");
    }
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| SyncError::Request(format!("TLS set-up: {e}")))?
    .with_root_certificates(store)
    .with_no_client_auth();
    reqwest::Client::builder()
        .use_preconfigured_tls(tls)
        .user_agent(USER_AGENT)
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()
        .map_err(|e| SyncError::Request(e.without_url().to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_client_builds_with_the_system_roots() {
        super::client().unwrap();
        super::client_with_roots(Vec::new()).unwrap();
    }
}
