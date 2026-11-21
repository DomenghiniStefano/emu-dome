//! Tauri side of the Saves section (boards V01–V05, E03, E04; `emudome_core::saves`): the
//! commands the interface calls (through `app/src/emu.ts` only), the launch backup and the E04
//! check run before a game opens, quick save thumbnails of any game on `emu://savethumb/...`.
//!
//! Games are named by their key (`GameId` in app/src/emu.ts), never by a path or a file name.
//! Every command waits for the one-time move of the old save names first (`crate::migration`).

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use ts_rs::TS;

use emudome_core::saves::{self, BackupReason, FolderImport, ImportFormat, SaveHealth};
use emudome_core::settings::{Scope, Value};
use emudome_core::{ErrorCode, GameKey};
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

/// `saves.backups`: how many backups of each in-game save to keep (0 = none).
fn keep_backups(app: &AppHandle) -> usize {
    let value = app.try_state::<SettingsState>().and_then(|state| {
        let resolved = state.0.lock().resolve("saves.backups", &Scope::Global);
        resolved
            .inspect_err(|e| log::warn!("saves.backups unavailable: {e:?}"))
            .ok()?
            .value
    });
    match value {
        Some(Value::Int(n)) => usize::try_from(n).unwrap_or(0),
        Some(Value::Bool(_) | Value::Text(_) | Value::List(_)) | None => 5,
    }
}

/// Refuses to change the saves of the game that is open: its session rewrites the `.sav`.
fn ensure_closed(state: &AppState, game: &GameKey) -> Result<(), ErrorCode> {
    match state.game.lock().as_ref() {
        Some(open) if open == game => Err(ErrorCode::SaveGameOpen),
        Some(_) | None => Ok(()),
    }
}

// ---- Before a game opens ----

/// E04 and the launch backup, before the session reads the in-game save: a broken one is moved
/// aside and the open refused with `saveUnreadable`; a good one is copied into the backups.
pub fn before_open(app: &AppHandle, save_dir: &Path, game: &GameKey) -> Result<(), ErrorCode> {
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
            log::warn!("in-game save of {game} unusable: {why}");
            let aside = saves::set_aside(save_dir, game, now_ms()).map_err(file_failed)?;
            let backup_at = saves::backups(save_dir, game).first().map(|b| b.at_ms);
            Err(ErrorCode::SaveUnreadable {
                game: game.clone(),
                aside,
                backup_at,
            })
        }
    }
}

// ---- DTOs (TypeScript generated into app/src/generated/app/ by src/contract.rs) ----

/// TypeScript: `SavesSummary` (generated). One row of V01.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SavesSummary")]
pub struct SummaryDto {
    game: GameKey,
    in_game: bool,
    quick_count: usize,
    /// In-game save, quick saves and thumbnails.
    bytes: u64,
    /// Unix ms of the newest change.
    last_change: Option<u64>,
    /// An in-game save was set aside as unreadable (E04).
    broken: bool,
}

/// TypeScript: `SaveFile` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SaveFile")]
pub(crate) struct FileDto {
    size: u64,
    modified: Option<u64>,
}

/// TypeScript: `QuickSave` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "QuickSave")]
pub(crate) struct QuickDto {
    slot: u8,
    size: u64,
    saved_at: Option<u64>,
    thumb_url: Option<String>,
    /// Made by an older version of its core: «Potrebbe non caricarsi».
    outdated: bool,
}

/// TypeScript: `SaveBackup` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SaveBackup")]
pub(crate) struct BackupDto {
    name: String,
    at: u64,
    reason: BackupReason,
    size: u64,
}

/// TypeScript: `GameSaves` (generated). V02.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "GameSaves")]
pub struct GameSavesDto {
    in_game: Option<FileDto>,
    quick: Vec<QuickDto>,
    backups: Vec<BackupDto>,
    broken: Vec<String>,
}

/// TypeScript: `ImportCandidate` (generated). The file chosen in V05, already read and recognised.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "ImportCandidate")]
pub struct ImportDto {
    path: String,
    file_name: String,
    format: ImportFormat,
    /// Bytes of the save inside the file.
    size: u64,
}

/// What to export. TypeScript: `SaveExport` (generated).
#[derive(Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SaveExport")]
pub enum ExportWhat {
    InGame,
    Quick { slot: u8 },
}

// ---- Thumbnails of any game: emu://savethumb/<console>-<hex>/<slot>?v=<ms> ----

fn thumb_url(game: &GameKey, slot: u8, saved_at: Option<u64>) -> String {
    emudome_platform::protocol_url(
        "emu",
        &format!(
            "savethumb/{}/{slot}?v={}",
            game.file_stem(),
            saved_at.unwrap_or(0)
        ),
    )
}

/// `emu://savethumb/<console>-<hex>/<slot>`.
pub fn thumb_response(app: &AppHandle, rest: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header, Response, StatusCode};
    let bytes = rest.split_once('/').and_then(|(game, slot)| {
        let game = GameKey::from_file_stem(game)?;
        let slot: u8 = slot.parse().ok()?;
        let state = app.try_state::<AppState>()?;
        let thumb = saves::game_saves(&state.save_dir(), &game, slot)
            .quick
            .into_iter()
            .find(|q| q.slot == slot)?
            .thumb?;
        std::fs::read(thumb).ok()
    });
    let builder = Response::builder();
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
pub fn saves_summary(
    app: AppHandle,
    state: State<'_, AppState>,
    games: Vec<GameKey>,
    slots: u8,
) -> Vec<SummaryDto> {
    crate::migration::wait(&app);
    let save_dir = state.save_dir();
    games
        .into_iter()
        .map(|game| {
            let s = saves::game_saves(&save_dir, &game, slots);
            SummaryDto {
                in_game: s.in_game.is_some(),
                quick_count: s.quick.len(),
                bytes: s.bytes(),
                last_change: s.last_change().and_then(crate::unix_ms),
                broken: !s.broken.is_empty(),
                game,
            }
        })
        .collect()
}

/// V02: everything saved for one game.
#[tauri::command]
pub fn saves_game(
    app: AppHandle,
    state: State<'_, AppState>,
    game: GameKey,
    slots: u8,
) -> GameSavesDto {
    crate::migration::wait(&app);
    let s = saves::game_saves(&state.save_dir(), &game, slots);
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
                    thumb_url: q.thumb.map(|_| thumb_url(&game, q.slot, saved_at)),
                    outdated: q.outdated,
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
    game: GameKey,
    name: String,
) -> Result<(), ErrorCode> {
    crate::migration::wait(&app);
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
    game: GameKey,
) -> Result<(), ErrorCode> {
    crate::migration::wait(&app);
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
    app: AppHandle,
    state: State<'_, AppState>,
    game: GameKey,
    slot: u8,
) -> Result<(), ErrorCode> {
    crate::migration::wait(&app);
    ensure_closed(&state, &game)?;
    saves::delete_quick(&state.save_dir(), &game, slot).map_err(file_failed)
}

/// V03 / V02: copies the in-game save or a quick save where the user chooses. `false` = the
/// save dialog was cancelled.
#[tauri::command]
pub async fn saves_export(
    app: AppHandle,
    game: GameKey,
    what: ExportWhat,
) -> Result<bool, ErrorCode> {
    crate::migration::wait(&app);
    let save_dir = app.state::<AppState>().save_dir();
    let files = emudome_core::GameFiles::new(&save_dir, &game);
    let source = match what {
        ExportWhat::InGame => files.sram.clone(),
        ExportWhat::Quick { slot } => files.slot(slot),
    };
    let name = export_name(&app, &game, &what);
    let Some(dest) = pick_save_target(&app, &name) else {
        return Ok(false);
    };
    std::fs::copy(&source, &dest).map_err(file_failed)?;
    Ok(true)
}

/// The name the export dialog suggests: the ROM's file name, as other emulators name their
/// saves (`Gioco.sav`, `Gioco.slot2.state`); the key when the game is not in the library.
fn export_name(app: &AppHandle, game: &GameKey, what: &ExportWhat) -> String {
    let base = crate::library::path_of(app, game)
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| game.file_stem());
    match what {
        ExportWhat::InGame => format!("{base}.sav"),
        ExportWhat::Quick { slot } => format!("{base}.slot{slot}.state"),
    }
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

/// V05 «Importa e sostituisci»: `file` becomes the in-game save of `game`; the current one
/// becomes a backup first.
#[tauri::command]
pub fn saves_import(
    app: AppHandle,
    state: State<'_, AppState>,
    file: String,
    game: GameKey,
) -> Result<(), ErrorCode> {
    crate::migration::wait(&app);
    import_one(&app, &state, Path::new(&file), &game)
}

fn import_one(
    app: &AppHandle,
    state: &AppState,
    file: &Path,
    game: &GameKey,
) -> Result<(), ErrorCode> {
    ensure_closed(state, game)?;
    let file_name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let bytes = std::fs::read(file).map_err(file_failed)?;
    let (_, data) =
        saves::parse_import(&bytes, &file_name).map_err(|e| ErrorCode::SaveImportInvalid {
            detail: e.to_string(),
        })?;
    saves::import(&state.save_dir(), game, &data, now_ms(), keep_backups(app)).map_err(file_failed)
}

/// A found save paired with its game. TypeScript: `SaveMatch` (generated).
#[derive(Serialize, Deserialize, TS)]
#[ts(export_to = "app/", rename = "SaveMatch")]
pub struct MatchDto {
    file: String,
    game: GameKey,
}

/// What a chosen folder offers. TypeScript: `FolderSaves` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "FolderSaves")]
pub struct FolderSavesDto {
    folder: String,
    /// Saves found that can be imported.
    found: usize,
    /// The ones that belong to a game of `games`, one per game.
    matches: Vec<MatchDto>,
}

/// First run S06 (DeSmuME, «Un’altra cartella o un altro emulatore»): picks a folder and pairs
/// the saves in it with the library games `games` by the file name of their ROM, as other
/// emulators name saves. `None` = cancelled.
#[tauri::command]
pub async fn saves_import_folder(
    app: AppHandle,
    what: FolderImport,
    games: Vec<GameKey>,
) -> Option<FolderSavesDto> {
    let folder = crate::library::pick_folder(&app)?;
    let found = saves::find_saves(&folder, what);
    let roms: Vec<(GameKey, PathBuf)> = games
        .into_iter()
        .filter_map(|g| Some((g.clone(), crate::library::path_of(&app, &g)?)))
        .collect();
    let paths: Vec<PathBuf> = roms.iter().map(|(_, p)| p.clone()).collect();
    let matches = saves::match_saves(&found, &paths)
        .into_iter()
        .filter_map(|m| {
            let (game, _) = roms.iter().find(|(_, p)| *p == m.game)?;
            Some(MatchDto {
                file: m.file.to_string_lossy().into_owned(),
                game: game.clone(),
            })
        })
        .collect();
    Some(FolderSavesDto {
        folder: folder.to_string_lossy().into_owned(),
        found: found.len(),
        matches,
    })
}

/// Imports every pair (each current in-game save becomes a backup first). Answers how many
/// were imported; the first failure stops and is the error.
#[tauri::command]
pub fn saves_import_many(
    app: AppHandle,
    state: State<'_, AppState>,
    matches: Vec<MatchDto>,
) -> Result<usize, ErrorCode> {
    crate::migration::wait(&app);
    matches.iter().try_fold(0, |done, m| {
        import_one(&app, &state, Path::new(&m.file), &m.game).map(|()| done + 1)
    })
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
