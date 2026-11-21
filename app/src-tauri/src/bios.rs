//! Tauri side of the user's BIOS and firmware files (`emudome_core::bios`, Trello #68).
//!
//! - `bios_choose`: the row's picker. The file is checked, and a recognised one is copied into
//!   the core's system folder (`<app local data>/system/<core>/`, per device like the `bios.*`
//!   settings); the setting keeps the picked path. A file of the right size that EmuDome does not
//!   recognise is kept as the choice but not copied (the core's replacement stays in use).
//! - `bios_status`: what each wired row shows (`builtin`, `verified`, `unrecognised`).
//! - A `bios.*` row reset (its value empty again): EmuDome's copy is deleted, never the user's
//!   file.
//! - Opening a game: the valid copies in place and the game's settings give the core's options
//!   (`emudome_core::bios::plan`); what fell back is sent as `emu://bios-fallback`
//!   `{ code, params }` (an `ErrorCode`), once per fallback.

use std::path::{Path, PathBuf};

use emudome_core::bios::{self, files, Assessment, Status};
use emudome_core::core_profile::CoreId;
use emudome_core::settings::{ConsoleId, Scope, Value};
use emudome_core::ErrorCode;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::settings::SettingsState;

/// The system folder of `core` (`GET_SYSTEM_DIRECTORY`), `None` when the app's local data
/// folder is unknown.
pub fn core_dir(app: &AppHandle, core: &str) -> Option<PathBuf> {
    crate::dirs::local_or_log(app, "system folder")
        .map(|local| files::core_dir(&local.join("system"), core))
}

/// The folder `core` gets as `GET_SYSTEM_DIRECTORY`: its own system folder when it has a BIOS
/// table, `shared` (the folder every core got before #68) otherwise, so a core without one
/// (Azahar keeps its own data there) is not moved.
pub fn system_dir(app: &AppHandle, core: CoreId, shared: PathBuf) -> PathBuf {
    bios::table(core)
        .and_then(|_| core_dir(app, core.name()))
        .unwrap_or(shared)
}

/// The value of a `bios.*` row (the picked file's path; empty = none).
fn chosen(app: &AppHandle, key: &str) -> bool {
    let state = app.state::<SettingsState>();
    let resolved = state.0.lock().resolve(key, &Scope::Global);
    match resolved {
        Ok(r) => matches!(r.value, Some(Value::Text(t)) if !t.is_empty()),
        Err(e) => {
            log::warn!("{key} unavailable: {e:?}");
            false
        }
    }
}

/// One wired row and its status. TypeScript: `BiosRow` (generated).
#[derive(Serialize, TS)]
#[ts(export_to = "app/")]
pub struct BiosRow {
    #[ts(as = "emudome_core::contract::SettingKey")]
    key: &'static str,
    status: Status,
}

/// Every `bios.*` row EmuDome handles, with what it shows.
#[tauri::command]
pub fn bios_status(app: AppHandle) -> Vec<BiosRow> {
    bios::TABLES
        .iter()
        .flat_map(|t| t.slots.iter().map(move |s| (t.core, s)))
        .filter_map(|(core, spec)| {
            let key = spec.setting?;
            let in_place =
                core_dir(&app, core.name()).is_some_and(|d| files::is_in_place(&d, spec));
            Some(BiosRow {
                key,
                status: bios::status(chosen(&app, key), in_place),
            })
        })
        .collect()
}

/// The picker of a `bios.*` row: resolves the row's new status, or `None` when cancelled.
#[tauri::command]
pub async fn bios_choose(app: AppHandle, key: String) -> Result<Option<Status>, ErrorCode> {
    let Some(file) = crate::library::pick_any_file(&app) else {
        return Ok(None);
    };
    choose(&app, &key, &file).map(Some)
}

/// Checks `file` for the row `key`, copies it when recognised and stores the choice.
fn choose(app: &AppHandle, key: &str, file: &Path) -> Result<Status, ErrorCode> {
    let (table, spec) = bios::slot_for_setting(key).ok_or_else(|| ErrorCode::SettingUnknown {
        key: key.to_owned(),
    })?;
    let probe = files::probe(file).map_err(|e| ErrorCode::BiosUnreadable {
        detail: e.to_string(),
    })?;
    let all = bios::TABLES.iter().flat_map(|t| t.slots.iter());
    let dir = core_dir(app, table.core.name()).ok_or_else(|| ErrorCode::BiosFileFailed {
        detail: "no app local data folder".into(),
    })?;
    let failed = |e: std::io::Error| ErrorCode::BiosFileFailed {
        detail: e.to_string(),
    };
    let status = match bios::assess(spec, all, &probe) {
        Assessment::Install => {
            files::install(&dir, spec, file).map_err(failed)?;
            Status::Verified
        }
        Assessment::KeepUnrecognised => {
            files::remove(&dir, spec).map_err(failed)?;
            Status::Unrecognised
        }
        Assessment::Refuse(code) => return Err(code),
    };
    let path = Value::Text(file.to_string_lossy().into_owned());
    let stored = {
        let state = app.state::<SettingsState>();
        let mut settings = state.0.lock();
        settings.set(key, &Scope::Global, path)
    };
    // Not stored: no copy without its choice.
    let level = stored.inspect_err(|_| {
        if let Err(e) = files::remove(&dir, spec) {
            log::warn!("{key}: copy not deleted: {e}");
        }
    })?;
    log::info!("{key}: {status:?}");
    crate::settings::emit_changed(app, &[key], level);
    Ok(status)
}

/// Settings changed: a `bios.*` row with no file any more loses EmuDome's copy.
pub fn settings_changed(app: &AppHandle, keys: &[&str]) {
    for key in keys {
        let Some((table, spec)) = bios::slot_for_setting(key) else {
            continue;
        };
        if chosen(app, key) {
            continue;
        }
        if let Some(dir) = core_dir(app, table.core.name()) {
            match files::remove(&dir, spec) {
                Ok(()) => log::info!("{key}: back to the built-in replacement"),
                Err(e) => log::warn!("{key}: copy not deleted: {e}"),
            }
        }
    }
}

/// The core options and fallbacks for a game of `console` run by `core`, from the valid files
/// in place and the game's settings (`read`).
pub fn plan(
    app: &AppHandle,
    core: CoreId,
    console: ConsoleId,
    read: &dyn Fn(&str) -> Option<Value>,
) -> bios::Plan {
    let in_place = bios::table(core)
        .zip(core_dir(app, core.name()))
        .map(|(table, dir)| files::in_place(&dir, table))
        .unwrap_or_default();
    let plan = bios::plan(core, console, &in_place, read);
    log::info!(
        "{core} system files in place: {in_place:?}; options {:?}",
        plan.options
    );
    plan
}

/// Tells the interface what fell back when the game opened.
pub fn emit_fallbacks(app: &AppHandle, fallbacks: &[ErrorCode]) {
    for code in fallbacks {
        log::warn!("system files: {code:?}");
        if let Err(e) = app.emit(crate::events::BIOS_FALLBACK, code) {
            log::warn!("emu://bios-fallback: {e}");
        }
    }
}
