//! Windows: core DLLs next to the executable or in `cores/`.
use std::path::PathBuf;

pub const OS_NAME: &str = "Windows";
pub const DYLIB_EXT: &str = "dll";

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
