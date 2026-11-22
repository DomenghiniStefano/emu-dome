//! Screenshots (L3 + B, F12, quick menu › Strumenti): the game's frame as a PNG at the core's
//! native resolution, in the game's own screenshots folder, keyed by its [`GameKey`] (engineering
//! rule 2, Trello #85): `<saves root>/screenshots/<console>/<hex>/<YYYY-MM-DD_HH-MM-SS>.png`,
//! the time in UTC (no time zone database here; the name only has to sort and stay unique),
//! `-2`, `-3`... when taken within the same second. Outside the game's saves folder on purpose:
//! screenshots are not synced with the saves and never count against a core's path budget.
//!
//! Before #85 they were `<saves root>/screenshots/<ROM file stem> <YYYY-MM-DD HH-MM-SS>.png`;
//! [`legacy_name`] reads those names for the one-time move (`crate::migration::run_screenshots`).
//!
//! The quick menu and the HUD are not in the picture: they are drawn by the webview, the frame
//! is the core's alone (on DS / 3DS both screens as the core stacks them, whatever layout is on
//! screen).

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::GameKey;

/// The screenshots' root, inside the saves root.
pub fn screenshots_dir(saves_root: &Path) -> PathBuf {
    saves_root.join("screenshots")
}

/// The screenshots of one game.
pub fn game_dir(saves_root: &Path, game: &GameKey) -> PathBuf {
    screenshots_dir(saves_root).join(game.dir())
}

/// `YYYY-MM-DD HH-MM-SS` (UTC) of a Unix time in seconds.
pub fn timestamp(unix_secs: u64) -> String {
    let days = unix_secs / 86_400;
    let secs = unix_secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}-{:02}-{:02}",
        secs / 3600,
        secs / 60 % 60,
        secs % 60
    )
}

/// Year, month, day of a count of days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}

/// Writes `png` as a new screenshot into the game's folder `dir` and answers its path. Never
/// replaces a file.
pub fn write(dir: &Path, png: &[u8], now: SystemTime) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let base = timestamp(secs).replace(' ', "_");
    for n in 1u32.. {
        let name = if n == 1 {
            format!("{base}.png")
        } else {
            format!("{base}-{n}.png")
        };
        let path = dir.join(name);
        // `create_new`: a screenshot taken at the same moment by another write is not replaced.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                io::Write::write_all(&mut file, png)?;
                return Ok(path);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && n < 1000 => {}
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("no free screenshot name"))
}

/// Whether `name` can be a screenshot of a game's folder: a plain `.png` file name of ASCII
/// letters, digits, `-`, `_` and `.`, not starting with a dot. For names from the interface or a
/// URL: no separator, drive or `..` gets through.
pub fn is_name(name: &str) -> bool {
    name.len() > 4
        && name.ends_with(".png")
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// A screenshot of a game, as listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shot {
    /// The file name in the game's folder ([`is_name`]).
    pub name: String,
    /// When it was written, Unix ms (its modification time).
    pub taken_ms: u64,
    /// Its size from the PNG header, `None` if the file is not a readable PNG.
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// The size in a PNG's header (`IHDR`), without decoding it.
fn png_size(file: &Path) -> Option<(u32, u32)> {
    let mut head = [0u8; 24];
    std::fs::File::open(file).ok()?.read_exact(&mut head).ok()?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([head[16], head[17], head[18], head[19]]);
    let h = u32::from_be_bytes([head[20], head[21], head[22], head[23]]);
    Some((w, h))
}

/// The screenshots in a game's folder `dir`, newest first; a missing folder has none.
pub fn list(dir: &Path) -> io::Result<Vec<Shot>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut shots: Vec<Shot> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let meta = entry.metadata().ok()?;
            if !meta.is_file() || !is_name(&name) {
                return None;
            }
            let taken_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .and_then(|d| u64::try_from(d.as_millis()).ok())
                .unwrap_or_default();
            let size = png_size(&entry.path());
            Some(Shot {
                name,
                taken_ms,
                width: size.map(|s| s.0),
                height: size.map(|s| s.1),
            })
        })
        .collect();
    shots.sort_by(|a, b| b.taken_ms.cmp(&a.taken_ms).then(b.name.cmp(&a.name)));
    Ok(shots)
}

/// A screenshot of a game's folder by its name ([`is_name`], and a file there); `None` for
/// anything else.
pub fn file(dir: &Path, name: &str) -> Option<PathBuf> {
    let path = is_name(name).then(|| dir.join(name))?;
    path.is_file().then_some(path)
}

/// `true` when `s` is `YYYY-MM-DD HH-MM-SS` as [`timestamp`] writes it.
fn is_old_timestamp(s: &str) -> bool {
    s.len() == 19
        && s.bytes().enumerate().all(|(i, b)| match i {
            4 | 7 | 13 | 16 => b == b'-',
            10 => b == b' ',
            _ => b.is_ascii_digit(),
        })
}

/// An old screenshot name (`<ROM file stem> <YYYY-MM-DD HH-MM-SS>[-N].png`, before #85): the stem
/// and its name in the game's folder.
pub fn legacy_name(name: &str) -> Option<(String, String)> {
    let base = name.strip_suffix(".png")?;
    let split = |base: &str| -> Option<(String, String)> {
        let at = base.len().checked_sub(19)?;
        let (head, time) = (base.get(..at)?, base.get(at..)?);
        let stem = head.strip_suffix(' ')?;
        (is_old_timestamp(time) && !stem.is_empty())
            .then(|| (stem.to_owned(), time.replace(' ', "_")))
    };
    if let Some((rest, n)) = base.rsplit_once('-') {
        let numbered = !n.is_empty() && n.len() <= 4 && n.bytes().all(|b| b.is_ascii_digit());
        if let Some((stem, time)) = split(rest).filter(|_| numbered) {
            return Some((stem, format!("{time}-{n}.png")));
        }
    }
    split(base).map(|(stem, time)| (stem, format!("{time}.png")))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::{file, game_dir, is_name, legacy_name, list, screenshots_dir, timestamp, write};
    use crate::settings::test_dir::TestDir;
    use crate::GameKey;

    #[test]
    fn timestamps_are_utc_dates() {
        assert_eq!(timestamp(0), "1970-01-01 00-00-00");
        // 2000-02-29 (leap day) 12:34:56.
        assert_eq!(timestamp(951_827_696), "2000-02-29 12-34-56");
        // 2026-11-18 23:59:59.
        assert_eq!(timestamp(1_795_046_399), "2026-11-18 23-59-59");
    }

    #[test]
    fn screenshots_go_to_the_games_folder_and_never_replace() {
        let dir = TestDir::new("screenshot");
        let key = GameKey::parse("gba:1a2b").unwrap();
        let shots = game_dir(&dir.path("saves"), &key);
        assert_eq!(
            shots,
            screenshots_dir(&dir.path("saves")).join("gba").join("1a2b")
        );
        let now = UNIX_EPOCH + Duration::from_secs(951_827_696);
        let first = write(&shots, b"one", now).unwrap();
        let second = write(&shots, b"two", now).unwrap();
        assert_eq!(first.parent(), Some(shots.as_path()));
        assert!(first.ends_with("2000-02-29_12-34-56.png"));
        assert!(second.ends_with("2000-02-29_12-34-56-2.png"));
        assert_eq!(std::fs::read(&first).unwrap(), b"one");
        assert_eq!(std::fs::read(&second).unwrap(), b"two");
        // Both listed, their size unknown (not PNGs); the newest name first at the same time.
        let listed = list(&shots).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|s| s.width.is_none()));
        assert_eq!(file(&shots, "2000-02-29_12-34-56.png"), Some(first));
        assert_eq!(file(&shots, "missing.png"), None);
        assert!(list(&dir.path("nothing")).unwrap().is_empty());
    }

    #[test]
    fn the_list_reads_the_png_size_and_skips_other_files() {
        let dir = TestDir::new("screenshot-list");
        let shots = dir.path("shots");
        let png = crate::rgba_png(3, 2, &[0u8; 3 * 2 * 4]).unwrap();
        write(&shots, &png, UNIX_EPOCH).unwrap();
        dir.write("shots/notes.txt", "x");
        dir.write("shots/.hidden.png", "x");
        let listed = list(&shots).unwrap();
        assert_eq!(listed.len(), 1, "{listed:?}");
        assert_eq!(listed[0].name, "1970-01-01_00-00-00.png");
        assert_eq!((listed[0].width, listed[0].height), (Some(3), Some(2)));
    }

    #[test]
    fn names_from_outside_are_plain_png_names_only() {
        assert!(is_name("2026-11-22_10-11-12.png"));
        assert!(is_name("2026-11-22_10-11-12-2.png"));
        for bad in [
            "",
            ".png",
            "../x.png",
            r"..\x.png",
            "C:x.png",
            "/etc/x.png",
            "a b.png",
            "x.jpg",
            ".hidden.png",
        ] {
            assert!(!is_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn old_names_give_the_rom_stem_and_the_new_name() {
        let s = |stem: &str, new: &str| Some((stem.to_owned(), new.to_owned()));
        assert_eq!(
            legacy_name("Lanterna 2026-11-18 23-59-59.png"),
            s("Lanterna", "2026-11-18_23-59-59.png")
        );
        assert_eq!(
            legacy_name("Pokemon v1.2 - Hack 2026-11-18 23-59-59-3.png"),
            s("Pokemon v1.2 - Hack", "2026-11-18_23-59-59-3.png")
        );
        assert_eq!(legacy_name("Lanterna 2026.png"), None);
        assert_eq!(legacy_name(" 2026-11-18 23-59-59.png"), None);
        assert_eq!(legacy_name("2026-11-18 23-59-59.png"), None);
        assert_eq!(legacy_name("Lanterna 2026-11-18 23-59-59.jpg"), None);
    }
}
