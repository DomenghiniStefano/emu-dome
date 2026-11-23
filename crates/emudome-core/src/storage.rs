//! «Gestisci lo spazio» (Trello #29): what EmuDome's files weigh on this device, and the few
//! things the screen may delete. Pure file work, no core, no Tauri: the shell calls it off the
//! event loop with the folders it owns, and never while the game concerned is open.
//!
//! What may be deleted here, and nothing else (Engineering rule 1, user data is never lost):
//! - a game file in EmuDome's own games folder (a game downloaded from Google Drive), only one
//!   whose path really lies inside that folder ([`remove_game_file`]); the shell checks first
//!   that the same game is on Drive. A file of the user's own folders is never touched;
//! - quick saves the user picked ([`delete_quick_saves`]): the slot's state, its thumbnail and
//!   its record. Never `game.sav`, its backups, a save set aside, or the core's own files;
//! - screenshots the user picked ([`delete_screenshots`]), by their checked names only;
//! - files that rebuild themselves ([`temp_paths`]): the named links to opened documents
//!   (Android), a download's leftover temporary file, Azahar's shader cache of a game not open.
//!
//! Sizes are walked without following symbolic links or junctions, so a link never makes a
//! folder count (or get deleted) twice.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::GameFiles;
use crate::{saves, screenshot, ConsoleId, GameKey, AUTO_SLOT};

/// A download's temporary file younger than this may still be written: left alone.
pub const TEMP_MIN_AGE: Duration = Duration::from_secs(60 * 60);

/// The file a download writes before its rename (`emudome_sync`'s `download_game`), and the
/// temporary file of [`crate::write_atomic`]: a leftover of either is temporary.
const TEMP_SUFFIX: &str = ".emudome-tmp";

/// Azahar's shader cache under a 3DS game's folder (its user tree is `<game>/Azahar/`, the
/// cache Citra's `shaders/` there): rebuilt by the core, a first start is slower meanwhile.
pub const AZAHAR_SHADERS: &str = "Azahar/shaders";

/// The bytes under `path`: a file's size, or a folder's files summed, without following links.
/// What cannot be read counts as nothing.
pub fn tree_size(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.is_file() {
        return meta.len();
    }
    if !meta.is_dir() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| tree_size(&e.path()))
        .sum()
}

/// [`tree_size`] of `path` without the subtrees `skip` (each counted nowhere).
pub fn tree_size_except(path: &Path, skip: &[&Path]) -> u64 {
    if skip.contains(&path) {
        return 0;
    }
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !meta.is_dir() {
        return tree_size(path);
    }
    if !skip.iter().any(|s| s.starts_with(path)) {
        return tree_size(path);
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| tree_size_except(&e.path(), skip))
        .sum()
}

fn modified_ms(meta: &std::fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| u64::try_from(d.as_millis()).ok())
}

// ---- Quick saves ----

/// One quick save on disk: its state with the thumbnail and record beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickFile {
    /// [`AUTO_SLOT`] for the auto quick save.
    pub slot: u8,
    /// The state, its thumbnail and its record together.
    pub bytes: u64,
    /// When the state was written, Unix ms.
    pub modified_ms: Option<u64>,
}

/// The slot a quick save's file name stands for: `auto.state` the auto quick save,
/// `slotN.state` slot N (1 to 255); anything else is not a quick save.
pub fn parse_slot(name: &str) -> Option<u8> {
    if name == "auto.state" {
        return Some(AUTO_SLOT);
    }
    let n = name.strip_prefix("slot")?.strip_suffix(".state")?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) || n.starts_with('0') {
        return None;
    }
    n.parse().ok().filter(|slot| *slot != AUTO_SLOT)
}

/// The quick saves in one game's folder, by slot (the auto quick save first).
pub fn game_quick_saves(saves_root: &Path, game: &GameKey) -> Vec<QuickFile> {
    let files = GameFiles::new(saves_root, game);
    let Ok(entries) = std::fs::read_dir(files.dir()) else {
        return Vec::new();
    };
    let mut list: Vec<QuickFile> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let slot = parse_slot(e.file_name().to_str()?)?;
            let meta = std::fs::symlink_metadata(e.path()).ok()?;
            if !meta.is_file() {
                return None;
            }
            let extra: u64 = [files.thumb(slot), files.meta(slot)]
                .iter()
                .filter_map(|p| std::fs::symlink_metadata(p).ok())
                .filter(std::fs::Metadata::is_file)
                .map(|m| m.len())
                .sum();
            Some(QuickFile {
                slot,
                bytes: meta.len() + extra,
                modified_ms: modified_ms(&meta),
            })
        })
        .collect();
    list.sort_by_key(|q| q.slot);
    list
}

/// The game folders under the saves root (`<console>/<hex>`), by key.
fn game_folders(saves_root: &Path) -> Vec<GameKey> {
    let mut keys = Vec::new();
    for console in ConsoleId::ALL {
        let Ok(entries) = std::fs::read_dir(saves_root.join(console.id())) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
            let key = entry
                .file_name()
                .to_str()
                .and_then(|hex| GameKey::new(console, hex).ok());
            if let (true, Some(key)) = (is_dir, key) {
                keys.push(key);
            }
        }
    }
    keys.sort();
    keys
}

/// Every game's quick saves, games with none left out.
pub fn all_quick_saves(saves_root: &Path) -> Vec<(GameKey, Vec<QuickFile>)> {
    game_folders(saves_root)
        .into_iter()
        .map(|key| {
            let quick = game_quick_saves(saves_root, &key);
            (key, quick)
        })
        .filter(|(_, quick)| !quick.is_empty())
        .collect()
}

/// Deletes the quick saves picked: each one's state, thumbnail and record
/// ([`saves::delete_quick`]); one already gone counts as deleted. Stops at the first failure,
/// saying how many went before it. Nothing else of the game's folder is touched.
pub fn delete_quick_saves(
    saves_root: &Path,
    picked: &[(GameKey, u8)],
) -> Result<usize, (usize, io::Error)> {
    let mut done = 0;
    for (game, slot) in picked {
        match saves::delete_quick(saves_root, game, *slot) {
            Ok(()) => done += 1,
            Err(e) if e.kind() == io::ErrorKind::NotFound => done += 1,
            Err(e) => return Err((done, e)),
        }
    }
    Ok(done)
}

// ---- Screenshots ----

/// The screenshots on disk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Screenshots {
    pub bytes: u64,
    pub count: u32,
    /// How many each game has, games with none left out.
    pub games: Vec<(GameKey, u32)>,
}

/// The screenshots under `saves/screenshots/<console>/<hex>/`.
pub fn screenshots(saves_root: &Path) -> Screenshots {
    let root = screenshot::screenshots_dir(saves_root);
    let mut out = Screenshots::default();
    for key in game_folders(&root) {
        let dir = screenshot::game_dir(saves_root, &key);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut count = 0u32;
        for entry in entries.filter_map(Result::ok) {
            let named = entry.file_name().to_str().is_some_and(screenshot::is_name);
            let meta = std::fs::symlink_metadata(entry.path());
            if let (true, Ok(meta)) = (named, meta) {
                if meta.is_file() {
                    count += 1;
                    out.bytes += meta.len();
                }
            }
        }
        if count > 0 {
            out.count += count;
            out.games.push((key, count));
        }
    }
    out
}

/// Deletes the screenshots of `game` named in `names` (each a plain screenshot name of its
/// folder, [`screenshot::file`]; any other name is skipped). Answers how many went.
pub fn delete_screenshots(
    saves_root: &Path,
    game: &GameKey,
    names: &[String],
) -> Result<usize, (usize, io::Error)> {
    let dir = screenshot::game_dir(saves_root, game);
    let mut done = 0;
    for name in names {
        let Some(path) = screenshot::file(&dir, name) else {
            continue;
        };
        match std::fs::remove_file(path) {
            Ok(()) => done += 1,
            Err(e) if e.kind() == io::ErrorKind::NotFound => done += 1,
            Err(e) => return Err((done, e)),
        }
    }
    Ok(done)
}

// ---- Temporary files ----

/// Where the temporary files are looked for.
#[derive(Clone, Debug)]
pub struct TempPlaces<'a> {
    /// The folder of the named links to opened documents (`<cache>/content`, Android); `None`
    /// where there is no cache folder.
    pub content_links: Option<PathBuf>,
    /// EmuDome's own games folder (downloads from Drive).
    pub games: &'a Path,
    /// The saves root (each 3DS game's Azahar shader cache).
    pub saves_root: &'a Path,
    /// The game open now: its files are in use, never counted or deleted.
    pub open: Option<&'a GameKey>,
    pub now: SystemTime,
}

/// The temporary files and folders that exist now: each rebuilds itself when needed.
pub fn temp_paths(places: &TempPlaces<'_>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    // A document's link is the path a running core may hold: none while a game is open.
    if let (None, Some(links)) = (places.open, &places.content_links) {
        if std::fs::symlink_metadata(links).is_ok_and(|m| m.is_dir()) {
            out.push(links.clone());
        }
    }
    for console in ConsoleId::ALL {
        let Ok(entries) = std::fs::read_dir(places.games.join(console.id())) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let leftover = entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with('.') && n.ends_with(TEMP_SUFFIX));
            let Ok(meta) = std::fs::symlink_metadata(entry.path()) else {
                continue;
            };
            let old = meta
                .modified()
                .ok()
                .and_then(|t| places.now.duration_since(t).ok())
                .is_some_and(|age| age >= TEMP_MIN_AGE);
            if leftover && meta.is_file() && old {
                out.push(entry.path());
            }
        }
    }
    for key in game_folders(places.saves_root) {
        if key.console() != ConsoleId::N3ds || places.open == Some(&key) {
            continue;
        }
        let shaders = GameFiles::new(places.saves_root, &key)
            .dir()
            .join(AZAHAR_SHADERS);
        if std::fs::symlink_metadata(&shaders).is_ok_and(|m| m.is_dir()) {
            out.push(shaders);
        }
    }
    out
}

/// Deletes the temporary `paths` ([`temp_paths`]); a link inside is removed, never followed.
/// Answers the bytes freed; a path that could not go is logged and left.
pub fn clear_temp(paths: &[PathBuf]) -> u64 {
    let mut freed = 0;
    for path in paths {
        let size = tree_size(path);
        let removed = match std::fs::symlink_metadata(path) {
            Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(path),
            Ok(_) => std::fs::remove_file(path),
            Err(e) => Err(e),
        };
        match removed {
            Ok(()) => freed += size,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => log::warn!("storage: {} not removed: {e}", path.display()),
        }
    }
    freed
}

// ---- A game file in EmuDome's own folder ----

/// Why a game file was not removed.
#[derive(Debug)]
pub enum RemoveError {
    /// The file is not a plain file inside EmuDome's games folder (a user's file, a link).
    NotOurs,
    Io(io::Error),
}

/// Whether `file` is a plain file (not a link) whose real path lies inside `folder`.
pub fn is_in_folder(folder: &Path, file: &Path) -> bool {
    let plain = std::fs::symlink_metadata(file).is_ok_and(|m| m.is_file());
    let inside = match (folder.canonicalize(), file.canonicalize()) {
        (Ok(folder), Ok(file)) => file.starts_with(&folder) && file != folder,
        (Err(_), _) | (_, Err(_)) => false,
    };
    plain && inside
}

/// «Togli dal telefono»: deletes the game file `file`, only when it lies in EmuDome's games
/// folder `games` ([`is_in_folder`]). Answers its size. The game's saves live elsewhere (the
/// saves root) and are not touched.
pub fn remove_game_file(games: &Path, file: &Path) -> Result<u64, RemoveError> {
    if !is_in_folder(games, file) {
        return Err(RemoveError::NotOurs);
    }
    let size = tree_size(file);
    std::fs::remove_file(file).map_err(RemoveError::Io)?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{
        all_quick_saves, clear_temp, delete_quick_saves, delete_screenshots, game_quick_saves,
        is_in_folder, parse_slot, remove_game_file, screenshots, temp_paths, tree_size,
        tree_size_except, RemoveError, TempPlaces, AZAHAR_SHADERS, TEMP_MIN_AGE,
    };
    use crate::settings::test_dir::TestDir;
    use crate::{ConsoleId, GameKey, AUTO_SLOT};

    fn key(console: ConsoleId, hex: &str) -> GameKey {
        GameKey::new(console, hex).unwrap()
    }

    /// A game's saves folder as EmuDome writes it: the in-game save, its backups, a save set
    /// aside, the core's own files, two quick saves (state, thumbnail, record) and the auto one.
    fn full_game(dir: &TestDir, game: &str) {
        for (rel, text) in [
            ("game.sav", "battery"),
            ("backups/1700000000000.launch.sav", "backup"),
            ("broken-1700000000000.sav", "aside"),
            ("quick-slot.json", "{\"slot\":2}"),
            ("melonDS DS/dldi.bin", "core file"),
            ("slot1.state", "state one"),
            ("slot1.png", "png"),
            ("slot1.json", "{}"),
            ("slot2.state", "state two"),
            ("auto.state", "auto"),
        ] {
            dir.write(&format!("saves/{game}/{rel}"), text);
        }
    }

    /// What must survive every deletion of this screen.
    fn assert_saves_kept(dir: &TestDir, game: &str) {
        for (rel, text) in [
            ("game.sav", "battery"),
            ("backups/1700000000000.launch.sav", "backup"),
            ("broken-1700000000000.sav", "aside"),
            ("quick-slot.json", "{\"slot\":2}"),
            ("melonDS DS/dldi.bin", "core file"),
        ] {
            assert_eq!(dir.read(&format!("saves/{game}/{rel}")), text, "{rel}");
        }
    }

    #[test]
    fn slots_are_read_from_their_file_names_only() {
        assert_eq!(parse_slot("auto.state"), Some(AUTO_SLOT));
        assert_eq!(parse_slot("slot1.state"), Some(1));
        assert_eq!(parse_slot("slot10.state"), Some(10));
        assert_eq!(parse_slot("slot0.state"), None);
        assert_eq!(parse_slot("slot01.state"), None);
        assert_eq!(parse_slot("slot.state"), None);
        assert_eq!(parse_slot("slot300.state"), None);
        assert_eq!(parse_slot("slot1.png"), None);
        assert_eq!(parse_slot("game.sav"), None);
    }

    #[test]
    fn a_games_quick_saves_count_their_thumbnail_and_record() {
        let dir = TestDir::new("storage-quick");
        full_game(&dir, "nds/abcd");
        let quick = game_quick_saves(&dir.path("saves"), &key(ConsoleId::Nds, "abcd"));
        let slots: Vec<(u8, u64)> = quick.iter().map(|q| (q.slot, q.bytes)).collect();
        assert_eq!(slots, vec![(AUTO_SLOT, 4), (1, 9 + 3 + 2), (2, 9)]);
        let all = all_quick_saves(&dir.path("saves"));
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, key(ConsoleId::Nds, "abcd"));
    }

    #[test]
    fn deleting_quick_saves_keeps_the_in_game_save_and_everything_else() {
        let dir = TestDir::new("storage-quick-delete");
        full_game(&dir, "nds/abcd");
        full_game(&dir, "gba/ef01");
        let nds = key(ConsoleId::Nds, "abcd");
        let done = delete_quick_saves(
            &dir.path("saves"),
            &[(nds.clone(), 1), (nds.clone(), AUTO_SLOT), (nds.clone(), 7)],
        )
        .unwrap();
        assert_eq!(done, 3, "a slot already gone counts as deleted");
        for gone in ["slot1.state", "slot1.png", "slot1.json", "auto.state"] {
            assert!(
                !dir.path(&format!("saves/nds/abcd/{gone}")).exists(),
                "{gone}"
            );
        }
        assert_eq!(dir.read("saves/nds/abcd/slot2.state"), "state two");
        assert_saves_kept(&dir, "nds/abcd");
        // The other game is not touched at all.
        assert_saves_kept(&dir, "gba/ef01");
        assert_eq!(
            game_quick_saves(&dir.path("saves"), &key(ConsoleId::Gba, "ef01")).len(),
            3
        );
    }

    #[test]
    fn screenshots_are_counted_and_deleted_by_their_names_only() {
        let dir = TestDir::new("storage-shots");
        full_game(&dir, "gba/ef01");
        dir.write("saves/screenshots/gba/ef01/2026-11-20_10-00-00.png", "aaaa");
        dir.write("saves/screenshots/gba/ef01/2026-11-21_10-00-00.png", "bb");
        dir.write("saves/screenshots/gba/ef01/notes.txt", "kept");
        let shots = screenshots(&dir.path("saves"));
        assert_eq!(shots.count, 2);
        assert_eq!(shots.bytes, 6);
        let gba = key(ConsoleId::Gba, "ef01");
        assert_eq!(shots.games, vec![(gba.clone(), 2)]);
        let done = delete_screenshots(
            &dir.path("saves"),
            &gba,
            &[
                "2026-11-20_10-00-00.png".to_owned(),
                "../../gba/ef01/game.sav".to_owned(),
                "notes.txt".to_owned(),
            ],
        )
        .unwrap();
        assert_eq!(done, 1);
        assert!(!dir
            .path("saves/screenshots/gba/ef01/2026-11-20_10-00-00.png")
            .exists());
        assert_eq!(
            dir.read("saves/screenshots/gba/ef01/2026-11-21_10-00-00.png"),
            "bb"
        );
        assert_eq!(dir.read("saves/screenshots/gba/ef01/notes.txt"), "kept");
        assert_saves_kept(&dir, "gba/ef01");
        assert_eq!(dir.read("saves/gba/ef01/slot1.state"), "state one");
    }

    #[test]
    fn temporary_files_are_the_links_old_downloads_and_shader_caches_only() {
        let dir = TestDir::new("storage-temp");
        full_game(&dir, "3ds/0004000000aaaa00");
        full_game(&dir, "3ds/0004000000bbbb00");
        dir.write(
            "saves/3ds/0004000000aaaa00/Azahar/shaders/vulkan/cache.bin",
            "123456",
        );
        dir.write(
            "saves/3ds/0004000000aaaa00/Azahar/nand/data/save.bin",
            "nand save",
        );
        dir.write(
            "saves/3ds/0004000000bbbb00/Azahar/shaders/vulkan/cache.bin",
            "open",
        );
        dir.write("cache/content/7/game.3ds", "link");
        dir.write("games/gba/.ef01.1-download.emudome-tmp", "partial");
        dir.write("games/gba/Game.gba", "rom");
        let open = key(ConsoleId::N3ds, "0004000000bbbb00");
        let saves = dir.path("saves");
        let games = dir.path("games");
        let later = SystemTime::now() + TEMP_MIN_AGE + Duration::from_secs(5);
        let places = |open, now| TempPlaces {
            content_links: Some(dir.path("cache/content")),
            games: &games,
            saves_root: &saves,
            open,
            now,
        };

        // A download written just now may still be running; the open game's files are in use.
        let now = temp_paths(&places(Some(&open), SystemTime::now()));
        assert_eq!(
            now,
            vec![dir.path("saves/3ds/0004000000aaaa00").join(AZAHAR_SHADERS)]
        );

        let paths = temp_paths(&places(None, later));
        assert_eq!(paths.len(), 4);
        let freed = clear_temp(&paths);
        assert_eq!(freed, 6 + 4 + 7 + 4);
        assert!(!dir.path("cache/content").exists());
        assert!(!dir.path("games/gba/.ef01.1-download.emudome-tmp").exists());
        assert!(!dir
            .path("saves/3ds/0004000000aaaa00/Azahar/shaders")
            .exists());
        assert_eq!(dir.read("games/gba/Game.gba"), "rom");
        assert_eq!(
            dir.read("saves/3ds/0004000000aaaa00/Azahar/nand/data/save.bin"),
            "nand save"
        );
        for game in ["3ds/0004000000aaaa00", "3ds/0004000000bbbb00"] {
            assert_saves_kept(&dir, game);
            assert_eq!(dir.read(&format!("saves/{game}/slot2.state")), "state two");
        }
    }

    #[test]
    fn a_game_file_goes_only_from_emudomes_own_folder() {
        let dir = TestDir::new("storage-remove");
        full_game(&dir, "gba/ef01");
        dir.write("games/gba/Game.gba", "rom bytes");
        dir.write("mine/Game.gba", "the user's own file");
        let games = dir.path("games");

        let outside = remove_game_file(&games, &dir.path("mine/Game.gba"));
        assert!(matches!(outside, Err(RemoveError::NotOurs)));
        assert_eq!(dir.read("mine/Game.gba"), "the user's own file");
        // A path that climbs out of the folder is not inside it either.
        let climbing = games.join("gba/../../mine/Game.gba");
        assert!(!is_in_folder(&games, &climbing));
        assert!(matches!(
            remove_game_file(&games, &climbing),
            Err(RemoveError::NotOurs)
        ));
        assert!(!is_in_folder(&games, &games));

        assert_eq!(
            remove_game_file(&games, &dir.path("games/gba/Game.gba")).unwrap(),
            9
        );
        assert!(!dir.path("games/gba/Game.gba").exists());
        assert_saves_kept(&dir, "gba/ef01");
        assert_eq!(dir.read("saves/gba/ef01/slot1.state"), "state one");
        assert_eq!(dir.read("saves/gba/ef01/auto.state"), "auto");
    }

    #[test]
    fn sizes_skip_the_subtrees_asked() {
        let dir = TestDir::new("storage-size");
        dir.write("local/library/library.json", "12345");
        dir.write("local/games/gba/Game.gba", "1234567890");
        assert_eq!(tree_size(&dir.path("local")), 15);
        let games = dir.path("local/games");
        assert_eq!(tree_size_except(&dir.path("local"), &[&games]), 5);
        assert_eq!(tree_size(&dir.path("missing")), 0);
    }
}
