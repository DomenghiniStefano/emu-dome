//! Android: the cores are `.so` files inside the APK (jniLibs), loaded by name.
//! Actual loading from the app's native folder comes in Phase 0, step 4.
use std::path::PathBuf;

pub const OS_NAME: &str = "Android";
pub const DYLIB_EXT: &str = "so";

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// Game drawn natively under the transparent webview: to be verified in Phase 0.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
