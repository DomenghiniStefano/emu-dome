//! The one-time move of saves and settings named the old way to the game key
//! (`emudome_core::migration`, Trello #70), and of screenshots named by ROM file stem to each
//! game's screenshots folder (Trello #85, its own marker `migration-85/done`), run at start on
//! its own thread.
//!
//! - It starts after the library and the settings are open, before any game can open: opening a
//!   game and the Saves commands wait for it ([`wait`], at most [`WAIT_AT_MOST`], so a stuck disk
//!   never blocks a game for good).
//! - It does not run while `library.json` cannot be read: what the library then holds is not the
//!   user's games, so no file may be judged from it. It is tried again at the next start.
//! - It holds the settings lock while it moves the settings files, and makes the engine read
//!   them again afterwards.
//! - What went wrong, or what had to stay where it was, reaches the interface as an `ErrorCode`
//!   (`savesMigrationFailed`, `savesMigrationIncomplete`) in `saves_migration_status` and in the
//!   `saves://migration` event; it never stops a game from opening.

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use emudome_core::migration::{self, Paths};
use emudome_core::ErrorCode;
use parking_lot::{Condvar, Mutex};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::settings::SettingsState;

/// The longest a game or the Saves screen waits for the move.
const WAIT_AT_MOST: Duration = Duration::from_secs(20);

/// Where the move stands. TypeScript: `SavesMigration` (generated).
#[derive(Clone, Debug, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SavesMigration")]
pub struct Status {
    pub running: bool,
    /// What the user should know: it failed, or some old files stayed where they were.
    pub problem: Option<ErrorCode>,
}

#[derive(Default)]
pub struct MigrationState {
    status: Mutex<Status>,
    ended: Condvar,
}

/// Runs the move on its own thread (managed state first, so [`wait`] sees it running).
pub fn start(app: &tauri::App) {
    app.manage(MigrationState {
        status: Mutex::new(Status {
            running: true,
            problem: None,
        }),
        ended: Condvar::new(),
    });
    let handle = app.handle().clone();
    let spawned = std::thread::Builder::new()
        .name("game-key-migration".into())
        .spawn(move || {
            let problem = run(&handle);
            finish(&handle, problem);
        });
    if let Err(e) = spawned {
        log::error!("game key migration thread: {e}");
        finish(app.handle(), Some(ErrorCode::SavesMigrationFailed));
    }
}

fn finish(app: &AppHandle, problem: Option<ErrorCode>) {
    let Some(state) = app.try_state::<MigrationState>() else {
        return;
    };
    let status = {
        let mut status = state.status.lock();
        *status = Status {
            running: false,
            problem,
        };
        status.clone()
    };
    state.ended.notify_all();
    if let Err(e) = app.emit(crate::events::SAVES_MIGRATION, status) {
        log::warn!("saves://migration: {e}");
    }
}

/// Waits for the move to end (at most [`WAIT_AT_MOST`]).
pub fn wait(app: &AppHandle) {
    let Some(state) = app.try_state::<MigrationState>() else {
        return;
    };
    let deadline = Instant::now() + WAIT_AT_MOST;
    let mut status = state.status.lock();
    while status.running {
        if state.ended.wait_until(&mut status, deadline).timed_out() {
            log::warn!("game key migration still running after {WAIT_AT_MOST:?}: not waiting");
            return;
        }
    }
}

fn now_ms() -> u64 {
    crate::unix_ms(SystemTime::now()).unwrap_or_default()
}

fn paths(app: &AppHandle) -> tauri::Result<Paths> {
    let data = crate::dirs::data(app)?;
    let local = crate::dirs::local(app)?;
    Ok(Paths {
        saves_root: data.join("saves"),
        settings_games: data.join("settings").join("game"),
        device_file: local.join("local-settings").join("device.json"),
        work_dir: data.join("migration-70"),
    })
}

fn failed(detail: String) -> Option<ErrorCode> {
    log::warn!("savesMigrationFailed: {detail}");
    Some(ErrorCode::SavesMigrationFailed)
}

/// One pass of each move; the problem to show, if any (the saves' first).
fn run(app: &AppHandle) -> Option<ErrorCode> {
    let paths = match paths(app) {
        Ok(p) => p,
        Err(e) => return failed(e.to_string()),
    };
    let saves = run_saves(app, &paths);
    let screenshots = run_screenshots(app, &paths);
    saves.or(screenshots)
}

/// The screenshots named by ROM file stem moved to each game's folder (Trello #85), once:
/// `<data>/migration-85/`. Those it cannot give to one game stay where they are (listed in its
/// report, not shown: they are still in the screenshots folder for the user to browse); an
/// error is shown as `savesMigrationFailed`.
fn run_screenshots(app: &AppHandle, paths: &Paths) -> Option<ErrorCode> {
    let work_dir = paths.work_dir.with_file_name("migration-85");
    if work_dir.join("done").is_file() {
        return None;
    }
    let known = match crate::library::known_files(app) {
        Ok(k) => k,
        Err(e) => return failed(format!("library not read, no screenshot moved: {e:?}")),
    };
    let report = migration::run_screenshots(&paths.saves_root, &work_dir, &known, now_ms());
    log::info!(
        "screenshots migration: {} moved, {} left, {} errors, done: {}",
        report.moved.len(),
        report.left.len(),
        report.errors.len(),
        report.done
    );
    (!report.errors.is_empty()).then(|| {
        log::warn!("savesMigrationFailed: {}", report.errors.join("; "));
        ErrorCode::SavesMigrationFailed
    })
}

/// The saves and settings named the old way moved to the game key (Trello #70), once.
fn run_saves(app: &AppHandle, paths: &Paths) -> Option<ErrorCode> {
    if migration::is_done(paths) {
        return None;
    }
    let known = match crate::library::known_files(app) {
        Ok(k) => k,
        Err(e) => return failed(format!("library not read, nothing moved: {e:?}")),
    };
    let report = match app.try_state::<SettingsState>() {
        Some(settings) => {
            let mut settings = settings.0.lock();
            let report = migration::run(paths, &known, now_ms());
            settings.forget_files();
            report
        }
        None => return failed("settings not open".into()),
    };
    log::info!(
        "game key migration: {} moved, {} left, {} shared folders, {} errors, done: {}",
        report.moved.len(),
        report.left.len(),
        report.shared.len(),
        report.errors.len(),
        report.done
    );
    if !report.errors.is_empty() {
        return failed(report.errors.join("; "));
    }
    (!report.left.is_empty()).then(|| ErrorCode::SavesMigrationIncomplete {
        left: report.left.len(),
        report: report_path(paths),
    })
}

fn report_path(paths: &Paths) -> String {
    let report: PathBuf = paths.report();
    report.to_string_lossy().into_owned()
}

/// Where the move stands (the interface asks once at start, then listens to
/// `saves://migration`).
#[tauri::command]
pub fn saves_migration_status(app: AppHandle) -> Status {
    app.try_state::<MigrationState>()
        .map(|s| s.status.lock().clone())
        .unwrap_or_default()
}
