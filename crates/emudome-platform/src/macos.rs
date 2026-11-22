//! macOS: planned but not compiled in the pipeline (no Apple license).
//! When needed: `.dylib` cores in `Contents/Resources/cores`, signing and notarization.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "macOS";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "dylib";

/// Not packaged: `cores.lock` has no macOS build.
pub const CORES_LOCK_PLATFORM: Option<&str> = None;

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Azahar's default user folder: the XDG data folder, else Application Support.
pub const AZAHAR_LAYOUT: crate::azahar::Layout = crate::azahar::Layout::MacOs;

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;

/// WKWebView keeps its profile outside the app's data folder.
pub const WEBVIEW_PROFILE_DIRS: &[&str] = &[];

/// The platform key of `cores.lock` this build ships cores for.
pub const CORES_PLATFORM: Option<&str> = None;
