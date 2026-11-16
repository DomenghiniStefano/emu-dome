//! Linux: besides `cores/`, also the folders where distributions install libretro cores.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "Linux";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "so";

pub fn extra_core_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/lib/libretro"),
        PathBuf::from("/usr/lib/x86_64-linux-gnu/libretro"),
    ]
}

/// On Linux (GTK) the native surface covers the webview: the canvas in the webview is used.
/// Verified in Phase 0 on X11: wgpu drawing on the window hides the interface.
pub const NATIVE_VIDEO_DEFAULT: bool = false;
