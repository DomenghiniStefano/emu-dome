//! Walking the library folders: every ROM file found is identified (from the hash cache when
//! its size and modification time did not change). Runs without the library: it takes the
//! cache and gives back a [`ScanOutcome`] that [`super::Library::apply_scan`] merges, so the
//! shell can scan on a background thread without holding the library's lock.
//!
//! Android's library folders are document trees picked through the system (Trello #47): not
//! paths, so they are walked through [`Documents`] (the shell's bridge to the platform), one
//! listing per folder, each document keyed by its URI and identified from the bytes of its
//! descriptor ([`OpenedDocument`]). Same rules as a folder: unknown extensions skipped, the
//! cache by key + size + mtime, an unreadable tree reported unreachable with its games kept.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::identity::{self, Identity};
use super::store::{CacheEntry, HashCache};

/// What to scan.
pub struct ScanInput {
    /// `library.folders` that are paths, walked recursively.
    pub folders: Vec<PathBuf>,
    /// `library.folders` that are picked document trees (Android `content://` tree URIs),
    /// walked through the [`Documents`] given to [`scan_with_documents`].
    pub trees: Vec<String>,
    /// Single files (added by drag and drop), checked one by one.
    pub files: Vec<PathBuf>,
    pub cache: HashCache,
}

/// A game file found.
#[derive(Clone, Debug)]
pub struct Found {
    /// Its path, or its document URI (a tree's document).
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
    /// Every folder of the input (reachable or not), trees included.
    pub folders: Vec<String>,
    /// Folders that could not be read.
    pub unreachable: Vec<String>,
    /// The cache to keep: the old one minus the files gone from reachable folders, plus the
    /// files identified now.
    pub cache: HashCache,
}

/// One child of a folder of a document tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocEntry {
    /// Its document URI: the game's path in the library, its cache key, how it is opened.
    pub uri: String,
    /// Its display name: the extension decides whether it is looked at, the stem its title.
    pub name: String,
    /// Bytes; 0 when the provider does not say (then never served from the cache).
    pub size: u64,
    /// Last modified, Unix ms; 0 when the provider does not say.
    pub mtime: u64,
    pub dir: bool,
}

/// A document opened for reading: `file` at its start, valid while `hold` lives.
pub struct OpenedDocument {
    pub file: File,
    pub hold: Box<dyn Send>,
}

/// The document trees, as the platform sees them.
pub trait Documents {
    /// The children of `dir` (a folder's [`DocEntry::uri`]) in `tree`, of its root for `None`.
    /// `Err` when it cannot be listed (for the root: the tree is unreachable).
    fn children(&self, tree: &str, dir: Option<&str>) -> Result<Vec<DocEntry>, String>;
    /// Opens the document `uri` for reading.
    fn open(&self, uri: &str) -> Result<OpenedDocument, String>;
}

/// Where no document tree exists (the desktop, the tests of plain folders).
struct NoDocuments;

impl Documents for NoDocuments {
    fn children(&self, tree: &str, _dir: Option<&str>) -> Result<Vec<DocEntry>, String> {
        Err(format!("{tree}: document trees are not supported here"))
    }

    fn open(&self, uri: &str) -> Result<OpenedDocument, String> {
        Err(format!("{uri}: documents are not supported here"))
    }
}

/// How deep a tree is walked (a provider that lists a link to a parent folder never loops).
const MAX_TREE_DEPTH: usize = 32;

/// Walks the folders and files (no document trees: those are unreachable here).
pub fn scan(input: ScanInput, progress: &mut dyn FnMut(usize, usize)) -> ScanOutcome {
    scan_with_documents(input, &NoDocuments, progress)
}

/// Walks the folders, the document trees through `documents`, and the files.
/// `progress(checked, found)` is called after every ROM file.
pub fn scan_with_documents(
    input: ScanInput,
    documents: &dyn Documents,
    progress: &mut dyn FnMut(usize, usize),
) -> ScanOutcome {
    let mut walker = Walker {
        old: input.cache,
        cache: HashCache::new(),
        found: Vec::new(),
        checked: 0,
        progress,
        documents,
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
    let mut reachable_trees = Vec::new();
    for tree in &input.trees {
        match documents.children(tree, None) {
            Ok(entries) => {
                let before = (walker.checked, walker.found.len());
                walker.tree(tree, entries, 0);
                log::info!(
                    "library scan: tree {tree}: {} ROM files, {} games",
                    walker.checked - before.0,
                    walker.found.len() - before.1
                );
                reachable_trees.push(tree.as_str());
            }
            Err(e) => {
                log::warn!("library scan: tree unreachable: {e}");
                unreachable.push(tree.clone());
            }
        }
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
            || reachable_trees.iter().any(|t| in_tree(&path, t))
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
            .chain(input.trees.iter().cloned())
            .collect(),
        unreachable,
        cache,
    }
}

/// Whether `uri` is a document of the tree `tree`: Android spells a tree's documents
/// `<tree URI>/document/<id>`. One spelled otherwise only stays in the cache longer.
fn in_tree(uri: &str, tree: &str) -> bool {
    uri.strip_prefix(tree)
        .is_some_and(|rest| rest.starts_with("/document/"))
}

/// The modification time of a file, Unix ms (0 when the system has none).
fn mtime_ms(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or_default()
}

/// The cached identity, when the file's size and modification time did not change. The display
/// rule may have changed since it was cached: it is applied again, to `name`.
fn from_cache(name: &Path, size: u64, mtime: u64, cached: Option<&CacheEntry>) -> Option<Identity> {
    let c = cached.filter(|c| c.size == size && c.mtime == mtime)?;
    let mut identity = c.identity.clone();
    identity.title = identity::display_title(identity.console, &identity.title, name);
    Some(identity)
}

/// The identity of the file at `path`, from `cached` when its size and modification time did
/// not change, else read from the file.
fn identify_stamped(
    path: &Path,
    size: u64,
    mtime: u64,
    cached: Option<&CacheEntry>,
) -> Result<Identity, identity::NotAGame> {
    match from_cache(path, size, mtime, cached) {
        Some(identity) => Ok(identity),
        None => identity::identify(path),
    }
}

/// The identity of one file (a game opened by its path), from the library's hash cache entry
/// for that path when the file did not change, else hashed now. Takes no lock: the caller copies
/// the cache entry out first, so hashing a large ROM never holds the library.
pub fn identify_cached(
    path: &Path,
    cached: Option<&CacheEntry>,
) -> Result<Identity, identity::NotAGame> {
    let meta = fs::metadata(path)?;
    identify_stamped(path, meta.len(), mtime_ms(&meta), cached)
}

/// [`identify_cached`] for a file already open at its start (an Android document read through
/// its descriptor): its size and modification time from the descriptor, `name` only names it.
pub fn identify_open_file(
    name: &Path,
    file: &mut File,
    cached: Option<&CacheEntry>,
) -> Result<Identity, identity::NotAGame> {
    let meta = file.metadata()?;
    match from_cache(name, meta.len(), mtime_ms(&meta), cached) {
        Some(identity) => Ok(identity),
        None => identity::identify_file(name, file),
    }
}

struct Walker<'a> {
    old: HashCache,
    cache: HashCache,
    found: Vec<Found>,
    checked: usize,
    progress: &'a mut dyn FnMut(usize, usize),
    documents: &'a dyn Documents,
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

    /// The entries of one folder of `tree`, its subfolders listed in turn.
    fn tree(&mut self, tree: &str, mut entries: Vec<DocEntry>, depth: usize) {
        entries.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uri.cmp(&b.uri)));
        for entry in entries {
            if !entry.dir {
                self.document(tree, entry);
            } else if depth >= MAX_TREE_DEPTH {
                log::warn!("library scan: {} not walked: too deep", entry.uri);
            } else {
                match self.documents.children(tree, Some(&entry.uri)) {
                    Ok(children) => self.tree(tree, children, depth + 1),
                    Err(e) => log::warn!("library scan: {e}"),
                }
            }
        }
    }

    fn document(&mut self, tree: &str, entry: DocEntry) {
        let name = Path::new(&entry.name);
        if !identity::is_rom(name) {
            return;
        }
        self.checked += 1;
        // A size the provider does not tell could match a cached 0 by chance: hashed again.
        let cached = (entry.size > 0)
            .then(|| from_cache(name, entry.size, entry.mtime, self.old.get(&entry.uri)))
            .flatten();
        let identity = match cached {
            Some(identity) => Ok(identity),
            None => self
                .documents
                .open(&entry.uri)
                .map_err(|e| identity::NotAGame::Io(std::io::Error::other(e)))
                .and_then(|mut doc| identity::identify_file(name, &mut doc.file)),
        };
        let DocEntry {
            uri,
            name,
            size,
            mtime,
            ..
        } = entry;
        self.record(uri, Some(tree), size, mtime, identity, &name);
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
        let mtime = mtime_ms(&meta);
        let identity = identify_stamped(path, size, mtime, self.old.get(&key));
        let shown = path.display().to_string();
        self.record(key, folder, size, mtime, identity, &shown);
    }

    /// One ROM file checked: cached and found when it is a game, logged when it is not.
    fn record(
        &mut self,
        key: String,
        folder: Option<&str>,
        size: u64,
        mtime: u64,
        identity: Result<Identity, identity::NotAGame>,
        shown: &str,
    ) {
        match identity {
            Ok(identity) => {
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
            Err(e) => log::info!("library scan: {shown} skipped: {e}"),
        }
        (self.progress)(self.checked, self.found.len());
    }
}
