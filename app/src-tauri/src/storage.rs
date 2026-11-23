//! E13 «Telefono quasi pieno» (Trello #29, the owner's option C): how much room is left where
//! the app keeps its data, and whether that is little enough to warn. The interface asks at
//! startup, when the shell shows again after a game, and when the library changes (a game
//! downloaded); it shows the warning on the phone only, but the answer is the same everywhere.
//!
//! «Gestisci lo spazio» (Trello #29, S1–S3): what EmuDome's files weigh (`storage_breakdown`:
//! the disk's room, the games from the heaviest with where each one is, the quick saves, the
//! screenshots, the temporary files) and the deletions the screen offers, each one confirmed by
//! the interface first and carried out by `emudome_core::storage`, which never touches an
//! in-game save:
//! - `storage_remove_game` («Togli dal telefono»): only a game file in EmuDome's own games folder
//!   (downloaded from Drive) and only while Drive has the same game; the game stays in the
//!   library, unavailable (faded), its saves kept;
//! - `storage_delete_quick`: the quick saves the user picked;
//! - `storage_delete_screenshots`: the screenshots the user picked;
//! - `storage_clear_temp` («Svuota»): the files that rebuild themselves.
//!
//! Every command reads or deletes files: off the event loop (`crate::blocking`). None of them
//! touches the game open now.

use std::path::{Path, PathBuf};

use emudome_core::library::GameRecord;
use emudome_core::settings::ConsoleId;
use emudome_core::storage::{self as files, QuickFile, RemoveError, TempPlaces};
use emudome_core::{ErrorCode, GameKey, IoCause};
use emudome_platform::DiskSpace;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::{cloud, library, AppState};

/// Below this much free room the warning shows, whatever the disk's size (the owner's mockup:
/// 500 MB; a share of the disk would nag on a large phone): a save that cannot be written is
/// lost.
pub const LOW_BYTES: u64 = 500 * 1024 * 1024;

/// The room on the data folder's disk. TypeScript: `StorageStatus` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub struct StorageStatus {
    /// Bytes the app may still write there.
    pub free: u64,
    /// The disk's size in bytes.
    pub total: u64,
    /// Little enough to warn ([`is_low`]).
    pub low: bool,
}

/// Whether `free` bytes are little enough to warn: under 500 MB.
pub fn is_low(free: u64) -> bool {
    free < LOW_BYTES
}

impl From<DiskSpace> for StorageStatus {
    fn from(space: DiskSpace) -> Self {
        StorageStatus {
            free: space.free,
            total: space.total,
            low: is_low(space.free),
        }
    }
}

/// The room left where the saves are written (the synced data folder). `None` when the system
/// does not say (logged): no warning then, since nothing the user can do follows from it.
#[tauri::command]
pub fn storage_status(app: AppHandle) -> Option<StorageStatus> {
    let dir = match crate::dirs::data(&app) {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("storage: no data folder: {e}");
            return None;
        }
    };
    match emudome_platform::disk_space(&dir) {
        Ok(space) => Some(space.into()),
        Err(e) => {
            log::warn!("storage: room on {} not read: {e}", dir.display());
            None
        }
    }
}

// ---- «Gestisci lo spazio» ----

/// Where a game's file is. TypeScript: `StorageGameLocation` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageGameLocation")]
pub enum GameLocation {
    /// EmuDome's own games folder (a game downloaded from Google Drive): «Togli dal telefono»
    /// when Drive has it too.
    App,
    /// One of the user's folders: «In <name>», with «Apri la cartella» when `openable` (the
    /// desktop). `name`: none when the system does not tell it (a picked Android document).
    Folder {
        name: Option<String>,
        openable: bool,
    },
}

/// One game of «Giochi, dal più pesante». TypeScript: `StorageGame` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageGame")]
pub struct StorageGame {
    pub id: GameKey,
    pub title: String,
    pub console: ConsoleId,
    /// The game file's size in bytes.
    pub size: u64,
    pub location: GameLocation,
}

/// What EmuDome's files weigh. TypeScript: `StorageBreakdown` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageBreakdown")]
pub struct Breakdown {
    /// The data folder's disk: free and total bytes (`null` when the system does not say).
    pub free: Option<u64>,
    pub total: Option<u64>,
    /// The bar's parts, in bytes: the games here, the quick saves, EmuDome's other files (saves,
    /// screenshots, covers, settings, temporary files), and the rest of the disk in use (other
    /// apps; `null` without the disk's size).
    pub games_bytes: u64,
    pub quick_bytes: u64,
    pub other_bytes: u64,
    pub other_apps_bytes: Option<u64>,
    /// The library's games whose file is on this device, the heaviest first.
    pub games: Vec<StorageGame>,
    /// How many quick saves, in how many games.
    pub quick_slots: u32,
    pub quick_games: u32,
    pub screenshot_bytes: u64,
    pub screenshot_count: u32,
    /// The games that have screenshots, with how many.
    pub screenshot_games: Vec<ScreenshotCount>,
    /// The temporary files' bytes («Svuota»).
    pub temp_bytes: u64,
}

/// How many screenshots a game has. TypeScript: `StorageScreenshotCount` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageScreenshotCount")]
pub struct ScreenshotCount {
    pub id: GameKey,
    pub count: u32,
}

/// One game's quick saves, for «Salvataggi rapidi › Scegli». TypeScript: `StorageQuickGame`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageQuickGame")]
pub struct QuickGame {
    pub id: GameKey,
    pub slots: Vec<QuickSlot>,
}

/// One quick save. TypeScript: `StorageQuickSlot` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageQuickSlot")]
pub struct QuickSlot {
    /// 0 is the auto quick save.
    pub slot: u8,
    /// Its state, thumbnail and record, in bytes.
    pub size: u64,
    /// When it was saved, Unix ms.
    pub saved_at: Option<u64>,
}

impl From<QuickFile> for QuickSlot {
    fn from(q: QuickFile) -> Self {
        QuickSlot {
            slot: q.slot,
            size: q.bytes,
            saved_at: q.modified_ms,
        }
    }
}

/// A quick save picked for deletion. TypeScript: `StorageQuickPick` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "StorageQuickPick")]
pub struct QuickPick {
    pub game: GameKey,
    pub slot: u8,
}

/// Where a library game's file is, given EmuDome's games folder and the name of a user folder.
fn location_of(
    game: &GameRecord,
    in_app_folder: bool,
    folder_name: impl Fn(&str) -> Option<String>,
) -> GameLocation {
    if in_app_folder {
        return GameLocation::App;
    }
    let document = emudome_platform::is_content_uri(&game.path);
    match &game.folder {
        Some(folder) => GameLocation::Folder {
            name: folder_name(folder),
            openable: emudome_platform::OPEN_FOLDER && !emudome_platform::is_content_uri(folder),
        },
        None if document => GameLocation::Folder {
            name: None,
            openable: false,
        },
        None => GameLocation::Folder {
            name: Path::new(&game.path)
                .parent()
                .and_then(Path::file_name)
                .map(|n| n.to_string_lossy().into_owned()),
            openable: emudome_platform::OPEN_FOLDER,
        },
    }
}

/// The folder «Apri la cartella» shows for a game: its library folder, or a single file's own
/// folder; `None` for a picked document.
fn folder_to_open(game: &GameRecord) -> Option<PathBuf> {
    match &game.folder {
        Some(folder) if !emudome_platform::is_content_uri(folder) => Some(PathBuf::from(folder)),
        Some(_) => None,
        None if emudome_platform::is_content_uri(&game.path) => None,
        None => Path::new(&game.path).parent().map(Path::to_path_buf),
    }
}

/// The rest of the disk in use: everything taken but EmuDome's own parts.
fn other_apps(space: Option<DiskSpace>, ours: u64) -> Option<u64> {
    space.map(|s| s.total.saturating_sub(s.free).saturating_sub(ours))
}

/// The heaviest first; equal sizes by title.
fn sort_games(games: &mut [StorageGame]) {
    games.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.title.cmp(&b.title)));
}

/// The folders the screen works on.
struct Places {
    data: PathBuf,
    local: PathBuf,
    games: PathBuf,
    saves: PathBuf,
    /// The named links of opened documents (Android only).
    content_links: Option<PathBuf>,
}

fn places(app: &AppHandle) -> Result<Places, ErrorCode> {
    let fail = |e: tauri::Error| {
        log::warn!("storage: no data folder: {e}");
        ErrorCode::StorageDeleteFailed {
            cause: IoCause::Other,
        }
    };
    let data = crate::dirs::data(app).map_err(fail)?;
    let local = crate::dirs::local(app).map_err(fail)?;
    let games = cloud::games_dir(app).map_err(fail)?;
    // Only where documents are opened: elsewhere the cache folder can be the data folder itself
    // (Windows), and nothing named `content` there is ours to delete.
    let content_links = emudome_platform::DOCUMENT_PICKERS
        .then(|| app.path().app_cache_dir().ok())
        .flatten()
        .map(|cache| cache.join(emudome_platform::CONTENT_LINKS_DIR));
    Ok(Places {
        saves: app.state::<AppState>().save_dir(),
        data,
        local,
        games,
        content_links,
    })
}

fn open_game(app: &AppHandle) -> Option<GameKey> {
    app.try_state::<AppState>().and_then(|s| s.games.in_use())
}

fn temp_paths(places: &Places, open: Option<&GameKey>) -> Vec<PathBuf> {
    files::temp_paths(&TempPlaces {
        content_links: places.content_links.clone(),
        games: &places.games,
        saves_root: &places.saves,
        open,
        now: std::time::SystemTime::now(),
    })
}

fn breakdown(app: &AppHandle) -> Result<Breakdown, ErrorCode> {
    crate::migration::wait(app);
    let places = places(app)?;
    let open = open_game(app);
    let mut games: Vec<StorageGame> = library::records(app)
        .into_iter()
        .filter(|g| g.listed && g.available)
        .map(|g| {
            let mine = !emudome_platform::is_content_uri(&g.path)
                && files::is_in_folder(&places.games, Path::new(&g.path));
            StorageGame {
                location: location_of(&g, mine, |f| library::folder_name(app, f)),
                title: g.shown_title().to_owned(),
                id: g.id,
                console: g.console,
                size: g.size,
            }
        })
        .collect();
    sort_games(&mut games);
    let games_bytes = games.iter().map(|g| g.size).sum();

    let quick = files::all_quick_saves(&places.saves);
    let quick_bytes = quick.iter().flat_map(|(_, q)| q).map(|q| q.bytes).sum();
    let quick_slots = quick.iter().map(|(_, q)| q.len()).sum::<usize>();
    let shots = files::screenshots(&places.saves);
    let temp_bytes = temp_paths(&places, open.as_ref())
        .iter()
        .map(|p| files::tree_size(p))
        .sum();

    // EmuDome's files, the games folder left out (its games are counted above).
    let skip = [places.games.as_path()];
    let mut ours = files::tree_size_except(&places.data, &skip);
    if !places.local.starts_with(&places.data) {
        ours += files::tree_size_except(&places.local, &skip);
    }
    let other_bytes = ours.saturating_sub(quick_bytes);
    let space = emudome_platform::disk_space(&places.data)
        .inspect_err(|e| log::warn!("storage: room on {} not read: {e}", places.data.display()))
        .ok();
    Ok(Breakdown {
        free: space.map(|s| s.free),
        total: space.map(|s| s.total),
        games_bytes,
        quick_bytes,
        other_bytes,
        other_apps_bytes: other_apps(space, games_bytes + quick_bytes + other_bytes),
        games,
        quick_slots: u32::try_from(quick_slots).unwrap_or(u32::MAX),
        quick_games: u32::try_from(quick.len()).unwrap_or(u32::MAX),
        screenshot_bytes: shots.bytes,
        screenshot_count: shots.count,
        screenshot_games: shots
            .games
            .into_iter()
            .map(|(id, count)| ScreenshotCount { id, count })
            .collect(),
        temp_bytes,
    })
}

/// «Gestisci lo spazio»: the disk's room and what EmuDome's files weigh. Whether a game is on
/// Drive is not asked here (no network wait): the interface asks `cloud_games` after.
#[tauri::command]
pub async fn storage_breakdown(app: AppHandle) -> Result<Breakdown, ErrorCode> {
    crate::blocking(app, breakdown).await?
}

/// «Salvataggi rapidi › Scegli»: every game's quick saves (the open game's too, shown; deleting
/// them is refused while it runs).
#[tauri::command]
pub async fn storage_quick_saves(app: AppHandle) -> Result<Vec<QuickGame>, ErrorCode> {
    crate::blocking(app, |app| {
        crate::migration::wait(app);
        let saves = app.state::<AppState>().save_dir();
        files::all_quick_saves(&saves)
            .into_iter()
            .map(|(id, slots)| QuickGame {
                id,
                slots: slots.into_iter().map(QuickSlot::from).collect(),
            })
            .collect()
    })
    .await
}

/// `saveGameOpen` when `games` includes the game open now.
fn refuse_open<'a>(
    app: &AppHandle,
    mut games: impl Iterator<Item = &'a GameKey>,
) -> Result<(), ErrorCode> {
    match open_game(app) {
        Some(open) if games.any(|g| *g == open) => Err(ErrorCode::SaveGameOpen),
        Some(_) | None => Ok(()),
    }
}

fn delete_failed((done, e): (usize, std::io::Error)) -> ErrorCode {
    log::warn!("storage: deletion stopped after {done}: {e}");
    ErrorCode::StorageDeleteFailed {
        cause: IoCause::of(&e),
    }
}

/// Deletes the quick saves picked (their state, thumbnail and record; never the in-game save).
/// Answers how many went.
#[tauri::command]
pub async fn storage_delete_quick(
    app: AppHandle,
    picked: Vec<QuickPick>,
) -> Result<usize, ErrorCode> {
    crate::blocking(app, move |app| {
        crate::migration::wait(app);
        refuse_open(app, picked.iter().map(|p| &p.game))?;
        let saves = app.state::<AppState>().save_dir();
        let picked: Vec<(GameKey, u8)> = picked.into_iter().map(|p| (p.game, p.slot)).collect();
        files::delete_quick_saves(&saves, &picked).map_err(delete_failed)
    })
    .await?
}

/// Deletes the screenshots `names` of `game`. Answers how many went.
#[tauri::command]
pub async fn storage_delete_screenshots(
    app: AppHandle,
    game: GameKey,
    names: Vec<String>,
) -> Result<usize, ErrorCode> {
    crate::blocking(app, move |app| {
        let saves = app.state::<AppState>().save_dir();
        files::delete_screenshots(&saves, &game, &names).map_err(delete_failed)
    })
    .await?
}

/// «File temporanei › Svuota»: the files that rebuild themselves (never the open game's).
/// Answers the bytes freed.
#[tauri::command]
pub async fn storage_clear_temp(app: AppHandle) -> Result<u64, ErrorCode> {
    crate::blocking(app, |app| {
        let places = places(app)?;
        let open = open_game(app);
        Ok(files::clear_temp(&temp_paths(&places, open.as_ref())))
    })
    .await?
}

/// «Togli dal telefono»: deletes the game file of `id`, only when it is in EmuDome's own games
/// folder and Google Drive has the same game (asked of Drive first: offline or signed out, the
/// sign-in's code and nothing deleted). The game stays in the library, unavailable; its saves
/// are not touched. Answers the bytes freed.
#[tauri::command]
pub async fn storage_remove_game(app: AppHandle, id: GameKey) -> Result<u64, ErrorCode> {
    let record = library::records(&app)
        .into_iter()
        .find(|g| g.id == id && g.listed)
        .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.to_string() })?;
    refuse_open(&app, std::iter::once(&id))?;
    let games = cloud::games_dir(&app).map_err(|e| {
        log::warn!("storage: no data folder: {e}");
        ErrorCode::StorageDeleteFailed {
            cause: IoCause::Other,
        }
    })?;
    let file = PathBuf::from(&record.path);
    if emudome_platform::is_content_uri(&record.path) || !files::is_in_folder(&games, &file) {
        return Err(ErrorCode::StorageGameNotRemovable);
    }
    if !cloud::on_drive(&app, &id).await? {
        return Err(ErrorCode::StorageGameNotRemovable);
    }
    crate::blocking(app, move |app| {
        // The game may have opened while Drive answered.
        refuse_open(app, std::iter::once(&id))?;
        let freed = files::remove_game_file(&games, &file).map_err(|e| match e {
            RemoveError::NotOurs => ErrorCode::StorageGameNotRemovable,
            RemoveError::Io(e) => {
                log::warn!("storage: {} not removed: {e}", file.display());
                ErrorCode::StorageDeleteFailed {
                    cause: IoCause::of(&e),
                }
            }
        })?;
        library::file_removed(app, &id)?;
        Ok(freed)
    })
    .await?
}

/// «Apri la cartella»: the folder of the game `id` in the system's file manager (the desktop;
/// `storageFolderUnavailable` elsewhere, where the button is not shown).
#[tauri::command]
pub fn storage_open_folder(app: AppHandle, id: GameKey) -> Result<(), ErrorCode> {
    let record = library::records(&app)
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.to_string() })?;
    let folder = folder_to_open(&record).ok_or_else(|| {
        log::warn!("storage: no folder to open for a document picked through the system");
        ErrorCode::StorageFolderUnavailable
    })?;
    emudome_platform::open_folder(&folder).map_err(|e| {
        log::warn!("storage: {} not opened: {e}", folder.display());
        ErrorCode::StorageFolderUnavailable
    })
}

#[cfg(test)]
mod tests {
    use super::{
        folder_to_open, is_low, location_of, other_apps, sort_games, GameLocation, StorageGame,
        StorageStatus, LOW_BYTES,
    };
    use emudome_core::library::{CoverKind, GameRecord};
    use emudome_core::settings::ConsoleId;
    use emudome_core::GameKey;
    use emudome_platform::DiskSpace;
    use std::path::PathBuf;

    const GIB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn under_500_mb_free_is_low() {
        assert!(is_low(380 * 1024 * 1024));
        assert!(is_low(LOW_BYTES - 1));
        assert!(!is_low(LOW_BYTES));
        assert!(is_low(0));
    }

    #[test]
    fn a_large_phone_with_a_few_gb_free_is_not_nagged() {
        // 6 GiB free of 128 GiB is under 5 %, but not low.
        assert!(!is_low(6 * GIB));
    }

    #[test]
    fn the_status_carries_the_decision() {
        let s = StorageStatus::from(DiskSpace {
            free: 40 * GIB,
            total: 128 * GIB,
        });
        assert_eq!(
            s,
            StorageStatus {
                free: 40 * GIB,
                total: 128 * GIB,
                low: false
            }
        );
    }

    fn record(path: &str, folder: Option<&str>) -> GameRecord {
        GameRecord {
            id: GameKey::new(ConsoleId::Gba, "1a2b").unwrap(),
            console: ConsoleId::Gba,
            title: "Game".into(),
            path: path.into(),
            folder: folder.map(str::to_owned),
            size: 1,
            mtime: 0,
            available: true,
            listed: true,
            play_time_secs: 0,
            last_played: None,
            added_at: 0,
            cover: CoverKind::Generated,
            user_cover: None,
            cover_rev: 0,
            has_icon: false,
            favourite: false,
            hidden: false,
            code: None,
            version: None,
            name: None,
        }
    }

    #[test]
    fn a_game_says_where_it_is() {
        let named = |f: &str| Some(format!("name of {f}"));
        let app_game = record("/data/games/gba/Game.gba", None);
        assert_eq!(location_of(&app_game, true, named), GameLocation::App);
        let in_folder = record("/home/me/Roms/Game.gba", Some("/home/me/Roms"));
        assert_eq!(
            location_of(&in_folder, false, named),
            GameLocation::Folder {
                name: Some("name of /home/me/Roms".into()),
                openable: emudome_platform::OPEN_FOLDER,
            }
        );
        let tree = "content://com.android.externalstorage.documents/tree/primary%3ADownload";
        let in_tree = record(&format!("{tree}/document/x"), Some(tree));
        assert_eq!(
            location_of(&in_tree, false, named),
            GameLocation::Folder {
                name: Some(format!("name of {tree}")),
                openable: false,
            }
        );
        let single = record("/home/me/Downloads/Game.gba", None);
        assert_eq!(
            location_of(&single, false, named),
            GameLocation::Folder {
                name: Some("Downloads".into()),
                openable: emudome_platform::OPEN_FOLDER,
            }
        );
        let document = record("content://media/document/7", None);
        assert_eq!(
            location_of(&document, false, named),
            GameLocation::Folder {
                name: None,
                openable: false,
            }
        );
        assert_eq!(
            folder_to_open(&in_folder),
            Some(PathBuf::from("/home/me/Roms"))
        );
        assert_eq!(
            folder_to_open(&single),
            Some(PathBuf::from("/home/me/Downloads"))
        );
        assert_eq!(folder_to_open(&in_tree), None);
        assert_eq!(folder_to_open(&document), None);
    }

    #[test]
    fn other_apps_are_what_is_taken_but_ours() {
        let space = DiskSpace {
            free: 10 * GIB,
            total: 128 * GIB,
        };
        assert_eq!(other_apps(Some(space), 30 * GIB), Some(88 * GIB));
        // Games on another disk can make ours larger than what is taken here.
        assert_eq!(other_apps(Some(space), 200 * GIB), Some(0));
        assert_eq!(other_apps(None, 1), None);
    }

    #[test]
    fn the_heaviest_games_come_first() {
        let game = |title: &str, size| StorageGame {
            id: GameKey::new(ConsoleId::Gba, "1a2b").unwrap(),
            title: title.into(),
            console: ConsoleId::Gba,
            size,
            location: GameLocation::App,
        };
        let mut games = vec![game("B", 5), game("C", 9), game("A", 5)];
        sort_games(&mut games);
        let order: Vec<&str> = games.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(order, ["C", "A", "B"]);
    }
}
