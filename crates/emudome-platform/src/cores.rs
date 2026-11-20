//! Where a libretro core is looked for, and the check of a shipped core against `cores.lock`.
//!
//! A core is native code run inside EmuDome, so where it comes from is a security boundary:
//! a release build loads only the cores in its own resource folder, each checked against the
//! sha256 `cores.lock` gives for this platform. The development places (`EMUDOME_CORES_DIR`,
//! `cores/` in any folder above the working directory, the system's libretro folders) exist in
//! debug builds only: in a release, launched by a file association from Downloads, a planted
//! `..\cores\mgba_libretro.dll` would otherwise run with the user's rights.
//!
//! Blind spot: the file is hashed, then loaded by path, so a process that can rewrite the
//! resource folder between the two can swap it; such a process already owns the installation.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// The lock compiled in: the cores this build was packaged with, and their checksums.
const CORES_LOCK: &str = include_str!("../../../cores.lock");

/// Everything the search looks at, gathered by the caller so the choice stays a pure function.
pub(crate) struct SearchPlaces {
    /// A debug build: the development places are searched too.
    pub debug: bool,
    /// `EMUDOME_CORES_DIR`.
    pub env_dir: Option<PathBuf>,
    /// The folder of the running executable.
    pub exe_dir: Option<PathBuf>,
    /// The app's resource folder (the Tauri shell passes it).
    pub resource_dir: Option<PathBuf>,
    /// The working directory, whose ancestors are walked.
    pub cwd: Option<PathBuf>,
    /// The system's own folders (Linux: `/usr/lib/libretro`).
    pub system_dirs: Vec<PathBuf>,
}

/// The folders to look in, in order; the first one holding the file wins.
///
/// Release: the `cores` folder in the app resources only. Debug, after it: `EMUDOME_CORES_DIR`
/// (first, so it can override), the `cores` folder next to the executable, `cores/` in the
/// working directory and every folder above it (the repo root), the system's folders.
pub(crate) fn search_dirs(places: SearchPlaces) -> Vec<PathBuf> {
    let resources = places.resource_dir.map(|r| r.join("cores"));
    if !places.debug {
        return resources.into_iter().collect();
    }
    let mut dirs: Vec<PathBuf> = places.env_dir.into_iter().collect();
    dirs.extend(places.exe_dir.map(|e| e.join("cores")));
    dirs.extend(resources);
    if let Some(mut cwd) = places.cwd {
        loop {
            dirs.push(cwd.join("cores"));
            if !cwd.pop() {
                break;
            }
        }
    }
    dirs.extend(places.system_dirs);
    dirs
}

/// Why no core can be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreLookupError {
    /// No search folder holds the file.
    NotFound,
    /// The file found is not the one `cores.lock` pins for this platform (or could not be read
    /// to tell): it is never loaded.
    Mismatch { path: PathBuf },
}

/// The sha256 (lowercase hex) `lock` pins for `core`'s library `file_name` on `platform`, if the
/// lock lists that core for that platform under that file name.
pub(crate) fn locked_sha256(
    lock: &str,
    core: &str,
    platform: &str,
    file_name: &str,
) -> Option<String> {
    let lock: serde_json::Value = serde_json::from_str(lock).ok()?;
    let library = lock
        .get("cores")?
        .get(core)?
        .get("builds")?
        .get(platform)?
        .get("library")?;
    if library.get("file")?.as_str()? != file_name {
        return None;
    }
    library.get("sha256")?.as_str().map(str::to_ascii_lowercase)
}

/// Lowercase hex sha256 of a file's bytes.
fn file_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Finds `file_name` in `dirs`. With `verify` (release), the file found is accepted only if its
/// sha256 is the one `lock` pins for `core` on `platform`; a core the lock does not pin for this
/// platform is accepted as it is (logged).
pub(crate) fn find_in(
    dirs: &[PathBuf],
    file_name: &str,
    core: &str,
    verify: Option<(&str, &str)>,
) -> Result<PathBuf, CoreLookupError> {
    let path = dirs
        .iter()
        .map(|d| d.join(file_name))
        .find(|p| p.is_file())
        .ok_or(CoreLookupError::NotFound)?;
    let Some((lock, platform)) = verify else {
        return Ok(path);
    };
    let Some(expected) = locked_sha256(lock, core, platform, file_name) else {
        log::warn!("cores.lock pins no {file_name} for {platform}: loaded unchecked");
        return Ok(path);
    };
    match file_sha256(&path) {
        Ok(actual) if actual == expected => Ok(path),
        Ok(actual) => {
            log::error!(
                "core {} refused: sha256 {actual}, cores.lock pins {expected}",
                path.display()
            );
            Err(CoreLookupError::Mismatch { path })
        }
        Err(e) => {
            log::error!("core {} refused: cannot read it ({e})", path.display());
            Err(CoreLookupError::Mismatch { path })
        }
    }
}

/// The lock compiled into this build.
pub(crate) fn cores_lock() -> &'static str {
    CORES_LOCK
}

#[cfg(test)]
mod tests {
    use super::{find_in, locked_sha256, search_dirs, CoreLookupError, SearchPlaces};
    use std::path::{Path, PathBuf};

    const NAME: &str = "mgba_libretro.dll";
    const PLATFORM: &str = "windows-x86_64";
    /// The bytes the test lock pins.
    const GOOD: &[u8] = b"good core";

    fn sha(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn lock_for(bytes: &[u8]) -> String {
        format!(
            r#"{{"cores":{{"mgba":{{"builds":{{"{PLATFORM}":{{"library":{{"file":"{NAME}","sha256":"{}"}}}}}}}}}}}}"#,
            sha(bytes)
        )
    }

    /// A fresh folder under the system temp dir, removed when dropped.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "emudome-cores-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn file(&self, rel: &str, bytes: &[u8]) -> PathBuf {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, bytes).unwrap();
            p
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The layout of the attack: the app installed in `app/`, launched from `Downloads/game/`
    /// (working directory) with a core planted in `cores/` above it and in `EMUDOME_CORES_DIR`.
    fn places(root: &Path, debug: bool) -> SearchPlaces {
        SearchPlaces {
            debug,
            env_dir: Some(root.join("env")),
            exe_dir: Some(root.join("app")),
            resource_dir: Some(root.join("app")),
            cwd: Some(root.join("Downloads").join("game")),
            system_dirs: vec![root.join("system")],
        }
    }

    #[test]
    fn release_ignores_a_core_planted_above_the_working_directory() {
        let s = Scratch::new("planted");
        s.file(&format!("Downloads/cores/{NAME}"), b"evil");
        s.file(&format!("env/{NAME}"), b"evil");
        s.file(&format!("system/{NAME}"), b"evil");
        std::fs::create_dir_all(s.0.join("Downloads/game")).unwrap();
        let lock = lock_for(GOOD);

        let release = search_dirs(places(&s.0, false));
        assert_eq!(release, vec![s.0.join("app").join("cores")]);
        assert_eq!(
            find_in(&release, NAME, "mgba", Some((&lock, PLATFORM))),
            Err(CoreLookupError::NotFound)
        );

        // The same layout in a debug build finds the planted file: the walk is development only.
        let debug = search_dirs(places(&s.0, true));
        assert_eq!(
            find_in(&debug, NAME, "mgba", None),
            Ok(s.0.join("env").join(NAME))
        );
        assert!(debug.contains(&s.0.join("Downloads").join("cores")));
        assert!(debug.contains(&s.0.join("system")));
    }

    #[test]
    fn release_loads_the_shipped_core_only_when_its_hash_matches() {
        let s = Scratch::new("hash");
        let lock = lock_for(GOOD);
        let dirs = search_dirs(places(&s.0, false));

        let good = s.file(&format!("app/cores/{NAME}"), GOOD);
        assert_eq!(
            find_in(&dirs, NAME, "mgba", Some((&lock, PLATFORM))),
            Ok(good.clone())
        );

        s.file(&format!("app/cores/{NAME}"), b"tampered");
        assert_eq!(
            find_in(&dirs, NAME, "mgba", Some((&lock, PLATFORM))),
            Err(CoreLookupError::Mismatch { path: good })
        );
    }

    #[test]
    fn a_core_the_lock_does_not_pin_here_is_loaded_unchecked() {
        let s = Scratch::new("unpinned");
        let dirs = search_dirs(places(&s.0, false));
        let path = s.file(&format!("app/cores/{NAME}"), b"anything");
        let lock = lock_for(GOOD);
        assert_eq!(
            find_in(&dirs, NAME, "mgba", Some((&lock, "linux-x86_64"))),
            Ok(path.clone())
        );
        assert_eq!(
            find_in(&dirs, NAME, "azahar", Some((&lock, PLATFORM))),
            Ok(path)
        );
    }

    #[test]
    fn the_real_lock_pins_the_shipped_cores() {
        let lock = super::cores_lock();
        let mgba = locked_sha256(lock, "mgba", "windows-x86_64", "mgba_libretro.dll");
        assert_eq!(mgba.map(|h| h.len()), Some(64));
        assert!(locked_sha256(lock, "mgba", "linux-x86_64", "mgba_libretro.so").is_some());
        assert_eq!(
            locked_sha256(lock, "mgba", "windows-x86_64", "other.dll"),
            None
        );
    }
}
