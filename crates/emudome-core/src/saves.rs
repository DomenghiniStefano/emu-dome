//! The saves of a game on disk (Saves section, boards V01–V05, E03, E04): what exists, the
//! backups of the in-game save and their rotation, restoring one, importing a save from another
//! emulator, setting a broken save aside. Pure file work, no core: the shell calls it while the
//! game is closed (the open session owns the `.sav` and rewrites it).
//!
//! Files, each game in its own folder named by its key ([`GameKey`], never the file name),
//! `<game>` = `<save_dir>/<console>/<hex>` ([`GameFiles`]):
//! - `<game>/game.sav`: the in-game save («Salvataggio del gioco»);
//! - `<game>/slotN.state` / `auto.state` (+ `.png` thumbnail, + `.json` [`SlotMeta`]): quick saves;
//! - `<game>/backups/<unix ms>.<reason>.sav`: backups of the in-game save;
//! - `<game>/broken-<unix ms>.sav`: an unreadable in-game save, set aside (E04);
//! - `<save_dir>/cores.json`: the last version seen of each core ([`CoreVersions`]), to tell
//!   which quick saves an older core made («Potrebbe non caricarsi»).
//!
//! Saves written before 2026-11-20 were named by the ROM's file stem, flat in `<save_dir>`:
//! [`crate::migration`] moves them once.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::core::GameFiles;
use crate::session::AUTO_SLOT;
use crate::{write_atomic, GameKey};

/// Why a backup was made (the board's «Avvio del gioco», «Prima dell’importazione», …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupReason {
    /// The game started: the save as it was before this session rewrote it.
    Launch,
    /// Before a save from another emulator replaced it.
    Import,
    /// Before an older backup replaced it.
    Restore,
}

impl BackupReason {
    const ALL: [BackupReason; 3] = [Self::Launch, Self::Import, Self::Restore];

    fn tag(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Import => "import",
            Self::Restore => "restore",
        }
    }
}

/// Size and last change of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStamp {
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// One quick save that exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSave {
    /// [`AUTO_SLOT`] for the auto quick save.
    pub slot: u8,
    pub file: FileStamp,
    pub thumb: Option<PathBuf>,
    /// Made by another version of its core than the last one seen: it might not load
    /// ([`is_outdated`]).
    pub outdated: bool,
}

// ---- What wrote a quick save («Potrebbe non caricarsi») ----

/// What wrote a quick save, kept beside it (`slotN.json`). A core's states often do not
/// load in another version of the same core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotMeta {
    /// The core's `library_name` («mGBA»).
    pub core: String,
    /// The core's `library_version`.
    pub core_version: String,
    /// EmuDome's version.
    pub app_version: String,
}

/// The last version seen of each core, by `library_name`, written at every game start.
pub type CoreVersions = BTreeMap<String, String>;

const CORE_VERSIONS: &str = "cores.json";

/// Whether a quick save might not load: its core has been seen since in another version.
/// No record (a save from before EmuDome kept them) or a core never seen: not outdated, so an
/// old save never raises a false alarm.
pub fn is_outdated(meta: Option<&SlotMeta>, seen: &CoreVersions) -> bool {
    meta.and_then(|m| seen.get(&m.core).map(|v| *v != m.core_version))
        .unwrap_or(false)
}

/// `seen` with `core` at `version`; `None` when it already was (nothing to write).
pub fn with_core_version(seen: &CoreVersions, core: &str, version: &str) -> Option<CoreVersions> {
    if seen.get(core).is_some_and(|v| v == version) {
        return None;
    }
    let mut next = seen.clone();
    next.insert(core.to_owned(), version.to_owned());
    Some(next)
}

/// The core versions seen so far (empty when none, or unreadable).
pub fn core_versions(save_dir: &Path) -> CoreVersions {
    std::fs::read(save_dir.join(CORE_VERSIONS))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Records that `core` runs at `version` now (at a game start).
pub fn note_core_version(save_dir: &Path, core: &str, version: &str) -> io::Result<()> {
    let Some(next) = with_core_version(&core_versions(save_dir), core, version) else {
        return Ok(());
    };
    let bytes = serde_json::to_vec_pretty(&next).map_err(io::Error::other)?;
    std::fs::create_dir_all(save_dir)?;
    write_atomic(&save_dir.join(CORE_VERSIONS), &bytes)
}

/// What wrote slot `slot`, if recorded.
pub fn slot_meta(files: &GameFiles, slot: u8) -> Option<SlotMeta> {
    std::fs::read(files.meta(slot))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
}

/// Records what wrote slot `slot`.
pub fn write_slot_meta(files: &GameFiles, slot: u8, meta: &SlotMeta) -> io::Result<()> {
    let bytes = serde_json::to_vec(meta).map_err(io::Error::other)?;
    write_atomic(&files.meta(slot), &bytes)
}

/// One backup of the in-game save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    /// File name inside the backups folder: what [`restore`] takes.
    pub name: String,
    /// Unix ms of when it was made.
    pub at_ms: u64,
    pub reason: BackupReason,
    pub size: u64,
}

/// Everything saved for one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSaves {
    pub in_game: Option<FileStamp>,
    /// The quick saves that exist, auto quick save first, then by slot.
    pub quick: Vec<QuickSave>,
    /// Newest first.
    pub backups: Vec<Backup>,
    /// File names of in-game saves set aside as unreadable (E04), newest first.
    pub broken: Vec<String>,
}

impl GameSaves {
    pub fn is_empty(&self) -> bool {
        self.in_game.is_none() && self.quick.is_empty()
    }

    /// In-game save plus quick saves and their thumbnails, in bytes (backups not counted).
    pub fn bytes(&self) -> u64 {
        let thumbs: u64 = self
            .quick
            .iter()
            .filter_map(|q| q.thumb.as_ref())
            .filter_map(|t| std::fs::metadata(t).ok())
            .map(|m| m.len())
            .sum();
        self.in_game.as_ref().map_or(0, |f| f.size)
            + self.quick.iter().map(|q| q.file.size).sum::<u64>()
            + thumbs
    }

    /// The newest change among the in-game save and the quick saves.
    pub fn last_change(&self) -> Option<SystemTime> {
        self.in_game
            .iter()
            .chain(self.quick.iter().map(|q| &q.file))
            .filter_map(|f| f.modified)
            .max()
    }
}

fn stamp(path: &Path) -> Option<FileStamp> {
    let meta = std::fs::metadata(path).ok().filter(|m| m.is_file())?;
    Some(FileStamp {
        size: meta.len(),
        modified: meta.modified().ok(),
    })
}

fn backups_dir(files: &GameFiles) -> PathBuf {
    files.dir().join("backups")
}

/// The backup a file name stands for (`<unix ms>.<reason>.sav`).
fn parse_backup(name: &str) -> Option<(u64, BackupReason)> {
    let rest = name.strip_suffix(".sav")?;
    let (ms, tag) = rest.split_once('.')?;
    if ms.is_empty() || !ms.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let reason = BackupReason::ALL.into_iter().find(|r| r.tag() == tag)?;
    Some((ms.parse().ok()?, reason))
}

/// The time of an in-game save set aside (`broken-<unix ms>.sav`).
fn parse_broken(name: &str) -> Option<u64> {
    let ms = name.strip_prefix("broken-")?.strip_suffix(".sav")?;
    if ms.is_empty() || !ms.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    ms.parse().ok()
}

fn file_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// The backups of a game, newest first.
pub fn backups(save_dir: &Path, game: &GameKey) -> Vec<Backup> {
    let files = GameFiles::new(save_dir, game);
    let dir = backups_dir(&files);
    let mut list: Vec<Backup> = file_names(&dir)
        .into_iter()
        .filter_map(|name| {
            let (at_ms, reason) = parse_backup(&name)?;
            let size = std::fs::metadata(dir.join(&name)).ok()?.len();
            Some(Backup {
                name,
                at_ms,
                reason,
                size,
            })
        })
        .collect();
    list.sort_by(|a, b| b.at_ms.cmp(&a.at_ms).then_with(|| b.name.cmp(&a.name)));
    list
}

/// What is saved for a game: in-game save, quick saves among the auto one and `1..=slots`,
/// backups, saves set aside.
pub fn game_saves(save_dir: &Path, game: &GameKey, slots: u8) -> GameSaves {
    let files = GameFiles::new(save_dir, game);
    let seen = core_versions(save_dir);
    let quick = std::iter::once(AUTO_SLOT)
        .chain(1..=slots)
        .filter_map(|slot| {
            let file = stamp(&files.slot(slot))?;
            let thumb = Some(files.thumb(slot)).filter(|t| t.is_file());
            let outdated = is_outdated(slot_meta(&files, slot).as_ref(), &seen);
            Some(QuickSave {
                slot,
                file,
                thumb,
                outdated,
            })
        })
        .collect();
    let mut broken: Vec<(u64, String)> = file_names(files.dir())
        .into_iter()
        .filter_map(|name| Some((parse_broken(&name)?, name)))
        .collect();
    broken.sort_by_key(|b| std::cmp::Reverse(b.0));
    GameSaves {
        in_game: stamp(&files.sram),
        quick,
        backups: backups(save_dir, game),
        broken: broken.into_iter().map(|(_, n)| n).collect(),
    }
}

/// What a quick save slot held before a save replaced it, so «Salvato nello slot N · Annulla»
/// can put it back: its state, thumbnail and record, `None` where the file did not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContent {
    pub state: Option<Vec<u8>>,
    pub thumb: Option<Vec<u8>>,
    pub meta: Option<Vec<u8>>,
}

/// A file's bytes, `None` when it does not exist.
fn read_if_any(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Writes `bytes` to `path`, or removes the file when there are none.
fn put_back(path: &Path, bytes: Option<&[u8]>) -> io::Result<()> {
    match bytes {
        Some(b) => write_atomic(path, b),
        None => match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        },
    }
}

impl SlotContent {
    /// What slot `slot` holds now.
    pub fn read(files: &GameFiles, slot: u8) -> io::Result<Self> {
        Ok(Self {
            state: read_if_any(&files.slot(slot))?,
            thumb: read_if_any(&files.thumb(slot))?,
            meta: read_if_any(&files.meta(slot))?,
        })
    }

    /// Puts this content back into slot `slot`: a file that did not exist is removed.
    pub fn restore(&self, files: &GameFiles, slot: u8) -> io::Result<()> {
        put_back(&files.slot(slot), self.state.as_deref())?;
        put_back(&files.thumb(slot), self.thumb.as_deref())?;
        put_back(&files.meta(slot), self.meta.as_deref())
    }
}

/// Copies the in-game save into the backups (when there is one and it differs from the newest
/// backup), then keeps only the `keep` newest. `None` when nothing was copied; `keep` = 0
/// (`saves.backups` «0 = none») copies nothing and removes the old ones.
pub fn backup(
    save_dir: &Path,
    game: &GameKey,
    reason: BackupReason,
    now_ms: u64,
    keep: usize,
) -> io::Result<Option<Backup>> {
    if keep == 0 {
        rotate(save_dir, game, 0);
        return Ok(None);
    }
    let files = GameFiles::new(save_dir, game);
    let data = match std::fs::read(&files.sram) {
        Ok(d) => d,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let dir = backups_dir(&files);
    let existing = backups(save_dir, game);
    // The same bytes as the newest backup: opening a game ten times without playing does not
    // push the older, different backups out.
    let same = existing
        .first()
        .and_then(|b| std::fs::read(dir.join(&b.name)).ok())
        .is_some_and(|newest| newest == data);
    let made = if same {
        None
    } else {
        std::fs::create_dir_all(&dir)?;
        // Never overwrite: two backups in the same millisecond get the next one.
        let mut at_ms = now_ms;
        while existing.iter().any(|b| b.at_ms == at_ms) {
            at_ms += 1;
        }
        let name = format!("{at_ms}.{}.sav", reason.tag());
        write_atomic(&dir.join(&name), &data)?;
        Some(Backup {
            name,
            at_ms,
            reason,
            size: data.len() as u64,
        })
    };
    rotate(save_dir, game, keep);
    Ok(made)
}

/// Removes the backups past the `keep` newest (`saves.backups`; 0 removes them all).
pub fn rotate(save_dir: &Path, game: &GameKey, keep: usize) {
    let files = GameFiles::new(save_dir, game);
    let dir = backups_dir(&files);
    for old in backups(save_dir, game).iter().skip(keep) {
        if let Err(e) = std::fs::remove_file(dir.join(&old.name)) {
            log::warn!("old backup {} not removed: {e}", old.name);
        }
    }
}

/// Puts a backup back as the in-game save; the save it replaces becomes a backup first.
pub fn restore(
    save_dir: &Path,
    game: &GameKey,
    name: &str,
    now_ms: u64,
    keep: usize,
) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    if parse_backup(name).is_none() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name} is not a backup of this game"),
        ));
    }
    let data = std::fs::read(backups_dir(&files).join(name))?;
    // `data` is already read: the rotation may remove the backup it came from.
    backup(save_dir, game, BackupReason::Restore, now_ms, keep + 1)?;
    write_atomic(&files.sram, &data)?;
    rotate(save_dir, game, keep);
    Ok(())
}

/// Replaces the in-game save with imported data; the current one becomes a backup first.
pub fn import(
    save_dir: &Path,
    game: &GameKey,
    data: &[u8],
    now_ms: u64,
    keep: usize,
) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    std::fs::create_dir_all(files.dir())?;
    backup(save_dir, game, BackupReason::Import, now_ms, keep)?;
    write_atomic(&files.sram, data)
}

/// Deletes a quick save, its thumbnail and its record.
pub fn delete_quick(save_dir: &Path, game: &GameKey, slot: u8) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    std::fs::remove_file(files.slot(slot))?;
    for (what, path) in [
        ("thumbnail", files.thumb(slot)),
        ("record", files.meta(slot)),
    ] {
        match std::fs::remove_file(path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                log::warn!("{what} of slot {slot} not removed: {e}")
            }
            Ok(()) | Err(_) => {}
        }
    }
    Ok(())
}

// ---- E04: an in-game save that cannot be used ----

/// Largest in-game save accepted (the biggest NDS flash is 8 MiB; 3DS saves are folders).
const MAX_SAVE_BYTES: u64 = 16 * 1024 * 1024;

/// The state of the in-game save before a game opens.
#[derive(Debug, PartialEq, Eq)]
pub enum SaveHealth {
    Missing,
    Fine,
    /// Unreadable, empty, or far too large to be a save: the core would start from garbage.
    Broken(String),
}

/// Checks the in-game save. The core reads it without telling whether it fits, so this only
/// sees what is certainly wrong: a file that cannot be read, an empty one (an interrupted
/// write), one larger than any cartridge save. A wrong but plausible size is not caught.
pub fn check_in_game(save_dir: &Path, game: &GameKey) -> SaveHealth {
    let files = GameFiles::new(save_dir, game);
    let meta = match std::fs::metadata(&files.sram) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return SaveHealth::Missing,
        Err(e) => return SaveHealth::Broken(e.to_string()),
    };
    if !meta.is_file() {
        return SaveHealth::Broken("not a file".into());
    }
    if meta.len() == 0 {
        return SaveHealth::Broken("empty file".into());
    }
    if meta.len() > MAX_SAVE_BYTES {
        return SaveHealth::Broken(format!("{} bytes", meta.len()));
    }
    // Readable at all (a file locked or with bad sectors fails here).
    match std::fs::File::open(&files.sram) {
        Ok(_) => SaveHealth::Fine,
        Err(e) => SaveHealth::Broken(e.to_string()),
    }
}

/// Moves the in-game save aside as `broken-<ms>.sav`, in the game's folder; returns that file
/// name.
pub fn set_aside(save_dir: &Path, game: &GameKey, now_ms: u64) -> io::Result<String> {
    let files = GameFiles::new(save_dir, game);
    let name = format!("broken-{now_ms}.sav");
    std::fs::rename(&files.sram, files.dir().join(&name))?;
    Ok(name)
}

// ---- Import (V05) ----

/// Where an imported file comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportFormat {
    /// A plain `.sav` (mGBA, VBA, no$gba raw, melonDS, a flash cart).
    Raw,
    /// DeSmuME `.dsv`: the save followed by a 122-byte footer.
    Desmume,
    /// Action Replay DS `.duc`, version 1 (`ARDS000000000001`, 500-byte header).
    ActionReplay,
}

/// Why a file cannot be imported. Technical, for `detail` parameters.
#[derive(Debug, PartialEq, Eq)]
pub enum ImportError {
    Empty,
    TooLarge(u64),
    /// The extension says DeSmuME but the footer is not there.
    NotDesmume,
    /// The extension says Action Replay but the header is not the known one.
    UnknownActionReplay,
    /// The extension is not a save format EmuDome reads.
    UnknownExtension(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty file"),
            Self::TooLarge(n) => write!(f, "{n} bytes, larger than any save"),
            Self::NotDesmume => write!(f, "no DeSmuME footer"),
            Self::UnknownActionReplay => {
                write!(
                    f,
                    "not an Action Replay DS file of version 1 (ARDS000000000001)"
                )
            }
            Self::UnknownExtension(ext) => write!(f, "unknown extension .{ext}"),
        }
    }
}

/// DeSmuME's footer (desmume/src/mc.cpp, `BackupDevice::ensure` and `readFooter`): this text
/// (82 bytes), six little-endian u32 (data size, padded size, type, address size, memory size,
/// version 0), then the 16-byte cookie.
const DSV_TEXT: &[u8] =
    b"|<--Snip above here to create a raw sav by excluding this DeSmuME savedata footer:";
const DSV_COOKIE: &[u8] = b"|-DESMUME SAVE-|";
const DSV_FOOTER: usize = 82 + 24 + 16;

/// Action Replay DS `.duc` version 1 (DeSmuME's `import_duc`): this id, the save at byte 500.
/// Version 2 files (an id at 0xA1) are not taken: their size field is not documented clearly.
const DUC_ID: &[u8] = b"ARDS000000000001";
const DUC_HEADER: usize = 500;

/// The raw save inside an imported file, chosen by its extension.
pub fn parse_import(bytes: &[u8], file_name: &str) -> Result<(ImportFormat, Vec<u8>), ImportError> {
    let ext = Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if bytes.is_empty() {
        return Err(ImportError::Empty);
    }
    if bytes.len() as u64 > MAX_SAVE_BYTES + DUC_HEADER as u64 {
        return Err(ImportError::TooLarge(bytes.len() as u64));
    }
    let (format, data) = match ext.as_str() {
        "sav" | "srm" => {
            // A `.sav` renamed from a `.dsv` still carries the footer: strip it too.
            match strip_dsv(bytes) {
                Some(d) => (ImportFormat::Desmume, d),
                None => (ImportFormat::Raw, bytes),
            }
        }
        "dsv" => (
            ImportFormat::Desmume,
            strip_dsv(bytes).ok_or(ImportError::NotDesmume)?,
        ),
        "duc" => {
            if bytes.len() <= DUC_HEADER || !bytes.starts_with(DUC_ID) {
                return Err(ImportError::UnknownActionReplay);
            }
            (ImportFormat::ActionReplay, &bytes[DUC_HEADER..])
        }
        other => return Err(ImportError::UnknownExtension(other.to_owned())),
    };
    if data.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok((format, data.to_vec()))
}

fn strip_dsv(bytes: &[u8]) -> Option<&[u8]> {
    let data_len = bytes.len().checked_sub(DSV_FOOTER)?;
    let footer = &bytes[data_len..];
    (footer.starts_with(DSV_TEXT) && footer.ends_with(DSV_COOKIE)).then_some(&bytes[..data_len])
}

// ---- Import from a folder (first run S06: DeSmuME, another folder or emulator) ----

/// Which files a folder import reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FolderImport {
    /// DeSmuME's `.dsv` files only.
    Desmume,
    /// Any save EmuDome imports: `.sav`, `.srm`, `.dsv`, `.duc`.
    Any,
}

impl FolderImport {
    fn takes(self, ext: &str) -> bool {
        match self {
            Self::Desmume => ext == "dsv",
            Self::Any => matches!(ext, "sav" | "srm" | "dsv" | "duc"),
        }
    }
}

/// A save file found in a folder that [`parse_import`] accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundSave {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
}

/// A found save and the game (ROM path) it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveMatch {
    pub file: PathBuf,
    pub game: PathBuf,
}

fn lower_ext(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn lower_stem(path: &Path) -> String {
    path.file_stem()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// The saves directly inside `dir` (not in subfolders) that `what` reads and that can be
/// imported, sorted by path. A folder that cannot be read gives none.
pub fn find_saves(dir: &Path, what: FolderImport) -> Vec<FoundSave> {
    let mut found: Vec<FoundSave> = std::fs::read_dir(dir)
        .inspect_err(|e| log::warn!("save folder {}: {e}", dir.display()))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && what.takes(&lower_ext(path)))
        .filter(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            std::fs::read(path)
                .ok()
                .is_some_and(|bytes| parse_import(&bytes, &name).is_ok())
        })
        .map(|path| FoundSave {
            modified: std::fs::metadata(&path).and_then(|m| m.modified()).ok(),
            path,
        })
        .collect();
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}

/// Pairs found saves with games by file name, as emulators name them (`Gioco.nds` ↔
/// `Gioco.dsv`), ignoring case. `.dsv` and `.duc` are Nintendo DS formats: only `.nds` games
/// take them. A game gets one save: the newest one when several match. Ordered by game.
pub fn match_saves(found: &[FoundSave], games: &[PathBuf]) -> Vec<SaveMatch> {
    let mut matches: Vec<SaveMatch> = games
        .iter()
        .filter_map(|game| {
            let stem = lower_stem(game);
            let nds = lower_ext(game) == "nds";
            found
                .iter()
                .filter(|f| lower_stem(&f.path) == stem)
                .filter(|f| nds || !matches!(lower_ext(&f.path).as_str(), "dsv" | "duc"))
                .fold(None::<&FoundSave>, |best, f| match best {
                    Some(b) if b.modified >= f.modified => Some(b),
                    Some(_) | None => Some(f),
                })
                .map(|f| SaveMatch {
                    file: f.path.clone(),
                    game: game.clone(),
                })
        })
        .collect();
    matches.sort_by(|a, b| a.game.cmp(&b.game));
    matches.dedup_by(|a, b| a.game == b.game);
    matches
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        backup, backups, check_in_game, core_versions, delete_quick, find_saves, game_saves,
        import, is_outdated, match_saves, note_core_version, parse_backup, parse_import, restore,
        set_aside, slot_meta, with_core_version, write_slot_meta, BackupReason, CoreVersions,
        FolderImport, FoundSave, ImportError, ImportFormat, SaveHealth, SaveMatch, SlotContent,
        SlotMeta, DSV_COOKIE, DSV_FOOTER, DSV_TEXT, DUC_HEADER, DUC_ID,
    };
    use crate::core::GameFiles;
    use crate::settings::test_dir::TestDir;
    use crate::GameKey;

    /// The game the tests save for, and another one.
    fn lanterna() -> GameKey {
        GameKey::parse("gba:1a").unwrap()
    }

    fn write(dir: &TestDir, rel: &str, bytes: &[u8]) {
        let path = dir.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn the_footer_is_122_bytes() {
        assert_eq!(DSV_TEXT.len(), 82);
        assert_eq!(DSV_FOOTER, 122);
    }

    #[test]
    fn backup_names_are_read_back() {
        assert_eq!(
            parse_backup("1700000000000.launch.sav"),
            Some((1_700_000_000_000, BackupReason::Launch))
        );
        assert_eq!(parse_backup("2.x.123.launch.sav"), None);
        assert_eq!(parse_backup("123.other.sav"), None);
        assert_eq!(parse_backup("Lanterna.123.import.sav"), None);
    }

    #[test]
    fn lists_what_a_game_has() {
        let dir = TestDir::new("saves-list");
        let save_dir = dir.path("saves");
        write(&dir, "saves/gba/1a/game.sav", &[1; 32]);
        write(&dir, "saves/gba/1a/auto.state", &[0; 100]);
        write(&dir, "saves/gba/1a/slot2.state", &[0; 100]);
        write(&dir, "saves/gba/1a/slot2.png", &[0; 10]);
        write(&dir, "saves/gba/1a/slot9.state", &[0; 100]);
        write(&dir, "saves/gba/2b/game.sav", &[1; 32]);
        let saves = game_saves(&save_dir, &lanterna(), 5);
        assert_eq!(saves.in_game.as_ref().map(|f| f.size), Some(32));
        // Slot 9 is past `saves.slots`.
        let slots: Vec<u8> = saves.quick.iter().map(|q| q.slot).collect();
        assert_eq!(slots, vec![0, 2]);
        assert!(saves.quick[1].thumb.is_some() && saves.quick[0].thumb.is_none());
        assert_eq!(saves.bytes(), 32 + 200 + 10);
        assert!(saves.last_change().is_some());
        assert!(game_saves(&save_dir, &GameKey::parse("gba:ee").unwrap(), 5).is_empty());
    }

    #[test]
    fn two_games_never_share_a_file() {
        // `Tetris.gb` and `Tetris.gba`: same file name, two contents, two keys.
        let dir = TestDir::new("saves-two-games");
        let save_dir = dir.path("saves");
        let gb = GameKey::parse("gb:7e").unwrap();
        let gba = GameKey::parse("gba:7e").unwrap();
        assert_ne!(
            GameFiles::new(&save_dir, &gb).sram,
            GameFiles::new(&save_dir, &gba).sram
        );
        import(&save_dir, &gb, b"gb save", 1, 5).unwrap();
        assert!(game_saves(&save_dir, &gba, 5).is_empty());
        assert_eq!(
            game_saves(&save_dir, &gb, 5).in_game.map(|f| f.size),
            Some(7)
        );
    }

    #[test]
    fn backups_skip_duplicates_and_rotate() {
        let dir = TestDir::new("saves-backup");
        let save_dir = dir.path("saves");
        let game = &lanterna();
        assert_eq!(
            backup(&save_dir, game, BackupReason::Launch, 1, 3).unwrap(),
            None
        );
        write(&dir, "saves/gba/1a/game.sav", b"v1");
        assert!(backup(&save_dir, game, BackupReason::Launch, 10, 3)
            .unwrap()
            .is_some());
        // Same bytes: no new backup.
        assert!(backup(&save_dir, game, BackupReason::Launch, 20, 3)
            .unwrap()
            .is_none());
        for (i, v) in [b"v2", b"v3", b"v4"].iter().enumerate() {
            write(&dir, "saves/gba/1a/game.sav", *v);
            backup(&save_dir, game, BackupReason::Launch, 30 + i as u64, 3).unwrap();
        }
        let list = backups(&save_dir, game);
        let times: Vec<u64> = list.iter().map(|b| b.at_ms).collect();
        assert_eq!(times, vec![32, 31, 30]);
        // `saves.backups` = 0: no new backup, and the old ones go.
        write(&dir, "saves/gba/1a/game.sav", b"v5");
        assert!(backup(&save_dir, game, BackupReason::Launch, 40, 0)
            .unwrap()
            .is_none());
        assert!(backups(&save_dir, game).is_empty());
    }

    #[test]
    fn restoring_keeps_the_replaced_save() {
        let dir = TestDir::new("saves-restore");
        let save_dir = dir.path("saves");
        let game = &lanterna();
        write(&dir, "saves/gba/1a/game.sav", b"old");
        let old = backup(&save_dir, game, BackupReason::Launch, 10, 5)
            .unwrap()
            .unwrap();
        write(&dir, "saves/gba/1a/game.sav", b"new");
        restore(&save_dir, game, &old.name, 20, 5).unwrap();
        assert_eq!(
            std::fs::read(dir.path("saves/gba/1a/game.sav")).unwrap(),
            b"old"
        );
        let list = backups(&save_dir, game);
        assert_eq!(list[0].reason, BackupReason::Restore);
        assert_eq!(
            std::fs::read(dir.path(&format!("saves/gba/1a/backups/{}", list[0].name))).unwrap(),
            b"new"
        );
        assert!(restore(&save_dir, game, "nope.sav", 30, 5).is_err());
        // keep = 1: the restored backup is the one that remains, not the one just restored from.
        restore(&save_dir, game, &old.name, 40, 1).unwrap();
        assert_eq!(backups(&save_dir, game).len(), 1);
    }

    #[test]
    fn import_backs_up_first() {
        let dir = TestDir::new("saves-import");
        let save_dir = dir.path("saves");
        let game = &lanterna();
        import(&save_dir, game, b"fresh", 5, 5).unwrap();
        assert!(backups(&save_dir, game).is_empty());
        import(&save_dir, game, b"second", 6, 5).unwrap();
        let list = backups(&save_dir, game);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].reason, BackupReason::Import);
        assert_eq!(
            std::fs::read(dir.path("saves/gba/1a/game.sav")).unwrap(),
            b"second"
        );
    }

    #[test]
    fn a_slot_put_back_after_a_save_is_as_it_was() {
        let dir = TestDir::new("saves-slot-undo");
        let files = GameFiles::new(&dir.path("saves"), &lanterna());
        write(&dir, "saves/gba/1a/slot2.state", b"old state");
        write(&dir, "saves/gba/1a/slot2.png", b"old thumb");
        let before = SlotContent::read(&files, 2).unwrap();
        // A save replaces both files...
        write(&dir, "saves/gba/1a/slot2.state", b"new state");
        write(&dir, "saves/gba/1a/slot2.png", b"new thumb");
        before.restore(&files, 2).unwrap();
        assert_eq!(std::fs::read(files.slot(2)).unwrap(), b"old state");
        assert_eq!(std::fs::read(files.thumb(2)).unwrap(), b"old thumb");
    }

    #[test]
    fn a_slot_that_was_empty_is_emptied_again() {
        let dir = TestDir::new("saves-slot-undo-empty");
        let files = GameFiles::new(&dir.path("saves"), &lanterna());
        let before = SlotContent::read(&files, 3).unwrap();
        assert_eq!(
            before,
            SlotContent {
                state: None,
                thumb: None,
                meta: None
            }
        );
        write(&dir, "saves/gba/1a/slot3.state", b"new state");
        write(&dir, "saves/gba/1a/slot3.png", b"new thumb");
        before.restore(&files, 3).unwrap();
        assert!(!files.slot(3).exists());
        assert!(!files.thumb(3).exists());
        // Putting back an empty slot twice is fine.
        before.restore(&files, 3).unwrap();
    }

    #[test]
    fn quick_saves_are_deleted_with_their_thumbnail() {
        let dir = TestDir::new("saves-delete");
        let save_dir = dir.path("saves");
        write(&dir, "saves/gba/1a/slot1.state", b"s");
        write(&dir, "saves/gba/1a/slot1.png", b"p");
        write(&dir, "saves/gba/1a/slot1.json", b"{}");
        delete_quick(&save_dir, &lanterna(), 1).unwrap();
        assert!(!dir.path("saves/gba/1a/slot1.state").exists());
        assert!(!dir.path("saves/gba/1a/slot1.png").exists());
        assert!(!dir.path("saves/gba/1a/slot1.json").exists());
        assert!(delete_quick(&save_dir, &lanterna(), 1).is_err());
    }

    fn meta(core: &str, version: &str) -> SlotMeta {
        SlotMeta {
            core: core.into(),
            core_version: version.into(),
            app_version: "0.1.0".into(),
        }
    }

    #[test]
    fn a_save_is_outdated_only_when_its_core_was_seen_in_another_version() {
        let seen: CoreVersions = [("mGBA".to_owned(), "0.11.0".to_owned())].into();
        assert!(!is_outdated(Some(&meta("mGBA", "0.11.0")), &seen));
        assert!(is_outdated(Some(&meta("mGBA", "0.10.2")), &seen));
        // No record (made before EmuDome kept them), or a core never seen: no false alarm.
        assert!(!is_outdated(None, &seen));
        assert!(!is_outdated(Some(&meta("melonDS DS", "1.0")), &seen));
        assert!(!is_outdated(
            Some(&meta("mGBA", "0.10.2")),
            &CoreVersions::new()
        ));
    }

    #[test]
    fn a_core_version_is_written_only_when_it_changes() {
        let seen: CoreVersions = [("mGBA".to_owned(), "0.10.2".to_owned())].into();
        assert_eq!(with_core_version(&seen, "mGBA", "0.10.2"), None);
        let next = with_core_version(&seen, "mGBA", "0.11.0");
        assert_eq!(
            next.as_ref()
                .and_then(|n| n.get("mGBA"))
                .map(String::as_str),
            Some("0.11.0")
        );
        let more = with_core_version(&seen, "Azahar", "2125");
        assert_eq!(more.map(|m| m.len()), Some(2));
    }

    #[test]
    fn the_saves_list_marks_the_slots_an_older_core_made() {
        let dir = TestDir::new("saves-outdated");
        let save_dir = dir.path("saves");
        let files = GameFiles::new(&save_dir, &lanterna());
        write(&dir, "saves/gba/1a/slot1.state", b"old");
        write(&dir, "saves/gba/1a/slot2.state", b"new");
        write(&dir, "saves/gba/1a/slot3.state", b"no record");
        write_slot_meta(&files, 1, &meta("mGBA", "0.10.2")).unwrap();
        write_slot_meta(&files, 2, &meta("mGBA", "0.11.0")).unwrap();
        // Before any core version is known: nothing is outdated.
        let outdated = |dir: &std::path::Path| -> Vec<(u8, bool)> {
            game_saves(dir, &lanterna(), 5)
                .quick
                .iter()
                .map(|q| (q.slot, q.outdated))
                .collect()
        };
        assert_eq!(outdated(&save_dir), [(1, false), (2, false), (3, false)]);
        note_core_version(&save_dir, "mGBA", "0.11.0").unwrap();
        assert_eq!(
            core_versions(&save_dir).get("mGBA").map(String::as_str),
            Some("0.11.0")
        );
        assert_eq!(outdated(&save_dir), [(1, true), (2, false), (3, false)]);
        assert_eq!(slot_meta(&files, 2), Some(meta("mGBA", "0.11.0")));
    }

    #[test]
    fn undoing_a_save_puts_its_record_back_too() {
        let dir = TestDir::new("saves-slot-undo-meta");
        let files = GameFiles::new(&dir.path("saves"), &lanterna());
        write(&dir, "saves/gba/1a/slot4.state", b"old state");
        write_slot_meta(&files, 4, &meta("mGBA", "0.10.2")).unwrap();
        let before = SlotContent::read(&files, 4).unwrap();
        write(&dir, "saves/gba/1a/slot4.state", b"new state");
        write_slot_meta(&files, 4, &meta("mGBA", "0.11.0")).unwrap();
        before.restore(&files, 4).unwrap();
        assert_eq!(slot_meta(&files, 4), Some(meta("mGBA", "0.10.2")));
    }

    #[test]
    fn a_broken_save_is_set_aside() {
        let dir = TestDir::new("saves-broken");
        let save_dir = dir.path("saves");
        let game = &lanterna();
        assert_eq!(check_in_game(&save_dir, game), SaveHealth::Missing);
        write(&dir, "saves/gba/1a/game.sav", b"");
        assert!(matches!(
            check_in_game(&save_dir, game),
            SaveHealth::Broken(_)
        ));
        let name = set_aside(&save_dir, game, 77).unwrap();
        assert_eq!(name, "broken-77.sav");
        assert!(dir.path("saves/gba/1a/broken-77.sav").is_file());
        assert_eq!(check_in_game(&save_dir, game), SaveHealth::Missing);
        assert_eq!(game_saves(&save_dir, game, 3).broken, vec![name]);
        write(&dir, "saves/gba/1a/game.sav", &[0xFF; 64]);
        assert_eq!(check_in_game(&save_dir, game), SaveHealth::Fine);
    }

    fn dsv(data: &[u8]) -> Vec<u8> {
        let mut file = data.to_vec();
        file.extend_from_slice(DSV_TEXT);
        for v in [data.len() as u32, data.len() as u32, 2, 2, 512, 0] {
            file.extend_from_slice(&v.to_le_bytes());
        }
        file.extend_from_slice(DSV_COOKIE);
        file
    }

    #[test]
    fn imports_raw_desmume_and_action_replay() {
        let save = vec![0xAB; 512];
        assert_eq!(
            parse_import(&save, "game.sav").unwrap(),
            (ImportFormat::Raw, save.clone())
        );
        assert_eq!(
            parse_import(&dsv(&save), "game.dsv").unwrap(),
            (ImportFormat::Desmume, save.clone())
        );
        // A .dsv renamed to .sav.
        assert_eq!(
            parse_import(&dsv(&save), "game.sav").unwrap(),
            (ImportFormat::Desmume, save.clone())
        );
        assert_eq!(
            parse_import(&save, "game.dsv"),
            Err(ImportError::NotDesmume)
        );
        let mut duc = DUC_ID.to_vec();
        duc.resize(DUC_HEADER, 0);
        duc.extend_from_slice(&save);
        assert_eq!(
            parse_import(&duc, "GAME.DUC").unwrap(),
            (ImportFormat::ActionReplay, save.clone())
        );
        assert_eq!(
            parse_import(&save, "game.duc"),
            Err(ImportError::UnknownActionReplay)
        );
        assert_eq!(parse_import(&[], "game.sav"), Err(ImportError::Empty));
        assert!(matches!(
            parse_import(&save, "game.zip"),
            Err(ImportError::UnknownExtension(_))
        ));
    }

    fn found(path: &str, secs: u64) -> FoundSave {
        FoundSave {
            path: PathBuf::from(path),
            modified: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs)),
        }
    }

    #[test]
    fn saves_match_games_by_file_name() {
        let found = [
            found("/d/Battery/Lanterna.dsv", 10),
            found("/d/Battery/ALTRO GIOCO.dsv", 10),
            found("/d/Battery/Nessuno.dsv", 10),
        ];
        let games = [
            PathBuf::from("/g/lanterna.nds"),
            PathBuf::from("/g/Altro gioco.nds"),
            PathBuf::from("/g/Senza.nds"),
        ];
        assert_eq!(
            match_saves(&found, &games),
            vec![
                SaveMatch {
                    file: PathBuf::from("/d/Battery/ALTRO GIOCO.dsv"),
                    game: PathBuf::from("/g/Altro gioco.nds"),
                },
                SaveMatch {
                    file: PathBuf::from("/d/Battery/Lanterna.dsv"),
                    game: PathBuf::from("/g/lanterna.nds"),
                },
            ]
        );
    }

    #[test]
    fn ds_formats_go_to_ds_games_only_and_the_newest_wins() {
        let found = [
            found("/d/Lanterna.dsv", 10),
            found("/d/Lanterna.sav", 20),
            found("/d/Volo.dsv", 30),
        ];
        let games = [
            PathBuf::from("/g/Lanterna.nds"),
            PathBuf::from("/g/Volo.gba"),
        ];
        assert_eq!(
            match_saves(&found, &games),
            vec![SaveMatch {
                file: PathBuf::from("/d/Lanterna.sav"),
                game: PathBuf::from("/g/Lanterna.nds"),
            }]
        );
    }

    #[test]
    fn a_folder_offers_only_the_saves_it_can_import() {
        let dir = TestDir::new("find-saves");
        let save = vec![0xAB; 512];
        write(&dir, "Battery/Lanterna.dsv", &dsv(&save));
        write(&dir, "Battery/Rotto.dsv", &save);
        write(&dir, "Battery/Volo.sav", &save);
        write(&dir, "Battery/note.txt", b"hello");
        write(&dir, "Battery/sub/Dentro.dsv", &dsv(&save));
        let names = |what| -> Vec<String> {
            find_saves(&dir.path("Battery"), what)
                .iter()
                .filter_map(|f| f.path.file_name().map(|n| n.to_string_lossy().into_owned()))
                .collect()
        };
        assert_eq!(names(FolderImport::Desmume), ["Lanterna.dsv"]);
        assert_eq!(names(FolderImport::Any), ["Lanterna.dsv", "Volo.sav"]);
        assert!(find_saves(&dir.path("missing"), FolderImport::Any).is_empty());
    }
}
