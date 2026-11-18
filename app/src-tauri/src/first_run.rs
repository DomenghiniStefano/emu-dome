//! Tauri side of the first run (`emudome_core::first_run`, Trello #35): whether to show it, its
//! steps, and the «done» marker.
//!
//! The marker is a file, `<app local data>/first-run/done`, not a setting: it is a fact about
//! this device (like `local-settings/device.json`, never synced), not a choice the user makes,
//! so it stays out of the settings catalogue. A library already set up (folders chosen or games
//! in it) counts as done, so an existing user upgrading never sees the flow; the marker is then
//! written at once, so removing every folder later does not bring the flow back.

use std::path::PathBuf;

use emudome_core::first_run::{self, Step};
use emudome_core::ErrorCode;
use tauri::{AppHandle, Manager};

fn marker(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_local_data_dir()
        .inspect_err(|e| log::warn!("first run marker: {e}"))
        .ok()
        .map(|dir| dir.join("first-run").join("done"))
}

fn write_marker(app: &AppHandle) -> Result<(), ErrorCode> {
    let Some(path) = marker(app) else {
        return Ok(());
    };
    let failed = |e: std::io::Error| ErrorCode::SettingsWriteFailed {
        detail: e.to_string(),
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(failed)?;
    }
    emudome_core::saves::write_atomic(&path, b"1\n").map_err(failed)
}

/// Whether the interface starts with the first run.
#[tauri::command]
pub fn first_run_show(app: AppHandle) -> bool {
    let done = marker(&app).is_some_and(|p| p.is_file());
    let set_up = crate::library::is_set_up(&app);
    if set_up && !done {
        if let Err(e) = write_marker(&app) {
            log::warn!("first run marker not written: {e:?}");
        }
    }
    first_run::should_show(done, set_up)
}

/// The steps, given whether a controller is connected now.
#[tauri::command]
pub fn first_run_steps(controller: bool) -> Vec<Step> {
    first_run::steps(controller)
}

/// «Vai alla libreria»: the first run never shows again on this device.
#[tauri::command]
pub fn first_run_done(app: AppHandle) -> Result<(), ErrorCode> {
    write_marker(&app)
}
