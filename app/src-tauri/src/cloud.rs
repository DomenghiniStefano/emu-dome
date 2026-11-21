//! Tauri side of the Google Drive account (Trello #20, `emudome-sync`): whether sync exists in
//! this build, whether this device is signed in, signing in and out. Nothing here syncs yet:
//! pull before launch and push after exit are `emudome_sync::sync::Syncer`'s, wired into the
//! game flow with the Cloud screens (#31). No event is sent.
//!
//! The refresh token lives in the system's credential store (`emudome_platform::cloud`), under
//! the service `it.domehub.emudome` and the account [`emudome_sync::auth::REFRESH_TOKEN_ACCOUNT`];
//! never in a file or a log.

use std::sync::Arc;

use emudome_core::ErrorCode;
use emudome_platform::cloud as system;
use emudome_sync::auth::{Auth, SecretStore, REFRESH_TOKEN_ACCOUNT, SIGN_IN_TIMEOUT};
use emudome_sync::loopback::{page, Pages};
use emudome_sync::oauth::{Endpoints, OAuthClient, Secret};
use emudome_sync::SyncError;
use parking_lot::Mutex;
use serde::Serialize;
use ts_rs::TS;

/// The account as the interface sees it. TypeScript: `CloudStatus`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub struct CloudStatus {
    /// This build has the Google OAuth client (otherwise sync does not exist in it).
    pub configured: bool,
    /// This system can sign in (the desktop; Android in Phase 4).
    pub supported: bool,
    /// A sign-in is kept on this device.
    pub signed_in: bool,
}

/// The refresh token in the system's credential store.
struct SystemStore;

fn store_error(e: system::CloudError) -> SyncError {
    match e {
        system::CloudError::Unsupported => SyncError::SignInUnsupported,
        system::CloudError::Failed(detail) => SyncError::Credentials(detail),
    }
}

impl SecretStore for SystemStore {
    fn load(&self) -> Result<Option<Secret>, SyncError> {
        system::credential_get(REFRESH_TOKEN_ACCOUNT)
            .map(|t| t.map(Secret::new))
            .map_err(store_error)
    }

    fn save(&self, token: &Secret) -> Result<(), SyncError> {
        system::credential_set(REFRESH_TOKEN_ACCOUNT, token.expose()).map_err(store_error)
    }

    fn delete(&self) -> Result<(), SyncError> {
        system::credential_delete(REFRESH_TOKEN_ACCOUNT).map_err(store_error)
    }
}

/// The account of this run, made at its first use (the HTTP client reads the system's
/// certificates once).
#[derive(Default)]
pub struct Cloud {
    auth: Mutex<Option<Arc<Auth>>>,
}

impl Cloud {
    fn auth(&self) -> Result<Arc<Auth>, ErrorCode> {
        let mut auth = self.auth.lock();
        if let Some(a) = auth.as_ref() {
            return Ok(a.clone());
        }
        let made = Arc::new(Auth::new(
            emudome_sync::http::client()?,
            Endpoints::google(),
            OAuthClient::from_build(),
            Arc::new(SystemStore),
        ));
        *auth = Some(made.clone());
        Ok(made)
    }

    async fn status(&self) -> Result<CloudStatus, ErrorCode> {
        let configured = OAuthClient::from_build().is_some();
        let supported = system::SIGN_IN_SUPPORTED;
        let signed_in = if configured && supported {
            self.auth()?.signed_in().await?
        } else {
            false
        };
        Ok(CloudStatus {
            configured,
            supported,
            signed_in,
        })
    }
}

/// Whether sync exists in this build, can sign in here, and is signed in.
#[tauri::command]
pub async fn cloud_status(cloud: tauri::State<'_, Cloud>) -> Result<CloudStatus, ErrorCode> {
    cloud.status().await
}

/// Signs in to Google Drive: the system browser opens Google's page, the user signs in there
/// and the browser comes back to a local page showing `done` (or `failed`), the interface's
/// texts. Waits up to five minutes.
#[tauri::command]
pub async fn cloud_sign_in(
    cloud: tauri::State<'_, Cloud>,
    done: String,
    failed: String,
) -> Result<CloudStatus, ErrorCode> {
    if OAuthClient::from_build().is_none() {
        return Err(ErrorCode::CloudNotConfigured);
    }
    if !system::SIGN_IN_SUPPORTED {
        return Err(ErrorCode::CloudSignInUnsupported);
    }
    let pages = Pages {
        done: page(&done),
        failed: page(&failed),
    };
    cloud
        .auth()?
        .sign_in(
            |url| system::open_url(url).map_err(|e| e.to_string()),
            &pages,
            SIGN_IN_TIMEOUT,
        )
        .await?;
    cloud.status().await
}

/// Signs this device out (the grant is revoked at Google when it can be reached).
#[tauri::command]
pub async fn cloud_sign_out(cloud: tauri::State<'_, Cloud>) -> Result<CloudStatus, ErrorCode> {
    if OAuthClient::from_build().is_some() && system::SIGN_IN_SUPPORTED {
        cloud.auth()?.sign_out().await?;
    }
    cloud.status().await
}
