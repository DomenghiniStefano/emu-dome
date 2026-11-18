//! The saves of a game on disk (Saves section, boards V01–V05, E03, E04): what exists, the
//! backups of the in-game save and their rotation, restoring one, importing a save from another
//! emulator, setting a broken save aside. Pure file work, no core: the shell calls it while the
//! game is closed (the open session owns the `.sav` and rewrites it).
//!
//! Files, all keyed by the ROM file name (TODO(game identity, Trello #21): by content later):
//! - `<save_dir>/<stem>.sav`: the in-game save («Salvataggio del gioco»);
//! - `<save_dir>/<stem>.slotN.state` / `.auto.state` (+ `.png`): quick saves ([`GameFiles`]);
//! - `<save_dir>/backups/<stem>.<unix ms>.<reason>.sav`: backups of the in-game save;
//! - `<save_dir>/<stem>.broken-<unix ms>.sav`: an unreadable in-game save, set aside (E04).

use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::core::GameFiles;
use crate::session::AUTO_SLOT;

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
    files.save_dir().join("backups")
}

/// The backup a file name stands for, when it belongs to this game.
fn parse_backup(stem: &str, name: &str) -> Option<(u64, BackupReason)> {
    let rest = name.strip_prefix(stem)?.strip_prefix('.')?;
    let rest = rest.strip_suffix(".sav")?;
    let (ms, tag) = rest.split_once('.')?;
    if ms.is_empty() || !ms.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let reason = BackupReason::ALL.into_iter().find(|r| r.tag() == tag)?;
    Some((ms.parse().ok()?, reason))
}

fn parse_broken(stem: &str, name: &str) -> Option<u64> {
    let ms = name
        .strip_prefix(stem)?
        .strip_prefix(".broken-")?
        .strip_suffix(".sav")?;
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
pub fn backups(save_dir: &Path, game: &Path) -> Vec<Backup> {
    let files = GameFiles::new(save_dir, game);
    let dir = backups_dir(&files);
    let mut list: Vec<Backup> = file_names(&dir)
        .into_iter()
        .filter_map(|name| {
            let (at_ms, reason) = parse_backup(files.stem(), &name)?;
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
pub fn game_saves(save_dir: &Path, game: &Path, slots: u8) -> GameSaves {
    let files = GameFiles::new(save_dir, game);
    let quick = std::iter::once(AUTO_SLOT)
        .chain(1..=slots)
        .filter_map(|slot| {
            let file = stamp(&files.slot(slot))?;
            let thumb = Some(files.thumb(slot)).filter(|t| t.is_file());
            Some(QuickSave { slot, file, thumb })
        })
        .collect();
    let mut broken: Vec<(u64, String)> = file_names(save_dir)
        .into_iter()
        .filter_map(|name| Some((parse_broken(files.stem(), &name)?, name)))
        .collect();
    broken.sort_by_key(|b| std::cmp::Reverse(b.0));
    GameSaves {
        in_game: stamp(&files.sram),
        quick,
        backups: backups(save_dir, game),
        broken: broken.into_iter().map(|(_, n)| n).collect(),
    }
}

/// Writes through a temporary file and a rename: an interrupted write leaves the old file whole
/// instead of a truncated one (the usual cause of E04).
pub fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// What a quick save slot held before a save replaced it, so «Salvato nello slot N · Annulla»
/// can put it back: its state and its thumbnail, `None` where the file did not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContent {
    pub state: Option<Vec<u8>>,
    pub thumb: Option<Vec<u8>>,
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
        })
    }

    /// Puts this content back into slot `slot`: a file that did not exist is removed.
    pub fn restore(&self, files: &GameFiles, slot: u8) -> io::Result<()> {
        put_back(&files.slot(slot), self.state.as_deref())?;
        put_back(&files.thumb(slot), self.thumb.as_deref())
    }
}

/// Copies the in-game save into the backups (when there is one and it differs from the newest
/// backup), then keeps only the `keep` newest. `None` when nothing was copied; `keep` = 0
/// (`saves.backups` «0 = none») copies nothing and removes the old ones.
pub fn backup(
    save_dir: &Path,
    game: &Path,
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
        let name = format!("{}.{at_ms}.{}.sav", files.stem(), reason.tag());
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
pub fn rotate(save_dir: &Path, game: &Path, keep: usize) {
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
    game: &Path,
    name: &str,
    now_ms: u64,
    keep: usize,
) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    if parse_backup(files.stem(), name).is_none() {
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
    game: &Path,
    data: &[u8],
    now_ms: u64,
    keep: usize,
) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    std::fs::create_dir_all(save_dir)?;
    backup(save_dir, game, BackupReason::Import, now_ms, keep)?;
    write_atomic(&files.sram, data)
}

/// Deletes a quick save and its thumbnail.
pub fn delete_quick(save_dir: &Path, game: &Path, slot: u8) -> io::Result<()> {
    let files = GameFiles::new(save_dir, game);
    std::fs::remove_file(files.slot(slot))?;
    match std::fs::remove_file(files.thumb(slot)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => {
            log::warn!("thumbnail of slot {slot} not removed: {e}")
        }
        Ok(()) | Err(_) => {}
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
pub fn check_in_game(save_dir: &Path, game: &Path) -> SaveHealth {
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

/// Moves the in-game save aside as `<stem>.broken-<ms>.sav`; returns that file name.
pub fn set_aside(save_dir: &Path, game: &Path, now_ms: u64) -> io::Result<String> {
    let files = GameFiles::new(save_dir, game);
    let name = format!("{}.broken-{now_ms}.sav", files.stem());
    std::fs::rename(&files.sram, save_dir.join(&name))?;
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        backup, backups, check_in_game, delete_quick, game_saves, import, parse_backup,
        parse_import, restore, set_aside, BackupReason, ImportError, ImportFormat, SaveHealth,
        SlotContent, DSV_COOKIE, DSV_FOOTER, DSV_TEXT, DUC_HEADER, DUC_ID,
    };
    use crate::core::GameFiles;
    use crate::settings::test_dir::TestDir;

    const GAME: &str = "games/Lanterna.gba";

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
    fn backup_names_belong_to_their_game_only() {
        assert_eq!(
            parse_backup("Lanterna", "Lanterna.1700000000000.launch.sav"),
            Some((1_700_000_000_000, BackupReason::Launch))
        );
        assert_eq!(
            parse_backup("Lanterna", "Lanterna.2.x.123.launch.sav"),
            None
        );
        assert_eq!(parse_backup("Lanterna", "Lanterna.123.other.sav"), None);
        assert_eq!(parse_backup("Lanterna", "Lanterna 2.123.import.sav"), None);
    }

    #[test]
    fn lists_what_a_game_has() {
        let dir = TestDir::new("saves-list");
        let save_dir = dir.path("saves");
        write(&dir, "saves/Lanterna.sav", &[1; 32]);
        write(&dir, "saves/Lanterna.auto.state", &[0; 100]);
        write(&dir, "saves/Lanterna.slot2.state", &[0; 100]);
        write(&dir, "saves/Lanterna.slot2.png", &[0; 10]);
        write(&dir, "saves/Lanterna.slot9.state", &[0; 100]);
        write(&dir, "saves/Other.sav", &[1; 32]);
        let saves = game_saves(&save_dir, Path::new(GAME), 5);
        assert_eq!(saves.in_game.as_ref().map(|f| f.size), Some(32));
        // Slot 9 is past `saves.slots`.
        let slots: Vec<u8> = saves.quick.iter().map(|q| q.slot).collect();
        assert_eq!(slots, vec![0, 2]);
        assert!(saves.quick[1].thumb.is_some() && saves.quick[0].thumb.is_none());
        assert_eq!(saves.bytes(), 32 + 200 + 10);
        assert!(saves.last_change().is_some());
        assert!(game_saves(&save_dir, Path::new("games/None.gba"), 5).is_empty());
    }

    #[test]
    fn backups_skip_duplicates_and_rotate() {
        let dir = TestDir::new("saves-backup");
        let save_dir = dir.path("saves");
        let game = Path::new(GAME);
        assert_eq!(
            backup(&save_dir, game, BackupReason::Launch, 1, 3).unwrap(),
            None
        );
        write(&dir, "saves/Lanterna.sav", b"v1");
        assert!(backup(&save_dir, game, BackupReason::Launch, 10, 3)
            .unwrap()
            .is_some());
        // Same bytes: no new backup.
        assert!(backup(&save_dir, game, BackupReason::Launch, 20, 3)
            .unwrap()
            .is_none());
        for (i, v) in [b"v2", b"v3", b"v4"].iter().enumerate() {
            write(&dir, "saves/Lanterna.sav", *v);
            backup(&save_dir, game, BackupReason::Launch, 30 + i as u64, 3).unwrap();
        }
        let list = backups(&save_dir, game);
        let times: Vec<u64> = list.iter().map(|b| b.at_ms).collect();
        assert_eq!(times, vec![32, 31, 30]);
        // `saves.backups` = 0: no new backup, and the old ones go.
        write(&dir, "saves/Lanterna.sav", b"v5");
        assert!(backup(&save_dir, game, BackupReason::Launch, 40, 0)
            .unwrap()
            .is_none());
        assert!(backups(&save_dir, game).is_empty());
    }

    #[test]
    fn restoring_keeps_the_replaced_save() {
        let dir = TestDir::new("saves-restore");
        let save_dir = dir.path("saves");
        let game = Path::new(GAME);
        write(&dir, "saves/Lanterna.sav", b"old");
        let old = backup(&save_dir, game, BackupReason::Launch, 10, 5)
            .unwrap()
            .unwrap();
        write(&dir, "saves/Lanterna.sav", b"new");
        restore(&save_dir, game, &old.name, 20, 5).unwrap();
        assert_eq!(
            std::fs::read(dir.path("saves/Lanterna.sav")).unwrap(),
            b"old"
        );
        let list = backups(&save_dir, game);
        assert_eq!(list[0].reason, BackupReason::Restore);
        assert_eq!(
            std::fs::read(dir.path(&format!("saves/backups/{}", list[0].name))).unwrap(),
            b"new"
        );
        assert!(restore(&save_dir, game, "Other.10.launch.sav", 30, 5).is_err());
        // keep = 1: the restored backup is the one that remains, not the one just restored from.
        restore(&save_dir, game, &old.name, 40, 1).unwrap();
        assert_eq!(backups(&save_dir, game).len(), 1);
    }

    #[test]
    fn import_backs_up_first() {
        let dir = TestDir::new("saves-import");
        let save_dir = dir.path("saves");
        let game = Path::new(GAME);
        import(&save_dir, game, b"fresh", 5, 5).unwrap();
        assert!(backups(&save_dir, game).is_empty());
        import(&save_dir, game, b"second", 6, 5).unwrap();
        let list = backups(&save_dir, game);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].reason, BackupReason::Import);
        assert_eq!(
            std::fs::read(dir.path("saves/Lanterna.sav")).unwrap(),
            b"second"
        );
    }

    #[test]
    fn a_slot_put_back_after_a_save_is_as_it_was() {
        let dir = TestDir::new("saves-slot-undo");
        let files = GameFiles::new(&dir.path("saves"), Path::new(GAME));
        write(&dir, "saves/Lanterna.slot2.state", b"old state");
        write(&dir, "saves/Lanterna.slot2.png", b"old thumb");
        let before = SlotContent::read(&files, 2).unwrap();
        // A save replaces both files...
        write(&dir, "saves/Lanterna.slot2.state", b"new state");
        write(&dir, "saves/Lanterna.slot2.png", b"new thumb");
        before.restore(&files, 2).unwrap();
        assert_eq!(std::fs::read(files.slot(2)).unwrap(), b"old state");
        assert_eq!(std::fs::read(files.thumb(2)).unwrap(), b"old thumb");
    }

    #[test]
    fn a_slot_that_was_empty_is_emptied_again() {
        let dir = TestDir::new("saves-slot-undo-empty");
        let files = GameFiles::new(&dir.path("saves"), Path::new(GAME));
        let before = SlotContent::read(&files, 3).unwrap();
        assert_eq!(
            before,
            SlotContent {
                state: None,
                thumb: None
            }
        );
        write(&dir, "saves/Lanterna.slot3.state", b"new state");
        write(&dir, "saves/Lanterna.slot3.png", b"new thumb");
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
        write(&dir, "saves/Lanterna.slot1.state", b"s");
        write(&dir, "saves/Lanterna.slot1.png", b"p");
        delete_quick(&save_dir, Path::new(GAME), 1).unwrap();
        assert!(!dir.path("saves/Lanterna.slot1.state").exists());
        assert!(!dir.path("saves/Lanterna.slot1.png").exists());
        assert!(delete_quick(&save_dir, Path::new(GAME), 1).is_err());
    }

    #[test]
    fn a_broken_save_is_set_aside() {
        let dir = TestDir::new("saves-broken");
        let save_dir = dir.path("saves");
        let game = Path::new(GAME);
        assert_eq!(check_in_game(&save_dir, game), SaveHealth::Missing);
        write(&dir, "saves/Lanterna.sav", b"");
        assert!(matches!(
            check_in_game(&save_dir, game),
            SaveHealth::Broken(_)
        ));
        let name = set_aside(&save_dir, game, 77).unwrap();
        assert_eq!(name, "Lanterna.broken-77.sav");
        assert_eq!(check_in_game(&save_dir, game), SaveHealth::Missing);
        assert_eq!(game_saves(&save_dir, game, 3).broken, vec![name]);
        write(&dir, "saves/Lanterna.sav", &[0xFF; 64]);
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
}
