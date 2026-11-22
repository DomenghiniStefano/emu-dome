//! The one-time move of the files named the old way to the game key (Trello #70), and of the
//! screenshots named by ROM file stem ([`run_screenshots`], Trello #85, its own marker).
//!
//! Before 2026-11-20 a game's files were not named by its [`GameKey`]:
//! - saves, flat in the saves root and named by the ROM's file stem: `<stem>.sav`,
//!   `<stem>.slotN.state|png|json`, `<stem>.auto.state|png|json`, `<stem>.broken-<ms>.sav`,
//!   `backups/<stem>.<ms>.<reason>.sav`;
//! - «Solo questo gioco» settings of a game opened by its path, named by a hash of the path:
//!   `settings/game/path-<fnv1a 64>.json`, and `"game": { "path-…": {..} }` in the device file.
//!
//! They move to the game's key ([`crate::GameFiles`], `settings/game/<console>-<hex>.json`) only
//! where facts say which game they belong to, never by guessing:
//! - the library's files ([`crate::library::Library::known_files`]: path → key) give
//!   stem → key, when exactly one key has that stem (ignoring case, as the old names did), and
//!   path hash → key;
//! - a stem shared by several games, a file no library game matches, a name that is not one of
//!   the old patterns, or a file whose new place is already taken: left where it is, untouched,
//!   and listed in the report. The app no longer reads it there; the user can move it by hand.
//!   A save given to the wrong game would be worse than one left where it is.
//! - folders a core made by itself (`melonDS DS/`, `Azahar/`) are shared by every game of that
//!   core: left where they are, listed apart (`shared`).
//!
//! Safety: before a file moves, a copy of it goes to `<work>/backup/` under its old relative
//! name; then it is renamed to its new place (an atomic rename on the same drive). An
//! interruption leaves each file either at its old place, or at its new place with its copy:
//! running again finishes the job. A file that already has its copy is not copied again.
//!
//! Once a pass ends without errors over a readable library that knows at least one game (or with
//! nothing to move), `<work>/done` is written and the move never runs again. Every pass writes
//! `<work>/report.json`. The caller runs it at start, after the library is read and before any
//! game opens, and never when the library could not be read (its games would look missing).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value as Json};

use crate::settings::ConsoleId;
use crate::{write_atomic, GameKey};

/// Where the old files are and where the move keeps its marker, report and copies.
pub struct Paths {
    /// `<data>/saves`.
    pub saves_root: PathBuf,
    /// `<data>/settings/game`.
    pub settings_games: PathBuf,
    /// `<local data>/local-settings/device.json`.
    pub device_file: PathBuf,
    /// `<data>/migration-70`: `done`, `report.json`, `backup/`.
    pub work_dir: PathBuf,
}

impl Paths {
    pub fn marker(&self) -> PathBuf {
        self.work_dir.join("done")
    }

    pub fn report(&self) -> PathBuf {
        self.work_dir.join("report.json")
    }

    fn backup(&self) -> PathBuf {
        self.work_dir.join("backup")
    }
}

/// Why a file stayed where it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LeftReason {
    /// No library game has this file name / path.
    NoGame,
    /// Several library games have this file name: which one is not known.
    Ambiguous,
    /// The game already has a file at the new place: neither is touched.
    DestinationExists,
    /// Not one of the old name patterns.
    UnknownName,
}

/// A file moved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Moved {
    pub from: String,
    pub to: String,
    pub game: GameKey,
}

/// A file left where it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Left {
    pub file: String,
    pub reason: LeftReason,
    /// The candidates when [`LeftReason::Ambiguous`].
    pub games: Vec<GameKey>,
}

/// What a pass did. Written to `report.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Unix ms of the pass.
    pub ran_at: u64,
    pub moved: Vec<Moved>,
    pub left: Vec<Left>,
    /// Folders shared by every game of a core, left where they are.
    pub shared: Vec<String>,
    /// What failed (technical text); a pass with errors is run again at the next start.
    pub errors: Vec<String>,
    /// The marker was written: the move will not run again.
    pub done: bool,
}

/// Whether the move already ended for good.
pub fn is_done(paths: &Paths) -> bool {
    paths.marker().is_file()
}

/// The old settings identity of a game opened by its path: `path-<FNV-1a 64 of the path>`.
pub fn legacy_path_id(path: &str) -> String {
    format!("path-{:016x}", fnv1a(path.as_bytes()))
}

/// FNV-1a, 64 bit, as the old settings identity used it.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The file stem of a path, lower case (as the old save names were matched).
fn lower_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// What the library's facts say: lower stem → keys, old path id → keys.
struct Facts {
    by_stem: BTreeMap<String, BTreeSet<GameKey>>,
    by_path_id: BTreeMap<String, BTreeSet<GameKey>>,
}

impl Facts {
    fn new(known: &[(String, GameKey)]) -> Self {
        let mut by_stem: BTreeMap<String, BTreeSet<GameKey>> = BTreeMap::new();
        let mut by_path_id: BTreeMap<String, BTreeSet<GameKey>> = BTreeMap::new();
        for (path, key) in known {
            by_stem
                .entry(lower_stem(path))
                .or_default()
                .insert(key.clone());
            by_path_id
                .entry(legacy_path_id(path))
                .or_default()
                .insert(key.clone());
        }
        Self {
            by_stem,
            by_path_id,
        }
    }
}

/// One key, or why there is none.
fn only(keys: Option<&BTreeSet<GameKey>>) -> Result<GameKey, (LeftReason, Vec<GameKey>)> {
    match keys {
        None => Err((LeftReason::NoGame, Vec::new())),
        Some(set) => match set.iter().collect::<Vec<_>>().as_slice() {
            [one] => Ok((*one).clone()),
            [] => Err((LeftReason::NoGame, Vec::new())),
            many => Err((
                LeftReason::Ambiguous,
                many.iter().map(|k| (*k).clone()).collect(),
            )),
        },
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// An old save name in the saves root: its stem and its name in the game's folder.
pub fn legacy_save(name: &str) -> Option<(String, String)> {
    if let Some(base) = name.strip_suffix(".sav") {
        if let Some((stem, ms)) = base.rsplit_once(".broken-") {
            if digits(ms) && !stem.is_empty() {
                return Some((stem.to_owned(), format!("broken-{ms}.sav")));
            }
        }
        return (!base.is_empty()).then(|| (base.to_owned(), "game.sav".to_owned()));
    }
    ["state", "png", "json"].into_iter().find_map(|ext| {
        let base = name.strip_suffix(ext)?.strip_suffix('.')?;
        if let Some(stem) = base.strip_suffix(".auto") {
            return (!stem.is_empty()).then(|| (stem.to_owned(), format!("auto.{ext}")));
        }
        let (stem, n) = base.rsplit_once(".slot")?;
        (digits(n) && n.parse::<u8>().is_ok() && !stem.is_empty())
            .then(|| (stem.to_owned(), format!("slot{n}.{ext}")))
    })
}

/// An old backup name (`backups/<stem>.<ms>.<reason>.sav`): its stem and its new name.
pub fn legacy_backup(name: &str) -> Option<(String, String)> {
    let base = name.strip_suffix(".sav")?;
    let (rest, reason) = base.rsplit_once('.')?;
    if !["launch", "import", "restore"].contains(&reason) {
        return None;
    }
    let (stem, ms) = rest.rsplit_once('.')?;
    (digits(ms) && !stem.is_empty()).then(|| (stem.to_owned(), format!("{ms}.{reason}.sav")))
}

/// Runs one pass (see the module comment) and writes the report, and the marker when done.
/// `known`: the library's files and their keys; the library must have been read.
pub fn run(paths: &Paths, known: &[(String, GameKey)], now_ms: u64) -> Report {
    let facts = Facts::new(known);
    let mut pass = Pass {
        paths,
        report: Report {
            ran_at: now_ms,
            ..Report::default()
        },
        found: 0,
    };
    pass.saves(&facts);
    pass.settings_files(&facts);
    pass.device_file(&facts);
    conclude(pass, known, b"70\n")
}

/// The one-time move of the screenshots named by ROM file stem
/// (`<saves root>/screenshots/<stem> <time>.png`) into each game's screenshots folder
/// ([`crate::screenshot::game_dir`], Trello #85), by the same rules as [`run`]: facts from the
/// library only, a copy in `<work_dir>/backup/screenshots/` before the rename, what cannot be
/// given to one game left where it is and reported, `<work_dir>/report.json`, and
/// `<work_dir>/done` once a pass ends without errors over a library that knows a game.
pub fn run_screenshots(
    saves_root: &Path,
    work_dir: &Path,
    known: &[(String, GameKey)],
    now_ms: u64,
) -> Report {
    let paths = Paths {
        saves_root: saves_root.to_path_buf(),
        settings_games: PathBuf::new(),
        device_file: PathBuf::new(),
        work_dir: work_dir.to_path_buf(),
    };
    let facts = Facts::new(known);
    let mut pass = Pass {
        paths: &paths,
        report: Report {
            ran_at: now_ms,
            ..Report::default()
        },
        found: 0,
    };
    pass.screenshots(&facts);
    conclude(pass, known, b"85\n")
}

/// The end of a pass: whether it is done, the report and the marker written.
fn conclude(pass: Pass<'_>, known: &[(String, GameKey)], marker: &[u8]) -> Report {
    let paths = pass.paths;
    let mut report = pass.report;
    let library_knows = !known.is_empty();
    report.done = report.errors.is_empty() && (library_knows || pass.found == 0);
    if !report.done && report.errors.is_empty() {
        log::info!(
            "game key migration: {} old files, but the library knows no game yet: tried again at the next start",
            pass.found
        );
    }
    let written = serde_json::to_vec_pretty(&report)
        .map_err(io::Error::other)
        .and_then(|bytes| write_atomic(&paths.report(), &bytes));
    if let Err(e) = written {
        log::warn!("game key migration report not written: {e}");
    }
    if report.done {
        if let Err(e) = write_atomic(&paths.marker(), marker) {
            log::warn!("game key migration marker not written: {e}");
            report.done = false;
        }
    }
    report
}

struct Pass<'a> {
    paths: &'a Paths,
    report: Report,
    /// Old files seen (moved or not).
    found: usize,
}

/// Folders of the saves root that are not old saves: the new layout, screenshots.
fn own_folder(name: &str) -> bool {
    name == "screenshots" || name == "backups" || ConsoleId::from_id(name).is_some()
}

impl Pass<'_> {
    fn error(&mut self, what: String) {
        log::warn!("game key migration: {what}");
        self.report.errors.push(what);
    }

    fn left(&mut self, file: &Path, reason: LeftReason, games: Vec<GameKey>) {
        log::warn!(
            "game key migration: {} left where it is ({reason:?} {games:?})",
            file.display()
        );
        self.report.left.push(Left {
            file: file.to_string_lossy().into_owned(),
            reason,
            games,
        });
    }

    /// The entries of `dir` (name, is a folder), sorted; a missing folder has none.
    fn entries(&mut self, dir: &Path) -> Vec<(String, bool)> {
        match fs::read_dir(dir) {
            Ok(entries) => {
                let mut list: Vec<(String, bool)> = entries
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        let name = e.file_name().into_string().ok()?;
                        let dir = e.file_type().ok()?.is_dir();
                        Some((name, dir))
                    })
                    .collect();
                list.sort();
                list
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                self.error(format!("{}: {e}", dir.display()));
                Vec::new()
            }
        }
    }

    fn saves(&mut self, facts: &Facts) {
        let root = self.paths.saves_root.clone();
        for (name, is_dir) in self.entries(&root) {
            if is_dir {
                if !own_folder(&name) && !name.starts_with('.') {
                    log::info!("game key migration: {name}/ is a core's own folder, left as it is");
                    self.report
                        .shared
                        .push(root.join(&name).to_string_lossy().into_owned());
                }
                continue;
            }
            // Shared by every game, or a temporary file of an interrupted write.
            if name == "cores.json" || name.starts_with('.') {
                continue;
            }
            self.found += 1;
            let file = root.join(&name);
            match legacy_save(&name) {
                Some((stem, new_name)) => self.move_save(facts, &file, &name, &stem, &new_name),
                None => self.left(&file, LeftReason::UnknownName, Vec::new()),
            }
        }
        let backups = root.join("backups");
        for (name, is_dir) in self.entries(&backups) {
            if is_dir || name.starts_with('.') {
                continue;
            }
            self.found += 1;
            let file = backups.join(&name);
            match legacy_backup(&name) {
                Some((stem, new_name)) => self.move_save(
                    facts,
                    &file,
                    &format!("backups/{name}"),
                    &stem,
                    &format!("backups/{new_name}"),
                ),
                None => self.left(&file, LeftReason::UnknownName, Vec::new()),
            }
        }
    }

    /// The old screenshots of the screenshots root (its folders are the new layout).
    fn screenshots(&mut self, facts: &Facts) {
        let root = self.paths.saves_root.clone();
        let dir = crate::screenshot::screenshots_dir(&root);
        for (name, is_dir) in self.entries(&dir) {
            if is_dir || name.starts_with('.') {
                continue;
            }
            self.found += 1;
            let file = dir.join(&name);
            let Some((stem, new_name)) = crate::screenshot::legacy_name(&name) else {
                self.left(&file, LeftReason::UnknownName, Vec::new());
                continue;
            };
            match only(facts.by_stem.get(&stem.to_lowercase())) {
                Ok(key) => {
                    let dest = crate::screenshot::game_dir(&root, &key).join(new_name);
                    let backup = self.paths.backup().join("screenshots").join(&name);
                    self.move_file(&file, &dest, &backup, key);
                }
                Err((reason, games)) => self.left(&file, reason, games),
            }
        }
    }

    /// `file` (`rel` under the saves root) to `new_rel` in the folder of the one game with
    /// `stem`.
    fn move_save(&mut self, facts: &Facts, file: &Path, rel: &str, stem: &str, new_rel: &str) {
        match only(facts.by_stem.get(&stem.to_lowercase())) {
            Ok(key) => {
                let dest = self.paths.saves_root.join(key.dir()).join(new_rel);
                let backup = self.paths.backup().join("saves").join(rel);
                self.move_file(file, &dest, &backup, key);
            }
            Err((reason, games)) => self.left(file, reason, games),
        }
    }

    fn settings_files(&mut self, facts: &Facts) {
        let dir = self.paths.settings_games.clone();
        for (name, is_dir) in self.entries(&dir) {
            let Some(id) = name.strip_suffix(".json").filter(|_| !is_dir) else {
                continue;
            };
            if !id.starts_with("path-") {
                continue;
            }
            self.found += 1;
            let file = dir.join(&name);
            match only(facts.by_path_id.get(id)) {
                Ok(key) => {
                    let dest = dir.join(format!("{}.json", key.file_stem()));
                    let backup = self.paths.backup().join("settings/game").join(&name);
                    self.move_file(&file, &dest, &backup, key);
                }
                Err((reason, games)) => self.left(&file, reason, games),
            }
        }
    }

    /// Copies `file` to `backup` (once), then renames it to `dest` unless `dest` is taken.
    fn move_file(&mut self, file: &Path, dest: &Path, backup: &Path, key: GameKey) {
        if dest.exists() {
            self.left(file, LeftReason::DestinationExists, vec![key]);
            return;
        }
        let moved = keep_copy(file, backup)
            .and_then(|()| fs::create_dir_all(dest.parent().unwrap_or(Path::new("."))))
            .and_then(|()| fs::rename(file, dest));
        match moved {
            Ok(()) => {
                log::info!(
                    "game key migration: {} → {}",
                    file.display(),
                    dest.display()
                );
                self.report.moved.push(Moved {
                    from: file.to_string_lossy().into_owned(),
                    to: dest.to_string_lossy().into_owned(),
                    game: key,
                });
            }
            Err(e) => self.error(format!("{}: {e}", file.display())),
        }
    }

    /// The per-device values of games opened by path: `"game": { "path-…": {..} }` renamed to
    /// the key's file stem, in one atomic write after a copy of the file.
    fn device_file(&mut self, facts: &Facts) {
        let path = self.paths.device_file.clone();
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return,
            Err(e) => return self.error(format!("{}: {e}", path.display())),
        };
        let mut root: Map<String, Json> = match serde_json::from_slice(&bytes) {
            Ok(Json::Object(map)) => map,
            Ok(_) => return self.error(format!("{}: not a JSON object", path.display())),
            Err(e) => return self.error(format!("{}: {e}", path.display())),
        };
        let Some(Json::Object(games)) = root.get_mut("game") else {
            return;
        };
        let old: Vec<String> = games
            .keys()
            .filter(|k| k.starts_with("path-"))
            .cloned()
            .collect();
        let mut moved = Vec::new();
        let mut left = Vec::new();
        for id in old {
            self.found += 1;
            let label = path.join(format!("game/{id}"));
            match only(facts.by_path_id.get(&id)) {
                Ok(key) if games.contains_key(&key.file_stem()) => {
                    left.push((label, LeftReason::DestinationExists, vec![key]));
                }
                Ok(key) => {
                    if let Some(values) = games.remove(&id) {
                        games.insert(key.file_stem(), values);
                        moved.push((label, key));
                    }
                }
                Err((reason, keys)) => left.push((label, reason, keys)),
            }
        }
        for (label, reason, keys) in left {
            self.left(&label, reason, keys);
        }
        if moved.is_empty() {
            return;
        }
        let written = serde_json::to_vec_pretty(&Json::Object(root))
            .map_err(io::Error::other)
            .and_then(|new| {
                keep_copy(
                    &path,
                    &self.paths.backup().join("local-settings/device.json"),
                )?;
                write_atomic(&path, &new)
            });
        match written {
            Ok(()) => {
                for (label, key) in moved {
                    self.report.moved.push(Moved {
                        to: path
                            .join(format!("game/{}", key.file_stem()))
                            .to_string_lossy()
                            .into_owned(),
                        from: label.to_string_lossy().into_owned(),
                        game: key,
                    });
                }
            }
            Err(e) => self.error(format!("{}: {e}", path.display())),
        }
    }
}

/// A copy of `file` at `backup`, unless one with the same bytes is there already; a different
/// file there is kept and the copy goes to `<backup>.1`, `.2`, ...
fn keep_copy(file: &Path, backup: &Path) -> io::Result<()> {
    let bytes = fs::read(file)?;
    let mut target = backup.to_path_buf();
    for n in 1u32.. {
        match fs::read(&target) {
            Ok(existing) if existing == bytes => return Ok(()),
            Ok(_) => {
                let mut name = backup.as_os_str().to_owned();
                name.push(format!(".{n}"));
                target = PathBuf::from(name);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => return write_atomic(&target, &bytes),
            Err(e) => return Err(e),
        }
        if n > 1000 {
            break;
        }
    }
    Err(io::Error::other("no free backup name"))
}

#[cfg(test)]
mod tests;
