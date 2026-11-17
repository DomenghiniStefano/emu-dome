//! Walking the library folders: every ROM file found is identified (from the hash cache when
//! its size and modification time did not change). Runs without the library: it takes the
//! cache and gives back a [`ScanOutcome`] that [`super::Library::apply_scan`] merges, so the
//! shell can scan on a background thread without holding the library's lock.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::identity::{self, Identity};
use super::store::{CacheEntry, HashCache};

/// What to scan.
pub struct ScanInput {
    /// `library.folders`, walked recursively.
    pub folders: Vec<PathBuf>,
    /// Single files (added by drag and drop), checked one by one.
    pub files: Vec<PathBuf>,
    pub cache: HashCache,
}

/// A game file found.
#[derive(Clone, Debug)]
pub struct Found {
    pub path: String,
    /// The library folder it is in, `None` for a single file.
    pub folder: Option<String>,
    pub size: u64,
    pub mtime: u64,
    pub identity: Identity,
}

/// What a scan saw.
pub struct ScanOutcome {
    pub found: Vec<Found>,
    /// Every folder of the input (reachable or not).
    pub folders: Vec<String>,
    /// Folders that could not be read.
    pub unreachable: Vec<String>,
    /// The cache to keep: the old one minus the files gone from reachable folders, plus the
    /// files identified now.
    pub cache: HashCache,
}

/// Walks the folders and files. `progress(checked, found)` is called after every ROM file.
pub fn scan(input: ScanInput, progress: &mut dyn FnMut(usize, usize)) -> ScanOutcome {
    let mut walker = Walker {
        old: input.cache,
        cache: HashCache::new(),
        found: Vec::new(),
        checked: 0,
        progress,
    };
    let mut unreachable = Vec::new();
    let mut reachable = Vec::new();
    for folder in &input.folders {
        let name = folder.to_string_lossy().into_owned();
        if fs::read_dir(folder).is_err() {
            unreachable.push(name);
            continue;
        }
        walker.walk(folder, &name);
        reachable.push(folder.clone());
    }
    for file in &input.files {
        walker.file(file, None);
    }
    // Cache entries of reachable folders and single files not seen now are gone; the others
    // (unreachable folders) are kept for when they come back.
    let Walker {
        old,
        mut cache,
        found,
        ..
    } = walker;
    for (path, entry) in old {
        let p = Path::new(&path);
        let checked_now = reachable.iter().any(|f| p.starts_with(f))
            || input.files.iter().any(|f| f.as_path() == p);
        if !checked_now {
            cache.entry(path).or_insert(entry);
        }
    }
    ScanOutcome {
        found,
        folders: input
            .folders
            .iter()
            .map(|f| f.to_string_lossy().into_owned())
            .collect(),
        unreachable,
        cache,
    }
}

struct Walker<'a> {
    old: HashCache,
    cache: HashCache,
    found: Vec<Found>,
    checked: usize,
    progress: &'a mut dyn FnMut(usize, usize),
}

impl Walker<'_> {
    fn walk(&mut self, dir: &Path, root: &str) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("library scan: {}: {e}", dir.display());
                return;
            }
        };
        let mut paths: Vec<(PathBuf, bool)> = entries
            .filter_map(Result::ok)
            .filter_map(|e| {
                // The entry's own type: symbolic links to folders are not followed (no loops).
                let kind = e.file_type().ok()?;
                Some((e.path(), kind.is_dir()))
            })
            .collect();
        paths.sort();
        for (path, is_dir) in paths {
            if is_dir {
                self.walk(&path, root);
            } else {
                self.file(&path, Some(root));
            }
        }
    }

    fn file(&mut self, path: &Path, folder: Option<&str>) {
        if !identity::is_rom(path) {
            return;
        }
        let Ok(meta) = fs::metadata(path) else {
            return;
        };
        if !meta.is_file() {
            return;
        }
        self.checked += 1;
        let key = path.to_string_lossy().into_owned();
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|d| u64::try_from(d.as_millis()).ok())
            .unwrap_or_default();
        let cached = self
            .old
            .get(&key)
            .filter(|c| c.size == size && c.mtime == mtime)
            .map(|c| c.identity.clone());
        let identity = match cached {
            Some(identity) => Some(identity),
            None => match identity::identify(path) {
                Ok(identity) => Some(identity),
                Err(e) => {
                    log::info!("library scan: {} skipped: {e}", path.display());
                    None
                }
            },
        };
        if let Some(identity) = identity {
            self.cache.insert(
                key.clone(),
                CacheEntry {
                    size,
                    mtime,
                    identity: identity.clone(),
                },
            );
            self.found.push(Found {
                path: key,
                folder: folder.map(str::to_owned),
                size,
                mtime,
                identity,
            });
        }
        (self.progress)(self.checked, self.found.len());
    }
}
