//! Where the app keeps its data: Tauri's app data / app local data folders, or, with
//! `EMUDOME_DATA_DIR` set, folders inside that one only.
//!
//! The override moves **everything** (saves, settings, library, covers, first-run marker,
//! webview profile), so a test or bench run never touches the user's real data. Before
//! 2026-11-18 it moved the saves only, and a test run rewrote the real `library.json`.

use std::path::PathBuf;
use std::sync::OnceLock;

use tauri::{AppHandle, Manager, Runtime};

static OVERRIDE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Sets the `EMUDOME_DATA_DIR` override, once, before anything opens its files.
pub fn set_override(dir: Option<PathBuf>) {
    if OVERRIDE.set(dir).is_err() {
        log::warn!("data folder override set twice: the first one stays");
    }
}

fn overridden() -> Option<&'static PathBuf> {
    OVERRIDE.get().and_then(Option::as_ref)
}

/// The synced data folder (saves, `settings/`).
pub fn data<R: Runtime, M: Manager<R>>(app: &M) -> tauri::Result<PathBuf> {
    match overridden() {
        Some(dir) => Ok(dir.clone()),
        None => app.path().app_data_dir(),
    }
}

/// The per-device data folder (library, covers, `local-settings/`, first-run marker).
pub fn local<R: Runtime, M: Manager<R>>(app: &M) -> tauri::Result<PathBuf> {
    match overridden() {
        Some(dir) => Ok(dir.join("local")),
        None => app.path().app_local_data_dir(),
    }
}

/// `local` for an `AppHandle`, logging instead of failing.
pub fn local_or_log(app: &AppHandle, what: &str) -> Option<PathBuf> {
    local(app).inspect_err(|e| log::warn!("{what}: {e}")).ok()
}
