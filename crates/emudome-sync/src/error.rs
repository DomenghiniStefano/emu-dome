//! What can go wrong while signing in or syncing, as data (the interface gets an
//! [`emudome_core::ErrorCode`] from it, never a sentence).
//!
//! `detail` strings carry technical text (an HTTP status and Google's reason, an OS error) for
//! the logs and the diagnostics; they never hold a token, a client secret or an authorization
//! code: the messages built here name what failed, not what was sent.

use std::fmt;

/// A sync or sign-in failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// This build has no Google OAuth client (`EMUDOME_GOOGLE_CLIENT_ID` was not set when it was
    /// compiled): Drive sync does not exist in it.
    NotConfigured,
    /// No Google account is signed in on this device.
    NotSignedIn,
    /// The sign-in did not finish: the user refused, closed the browser, the time ran out, or
    /// Google refused the code.
    SignInFailed(String),
    /// Signing in is not possible on this system yet (Android: Phase 4).
    SignInUnsupported,
    /// The refresh token was revoked or expired (`invalid_grant`): sign in again.
    AccessExpired,
    /// Google could not be reached (no network, DNS, timeout).
    Offline(String),
    /// Drive has no room for `needed` bytes (`available` left, `u64::MAX` when unlimited).
    QuotaFull { needed: u64, available: u64 },
    /// Drive answered with an error none of the others names (status and reason).
    Request(String),
    /// A Drive file the request named is gone (deleted on Drive, or never visible to the app).
    NotFound(String),
    /// A local file could not be read or written.
    Local(String),
    /// The sync index is there but could not be read: nothing may be decided from it, and it is
    /// never written over until it reads again.
    IndexReadFailed(String),
    /// The system's credential store refused (no Secret Service on a Linux desktop, a locked
    /// keyring): the sign-in cannot be kept.
    Credentials(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::NotConfigured => f.write_str("no Google OAuth client in this build"),
            SyncError::NotSignedIn => f.write_str("not signed in"),
            SyncError::SignInFailed(d) => write!(f, "sign-in failed: {d}"),
            SyncError::SignInUnsupported => f.write_str("sign-in not supported on this system"),
            SyncError::AccessExpired => f.write_str("access expired or revoked"),
            SyncError::Offline(d) => write!(f, "offline: {d}"),
            SyncError::QuotaFull { needed, available } => {
                write!(
                    f,
                    "Drive full: {needed} bytes needed, {available} available"
                )
            }
            SyncError::Request(d) => write!(f, "Drive request failed: {d}"),
            SyncError::NotFound(d) => write!(f, "not found on Drive: {d}"),
            SyncError::Local(d) => write!(f, "local file: {d}"),
            SyncError::IndexReadFailed(d) => write!(f, "sync index unreadable: {d}"),
            SyncError::Credentials(d) => write!(f, "credential store: {d}"),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<reqwest::Error> for SyncError {
    /// A transport failure: offline unless the request itself or the answer was wrong (a
    /// connection cut halfway through is offline too). The URL is left out of the text: a
    /// resumable upload's session URL is a credential of its own.
    fn from(e: reqwest::Error) -> Self {
        let e = e.without_url();
        if e.is_builder() || e.is_decode() || e.is_status() || e.is_redirect() {
            SyncError::Request(e.to_string())
        } else {
            SyncError::Offline(e.to_string())
        }
    }
}
