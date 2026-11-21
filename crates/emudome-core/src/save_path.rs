//! Whether the files a core creates by itself fit under a game's save folder on this system
//! (Trello #94).
//!
//! Azahar keeps its emulated NAND and SD card in the game's own folder ([`crate::GameFiles`]),
//! several folders deep. When one of its paths is longer than the system allows, it does not
//! report an error: it aborts the whole process inside `retro_load_game`. So the shell measures
//! first, before the core loads: the game's folder plus the deepest path the core creates under
//! it ([`SaveDepth`], measured on the pinned build) against the limits of this process
//! ([`PathLimits`], which `emudome_platform::path_limits` fills in). Pure: no file is touched.

use std::path::{Path, PathBuf};

/// What a path's length is counted in: the system's own unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathUnit {
    /// UTF-16 code units (Windows).
    Utf16,
    /// Bytes of the path as the system stores it (UTF-8 on Linux and Android).
    Byte,
}

/// The longest paths this process can create, without the terminating NUL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathLimits {
    /// The longest folder path (Windows without long paths: 247, `CreateDirectory` keeps room for
    /// an 8.3 file name).
    pub dir: usize,
    /// The longest file path (Windows without long paths: 259, `MAX_PATH` less the NUL).
    pub file: usize,
    pub unit: PathUnit,
}

/// The deepest folder and the deepest file a core creates under its save directory, relative to
/// it, as measured on the pinned build (`/` between the parts; every character is ASCII, so its
/// length is the same in every [`PathUnit`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveDepth {
    pub deepest_dir: &'static str,
    pub deepest_file: &'static str,
}

/// A game's save folder is too long for the core's own files on this system: `path` (the
/// deepest one the core would create) is `length` long, over `limit` (both in the system's
/// [`PathUnit`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavePathTooLong {
    pub save_dir: PathBuf,
    pub path: PathBuf,
    pub length: usize,
    pub limit: usize,
}

impl std::fmt::Display for SavePathTooLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the core would create {} ({} characters, the system allows {})",
            self.path.display(),
            self.length,
            self.limit
        )
    }
}

impl std::error::Error for SavePathTooLong {}

/// The length of `path` in `unit`.
pub fn path_len(path: &Path, unit: PathUnit) -> usize {
    let text = path.as_os_str().to_string_lossy();
    match unit {
        PathUnit::Utf16 => text.encode_utf16().count(),
        PathUnit::Byte => text.len(),
    }
}

/// Checks that the core's deepest folder and file under `save_dir` (the game's own folder, the
/// core's save directory) fit in `limits`. The folder is checked first: the core creates it
/// before the files in it.
pub fn check(
    save_dir: &Path,
    depth: &SaveDepth,
    limits: PathLimits,
) -> Result<(), SavePathTooLong> {
    for (tail, limit) in [
        (depth.deepest_dir, limits.dir),
        (depth.deepest_file, limits.file),
    ] {
        // Part by part: the system's own separator, as in the core's paths.
        let path = tail
            .split('/')
            .fold(save_dir.to_path_buf(), |p, c| p.join(c));
        let length = path_len(&path, limits.unit);
        if length > limit {
            return Err(SavePathTooLong {
                save_dir: save_dir.to_path_buf(),
                path,
                length,
                limit,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{check, path_len, PathLimits, PathUnit, SaveDepth};

    /// Windows without long paths.
    const LEGACY: PathLimits = PathLimits {
        dir: 247,
        file: 259,
        unit: PathUnit::Utf16,
    };

    const DEPTH: SaveDepth = crate::n3ds::SAVE_DEPTH;

    /// A save folder exactly `len` characters long (ASCII).
    fn dir_of(len: usize) -> PathBuf {
        let base = "C:/";
        PathBuf::from(format!("{base}{}", "a".repeat(len - base.len())))
    }

    #[test]
    fn the_measured_azahar_depth_is_the_one_its_lengths_say() {
        // 80 and 93 characters under the game's folder: the measure of 2026-11-21 (Trello #94).
        assert_eq!(DEPTH.deepest_dir.len(), 80);
        assert_eq!(DEPTH.deepest_file.len(), 93);
        assert!(DEPTH.deepest_file.is_ascii() && DEPTH.deepest_dir.is_ascii());
    }

    /// The boundary measured with the headless example on Windows without long paths: a game
    /// folder of 165 characters runs, one of 166 aborts the process (its deepest file would be
    /// 260 characters).
    #[test]
    fn the_guard_refuses_exactly_where_azahar_aborts() {
        assert_eq!(check(&dir_of(165), &DEPTH, LEGACY), Ok(()));
        let err = check(&dir_of(166), &DEPTH, LEGACY).unwrap_err();
        assert_eq!(err.length, 260);
        assert_eq!(err.limit, 259);
        assert!(err.path.ends_with("user/gamecoin.dat"));
        assert_eq!(err.save_dir, dir_of(166));
    }

    #[test]
    fn the_folder_is_checked_before_the_files() {
        let tight_dir = PathLimits {
            dir: 100,
            file: 10_000,
            unit: PathUnit::Utf16,
        };
        let err = check(&dir_of(50), &DEPTH, tight_dir).unwrap_err();
        assert!(err.path.ends_with("F000000B/boss"));
        assert_eq!(err.length, 50 + 1 + 80);
        assert_eq!(err.limit, 100);
    }

    #[test]
    fn long_paths_let_a_deep_folder_through() {
        let long = PathLimits {
            dir: 32_000,
            file: 32_000,
            unit: PathUnit::Utf16,
        };
        assert_eq!(check(&dir_of(1_000), &DEPTH, long), Ok(()));
    }

    /// Windows counts UTF-16 units, Linux bytes: a user name with accents is longer in bytes.
    #[test]
    fn a_path_is_counted_in_the_systems_unit() {
        let p = Path::new("C:/Users/Stéfano");
        assert_eq!(path_len(p, PathUnit::Utf16), 16);
        assert_eq!(path_len(p, PathUnit::Byte), 17);
        // Outside the basic plane: two UTF-16 units, four bytes.
        let p = Path::new("/home/\u{1F600}");
        assert_eq!(path_len(p, PathUnit::Utf16), 8);
        assert_eq!(path_len(p, PathUnit::Byte), 10);
    }
}
