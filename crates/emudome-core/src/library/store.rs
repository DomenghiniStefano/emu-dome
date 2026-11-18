//! The library database: one JSON file, `library.json`, in a folder the shell chooses.
//!
//! `{ "games": { "<id>": GameRecord }, "hashCache": { "<path>": CacheEntry } }`, plus any key a
//! newer version wrote (kept). Games are never deleted: a game whose file or folder is gone is
//! kept with `available: false`; one whose folder left `library.folders` is kept with
//! `listed: false` (hidden from the interface), so its play time, flags and cover come back with
//! it. Written atomically like the settings files (`settings::store::write_atomic`); a file that
//! cannot be parsed is moved aside to `library.json.unreadable` and the library starts empty.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

use super::identity::Identity;
use super::scan::ScanOutcome;
use crate::settings::ConsoleId;
use crate::ErrorCode;

/// Which picture a cover shows. Mirror: `CoverKind` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoverKind {
    /// An image the user chose (copied into the covers folder).
    User,
    /// The icon inside the ROM (NDS banner, 3DS SMDH).
    Icon,
    /// Drawn by the interface: `colour` and the title.
    Generated,
}

/// One game as stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecord {
    pub id: String,
    pub console: ConsoleId,
    pub title: String,
    pub path: String,
    /// The library folder it was found in; `None` for a file added on its own.
    pub folder: Option<String>,
    pub size: u64,
    /// Modification time of the file, Unix ms.
    pub mtime: u64,
    /// Reachable at the last scan.
    pub available: bool,
    /// Shown in the library (false once its folder left `library.folders`).
    pub listed: bool,
    pub play_time_secs: u64,
    /// Unix ms.
    pub last_played: Option<u64>,
    /// Unix ms.
    pub added_at: u64,
    pub cover: CoverKind,
    /// File name of the user's cover inside the covers folder.
    pub user_cover: Option<String>,
    /// Bumped at every cover change, so the webview reloads `emu://cover/<id>?v=<rev>`.
    pub cover_rev: u32,
    pub has_icon: bool,
    pub favourite: bool,
    pub hidden: bool,
    pub code: Option<String>,
    pub version: Option<u16>,
    /// «Nome mostrato» (L10): the user's name for the game, over the header's `title`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl GameRecord {
    /// The name the interface shows: the user's, else the one from the ROM.
    pub fn shown_title(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.title)
    }
}

/// Longest «Nome mostrato», in characters.
pub const NAME_MAX: usize = 128;

/// A file already identified: reused while its size and modification time do not change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub size: u64,
    pub mtime: u64,
    pub identity: Identity,
}

/// Path → identity of the files seen.
pub type HashCache = BTreeMap<String, CacheEntry>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Db {
    #[serde(default)]
    games: BTreeMap<String, GameRecord>,
    #[serde(default)]
    hash_cache: HashCache,
    /// Anything else a newer version wrote.
    #[serde(flatten)]
    other: Map<String, Json>,
}

/// What [`Library::apply_scan`] changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanReport {
    /// Games found in the scanned folders and files.
    pub found: usize,
    /// Games that were not in the library before (new, or back with their folder).
    pub added: usize,
    /// Folders that could not be read.
    pub unreachable: Vec<String>,
}

/// The library of this device.
pub struct Library {
    file: PathBuf,
    db: Db,
}

impl Library {
    /// Opens `<dir>/library.json` (empty if missing).
    pub fn open(dir: &Path) -> Self {
        let file = dir.join("library.json");
        let db = read_db(&file).unwrap_or_default();
        Self { file, db }
    }

    /// The games shown in the library.
    pub fn games(&self) -> impl Iterator<Item = &GameRecord> {
        self.db.games.values().filter(|g| g.listed)
    }

    /// A game shown in the library.
    pub fn game(&self, id: &str) -> Result<&GameRecord, ErrorCode> {
        self.db
            .games
            .get(id)
            .filter(|g| g.listed)
            .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.into() })
    }

    /// The listed game whose file is `path`, if any.
    pub fn game_at(&self, path: &str) -> Option<&GameRecord> {
        self.games().find(|g| g.path == path)
    }

    fn game_mut(&mut self, id: &str) -> Result<&mut GameRecord, ErrorCode> {
        self.db
            .games
            .get_mut(id)
            .filter(|g| g.listed)
            .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.into() })
    }

    /// The hash cache, for a scan running outside the lock.
    pub fn cache(&self) -> &HashCache {
        &self.db.hash_cache
    }

    /// Paths of the listed games added as single files (rechecked by every scan).
    pub fn single_files(&self) -> Vec<PathBuf> {
        self.games()
            .filter(|g| g.folder.is_none())
            .map(|g| PathBuf::from(&g.path))
            .collect()
    }

    /// Merges a scan: games found become listed and available (new ones are added), listed
    /// games not found become unavailable, games of folders no longer in `library.folders`
    /// leave the library (kept on disk, unlisted). Writes the file.
    pub fn apply_scan(&mut self, scan: ScanOutcome, now_ms: u64) -> Result<ScanReport, ErrorCode> {
        let mut seen: BTreeMap<&str, &super::scan::Found> = BTreeMap::new();
        for found in &scan.found {
            let current = self
                .db
                .games
                .get(&found.identity.id)
                .is_some_and(|g| g.path == found.path);
            // Duplicates: keep the path the game already has, else the first one found.
            if current || !seen.contains_key(found.identity.id.as_str()) {
                seen.insert(&found.identity.id, found);
            }
        }
        let folders: BTreeSet<&str> = scan.folders.iter().map(String::as_str).collect();
        let mut added = 0;
        for game in self.db.games.values_mut() {
            if let Some(found) = seen.get(game.id.as_str()) {
                if !game.listed {
                    game.listed = true;
                    added += 1;
                }
                game.available = true;
                game.path.clone_from(&found.path);
                game.folder.clone_from(&found.folder);
                game.size = found.size;
                game.mtime = found.mtime;
                game.title.clone_from(&found.identity.title);
                game.has_icon = found.identity.has_icon;
                if game.cover == CoverKind::Icon && !game.has_icon {
                    game.cover = CoverKind::Generated;
                }
            } else if game.folder.as_deref().is_some_and(|f| !folders.contains(f)) {
                game.listed = false;
            } else {
                game.available = false;
            }
        }
        for (id, found) in &seen {
            if !self.db.games.contains_key(*id) {
                added += 1;
                self.db
                    .games
                    .insert((*id).to_owned(), new_record(found, now_ms));
            }
        }
        let report = ScanReport {
            found: seen.len(),
            added,
            unreachable: scan.unreachable,
        };
        self.db.hash_cache = scan.cache;
        self.save()?;
        Ok(report)
    }

    /// A folder left `library.folders`: its games leave the library (kept, unlisted).
    pub fn unlist_folder(&mut self, folder: &str) -> Result<(), ErrorCode> {
        for game in self.db.games.values_mut() {
            if game.folder.as_deref() == Some(folder) {
                game.listed = false;
            }
        }
        self.save()
    }

    pub fn set_flags(
        &mut self,
        id: &str,
        favourite: Option<bool>,
        hidden: Option<bool>,
    ) -> Result<(), ErrorCode> {
        let game = self.game_mut(id)?;
        if let Some(f) = favourite {
            game.favourite = f;
        }
        if let Some(h) = hidden {
            game.hidden = h;
        }
        self.save()
    }

    /// «Nome mostrato» (L10). Blank, or the same as the ROM's title, goes back to the ROM's title;
    /// spaces around are dropped and a name longer than [`NAME_MAX`] is cut.
    pub fn set_name(&mut self, id: &str, name: &str) -> Result<(), ErrorCode> {
        let game = self.game_mut(id)?;
        let name: String = name.trim().chars().take(NAME_MAX).collect();
        let name = name.trim_end();
        game.name = (!name.is_empty() && name != game.title).then(|| name.to_owned());
        self.save()
    }

    /// Changes the cover. `user_file` is the file name in the covers folder, for `User`.
    pub fn set_cover(
        &mut self,
        id: &str,
        kind: CoverKind,
        user_file: Option<String>,
    ) -> Result<(), ErrorCode> {
        let game = self.game_mut(id)?;
        match kind {
            CoverKind::Icon if !game.has_icon => {
                return Err(ErrorCode::CoverInvalid {
                    detail: "the ROM has no icon".into(),
                })
            }
            CoverKind::User if user_file.is_none() => {
                return Err(ErrorCode::CoverInvalid {
                    detail: "no image".into(),
                })
            }
            CoverKind::Icon | CoverKind::User | CoverKind::Generated => {}
        }
        if kind == CoverKind::User {
            game.user_cover = user_file;
        }
        game.cover = kind;
        game.cover_rev = game.cover_rev.wrapping_add(1);
        self.save()
    }

    /// The file is not reachable now.
    pub fn set_unavailable(&mut self, id: &str) -> Result<(), ErrorCode> {
        self.game_mut(id)?.available = false;
        self.save()
    }

    /// The game was opened now.
    pub fn mark_played(&mut self, id: &str, now_ms: u64) -> Result<(), ErrorCode> {
        self.game_mut(id)?.last_played = Some(now_ms);
        self.save()
    }

    /// Adds a play stretch to the game's total.
    pub fn add_play_time(&mut self, id: &str, secs: u64) -> Result<(), ErrorCode> {
        let game = self.game_mut(id)?;
        game.play_time_secs = game.play_time_secs.saturating_add(secs);
        self.save()
    }

    /// Writes the file (the in-memory state stays as it is if this fails; the next write
    /// carries it).
    pub fn save(&self) -> Result<(), ErrorCode> {
        let write_failed = |e: String| ErrorCode::LibraryWriteFailed { detail: e };
        let mut bytes =
            serde_json::to_vec_pretty(&self.db).map_err(|e| write_failed(e.to_string()))?;
        bytes.push(b'\n');
        crate::settings::write_atomic(&self.file, &bytes).map_err(|e| write_failed(e.to_string()))
    }
}

fn new_record(found: &super::scan::Found, now_ms: u64) -> GameRecord {
    let id = &found.identity;
    GameRecord {
        id: id.id.clone(),
        console: id.console,
        title: id.title.clone(),
        path: found.path.clone(),
        folder: found.folder.clone(),
        size: found.size,
        mtime: found.mtime,
        available: true,
        listed: true,
        play_time_secs: 0,
        last_played: None,
        added_at: now_ms,
        cover: if id.has_icon {
            CoverKind::Icon
        } else {
            CoverKind::Generated
        },
        user_cover: None,
        cover_rev: 0,
        has_icon: id.has_icon,
        favourite: false,
        hidden: false,
        code: id.code.clone(),
        version: id.version,
        name: None,
    }
}

/// Background colours of generated covers: muted tones that read with white text.
const COVER_COLOURS: &[&str] = &[
    "#3d4a6b", "#5b3d6b", "#6b3d4f", "#6b4a3d", "#5e5a35", "#3d6b52", "#3d5f6b", "#4a4f5e",
    "#6b3d3d", "#3d6b6b", "#54406e", "#2f5a4a",
];

/// The generated cover's colour for a game, `#rrggbb`, always the same for the same id.
pub fn cover_colour(id: &str) -> &'static str {
    // FNV-1a: stable across Rust versions and platforms.
    let hash = id.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    let index = usize::try_from(hash % COVER_COLOURS.len() as u64).unwrap_or_default();
    COVER_COLOURS.get(index).copied().unwrap_or("#4a4f5e")
}

fn read_db(path: &Path) -> Option<Db> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("libraryReadFailed: {}: {e}", path.display());
            return None;
        }
    };
    match serde_json::from_str(&text) {
        Ok(db) => Some(db),
        Err(e) => {
            let aside = path.with_extension("json.unreadable");
            log::warn!(
                "libraryFileUnreadable: {}: {e}; moved to {}, starting empty",
                path.display(),
                aside.display()
            );
            if let Err(e) = fs::rename(path, &aside) {
                log::warn!("libraryReadFailed: could not move {}: {e}", path.display());
            }
            None
        }
    }
}
