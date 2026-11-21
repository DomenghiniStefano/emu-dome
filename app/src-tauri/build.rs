fn main() {
    // Windows: EmuDome's own application manifest (the other targets ignore it). It is
    // tauri-build's default (Common Controls v6, which the dialog APIs need) plus `longPathAware`:
    // the cores run in this process, and Azahar keeps its emulated NAND deep under the game's
    // save folder (Trello #94). The system policy (`LongPathsEnabled`) is needed too; EmuDome
    // only reads it (`emudome_platform::long_paths_policy`).
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("windows-app-manifest.xml"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    if let Err(e) = tauri_build::try_build(attributes) {
        // As `tauri_build::build` reports it.
        println!("{e:#}");
        std::process::exit(1);
    }
}
