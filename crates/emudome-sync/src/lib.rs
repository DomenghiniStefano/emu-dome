//! Google Drive sync (Trello #20; the decision: docs/decisions.md, «Product»; the plan:
//! docs/feasibility.md, «Google Drive sync»).
//!
//! Everything here is independent of the operating system; what is not (opening the system
//! browser, the system's credential store, Android's sign-in) is in `emudome-platform`, handed
//! in through small traits ([`auth::SecretStore`]) or closures.
//!
//! - [`oauth`], [`loopback`], [`auth`]: the installed-app sign-in (system browser, loopback
//!   redirect on `127.0.0.1:<random port>`, PKCE S256, scope `drive.file` only), the refresh
//!   token kept in the credential store, access tokens refreshed in memory.
//! - [`http`]: the one HTTP client (reqwest, rustls with ring, the system's root certificates).
//! - [`drive`]: Drive as the sync sees it ([`drive::DriveApi`]), over REST v3
//!   ([`drive::rest::RestDrive`]: multipart uploads for small files, resumable ones in chunks
//!   for big files) and in memory for the tests ([`drive::fake::FakeDrive`]).

pub mod auth;
pub mod drive;
mod error;
pub mod http;
pub mod loopback;
pub mod oauth;
pub mod time;

#[cfg(test)]
mod test_http;

pub use error::SyncError;
