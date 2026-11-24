//! The one way user data reaches the disk (engineering rule 1): saves, states, settings, the
//! library and their side files are all written by [`write_atomic`].
//!
//! A temporary file is created next to the target, written, flushed to the disk (`fsync`), then
//! renamed over the target (a replace on every OS); the folder is flushed too where the system
//! allows it. An interruption at any point leaves either the old file or the new one, never half
//! of one.
//!
//! The temporary file is named `.<file name>.<pid>-<n>.emudome-tmp`: hidden on Linux and
//! Android, it never ends in `.sav`, `.state`, `.json` or `.png`, so nothing that lists saves or
//! settings reads it. Temporary files of this pattern left by an interrupted write are removed by
//! the next write of the same target once they are older than [`STALE_AFTER`]; nothing else is
//! ever removed.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// The extension of the helper's temporary files.
pub(crate) const TMP_SUFFIX: &str = ".emudome-tmp";

/// A temporary file older than this is a leftover of an interrupted write, never one in flight.
pub const STALE_AFTER: Duration = Duration::from_secs(60 * 60);

/// Writes `bytes` to `path` atomically and durably (see the module comment). Creates the parent
/// folder if it is missing.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_with(path, bytes, |_| Ok(()))
}

/// [`write_atomic`] with a step run between the flushed temporary file and the rename: tests use
/// it to fail there, as a crash or a full disk would.
fn write_with(
    path: &Path,
    bytes: &[u8],
    before_rename: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let dir = parent_dir(path);
    fs::create_dir_all(&dir)?;
    let name = file_name(path)?;
    let tmp = dir.join(tmp_name(&name));
    let written = write_tmp(&tmp, bytes)
        .and_then(|()| before_rename(&tmp))
        .and_then(|()| fs::rename(&tmp, path));
    if let Err(e) = written {
        // The target was not touched; the temporary file goes.
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    sync_dir(&dir);
    remove_stale(&dir, &name);
    Ok(())
}

fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        Some(_) | None => PathBuf::from("."),
    }
}

fn file_name(path: &Path) -> io::Result<OsString> {
    path.file_name().map(ToOwned::to_owned).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("no file name in {}", path.display()),
        )
    })
}

/// `.<name>.` : what every temporary file of the target `name` starts with.
fn tmp_prefix(name: &OsString) -> OsString {
    let mut prefix = OsString::from(".");
    prefix.push(name);
    prefix.push(".");
    prefix
}

fn tmp_name(name: &OsString) -> OsString {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let mut tmp = tmp_prefix(name);
    tmp.push(format!(
        "{}-{}{TMP_SUFFIX}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    tmp
}

fn write_tmp(tmp: &Path, bytes: &[u8]) -> io::Result<()> {
    // `create_new`: never reuse a file that is there, whoever made it.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Flushes the folder's entries (the rename) where the system can: a folder opens as a file on
/// Linux and Android; on Windows the open or the flush is refused, and NTFS journals the rename.
pub(crate) fn sync_dir(dir: &Path) {
    if let Ok(d) = fs::File::open(dir) {
        let _ = d.sync_all();
    }
}

/// Removes the temporary files of the target `name` left by interrupted writes: only this helper's
/// pattern, only older than [`STALE_AFTER`]. Failures are only logged: the write itself succeeded.
fn remove_stale(dir: &Path, name: &OsString) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let prefix = tmp_prefix(name);
    for entry in entries.flatten() {
        if is_tmp_of(&entry.file_name(), &prefix) && is_stale(&entry) {
            if let Err(e) = fs::remove_file(entry.path()) {
                log::warn!(
                    "stale temporary file {} not removed: {e}",
                    entry.path().display()
                );
            }
        }
    }
}

fn is_tmp_of(file: &OsString, prefix: &OsString) -> bool {
    let (Some(file), Some(prefix)) = (file.to_str(), prefix.to_str()) else {
        return false;
    };
    file.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(TMP_SUFFIX))
        .is_some_and(|id| {
            id.split_once('-')
                .is_some_and(|(pid, n)| pid.parse::<u32>().is_ok() && n.parse::<u64>().is_ok())
        })
}

/// Opens `path` for reading. The stores read their file through here, so a test can make a file
/// unreadable without the OS's permissions ([`deny_reads`]).
pub(crate) fn open_for_read(path: &Path) -> io::Result<fs::File> {
    #[cfg(test)]
    if denied::is_denied(path) {
        return Err(io::Error::from(io::ErrorKind::PermissionDenied));
    }
    fs::File::open(path)
}

/// Reads the whole of `path` as text, through [`open_for_read`].
pub(crate) fn read_text(path: &Path) -> io::Result<String> {
    let mut text = String::new();
    open_for_read(path)?.read_to_string(&mut text)?;
    Ok(text)
}

/// Before a store writes its copy in memory over `path` (engineering rule 1): `Ok` when the file
/// can still be read, or when there is none (a missing file is created); the error otherwise.
///
/// A store trusts the copy it read earlier while the file's modification time and size stay the
/// same, or always; a permission taken away (`chmod 000`), a lock held by another program or a
/// folder put in its place changes neither, and the copy would replace a file that cannot be
/// read. So the file is opened and its first byte read (an open alone succeeds on a folder on
/// Linux and Android).
pub fn check_readable(path: &Path) -> io::Result<()> {
    let probed = open_for_read(path).and_then(|mut file| file.read(&mut [0_u8; 1]));
    match probed {
        Ok(_) => Ok(()),
        // No file, or a file in place of one of its folders: no file there either.
        Err(e)
            if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::NotADirectory =>
        {
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// While the returned guard lives, every read of `path` on this thread through
/// [`open_for_read`] fails with `PermissionDenied`: a file made unreadable after a store read it,
/// the same on every OS (the OS gate keeps permission and lock helpers out of this crate).
#[cfg(test)]
pub(crate) fn deny_reads(path: &Path) -> denied::Guard {
    denied::deny(path)
}

#[cfg(test)]
pub(crate) mod denied {
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    thread_local! {
        static DENIED: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn is_denied(path: &Path) -> bool {
        DENIED.with(|d| d.borrow().iter().any(|p| p == path))
    }

    pub(crate) fn deny(path: &Path) -> Guard {
        DENIED.with(|d| d.borrow_mut().push(path.to_path_buf()));
        Guard(path.to_path_buf())
    }

    /// Gives the reads back when dropped.
    pub(crate) struct Guard(PathBuf);

    impl Drop for Guard {
        fn drop(&mut self) {
            DENIED.with(|d| d.borrow_mut().retain(|p| p != &self.0));
        }
    }
}

fn is_stale(entry: &fs::DirEntry) -> bool {
    entry
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age >= STALE_AFTER)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::io;
    use std::time::{Duration, SystemTime};

    use super::{is_tmp_of, tmp_name, tmp_prefix, write_atomic, write_with, STALE_AFTER};
    use crate::settings::test_dir::TestDir;

    fn files_in(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn check_readable_refuses_only_a_file_that_is_there_and_cannot_be_read() {
        let dir = TestDir::new("atomic-check-readable");
        // Missing, or a file in place of a folder on its path: nothing to lose, writes create it.
        assert!(super::check_readable(&dir.path("missing.json")).is_ok());
        dir.write("plain", "x");
        assert!(super::check_readable(&dir.path("plain/below.json")).is_ok());
        // Readable, empty included.
        dir.write("there.json", "{}");
        assert!(super::check_readable(&dir.path("there.json")).is_ok());
        dir.write("empty.json", "");
        assert!(super::check_readable(&dir.path("empty.json")).is_ok());
        // A folder in its place, on every OS.
        dir.write("folder.json/inside.txt", "kept");
        assert!(super::check_readable(&dir.path("folder.json")).is_err());
        // Made unreadable (the test seam for a permission or a lock).
        let path = dir.path("there.json");
        {
            let _denied = super::deny_reads(&path);
            let e = super::check_readable(&path).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::PermissionDenied);
        }
        assert!(super::check_readable(&path).is_ok());
    }

    #[test]
    fn replaces_the_file_and_leaves_no_temporary_file() {
        let dir = TestDir::new("atomic-replace");
        let path = dir.path("a/b/Game.sav");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(files_in(&dir.path("a/b")), vec!["Game.sav".to_owned()]);
    }

    #[test]
    fn a_write_interrupted_before_the_rename_leaves_the_old_file_whole() {
        let dir = TestDir::new("atomic-interrupted");
        let path = dir.path("Game.sav");
        write_atomic(&path, b"the old save").unwrap();
        let result = write_with(&path, b"the new save", |tmp| {
            // The new bytes are on disk in the temporary file, the target still has the old ones.
            assert_eq!(fs::read(tmp).unwrap(), b"the new save");
            Err(io::Error::other("power cut"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"the old save");
        assert_eq!(files_in(&dir.path("")), vec!["Game.sav".to_owned()]);
    }

    #[test]
    fn a_failed_first_write_creates_nothing() {
        let dir = TestDir::new("atomic-first-fails");
        let path = dir.path("Game.auto.state");
        let result = write_with(&path, b"state", |_| Err(io::Error::other("disk full")));
        assert!(result.is_err());
        assert!(files_in(&dir.path("")).is_empty());
    }

    #[test]
    fn temporary_names_never_look_like_user_files() {
        let name = OsString::from("Game.sav");
        let tmp = tmp_name(&name).to_string_lossy().into_owned();
        assert!(tmp.starts_with(".Game.sav."), "{tmp}");
        assert!(tmp.ends_with(".emudome-tmp"), "{tmp}");
        for ext in [".sav", ".state", ".json", ".png"] {
            assert!(!tmp.ends_with(ext), "{tmp}");
        }
        assert_ne!(tmp_name(&name), tmp_name(&name), "two writes, two names");
    }

    #[test]
    fn only_this_helpers_own_pattern_counts_as_a_temporary_file() {
        let prefix = tmp_prefix(&OsString::from("Game.sav"));
        let yes = |s: &str| is_tmp_of(&OsString::from(s), &prefix);
        assert!(yes(".Game.sav.123-4.emudome-tmp"));
        assert!(!yes("Game.sav"));
        assert!(!yes("Game.sav.tmp"));
        assert!(!yes(".Game.sav.x-4.emudome-tmp"));
        assert!(!yes(".Game.sav.123-4.emudome-tmp.sav"));
        assert!(!yes(".Other.sav.123-4.emudome-tmp"));
        assert!(!yes(".Game.sav.slot1.123-4.emudome-tmp"));
    }

    #[test]
    fn stale_leftovers_of_the_same_target_are_removed_and_nothing_else() {
        let dir = TestDir::new("atomic-stale");
        let old = SystemTime::now() - STALE_AFTER - Duration::from_secs(60);
        let make = |name: &str, modified: SystemTime| {
            let path = dir.path(name);
            fs::write(&path, b"x").unwrap();
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(modified)
                .unwrap();
        };
        make(".Game.sav.1-1.emudome-tmp", old);
        make(".Game.sav.1-2.emudome-tmp", SystemTime::now());
        make(".Other.sav.1-1.emudome-tmp", old);
        make("Game.sav.tmp", old);
        make("Game.slot1.state", old);
        write_atomic(&dir.path("Game.sav"), b"save").unwrap();
        assert_eq!(
            files_in(&dir.path("")),
            vec![
                ".Game.sav.1-2.emudome-tmp".to_owned(),
                ".Other.sav.1-1.emudome-tmp".to_owned(),
                "Game.sav".to_owned(),
                "Game.sav.tmp".to_owned(),
                "Game.slot1.state".to_owned(),
            ]
        );
    }
}
