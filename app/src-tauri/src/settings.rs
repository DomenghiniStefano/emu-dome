//! Tauri side of the settings engine (`emudome_core::settings`): where the files live, the
//! commands the interface calls (through `app/src/emu.ts` only) and the `settings://changed`
//! event, `{ key, level }`, sent after every change so any screen showing the value refreshes.

use emudome_core::settings::{
    keys, CommitTarget, ConsoleId, Device, GameKey, GameOverride, Level, PageValue, RecentChange,
    SchemaDto, Scope, Settings, Value,
};
use emudome_core::speed_audio::FastAudio;
use emudome_core::ErrorCode;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use ts_rs::TS;

/// The engine, behind a lock: every command is short (in memory, or one small file write).
pub struct SettingsState(pub Mutex<Settings>);

/// Payload of `settings://changed`. TypeScript: `SettingsChanged` (generated).
#[derive(Clone, Serialize, TS)]
#[ts(export_to = "app/", rename = "SettingsChanged")]
pub(crate) struct Changed<'a> {
    #[ts(as = "emudome_core::contract::SettingKey")]
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
    let synced = crate::dirs::data(app)?.join("settings");
    let device_file = crate::dirs::local(app)?
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
    crate::core_settings::settings_changed(app, keys);
    crate::bios::settings_changed(app, keys);
    if keys.iter().any(|k| REWIND_KEYS.contains(k)) {
        // The open game follows at once (no game: nothing to tell).
        let seconds = rewind_seconds(app, &Scope::Session);
        crate::send_command(app, emudome_core::Command::SetRewindLength(seconds));
    }
    if keys.contains(&keys::AUDIO_FF.as_str()) {
        let policy = fast_audio(app, &Scope::Session);
        crate::send_command(app, emudome_core::Command::SetFastAudio(policy));
    }
    for key in keys {
        if let Err(e) = app.emit("settings://changed", Changed { key, level }) {
            log::warn!("settings://changed {key}: {e}");
        }
    }
}

/// A game opened: its session of changes «in prova» starts (the previous one is dropped).
pub fn begin_session(app: &AppHandle, console: ConsoleId, game: GameKey) {
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
    /// `audio.ff`: the audio in fast forward and slow motion.
    pub fast_audio: FastAudio,
    /// The core options the settings ask for (`settings::core_values`).
    pub core_options: Vec<emudome_core::settings::core_values::MappedOption>,
    /// The user's name on the console (`consoles.dsname` for a DS game), `None`: the OS user.
    pub username: Option<String>,
    /// What the system files in place allow (`emudome_core::bios`).
    pub bios: emudome_core::bios::Plan,
}

const REWIND_KEYS: [&str; 2] = [
    keys::SPEED_REWIND.as_str(),
    keys::SPEED_REWIND_LENGTH.as_str(),
];

/// A setting's value as `scope` resolves it, `None` when it has none or cannot be read (logged):
/// what [`keys::Reader`] takes, which puts the catalogue's default in its place.
fn resolver<'a>(
    app: &'a AppHandle,
    scope: &'a Scope,
) -> impl Fn(&str) -> Option<Value> + Copy + 'a {
    move |key| {
        let state = app.try_state::<SettingsState>()?;
        let resolved = state.0.lock().resolve(key, scope);
        match resolved {
            Ok(r) => r.value,
            Err(e) => {
                log::warn!("{key} unavailable: {e:?}");
                None
            }
        }
    }
}

/// `audio.ff` seen from `scope`.
fn fast_audio(app: &AppHandle, scope: &Scope) -> FastAudio {
    let id = keys::Reader::new(resolver(app, scope)).text(keys::AUDIO_FF);
    FastAudio::from_option(&id).unwrap_or_default()
}

/// `speed.rew` and `speed.rewlen` seen from `scope` (the open game's session, or the game
/// being opened); `None` = rewind off.
fn rewind_seconds(app: &AppHandle, scope: &Scope) -> Option<u32> {
    emudome_core::rewind::seconds_from_settings(resolver(app, scope))
}

/// What the game flow reads for a game of `console` run by `core`.
pub fn game_flow(
    app: &AppHandle,
    core: emudome_core::core_profile::CoreId,
    console: ConsoleId,
    game: &GameKey,
) -> GameFlow {
    let scope = Scope::Game {
        console,
        game: game.clone(),
    };
    let read = resolver(app, &scope);
    let settings = keys::Reader::new(&read);
    GameFlow {
        autosave_every: emudome_core::saves::autosave_every(&settings.text(keys::SAVES_AUTOSAVE)),
        autosave_on_exit: settings.bool(keys::SAVES_AUTOSAVE_ON_EXIT),
        resume: settings.text(keys::SAVES_RESUME),
        rewind_seconds: rewind_seconds(app, &scope),
        username: match console {
            ConsoleId::Nds => emudome_core::nds::username(read),
            ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba | ConsoleId::N3ds => None,
        },
        fast_audio: fast_audio(app, &scope),
        bios: crate::bios::plan(app, core, console, &read),
        core_options: emudome_core::settings::core_values::mapped(core, console, read),
    }
}

/// The value of `general.theme` for every game (`system`, `dark`, `black`, `light`).
pub fn theme_setting(state: &SettingsState) -> String {
    let read = |key: &str| match state.0.lock().resolve(key, &Scope::Global) {
        Ok(resolved) => resolved.value,
        Err(e) => {
            log::warn!("{key} unavailable: {e:?}");
            None
        }
    };
    keys::Reader::new(read).text(keys::GENERAL_THEME)
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

// ---- «Immagine tua» (`video.surroundImg`): the user's own background around the game ----

/// The one setting that holds a background image.
const BACKGROUND_KEY: &str = keys::VIDEO_SURROUND_IMG.as_str();

/// Where the chosen images are copied: `<app data>/backgrounds`, beside the synced settings, so
/// the setting (a file name there) keeps working when the original file moves.
fn backgrounds_dir(app: &AppHandle) -> Option<std::path::PathBuf> {
    crate::dirs::data(app)
        .inspect_err(|e| log::warn!("app data dir: {e}"))
        .ok()
        .map(|d| d.join("backgrounds"))
}

/// The name an image is stored under: its content's sha256, so choosing the same image twice
/// keeps one copy, on every device and Rust version (the folder and the setting sync).
/// Images stored before 2026-11-21 keep their old name (16 hex digits of `DefaultHasher`, which
/// changes between Rust versions): the setting still names them and they are still served.
fn background_name(bytes: &[u8], ext: &str) -> String {
    use sha2::{Digest, Sha256};
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("{hex}.{ext}")
}

/// «Scegli un’immagine…»: opens an image picker, copies the image into the backgrounds folder and
/// stores its name in `video.surroundImg` at `scope`. `false` if the picker was cancelled.
#[tauri::command]
pub async fn settings_choose_background(app: AppHandle, scope: Scope) -> Result<bool, ErrorCode> {
    let Some(source) = crate::library::pick_image(&app) else {
        return Ok(false);
    };
    let invalid = |detail: String| ErrorCode::CoverInvalid { detail };
    let size = std::fs::metadata(&source)
        .map_err(|e| invalid(e.to_string()))?
        .len();
    if size > crate::library::MAX_COVER_BYTES {
        return Err(invalid(format!(
            "{size} bytes, limit {}",
            crate::library::MAX_COVER_BYTES
        )));
    }
    let bytes = std::fs::read(&source).map_err(|e| invalid(e.to_string()))?;
    let ext = crate::library::image_ext(&bytes)
        .ok_or_else(|| invalid("not a PNG, JPEG or WebP image".into()))?;
    let name = background_name(&bytes, ext);
    let dir = backgrounds_dir(&app).ok_or_else(|| ErrorCode::SettingsWriteFailed {
        detail: "no app data folder".into(),
    })?;
    emudome_core::write_atomic(&dir.join(&name), &bytes).map_err(|e| {
        ErrorCode::SettingsWriteFailed {
            detail: e.to_string(),
        }
    })?;
    let state = app.state::<SettingsState>();
    let level = state
        .0
        .lock()
        .set(BACKGROUND_KEY, &scope, Value::Text(name))?;
    emit_changed(&app, &[BACKGROUND_KEY], level);
    Ok(true)
}

/// `emu://background/<name>`: an image of the backgrounds folder (only names it can hold).
pub fn background_response(app: &AppHandle, name: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let found = backgrounds_dir(app)
        .and_then(|dir| crate::dirs::file_in(&dir, name))
        .and_then(|file| {
            std::fs::read(&file)
                .inspect_err(|e| log::warn!("background {}: {e}", file.display()))
                .ok()
                .map(|bytes| (bytes, crate::library::image_type(&file)))
        });
    let builder = Response::builder();
    let response = match found {
        // The name is the content's hash: it never changes.
        Some((bytes, mime)) => builder
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(bytes),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.unwrap_or_else(crate::response_error)
}

#[cfg(test)]
mod tests {
    use super::background_name;
    use crate::dirs::file_name_ok;

    #[test]
    fn the_same_image_gets_the_same_name() {
        let a = background_name(b"abc", "png");
        assert_eq!(a, background_name(b"abc", "png"));
        assert_ne!(a, background_name(b"abd", "png"));
        assert!(file_name_ok(&a));
        // Stable across Rust versions and devices: sha256("abc").
        assert_eq!(
            a,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.png"
        );
    }

    #[test]
    fn a_background_stored_under_its_old_name_is_still_served() {
        // The 16-hex-digit names of `DefaultHasher` (before 2026-11-21) stay valid file names.
        assert!(file_name_ok("0123456789abcdef.png"));
    }
}
