//! Tauri side of the Saves section (boards V01–V05, E03, E04; `emudome_core::saves`): the
//! commands the interface calls (through `app/src/emu.ts` only), the launch backup and the E04
//! check run before a game opens, quick save thumbnails of any game on `emu://savethumb/...`.
//!
//! Games are named by their ROM path: the save files are found from its file name
//! (TODO(game identity, Trello #21): from the content identity later).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use emudome_core::saves::{self, BackupReason, ImportFormat, SaveHealth};
use emudome_core::settings::{Scope, Value};
use emudome_core::ErrorCode;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::settings::SettingsState;
use crate::AppState;

fn now_ms() -> u64 {
    crate::unix_ms(SystemTime::now()).unwrap_or_default()
}

fn file_failed(e: impl std::fmt::Display) -> ErrorCode {
    ErrorCode::SaveFileFailed {
        detail: e.to_string(),
    }
}

/// `saves.backups`: how many backups of each in-game save to keep.
fn keep_backups(app: &AppHandle) -> usize {
    let value = app.try_state::<SettingsState>().and_then(|state| {
        let resolved = state.0.lock().resolve("saves.backups", &Scope::Global);
        resolved
            .inspect_err(|e| log::warn!("saves.backups unavailable: {e:?}"))
            .ok()?
            .value
    });
    match value {
        Some(Value::Int(n)) => usize::try_from(n).unwrap_or(1).max(1),
        Some(Value::Bool(_) | Value::Text(_) | Value::List(_)) | None => 5,
    }
}

/// Refuses to change the saves of the game that is open: its session rewrites the `.sav`.
fn ensure_closed(state: &AppState, game: &Path) -> Result<(), ErrorCode> {
    match state.game_path.lock().as_deref() {
        Some(open) if open == game => Err(ErrorCode::SaveGameOpen),
        Some(_) | None => Ok(()),
    }
}

// ---- Before a game opens ----

/// E04 and the launch backup, before the session reads the in-game save: a broken one is moved
/// aside and the open refused with `saveUnreadable`; a good one is copied into the backups.
pub fn before_open(app: &AppHandle, save_dir: &Path, game: &Path) -> Result<(), ErrorCode> {
    match saves::check_in_game(save_dir, game) {
        SaveHealth::Missing => Ok(()),
        SaveHealth::Fine => {
            // A backup that cannot be written must not stop the game: logged only.
            if let Err(e) = saves::backup(
                save_dir,
                game,
                BackupReason::Launch,
                now_ms(),
                keep_backups(app),
            ) {
                log::warn!("launch backup not made: {e}");
            }
            Ok(())
        }
        SaveHealth::Broken(why) => {
            log::warn!("in-game save of {} unusable: {why}", game.display());
            let aside = saves::set_aside(save_dir, game, now_ms()).map_err(file_failed)?;
            let backup_at = saves::backups(save_dir, game).first().map(|b| b.at_ms);
            Err(ErrorCode::SaveUnreadable { aside, backup_at })
        }
    }
}

// ---- DTOs (mirrors in app/src/emu.ts) ----

/// Mirror: `SavesSummary`. One row of V01.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDto {
    path: String,
    in_game: bool,
    quick_count: usize,
    /// In-game save, quick saves and thumbnails.
    bytes: u64,
    /// Unix ms of the newest change.
    last_change: Option<u64>,
    /// An in-game save was set aside as unreadable (E04).
    broken: bool,
}

/// Mirror: `SaveFile`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDto {
    size: u64,
    modified: Option<u64>,
}

/// Mirror: `QuickSave`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuickDto {
    slot: u8,
    size: u64,
    saved_at: Option<u64>,
    thumb_url: Option<String>,
}

/// Mirror: `SaveBackup`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupDto {
    name: String,
    at: u64,
    reason: BackupReason,
    size: u64,
}

/// Mirror: `GameSaves`. V02.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSavesDto {
    in_game: Option<FileDto>,
    quick: Vec<QuickDto>,
    backups: Vec<BackupDto>,
    broken: Vec<String>,
}

/// Mirror: `ImportCandidate`. The file chosen in V05, already read and recognised.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDto {
    path: String,
    file_name: String,
    format: ImportFormat,
    /// Bytes of the save inside the file.
    size: u64,
}

/// What to export. Mirror: `SaveExport`.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportWhat {
    InGame,
    Quick { slot: u8 },
}

// ---- Thumbnails of any game: emu://savethumb/<hex of the ROM path>/<slot>?v=<ms> ----

fn hex(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<String> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

fn thumb_url(game: &str, slot: u8, saved_at: Option<u64>) -> String {
    emudome_platform::protocol_url(
        "emu",
        &format!("savethumb/{}/{slot}?v={}", hex(game), saved_at.unwrap_or(0)),
    )
}

/// `emu://savethumb/<hex path>/<slot>`.
pub fn thumb_response(app: &AppHandle, rest: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let bytes = rest.split_once('/').and_then(|(game, slot)| {
        let game = PathBuf::from(unhex(game)?);
        let slot: u8 = slot.parse().ok()?;
        let state = app.try_state::<AppState>()?;
        let thumb = saves::game_saves(&state.save_dir(), &game, slot)
            .quick
            .into_iter()
            .find(|q| q.slot == slot)?
            .thumb?;
        std::fs::read(thumb).ok()
    });
    let builder = Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    match bytes {
        Some(b) => builder
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(b),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    }
    .unwrap_or_else(crate::response_error)
}

// ---- Commands ----

/// V01: what each game has saved, for the list.
#[tauri::command]
pub fn saves_summary(state: State<'_, AppState>, paths: Vec<String>, slots: u8) -> Vec<SummaryDto> {
    let save_dir = state.save_dir();
    paths
        .into_iter()
        .map(|path| {
            let s = saves::game_saves(&save_dir, Path::new(&path), slots);
            SummaryDto {
                in_game: s.in_game.is_some(),
                quick_count: s.quick.len(),
                bytes: s.bytes(),
                last_change: s.last_change().and_then(crate::unix_ms),
                broken: !s.broken.is_empty(),
                path,
            }
        })
        .collect()
}

/// V02: everything saved for one game.
#[tauri::command]
pub fn saves_game(state: State<'_, AppState>, path: String, slots: u8) -> GameSavesDto {
    let s = saves::game_saves(&state.save_dir(), Path::new(&path), slots);
    GameSavesDto {
        in_game: s.in_game.map(|f| FileDto {
            size: f.size,
            modified: f.modified.and_then(crate::unix_ms),
        }),
        quick: s
            .quick
            .into_iter()
            .map(|q| {
                let saved_at = q.file.modified.and_then(crate::unix_ms);
                QuickDto {
                    slot: q.slot,
                    size: q.file.size,
                    saved_at,
                    thumb_url: q.thumb.map(|_| thumb_url(&path, q.slot, saved_at)),
                }
            })
            .collect(),
        backups: s
            .backups
            .into_iter()
            .map(|b| BackupDto {
                name: b.name,
                at: b.at_ms,
                reason: b.reason,
                size: b.size,
            })
            .collect(),
        broken: s.broken,
    }
}

/// V04: puts a backup back; the current in-game save becomes a backup first.
#[tauri::command]
pub fn saves_restore(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    name: String,
) -> Result<(), ErrorCode> {
    let game = PathBuf::from(path);
    ensure_closed(&state, &game)?;
    saves::restore(
        &state.save_dir(),
        &game,
        &name,
        now_ms(),
        keep_backups(&app),
    )
    .map_err(file_failed)
}

/// E04: puts the newest backup back (after the broken save was set aside).
#[tauri::command]
pub fn saves_restore_latest(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), ErrorCode> {
    let game = PathBuf::from(path);
    ensure_closed(&state, &game)?;
    let save_dir = state.save_dir();
    let newest = saves::backups(&save_dir, &game)
        .into_iter()
        .next()
        .ok_or_else(|| file_failed("no backup"))?;
    saves::restore(&save_dir, &game, &newest.name, now_ms(), keep_backups(&app))
        .map_err(file_failed)
}

/// V03: deletes a quick save and its thumbnail.
#[tauri::command]
pub fn saves_delete_quick(
    state: State<'_, AppState>,
    path: String,
    slot: u8,
) -> Result<(), ErrorCode> {
    let game = PathBuf::from(path);
    ensure_closed(&state, &game)?;
    saves::delete_quick(&state.save_dir(), &game, slot).map_err(file_failed)
}

/// V03 / V02: copies the in-game save or a quick save where the user chooses. `false` = the
/// save dialog was cancelled.
#[tauri::command]
pub async fn saves_export(
    app: AppHandle,
    path: String,
    what: ExportWhat,
) -> Result<bool, ErrorCode> {
    let save_dir = app.state::<AppState>().save_dir();
    let files = emudome_core::GameFiles::new(&save_dir, Path::new(&path));
    let source = match what {
        ExportWhat::InGame => files.sram.clone(),
        ExportWhat::Quick { slot } => files.slot(slot),
    };
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(dest) = pick_save_target(&app, &name) else {
        return Ok(false);
    };
    std::fs::copy(&source, &dest).map_err(file_failed)?;
    Ok(true)
}

/// V05 step 1: picks a file and recognises it. `None` = cancelled.
#[tauri::command]
pub async fn saves_import_pick(app: AppHandle) -> Result<Option<ImportDto>, ErrorCode> {
    let Some(file) = pick_import_file(&app) else {
        return Ok(None);
    };
    let file_name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let bytes = std::fs::read(&file).map_err(file_failed)?;
    let (format, data) =
        saves::parse_import(&bytes, &file_name).map_err(|e| ErrorCode::SaveImportInvalid {
            detail: e.to_string(),
        })?;
    Ok(Some(ImportDto {
        path: file.to_string_lossy().into_owned(),
        file_name,
        format,
        size: data.len() as u64,
    }))
}

/// V05 «Importa e sostituisci»: `file` becomes the in-game save of the game at `game`; the
/// current one becomes a backup first.
#[tauri::command]
pub fn saves_import(
    app: AppHandle,
    state: State<'_, AppState>,
    file: String,
    game: String,
) -> Result<(), ErrorCode> {
    let game = PathBuf::from(game);
    ensure_closed(&state, &game)?;
    let file_name = Path::new(&file)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let bytes = std::fs::read(&file).map_err(file_failed)?;
    let (_, data) =
        saves::parse_import(&bytes, &file_name).map_err(|e| ErrorCode::SaveImportInvalid {
            detail: e.to_string(),
        })?;
    saves::import(
        &state.save_dir(),
        &game,
        &data,
        now_ms(),
        keep_backups(&app),
    )
    .map_err(file_failed)
}

// ---- Native pickers (desktop; Android has no file-path pickers: TODO with the SAF) ----

#[cfg(desktop)]
fn pick_save_target(app: &AppHandle, name: &str) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_file_name(name)
        .blocking_save_file()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("export target: {e}"))
        .ok()
}

#[cfg(desktop)]
fn pick_import_file(app: &AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("Save", &["sav", "srm", "dsv", "duc"])
        .blocking_pick_file()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("import file: {e}"))
        .ok()
}

#[cfg(mobile)]
fn pick_save_target(_app: &AppHandle, _name: &str) -> Option<PathBuf> {
    log::warn!("save dialog not available on this platform yet");
    None
}

#[cfg(mobile)]
fn pick_import_file(_app: &AppHandle) -> Option<PathBuf> {
    log::warn!("file picker not available on this platform yet");
    None
}

#[cfg(test)]
mod tests {
    use super::{hex, unhex};

    #[test]
    fn paths_survive_the_url() {
        let p = r"C:\Giochi\Pokémon Smeraldo.gba";
        assert_eq!(unhex(&hex(p)).as_deref(), Some(p));
        assert_eq!(unhex("zz"), None);
        assert_eq!(unhex("abc"), None);
    }
}
