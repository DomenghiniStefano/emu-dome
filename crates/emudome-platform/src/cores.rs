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
    /// No search folder holds the file, and this build does not ship the core for this platform
    /// (`cores.lock`'s `ship` off, or no build for it): the console is not available on this
    /// device yet, nothing is broken. Only where no development place can hold it (a release,
    /// or Android, whose cores come from the package).
    NotShipped,
}

/// Whether `lock` ships `core` (`ship: true`) with a build for `platform`.
pub(crate) fn ships(lock: &str, core: &str, platform: &str) -> bool {
    let Ok(lock) = serde_json::from_str::<serde_json::Value>(lock) else {
        return false;
    };
    let Some(entry) = lock.get("cores").and_then(|c| c.get(core)) else {
        return false;
    };
    entry.get("ship").and_then(serde_json::Value::as_bool) == Some(true)
        && entry.get("builds").and_then(|b| b.get(platform)).is_some()
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

/// A core this build ships, as `cores.lock` describes it (the licences page, `about.lic`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShippedCore {
    /// The lock's id (`mgba`).
    pub id: String,
    /// Its display name (`mGBA`).
    pub name: String,
    /// The consoles it runs, as the lock writes them (`GB, GBC, GBA`).
    pub consoles: String,
    /// Its licence, as an SPDX expression (`MPL-2.0 AND BSD-3-Clause`).
    pub spdx: String,
    pub version: String,
    /// The upstream repository and the pinned commit (the source offer).
    pub repo: String,
    pub commit: String,
    /// The licence files packaged with it under `cores/notices/`: its own first, then the
    /// notices of what it bundles.
    pub notices: Vec<String>,
}

/// The cores `cores.lock` ships (`ship: true`) with a build for `platform`, by id.
/// One source of truth: nothing about a core is typed again in the app.
pub fn shipped_cores(platform: &str) -> Vec<ShippedCore> {
    shipped_in(CORES_LOCK, platform)
}

fn shipped_in(lock: &str, platform: &str) -> Vec<ShippedCore> {
    let Ok(lock) = serde_json::from_str::<serde_json::Value>(lock) else {
        log::error!("cores.lock compiled into this build is not JSON");
        return Vec::new();
    };
    let text = |v: &serde_json::Value, path: &[&str]| -> String {
        path.iter()
            .try_fold(v, |v, k| v.get(k))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let Some(cores) = lock.get("cores").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    cores
        .iter()
        .filter(|(_, core)| core.get("ship").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|(id, core)| {
            let build = core.get("builds")?.get(platform)?;
            let mut notices = vec![text(core, &["license", "file"])];
            notices.extend(
                build
                    .get("notices")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|n| text(n, &["file"])),
            );
            notices.retain(|n| !n.is_empty());
            Some(ShippedCore {
                id: id.clone(),
                name: text(core, &["name"]),
                consoles: text(core, &["consoles"]),
                spdx: text(core, &["spdx"]),
                version: text(core, &["source", "version"]),
                repo: text(core, &["source", "repo"]),
                commit: text(core, &["source", "commit"]),
                notices,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{find_in, locked_sha256, search_dirs, shipped_in, CoreLookupError, SearchPlaces};
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

    #[test]
    fn the_shipped_cores_come_from_the_lock_with_their_notices() {
        let cores = shipped_in(super::cores_lock(), "windows-x86_64");
        let ids: Vec<&str> = cores.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["melondsds", "mgba"], "Azahar is not shipped yet");
        let mgba = &cores[1];
        assert_eq!(mgba.name, "mGBA");
        assert!(mgba.spdx.contains("MPL-2.0"));
        assert_eq!(mgba.commit.len(), 40);
        assert_eq!(mgba.notices, ["mgba-LICENSE.txt", "mgba-inih-LICENSE.txt"]);
        assert!(shipped_in(super::cores_lock(), "nowhere").is_empty());
        assert!(shipped_in("not json", "windows-x86_64").is_empty());
        let lock = r#"{"cores":{"a":{"ship":false,"builds":{"p":{}}},
            "b":{"ship":true,"name":"B","license":{"file":"b.txt"},"builds":{"p":{"notices":[]}}}}}"#;
        let only = shipped_in(lock, "p");
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].notices, ["b.txt"]);
    }

    #[test]
    fn a_core_ships_only_with_ship_on_and_a_build_for_the_platform() {
        let lock = r#"{"cores":{"a":{"ship":false,"builds":{"p":{}}},
            "b":{"ship":true,"builds":{"p":{}}}}}"#;
        assert!(super::ships(lock, "b", "p"));
        assert!(!super::ships(lock, "b", "q"), "no build for that platform");
        assert!(!super::ships(lock, "a", "p"), "ship off");
        assert!(!super::ships(lock, "c", "p"), "not in the lock");
        assert!(!super::ships("not json", "b", "p"));
        // The real lock: Azahar is fetchable but not shipped, on Android too (#127).
        let real = super::cores_lock();
        assert!(super::ships(real, "mgba", "android-arm64-v8a"));
        assert!(super::ships(real, "melondsds", "android-arm64-v8a"));
        assert!(!super::ships(real, "azahar", "android-arm64-v8a"));
    }
}
