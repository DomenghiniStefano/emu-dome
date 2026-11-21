//! The open game's core options follow the settings while it runs.
//!
//! The options a game opens with come from `settings::game_flow` (Scope::Game). After that,
//! every change of a setting that some core table maps (`emudome_core::settings::core_values`)
//! is resolved again for the open game's session (so the quick menu's changes «in prova» count),
//! and [`core_values::plan_update`] decides:
//! - live options that changed go to the core (`Command::SetCoreOption`: it sees them at its next
//!   `GET_VARIABLE_UPDATE`, no restart);
//! - restart options are not sent: the settings whose value waits for the game to be opened again
//!   go to the interface as `emu://core-restart` `{ settings: [key, ...] }`, sent whenever that
//!   list changes (an empty list: nothing waits any more). TypeScript: `CoreRestart` (generated).

use emudome_core::settings::core_values::{self, MappedOption};
use emudome_core::settings::{ConsoleId, Scope, Value};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::settings::SettingsState;

/// The open game's core and options.
struct Running {
    core: &'static str,
    console: ConsoleId,
    /// What the core was opened with.
    started: Vec<MappedOption>,
    /// The options asked for at the last change (what the core has, for the live ones).
    previous: Vec<MappedOption>,
    /// Settings waiting for the game to be opened again (last sent to the interface).
    restart: Vec<&'static str>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

/// Payload of `emu://core-restart`.
#[derive(Clone, Serialize, TS)]
#[ts(export_to = "app/")]
pub(crate) struct CoreRestart<'a> {
    #[ts(as = "Vec<emudome_core::contract::SettingKey>")]
    settings: &'a [&'static str],
}

fn emit_restart(app: &AppHandle, settings: &[&'static str]) {
    if let Err(e) = app.emit("emu://core-restart", CoreRestart { settings }) {
        log::warn!("emu://core-restart: {e}");
    }
}

/// A game was opened by `core` with these options (from the settings).
pub fn started(
    app: &AppHandle,
    core: &'static str,
    console: ConsoleId,
    options: Vec<MappedOption>,
) {
    let old = RUNNING.lock().replace(Running {
        core,
        console,
        previous: options.clone(),
        started: options,
        restart: Vec::new(),
    });
    if old.is_some_and(|o| !o.restart.is_empty()) {
        emit_restart(app, &[]);
    }
}

/// The game was closed.
pub fn stopped(app: &AppHandle) {
    let old = RUNNING.lock().take();
    if old.is_some_and(|o| !o.restart.is_empty()) {
        emit_restart(app, &[]);
    }
}

/// Settings changed: the open game's core follows the ones it maps.
pub fn settings_changed(app: &AppHandle, keys: &[&str]) {
    if !keys.iter().any(|k| core_values::is_core_setting(k)) {
        return;
    }
    let Some((core, console)) = RUNNING.lock().as_ref().map(|r| (r.core, r.console)) else {
        return;
    };
    let Some(state) = app.try_state::<SettingsState>() else {
        return;
    };
    let now = {
        let mut settings = state.0.lock();
        core_values::mapped(core, console, |key| -> Option<Value> {
            match settings.resolve(key, &Scope::Session) {
                Ok(r) => r.value,
                Err(e) => {
                    log::warn!("{key} unavailable: {e:?}");
                    None
                }
            }
        })
    };
    let (push, restart) = {
        let mut running = RUNNING.lock();
        let Some(r) = running.as_mut().filter(|r| r.core == core) else {
            return; // the game closed meanwhile
        };
        let update = core_values::plan_update(&r.started, &r.previous, &now);
        r.previous = now;
        let restart = (update.restart != r.restart).then(|| update.restart.clone());
        r.restart = update.restart;
        (update.push, restart)
    };
    for (key, value) in push {
        log::info!("core option {key} = {value}");
        crate::send_command(app, emudome_core::Command::SetCoreOption { key, value });
    }
    if let Some(restart) = restart {
        log::info!("waiting for the game to be opened again: {restart:?}");
        emit_restart(app, &restart);
    }
}
