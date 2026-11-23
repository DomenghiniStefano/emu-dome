//! What can go wrong while signing in or syncing, as data (the interface gets an
//! [`emudome_core::ErrorCode`] from it, never a sentence).
//!
//! `detail` strings carry technical text (an HTTP status and Google's reason, an OS error) for
//! the logs only: the conversion to an [`ErrorCode`] logs them and keeps the values the
//! interface words (an [`IoCause`], a [`SignInProblem`]). They never hold a token, a client
//! secret or an authorization code: the messages built here name what failed, not what was sent.

use std::fmt;
use std::path::Path;

use emudome_core::{ErrorCode, IoCause, SignInProblem};

/// A sync or sign-in failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// This build has no Google OAuth client (`EMUDOME_GOOGLE_CLIENT_ID` was not set when it was
    /// compiled): Drive sync does not exist in it.
    NotConfigured,
    /// No Google account is signed in on this device.
    NotSignedIn,
    /// The sign-in did not finish (why, as far as EmuDome can tell, and the technical text): the
    /// user refused, closed the browser, the time ran out, or Google refused the code.
    SignInFailed(SignInProblem, String),
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
    /// A local file could not be read or written (the cause, and the technical text).
    Local(IoCause, String),
    /// The sync index is there but could not be read: nothing may be decided from it, and it is
    /// never written over until it reads again.
    IndexReadFailed(IoCause, String),
    /// The system's credential store refused (no Secret Service on a Linux desktop, a locked
    /// keyring): the sign-in cannot be kept.
    Credentials(String),
}

impl SyncError {
    /// The local file at `path` failed with `e`.
    pub fn local(path: &Path, e: &std::io::Error) -> Self {
        SyncError::Local(IoCause::of(e), format!("{}: {e}", path.display()))
    }

    /// A local failure that is not the file system's (a conversion, a serialization).
    pub fn local_other(detail: impl fmt::Display) -> Self {
        SyncError::Local(IoCause::Other, detail.to_string())
    }

    /// The sign-in failed for no reason EmuDome tells apart.
    pub fn sign_in(detail: impl fmt::Display) -> Self {
        SyncError::SignInFailed(SignInProblem::Other, detail.to_string())
    }
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::NotConfigured => f.write_str("no Google OAuth client in this build"),
            SyncError::NotSignedIn => f.write_str("not signed in"),
            SyncError::SignInFailed(p, d) => write!(f, "sign-in failed ({p:?}): {d}"),
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
            SyncError::Local(_, d) => write!(f, "local file: {d}"),
            SyncError::IndexReadFailed(_, d) => write!(f, "sync index unreadable: {d}"),
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

impl From<SyncError> for ErrorCode {
    /// What the interface shows. A local file that could not be read or written is the saves'
    /// own code; a Drive file gone is a request failure (the next sync finds the folder again).
    /// The technical text of a failure goes to the log here: the code carries values only.
    fn from(e: SyncError) -> Self {
        match &e {
            SyncError::SignInFailed(..)
            | SyncError::Offline(_)
            | SyncError::Request(_)
            | SyncError::NotFound(_)
            | SyncError::Local(..)
            | SyncError::IndexReadFailed(..)
            | SyncError::Credentials(_) => log::warn!("cloud: {e}"),
            SyncError::NotConfigured
            | SyncError::NotSignedIn
            | SyncError::SignInUnsupported
            | SyncError::AccessExpired
            | SyncError::QuotaFull { .. } => {}
        }
        match e {
            SyncError::NotConfigured => ErrorCode::CloudNotConfigured,
            SyncError::NotSignedIn => ErrorCode::CloudNotSignedIn,
            SyncError::SignInFailed(reason, _) => ErrorCode::CloudSignInFailed { reason },
            SyncError::SignInUnsupported => ErrorCode::CloudSignInUnsupported,
            SyncError::AccessExpired => ErrorCode::CloudAccessExpired,
            SyncError::Offline(_) => ErrorCode::CloudOffline,
            SyncError::QuotaFull { needed, available } => {
                ErrorCode::CloudQuotaFull { needed, available }
            }
            SyncError::Request(_) | SyncError::NotFound(_) => ErrorCode::CloudRequestFailed,
            SyncError::Local(cause, _) => ErrorCode::SaveFileFailed { cause },
            SyncError::IndexReadFailed(cause, _) => ErrorCode::CloudIndexReadFailed { cause },
            SyncError::Credentials(_) => ErrorCode::CloudCredentialsUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use emudome_core::{ErrorCode, IoCause, SignInProblem};
    use serde_json::json;

    use super::SyncError;

    #[test]
    fn every_failure_reaches_the_interface_as_a_code_without_its_text() {
        let cases = [
            (
                SyncError::NotConfigured,
                json!({ "code": "cloudNotConfigured" }),
            ),
            (
                SyncError::NotSignedIn,
                json!({ "code": "cloudNotSignedIn" }),
            ),
            (
                SyncError::SignInFailed(SignInProblem::TimedOut, "timed out".into()),
                json!({ "code": "cloudSignInFailed", "params": { "reason": "timedOut" } }),
            ),
            (
                SyncError::sign_in("loopback listener: denied"),
                json!({ "code": "cloudSignInFailed", "params": { "reason": "other" } }),
            ),
            (
                SyncError::SignInUnsupported,
                json!({ "code": "cloudSignInUnsupported" }),
            ),
            (
                SyncError::AccessExpired,
                json!({ "code": "cloudAccessExpired" }),
            ),
            (
                SyncError::Offline("dns".into()),
                json!({ "code": "cloudOffline" }),
            ),
            (
                SyncError::QuotaFull {
                    needed: 10,
                    available: 2,
                },
                json!({ "code": "cloudQuotaFull", "params": { "needed": 10, "available": 2 } }),
            ),
            (
                SyncError::Request("HTTP 500 backendError".into()),
                json!({ "code": "cloudRequestFailed" }),
            ),
            (
                SyncError::NotFound("file x".into()),
                json!({ "code": "cloudRequestFailed" }),
            ),
            (
                SyncError::local(
                    std::path::Path::new("D:/saves/gba/1a/game.sav"),
                    &std::io::Error::from(std::io::ErrorKind::PermissionDenied),
                ),
                json!({ "code": "saveFileFailed", "params": { "cause": "permissionDenied" } }),
            ),
            (
                SyncError::local_other("bad range"),
                json!({ "code": "saveFileFailed", "params": { "cause": "other" } }),
            ),
            (
                SyncError::IndexReadFailed(IoCause::NotFound, "io".into()),
                json!({ "code": "cloudIndexReadFailed", "params": { "cause": "notFound" } }),
            ),
            (
                SyncError::Credentials("no daemon".into()),
                json!({ "code": "cloudCredentialsUnavailable" }),
            ),
        ];
        for (e, want) in cases {
            assert_eq!(serde_json::to_value(ErrorCode::from(e)).unwrap(), want);
        }
    }
}
