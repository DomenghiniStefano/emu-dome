//! Windows: core DLLs next to the executable or in `cores/`.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "Windows";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "dll";

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
