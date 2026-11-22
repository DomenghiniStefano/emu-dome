//! Linux: besides `cores/`, also the folders where distributions install libretro cores.
use std::path::PathBuf;

pub use crate::desktop::*;

pub const OS_NAME: &str = "Linux";
pub fn core_file_name(core: &str) -> String {
    format!("{core}_libretro.{DYLIB_EXT}")
}

const DYLIB_EXT: &str = "so";

/// The platform key of `cores.lock` whose checksums a release build checks its cores against.
pub const CORES_LOCK_PLATFORM: Option<&str> = if cfg!(target_arch = "x86_64") {
    Some("linux-x86_64")
} else {
    None
};

pub fn extra_core_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/lib/libretro"),
        PathBuf::from("/usr/lib/x86_64-linux-gnu/libretro"),
    ]
}

/// Azahar's default user folder: in the working folder, else the XDG data folder.
pub const AZAHAR_LAYOUT: crate::azahar::Layout = crate::azahar::Layout::Xdg;

/// On Linux (GTK) the native surface covers the webview: the canvas in the webview is used.
/// Verified in Phase 0 on X11: wgpu drawing on the window hides the interface.
pub const NATIVE_VIDEO_DEFAULT: bool = false;

/// WebKitGTK's files have no single folder name to leave out: copied with the rest (small).
pub const WEBVIEW_PROFILE_DIRS: &[&str] = &[];

/// The platform key of `cores.lock` this build ships cores for.
pub const CORES_PLATFORM: Option<&str> = CORES_LOCK_PLATFORM;
