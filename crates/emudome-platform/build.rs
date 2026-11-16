//! On Android, hands the Gradle library in `android/` (the SurfaceView under the WebView)
//! to the app: tauri-build reads `DEP_EMUDOME_PLATFORM_ANDROID_LIBRARY_PATH` and adds the
//! module to `gen/android`. Nothing to do on the other systems.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        let Some(dir) = std::env::var_os("CARGO_MANIFEST_DIR") else {
            panic!("CARGO_MANIFEST_DIR is set by Cargo for build scripts");
        };
        let dir = std::path::PathBuf::from(dir).join("android");
        println!("cargo:android_library_path={}", dir.display());
    }
}
