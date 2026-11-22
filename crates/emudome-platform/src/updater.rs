//! EmuDome updating itself (Trello #44): Tauri's updater plugin on the desktop, nothing on
//! Android (updates come from Google Play there).
//!
//! The update key and the manifest's address are **build-time values**, like the Google client
//! id: `EMUDOME_UPDATER_PUBKEY` (the public half of the key the owner signs releases with, made
//! with `pnpm tauri signer generate` outside the repository; docs/release.md) and, optionally,
//! `EMUDOME_UPDATER_ENDPOINT` (default: [`DEFAULT_ENDPOINT`], the `latest.json` of the newest
//! GitHub release that is not a pre-release). A build without the key cannot update itself
//! ([`CONFIGURED`] false): the plugin is not even registered, and the interface shows
//! `general.updates` disabled with the reason. The private key never enters the repository nor
//! the app: only the release build's environment has it (`TAURI_SIGNING_PRIVATE_KEY`).
//!
//! The plugin reads `plugins.updater` of `tauri.conf.json`, which holds an empty `pubkey`: the
//! key given here at build time takes its place.
//!
//! Every function blocks (the network): call them off the main thread.

/// The update manifest by default: the newest GitHub release of the repository that is not a
/// pre-release (the cores' own releases are pre-releases, so they never show here).
pub const DEFAULT_ENDPOINT: &str =
    "https://github.com/DomenghiniStefano/emu-dome/releases/latest/download/latest.json";

/// Why an update step failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    /// This build cannot update itself (no key, or a system without updates of its own).
    Unavailable,
    /// The step failed (`String`: the plugin's message, English, for the logs and `detail`).
    Failed(String),
}

#[cfg(not(target_os = "android"))]
mod imp {
    use tauri_plugin_updater::UpdaterExt;

    use super::{UpdateError, DEFAULT_ENDPOINT};

    const PUBKEY: Option<&str> = option_env!("EMUDOME_UPDATER_PUBKEY");
    const ENDPOINT: Option<&str> = option_env!("EMUDOME_UPDATER_ENDPOINT");

    pub const CONFIGURED: bool = match PUBKEY {
        Some(key) => !key.is_empty(),
        None => false,
    };

    pub fn with_updater<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
        match PUBKEY.filter(|k| !k.is_empty()) {
            Some(key) => builder.plugin(tauri_plugin_updater::Builder::new().pubkey(key).build()),
            None => builder,
        }
    }

    pub struct Found(tauri_plugin_updater::Update);

    impl Found {
        pub fn version(&self) -> &str {
            &self.0.version
        }
    }

    pub struct Downloaded(Vec<u8>);

    fn failed(e: impl std::fmt::Display) -> UpdateError {
        UpdateError::Failed(e.to_string())
    }

    pub fn check(app: &tauri::AppHandle) -> Result<Option<Found>, UpdateError> {
        if !CONFIGURED {
            return Err(UpdateError::Unavailable);
        }
        let endpoint = ENDPOINT.unwrap_or(DEFAULT_ENDPOINT);
        let url = endpoint.parse().map_err(failed)?;
        let updater = app
            .updater_builder()
            .endpoints(vec![url])
            .map_err(failed)?
            .build()
            .map_err(failed)?;
        tauri::async_runtime::block_on(updater.check())
            .map(|found| found.map(Found))
            .map_err(failed)
    }

    pub fn download(found: &Found) -> Result<Downloaded, UpdateError> {
        // The plugin checks the signature against the key before it answers.
        tauri::async_runtime::block_on(found.0.download(|_, _| {}, || {}))
            .map(Downloaded)
            .map_err(failed)
    }

    pub fn install(
        app: &tauri::AppHandle,
        found: &Found,
        bytes: &Downloaded,
    ) -> Result<(), UpdateError> {
        // Windows: starts the NSIS installer (passive, it restarts EmuDome) and exits here.
        found.0.install(&bytes.0).map_err(failed)?;
        // Linux (the AppImage replaced in place): start the new version.
        app.restart()
    }
}

#[cfg(target_os = "android")]
mod imp {
    use super::UpdateError;

    pub const CONFIGURED: bool = false;

    pub fn with_updater<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
        builder
    }

    pub struct Found(String);

    impl Found {
        pub fn version(&self) -> &str {
            &self.0
        }
    }

    pub struct Downloaded;

    pub fn check(_app: &tauri::AppHandle) -> Result<Option<Found>, UpdateError> {
        Err(UpdateError::Unavailable)
    }

    pub fn download(_found: &Found) -> Result<Downloaded, UpdateError> {
        Err(UpdateError::Unavailable)
    }

    pub fn install(
        _app: &tauri::AppHandle,
        _found: &Found,
        _bytes: &Downloaded,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::Unavailable)
    }
}

/// Whether this build can update itself: the desktop, built with `EMUDOME_UPDATER_PUBKEY`.
pub const CONFIGURED: bool = imp::CONFIGURED;

/// Registers the updater plugin on `builder` when this build can update itself.
pub fn with_updater<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    imp::with_updater(builder)
}

/// A newer version the release server announced.
pub struct FoundUpdate(imp::Found);

impl FoundUpdate {
    /// Its version, as the release names it (`0.0.2`).
    pub fn version(&self) -> &str {
        self.0.version()
    }
}

/// A newer version downloaded, its signature checked against the build's key.
pub struct DownloadedUpdate(imp::Downloaded);

/// Asks the release server whether a version newer than this one exists.
pub fn check_update(app: &tauri::AppHandle) -> Result<Option<FoundUpdate>, UpdateError> {
    imp::check(app).map(|found| found.map(FoundUpdate))
}

/// Downloads `found` and checks its signature; nothing is installed.
pub fn download_update(found: &FoundUpdate) -> Result<DownloadedUpdate, UpdateError> {
    imp::download(&found.0).map(DownloadedUpdate)
}

/// Installs a downloaded update and restarts into it: on Windows the installer takes over and
/// this process exits; on Linux the new version starts. Returns only when it failed. The caller
/// closes the open game first: nothing here waits for its saves.
pub fn install_update(
    app: &tauri::AppHandle,
    found: &FoundUpdate,
    bytes: &DownloadedUpdate,
) -> Result<(), UpdateError> {
    imp::install(app, &found.0, &bytes.0)
}
