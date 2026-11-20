//! Where the app keeps its data: Tauri's app data / app local data folders, or, with
//! `EMUDOME_DATA_DIR` set, folders inside that one only.
//!
//! The override moves **everything** (saves, settings, library, covers, first-run marker,
//! webview profile), so a test or bench run never touches the user's real data. Before
//! 2026-11-18 it moved the saves only, and a test run rewrote the real `library.json`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{AppHandle, Manager, Runtime};

static OVERRIDE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Sets the `EMUDOME_DATA_DIR` override, once, before anything opens its files.
pub fn set_override(dir: Option<PathBuf>) {
    if OVERRIDE.set(dir).is_err() {
        log::warn!("data folder override set twice: the first one stays");
    }
}

fn overridden() -> Option<&'static PathBuf> {
    OVERRIDE.get().and_then(Option::as_ref)
}

/// The synced data folder (saves, `settings/`).
pub fn data<R: Runtime, M: Manager<R>>(app: &M) -> tauri::Result<PathBuf> {
    match overridden() {
        Some(dir) => Ok(dir.clone()),
        None => app.path().app_data_dir(),
    }
}

/// The per-device data folder (library, covers, `local-settings/`, first-run marker).
pub fn local<R: Runtime, M: Manager<R>>(app: &M) -> tauri::Result<PathBuf> {
    match overridden() {
        Some(dir) => Ok(dir.join("local")),
        None => app.path().app_local_data_dir(),
    }
}

/// `local` for an `AppHandle`, logging instead of failing.
pub fn local_or_log(app: &AppHandle, what: &str) -> Option<PathBuf> {
    local(app).inspect_err(|e| log::warn!("{what}: {e}")).ok()
}

/// `dir/name` when `name` is a plain file name of one of our folders ([`file_name_ok`]), else
/// `None`. For names read back from a file (`library.json`, the settings) or from a URL: those
/// may be synced or tampered with, and an absolute path or `..` would reach any file on the disk.
pub fn file_in(dir: &Path, name: &str) -> Option<PathBuf> {
    file_name_ok(name).then(|| dir.join(name))
}

/// Whether `name` is a plain file name: ASCII letters, digits, `-`, `_` and `.` only (so no
/// separator, drive or UNC prefix), not empty, not starting with a dot (so no `..`).
pub fn file_name_ok(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::{file_in, file_name_ok};
    use std::path::Path;

    #[test]
    fn only_plain_names_stay_inside_the_folder() {
        assert!(file_name_ok("0123abcd.png"));
        assert!(file_name_ok("gba-0a1b.user-3.png"));
        for bad in [
            "",
            ".hidden",
            "..",
            "../settings/global.json",
            "a/b.png",
            r"a\b.png",
            r"C:\Windows\win.ini",
            "C:win.ini",
            "/etc/passwd",
            r"\\server\share\x.png",
        ] {
            assert!(!file_name_ok(bad), "{bad:?} accepted");
            assert_eq!(file_in(Path::new("covers"), bad), None);
        }
        assert_eq!(
            file_in(Path::new("covers"), "x.png"),
            Some(Path::new("covers").join("x.png"))
        );
    }
}
