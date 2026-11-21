//! What the Drive sync (`emudome-sync`) needs from the system: opening the authorization URL in
//! the system browser, the credential store that keeps the refresh token, and this device's name.
//!
//! - Windows: the default browser through `url.dll`; Windows' Credential Manager (`keyring`,
//!   `windows-native`).
//! - Linux: `xdg-open`; the Secret Service over D-Bus (`keyring`, `async-secret-service` on
//!   zbus with `crypto-rust`: pure Rust, no libdbus, no OpenSSL). A desktop without a Secret
//!   Service (no GNOME Keyring / KWallet daemon) refuses: the sign-in cannot be kept there
//!   (to verify on such a system; not compiled on this machine, only on Linux).
//! - Android (Phase 4): nothing yet. The plan: a Kotlin `CloudBridge` in `android/` calls Google
//!   Identity Services' `AuthorizationClient.authorize()` with the `drive.file` scope and hands the
//!   access token to Rust through JNI (no refresh token kept on the phone, no loopback, no
//!   browser); `authorize()` gives a new token without asking while the grant lasts. Needs Play
//!   Services and the Android OAuth client (#49) of the same Cloud project. Until then
//!   [`open_url`] and the credential store answer [`CloudError::Unsupported`] there.
//! - macOS: not built yet; the same `keyring` crate has its backend (`apple-native`).
//!
//! The credential calls block (D-Bus, the Credential Manager): call them off the async runtime.

/// The service name the refresh token is kept under (the app identifier).
pub const CREDENTIAL_SERVICE: &str = "it.domehub.emudome";

/// Why a cloud call to the system failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudError {
    /// Not on this system (yet).
    Unsupported,
    /// The system refused; technical text from the OS or the library.
    Failed(String),
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudError::Unsupported => f.write_str("not supported on this system"),
            CloudError::Failed(d) => f.write_str(d),
        }
    }
}

/// Opens `url` in the system's default browser, without waiting for it. Only on the user's own
/// request (the sign-in button): it takes the focus.
pub fn open_url(url: &str) -> Result<(), CloudError> {
    imp::open_url(url)
}

/// This device's name as the user knows it (the computer's name), for «in use on <device>» and
/// the conflict question. Falls back to the system's name.
pub fn device_name() -> String {
    imp::device_name()
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| crate::OS_NAME.to_owned())
}

/// The secret kept for `account` under [`CREDENTIAL_SERVICE`]; `None` when there is none.
pub fn credential_get(account: &str) -> Result<Option<String>, CloudError> {
    imp::credential_get(account)
}

/// Keeps `secret` for `account`, replacing what was there.
pub fn credential_set(account: &str, secret: &str) -> Result<(), CloudError> {
    imp::credential_set(account, secret)
}

/// Forgets the secret of `account`; nothing there is not an error.
pub fn credential_delete(account: &str) -> Result<(), CloudError> {
    imp::credential_delete(account)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod keyring_store {
    use super::{CloudError, CREDENTIAL_SERVICE};

    fn entry(account: &str) -> Result<keyring::Entry, CloudError> {
        keyring::Entry::new(CREDENTIAL_SERVICE, account)
            .map_err(|e| CloudError::Failed(e.to_string()))
    }

    pub fn credential_get(account: &str) -> Result<Option<String>, CloudError> {
        match entry(account)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CloudError::Failed(e.to_string())),
        }
    }

    pub fn credential_set(account: &str, secret: &str) -> Result<(), CloudError> {
        entry(account)?
            .set_password(secret)
            .map_err(|e| CloudError::Failed(e.to_string()))
    }

    pub fn credential_delete(account: &str) -> Result<(), CloudError> {
        match entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(CloudError::Failed(e.to_string())),
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    pub use super::keyring_store::{credential_delete, credential_get, credential_set};
    use super::CloudError;

    pub fn open_url(url: &str) -> Result<(), CloudError> {
        // `url.dll` opens the default browser with the URL as one argument: no shell parses
        // the `&` of the query (as `cmd /c start` would).
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map(drop)
            .map_err(|e| CloudError::Failed(e.to_string()))
    }

    pub fn device_name() -> Option<String> {
        std::env::var("COMPUTERNAME").ok()
    }
}

#[cfg(target_os = "linux")]
mod imp {
    pub use super::keyring_store::{credential_delete, credential_get, credential_set};
    use super::CloudError;

    pub fn open_url(url: &str) -> Result<(), CloudError> {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(drop)
            .map_err(|e| CloudError::Failed(e.to_string()))
    }

    pub fn device_name() -> Option<String> {
        std::fs::read_to_string("/etc/hostname")
            .ok()
            .or_else(|| std::env::var("HOSTNAME").ok())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::CloudError;

    pub fn open_url(url: &str) -> Result<(), CloudError> {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(drop)
            .map_err(|e| CloudError::Failed(e.to_string()))
    }

    pub fn device_name() -> Option<String> {
        None
    }

    pub fn credential_get(_account: &str) -> Result<Option<String>, CloudError> {
        Err(CloudError::Unsupported)
    }

    pub fn credential_set(_account: &str, _secret: &str) -> Result<(), CloudError> {
        Err(CloudError::Unsupported)
    }

    pub fn credential_delete(_account: &str) -> Result<(), CloudError> {
        Err(CloudError::Unsupported)
    }
}

#[cfg(target_os = "android")]
mod imp {
    use super::CloudError;

    pub fn open_url(_url: &str) -> Result<(), CloudError> {
        Err(CloudError::Unsupported)
    }

    pub fn device_name() -> Option<String> {
        None
    }

    pub fn credential_get(_account: &str) -> Result<Option<String>, CloudError> {
        Err(CloudError::Unsupported)
    }

    pub fn credential_set(_account: &str, _secret: &str) -> Result<(), CloudError> {
        Err(CloudError::Unsupported)
    }

    pub fn credential_delete(_account: &str) -> Result<(), CloudError> {
        Err(CloudError::Unsupported)
    }
}

/// Whether the sign-in can run on this system: the desktop's loopback flow; Android's comes in
/// Phase 4.
pub const SIGN_IN_SUPPORTED: bool = cfg!(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos"
));

#[cfg(test)]
mod tests {
    #[test]
    fn the_device_has_a_name() {
        // Never the credential store here: tests do not write to the system's store.
        assert!(!super::device_name().is_empty());
    }
}
