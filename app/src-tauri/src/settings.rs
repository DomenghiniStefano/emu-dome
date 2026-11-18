//! Tauri side of the settings engine (`emudome_core::settings`): where the files live, the
//! commands the interface calls (through `app/src/emu.ts` only) and the `settings://changed`
//! event, `{ key, level }`, sent after every change so any screen showing the value refreshes.

use emudome_core::settings::{
    CommitTarget, ConsoleId, Device, GameId, GameOverride, Level, PageValue, RecentChange,
    SchemaDto, Scope, Settings, Value,
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

pub(crate) fn emit_changed(app: &AppHandle, keys: &[&str], level: Level) {
    if !keys.is_empty() {
        crate::pads::settings_changed();
    }
    if keys.iter().any(|k| crate::effects::is_effect_key(k)) {
        crate::effects::apply(app);
    }
    if keys.iter().any(|k| REWIND_KEYS.contains(k)) {
        // The open game follows at once (no game: nothing to tell).
        let seconds = rewind_seconds(app, &Scope::Session);
        crate::send_command(app, emudome_core::Command::SetRewindLength(seconds));
    }
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
        // The controllers and the screen effect now follow this game's settings.
        crate::pads::settings_changed();
        crate::effects::apply(app);
        emit_changed(app, &dropped, Level::Session);
    }
}

/// The game closed: unsaved changes are discarded.
pub fn end_session(app: &AppHandle) {
    if let Some(state) = app.try_state::<SettingsState>() {
        let dropped = state.0.lock().end_session();
        crate::pads::settings_changed();
        crate::effects::apply(app);
        emit_changed(app, &dropped, Level::Session);
    }
}

/// What the game flow reads from the settings for one game (step 4): `saves.autoq`,
/// `saves.autoexit`, `saves.resume`. Missing values fall back to the schema's defaults.
pub struct GameFlow {
    pub autosave_every: Option<std::time::Duration>,
    pub autosave_on_exit: bool,
    /// `ask`, `always` or `ingame`.
    pub resume: String,
    /// `speed.rew` / `speed.rewlen`: the seconds of rewind to keep, `None` = off.
    pub rewind_seconds: Option<u32>,
}

const REWIND_KEYS: [&str; 2] = ["speed.rew", "speed.rewlen"];

/// `speed.rew` and `speed.rewlen` seen from `scope` (the open game's session, or the game
/// being opened); `None` = rewind off.
fn rewind_seconds(app: &AppHandle, scope: &Scope) -> Option<u32> {
    let read = |key: &str| -> Option<Value> {
        let state = app.try_state::<SettingsState>()?;
        let resolved = state.0.lock().resolve(key, scope);
        match resolved {
            Ok(r) => r.value,
            Err(e) => {
                log::warn!("{key} unavailable: {e:?}");
                None
            }
        }
    };
    emudome_core::rewind::seconds_from_settings(read(REWIND_KEYS[0]), read(REWIND_KEYS[1]))
}

pub fn game_flow(app: &AppHandle, console: ConsoleId, game: &GameId) -> GameFlow {
    let scope = Scope::Game {
        console,
        game: game.clone(),
    };
    let read = |key: &str| -> Option<Value> {
        let state = app.try_state::<SettingsState>()?;
        let resolved = state.0.lock().resolve(key, &scope);
        match resolved {
            Ok(r) => r.value,
            Err(e) => {
                log::warn!("{key} unavailable: {e:?}");
                None
            }
        }
    };
    let text = |key: &str| match read(key) {
        Some(Value::Text(t)) => Some(t),
        Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => None,
    };
    GameFlow {
        autosave_every: match text("saves.autoq").as_deref() {
            Some("1min") => Some(std::time::Duration::from_secs(60)),
            Some("never") => None,
            Some(_) | None => Some(std::time::Duration::from_secs(300)),
        },
        autosave_on_exit: !matches!(read("saves.autoexit"), Some(Value::Bool(false))),
        resume: text("saves.resume").unwrap_or_else(|| "always".into()),
        rewind_seconds: rewind_seconds(app, &scope),
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

/// The `limit` values changed most recently, at any level, newest first («Modificate di
/// recente», T01).
#[tauri::command]
pub fn settings_recent(state: State<'_, SettingsState>, limit: usize) -> Vec<RecentChange> {
    state.0.lock().recent(limit)
}

/// The scope of the open game (`null` without one), for «Solo questo gioco».
#[tauri::command]
pub fn settings_game_scope(state: State<'_, SettingsState>) -> Option<Scope> {
    state.0.lock().session_scope()
}

/// Game details › «Impostazioni del gioco» (L06): the values stored for one game, each with
/// the value of its console's games. Any other scope has none.
#[tauri::command]
pub fn settings_game_overrides(state: State<'_, SettingsState>, scope: Scope) -> Vec<GameOverride> {
    match scope {
        Scope::Game { console, game } => state.0.lock().game_overrides(console, &game),
        Scope::Global | Scope::Console { .. } | Scope::Session => Vec::new(),
    }
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
    crate::library::on_setting_changed(&app, &key);
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
    crate::library::on_setting_changed(&app, &key);
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
