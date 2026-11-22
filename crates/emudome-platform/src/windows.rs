//! Windows: core DLLs next to the executable or in `cores/`.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "Windows";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "dll";

/// The platform key of `cores.lock` whose checksums a release build checks its cores against.
pub const CORES_LOCK_PLATFORM: Option<&str> = if cfg!(target_arch = "x86_64") {
    Some("windows-x86_64")
} else {
    None
};

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Azahar's default user folder: next to the executable, else Roaming AppData.
pub const AZAHAR_LAYOUT: crate::azahar::Layout = crate::azahar::Layout::Windows;

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;

/// WebView2's profile, which Tauri keeps in the local data folder: a cache the webview makes
/// again (the interface keeps nothing there that matters).
pub const WEBVIEW_PROFILE_DIRS: &[&str] = &["EBWebView"];

/// The platform key of `cores.lock` this build ships cores for.
pub const CORES_PLATFORM: Option<&str> = CORES_LOCK_PLATFORM;
