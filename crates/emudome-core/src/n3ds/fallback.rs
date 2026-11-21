//! Whether Azahar wrote into its own user folder during a session (Trello #97).
//!
//! Azahar keeps a game's emulated NAND and SD card under the save directory EmuDome gives it
//! (the game's folder). When it cannot create `<that folder>/Azahar/`, it silently uses its own
//! default user folder instead (`emudome_platform::azahar_user_dirs`), shared by every game: the
//! session's progress lands there. The session refuses to start when the game's folder cannot be
//! made, but a folder that exists and still cannot be written (permissions, a path the core
//! cannot use) is only seen afterwards: the shell looks at Azahar's own folders before the session
//! ([`FallbackWatch::before`]) and after it ([`FallbackWatch::written`]).
//!
//! Read only: the folders are walked for their entries' modification times and count, never
//! created, written or deleted. Symbolic links and junctions are not followed.
//!
//! Blind spots: another program writing there during the session (the standalone Azahar app uses
//! the same folder) looks the same; a write that keeps an entry's modification time and the
//! number of entries is not seen; past [`MAX_ENTRIES`] entries only the first ones are compared.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// At most this many entries are read under one folder, so a large tree (a standalone Azahar's
/// shader cache) costs a bounded time.
pub const MAX_ENTRIES: usize = 20_000;

/// What a folder looked like: its newest modification time (the folder's own and every entry's
/// under it) and how many entries it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FolderState {
    pub newest: Option<SystemTime>,
    pub entries: usize,
}

/// Reads what `dir` looks like now; `None` when nothing is there.
pub fn probe(dir: &Path) -> Option<FolderState> {
    let meta = std::fs::symlink_metadata(dir).ok()?;
    let mut state = FolderState {
        newest: meta.modified().ok(),
        entries: 0,
    };
    if !meta.is_dir() {
        return Some(state);
    }
    let mut pending = vec![dir.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let Ok(read) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in read.flatten() {
            if state.entries >= MAX_ENTRIES {
                return Some(state);
            }
            state.entries += 1;
            // `DirEntry::metadata` does not follow a link: a link or junction counts as one entry.
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            state.newest = state.newest.max(meta.modified().ok());
            if meta.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Some(state)
}

/// Whether something was written between `before` and `after`: the folder appeared, an entry is
/// newer, or entries were added or removed. A folder that is gone was not written to.
pub fn written(before: Option<FolderState>, after: Option<FolderState>) -> bool {
    match (before, after) {
        (_, None) => false,
        (None, Some(_)) => true,
        (Some(b), Some(a)) => a.entries != b.entries || a.newest > b.newest,
    }
}

/// Azahar's own folders as they were before a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FallbackWatch {
    folders: Vec<(PathBuf, Option<FolderState>)>,
}

impl FallbackWatch {
    /// Looks at `folders` (`emudome_platform::azahar_user_dirs`) before the session starts.
    pub fn before(folders: Vec<PathBuf>) -> Self {
        Self {
            folders: folders
                .into_iter()
                .map(|f| {
                    let state = probe(&f);
                    (f, state)
                })
                .collect(),
        }
    }

    /// After the session: the first folder that was written to, if any.
    pub fn written(&self) -> Option<&Path> {
        self.folders
            .iter()
            .find(|(folder, before)| written(*before, probe(folder)))
            .map(|(folder, _)| folder.as_path())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{probe, written, FallbackWatch, FolderState};
    use crate::settings::test_dir::TestDir;

    fn at(secs: u64, entries: usize) -> Option<FolderState> {
        Some(FolderState {
            newest: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs)),
            entries,
        })
    }

    #[test]
    fn the_decision() {
        // Created during the session.
        assert!(written(None, at(10, 3)));
        // Untouched, or never there.
        assert!(!written(at(10, 3), at(10, 3)));
        assert!(!written(None, None));
        // Removed by someone else: nothing written there.
        assert!(!written(at(10, 3), None));
        // A newer entry, or entries added or removed.
        assert!(written(at(10, 3), at(11, 3)));
        assert!(written(at(10, 3), at(10, 4)));
        assert!(written(at(10, 3), at(10, 2)));
        // An older time with the same entries (a clock moved back): not a write.
        assert!(!written(at(10, 3), at(9, 3)));
    }

    #[test]
    fn a_folder_created_during_the_session_is_seen() {
        let dir = TestDir::new("fallback-created");
        let azahar = dir.path("Azahar");
        let watch = FallbackWatch::before(vec![dir.path("user"), azahar.clone()]);
        assert_eq!(watch.written(), None);
        dir.write("Azahar/sdmc/game/options.ini", "x");
        assert_eq!(watch.written(), Some(azahar.as_path()));
    }

    #[test]
    fn a_deep_file_rewritten_in_an_existing_folder_is_seen() {
        let dir = TestDir::new("fallback-rewritten");
        dir.write("Azahar/sdmc/game/options.ini", "x");
        let file = dir.path("Azahar/sdmc/game/options.ini");
        // The file older than its folders: only its own time can be the newest after the write.
        std::fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(SystemTime::now() - Duration::from_secs(3600))
            .unwrap();
        assert_eq!(probe(&dir.path("Azahar")).unwrap().entries, 3);
        let watch = FallbackWatch::before(vec![dir.path("Azahar")]);
        assert_eq!(watch.written(), None);
        // Same entries, the file written again deep down: the folders' own times do not move.
        std::thread::sleep(Duration::from_millis(50));
        std::fs::write(&file, "y").unwrap();
        assert_eq!(watch.written(), Some(dir.path("Azahar").as_path()));
    }
}
