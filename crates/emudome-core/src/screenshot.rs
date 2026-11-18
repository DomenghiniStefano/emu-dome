//! Screenshots (L3 + B, F12, quick menu › Strumenti): the game's frame as a PNG at the core's
//! native resolution, next to the saves:
//! `<save_dir>/screenshots/<stem> <YYYY-MM-DD HH-MM-SS>.png`, the time in UTC (no time zone
//! database here; the name only has to sort and stay unique), `-2`, `-3`... when taken
//! within the same second.
//!
//! The quick menu and the HUD are not in the picture: they are drawn by the webview, the frame
//! is the core's alone.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::GameFiles;

/// The screenshots' folder, inside the save folder.
pub fn screenshots_dir(save_dir: &Path) -> PathBuf {
    save_dir.join("screenshots")
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

/// Writes `png` as a new screenshot of the game and answers its path. Never replaces a file.
pub fn write(files: &GameFiles, png: &[u8], now: SystemTime) -> io::Result<PathBuf> {
    let dir = screenshots_dir(files.save_dir());
    std::fs::create_dir_all(&dir)?;
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let base = format!("{} {}", files.stem(), timestamp(secs));
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{Duration, UNIX_EPOCH};

    use super::{screenshots_dir, timestamp, write};
    use crate::core::GameFiles;
    use crate::settings::test_dir::TestDir;

    #[test]
    fn timestamps_are_utc_dates() {
        assert_eq!(timestamp(0), "1970-01-01 00-00-00");
        // 2000-02-29 (leap day) 12:34:56.
        assert_eq!(timestamp(951_827_696), "2000-02-29 12-34-56");
        // 2026-11-18 23:59:59.
        assert_eq!(timestamp(1_795_046_399), "2026-11-18 23-59-59");
    }

    #[test]
    fn screenshots_go_next_to_the_saves_and_never_replace() {
        let dir = TestDir::new("screenshot");
        let files = GameFiles::new(&dir.path("saves"), Path::new("games/Lanterna.gba"));
        let now = UNIX_EPOCH + Duration::from_secs(951_827_696);
        let first = write(&files, b"one", now).unwrap();
        let second = write(&files, b"two", now).unwrap();
        assert_eq!(
            first.parent(),
            Some(screenshots_dir(&dir.path("saves")).as_path())
        );
        assert!(first.ends_with("Lanterna 2000-02-29 12-34-56.png"));
        assert!(second.ends_with("Lanterna 2000-02-29 12-34-56-2.png"));
        assert_eq!(std::fs::read(first).unwrap(), b"one");
        assert_eq!(std::fs::read(second).unwrap(), b"two");
    }
}
