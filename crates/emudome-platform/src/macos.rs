//! macOS: planned but not compiled in the pipeline (no Apple license).
//! When needed: `.dylib` cores in `Contents/Resources/cores`, signing and notarization.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "macOS";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "dylib";

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
