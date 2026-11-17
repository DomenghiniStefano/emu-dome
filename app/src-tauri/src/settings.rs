//! Tauri side of the settings engine (`emudome_core::settings`): where the files live, the
//! commands the interface calls (through `app/src/emu.ts` only) and the `settings://changed`
//! event, `{ key, level }`, sent after every change so any screen showing the value refreshes.

use emudome_core::settings::{
    CommitTarget, ConsoleId, Device, GameId, Level, PageValue, SchemaDto, Scope, Settings, Value,
};
use emudome_core::ErrorCode;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

/// The engine, behind a lock: every command is short (in memory, or one small file write).
pub struct SettingsState(pub Mutex<Settings>);

/// Payload of `settings://changed`. Mirror: `SettingsChanged` in app/src/emu.ts.
#[derive(Clone, Serialize)]
struct Changed<'a> {
    key: &'a str,
    level: Level,
}

fn device() -> Device {
    if emudome_platform::PHONE {
        Device::Phone
    } else {
        Device::Desktop
    }
}

/// Opens the settings: the synced files in `<app data>/settings`, the per-device file in
/// `<app local data>/local-settings/device.json` (a different folder even where the two data
/// dirs are the same, so syncing `settings/` never carries it).
pub fn open(app: &tauri::App) -> tauri::Result<SettingsState> {
    let synced = app.path().app_data_dir()?.join("settings");
    let device_file = app
        .path()
        .app_local_data_dir()?
        .join("local-settings")
        .join("device.json");
    log::info!(
        "settings in {} (this device: {})",
        synced.display(),
        device_file.display()
    );
    Ok(SettingsState(Mutex::new(Settings::open(
        synced,
        device_file,
        device(),
    ))))
}

fn emit_changed(app: &AppHandle, keys: &[&str], level: Level) {
    for key in keys {
        if let Err(e) = app.emit("settings://changed", Changed { key, level }) {
            log::warn!("settings://changed {key}: {e}");
        }
    }
}

/// A game opened: its session of changes «in prova» starts (the previous one is dropped).
pub fn begin_session(app: &AppHandle, console: ConsoleId, game: GameId) {
    if let Some(state) = app.try_state::<SettingsState>() {
        let dropped = {
            let mut settings = state.0.lock();
            let dropped = settings.end_session();
            settings.begin_session(console, game);
            dropped
        };
        emit_changed(app, &dropped, Level::Session);
    }
}

/// The game closed: unsaved changes are discarded.
pub fn end_session(app: &AppHandle) {
    if let Some(state) = app.try_state::<SettingsState>() {
        let dropped = state.0.lock().end_session();
        emit_changed(app, &dropped, Level::Session);
    }
}

/// The value of `general.theme` for every game (`system`, `dark`, `black`, `light`).
pub fn theme_setting(state: &SettingsState) -> Option<String> {
    match state.0.lock().resolve("general.theme", &Scope::Global) {
        Ok(resolved) => match resolved.value {
            Some(Value::Text(id)) => Some(id),
            Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => None,
        },
        Err(e) => {
            log::warn!("general.theme unavailable: {e:?}");
            None
        }
    }
}

/// The whole schema for this device: the interface builds every settings page from it.
#[tauri::command]
pub fn settings_schema(state: State<'_, SettingsState>) -> SchemaDto {
    state.0.lock().schema()
}

/// The rows of a page seen from a scope: value, level it comes from, changed from the default.
#[tauri::command]
pub fn settings_page(
    state: State<'_, SettingsState>,
    page: String,
    scope: Scope,
) -> Result<Vec<PageValue>, ErrorCode> {
    state.0.lock().page(&page, &scope)
}

/// The scope of the open game (`null` without one), for «Solo questo gioco».
#[tauri::command]
pub fn settings_game_scope(state: State<'_, SettingsState>) -> Option<Scope> {
    state.0.lock().session_scope()
}

/// Stores a value at the scope's level (`{ level: 'session' }`: «in prova», in memory).
#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<'_, SettingsState>,
    key: String,
    scope: Scope,
    value: Value,
) -> Result<(), ErrorCode> {
    let level = state.0.lock().set(&key, &scope, value)?;
    emit_changed(&app, &[&key], level);
    Ok(())
}

/// Removes the value at the scope's level: the weaker level shows through.
#[tauri::command]
pub fn settings_reset(
    app: AppHandle,
    state: State<'_, SettingsState>,
    key: String,
    scope: Scope,
) -> Result<(), ErrorCode> {
    let level = state.0.lock().reset(&key, &scope)?;
    emit_changed(&app, &[&key], level);
    Ok(())
}

/// Saves the changes «in prova» for this game or for its console (all or nothing).
#[tauri::command]
pub fn settings_session_commit(
    app: AppHandle,
    state: State<'_, SettingsState>,
    target: CommitTarget,
) -> Result<(), ErrorCode> {
    let (level, keys) = state.0.lock().commit_session(target)?;
    emit_changed(&app, &keys, level);
    Ok(())
}

/// Drops the changes «in prova», keeping the game open.
#[tauri::command]
pub fn settings_session_discard(
    app: AppHandle,
    state: State<'_, SettingsState>,
) -> Result<(), ErrorCode> {
    let keys = state.0.lock().discard_session()?;
    emit_changed(&app, &keys, Level::Session);
    Ok(())
}
