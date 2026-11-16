//! Everything that changes from one operating system to another lives here.
//!
//! Project rule: no `#[cfg(target_os)]` outside this crate.
//! Each system has its own module; macOS exists as a stub but is not in the
//! build pipeline (signing and notarization need the Apple license).

use std::path::{Path, PathBuf};

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
use android as imp;
#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(target_os = "macos")]
use macos as imp;
#[cfg(target_os = "windows")]
use windows as imp;

/// Whether the game is drawn natively under the webview (otherwise in a canvas
/// inside the webview). Can be forced with `EMUDOME_VIDEO=native|webview`.
pub fn native_video_default() -> bool {
    match std::env::var("EMUDOME_VIDEO").as_deref() {
        Ok("native") => true,
        Ok("webview") => false,
        _ => imp::NATIVE_VIDEO_DEFAULT,
    }
}

/// System name, for logs and diagnostics.
pub const OS_NAME: &str = imp::OS_NAME;

/// File name of a libretro core, e.g. `mgba` → `mgba_libretro.dll` on Windows.
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{}", imp::DYLIB_EXT)
}

/// Where to look for the cores, in order. The first one holding the file wins.
///
/// 1. `EMUDOME_CORES_DIR`, handy in development.
/// 2. The `cores` folder next to the executable (installation).
/// 3. The `cores` folder in the app resources (passed by the Tauri shell).
/// 4. `cores/` at the repo root, walking up from the current folder (development).
pub fn core_search_dirs(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = std::env::var("EMUDOME_CORES_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("cores"));
        }
    }
    if let Some(res) = resource_dir {
        dirs.push(res.join("cores"));
    }
    if let Ok(mut cwd) = std::env::current_dir() {
        loop {
            dirs.push(cwd.join("cores"));
            if !cwd.pop() {
                break;
            }
        }
    }
    dirs.extend(imp::extra_core_dirs());
    dirs
}

/// Finds the core file by searching the candidate folders.
pub fn find_core(core: &str, resource_dir: Option<&Path>) -> Option<PathBuf> {
    let name = core_file_name(core);
    core_search_dirs(resource_dir)
        .into_iter()
        .map(|d| d.join(&name))
        .find(|p| p.is_file())
}
