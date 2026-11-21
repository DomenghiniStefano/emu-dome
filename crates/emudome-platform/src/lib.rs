//! Everything that changes from one operating system to another lives here.
//!
//! Project rule: no `#[cfg(target_os)]` outside this crate.
//! Each system has its own module; macOS exists as a stub but is not in the
//! build pipeline (signing and notarization need the Apple license).

use std::path::{Path, PathBuf};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

mod cores;
mod cpu;

pub use cores::CoreLookupError;

#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod desktop;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
use android as imp;
#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(target_os = "macos")]
use macos as imp;
#[cfg(target_os = "windows")]
use windows as imp;

/// Whether the game is drawn natively under the webview (otherwise in a canvas
/// inside the webview) on this system. `EMUDOME_VIDEO=native|webview` overrides it
/// (read with the other knobs in the app's `tuning.rs`).
pub fn native_video_default() -> bool {
    imp::NATIVE_VIDEO_DEFAULT
}

/// CPU time used so far by this process (user + kernel, every thread), for the bench
/// reports; `None` where the system does not say.
pub fn process_cpu_time() -> Option<std::time::Duration> {
    cpu::process_cpu_time()
}

/// System name, for logs and diagnostics.
pub const OS_NAME: &str = imp::OS_NAME;

/// The URL the webview loads for `path` on the app's custom protocol `scheme` (what
/// `convertFileSrc(path, scheme)` gives in the interface): WebView2 and Android's WebView
/// reach custom protocols as `http://<scheme>.localhost/`, the others as `<scheme>://localhost/`.
/// `path` must already be URL-safe.
pub fn protocol_url(scheme: &str, path: &str) -> String {
    if cfg!(any(target_os = "windows", target_os = "android")) {
        format!("http://{scheme}.localhost/{path}")
    } else {
        format!("{scheme}://localhost/{path}")
    }
}

/// Starts the log output: stderr on the desktop (`RUST_LOG` filters it, `info` by default),
/// logcat on Android (tag `EmuDome`).
pub fn init_logging() {
    imp::init_logging();
}

/// Whether a window event says the app went to the background: Android's `onPause`, after which
/// the process may be killed without any other event (Tauri's `WindowEvent::Suspended`, mobile
/// only). The desktop has no such event: there the app ends by closing its window or quitting.
pub fn is_suspend(event: &tauri::WindowEvent) -> bool {
    #[cfg(target_os = "android")]
    {
        matches!(event, tauri::WindowEvent::Suspended)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = event;
        false
    }
}

/// The game to open at startup: the first non-flag argument on the desktop
/// (`emudome game.gba`), the `game` extra of the launch intent on Android
/// (`adb shell am start -n <package>/.MainActivity --es game /path/game.gba`).
pub fn startup_game() -> Option<String> {
    imp::startup_game()
}

/// File name of a libretro core, e.g. `mgba` → `mgba_libretro.dll` on Windows.
pub fn core_file_name(core: &str) -> String {
    imp::core_file_name(core)
}

/// Debug builds (and the `dev-core-paths` feature, for bench runs) search the development places
/// for the cores and do not check them against `cores.lock`.
const DEV_CORE_PATHS: bool = cfg!(any(debug_assertions, feature = "dev-core-paths"));

/// Where to look for the cores, in order. The first one holding the file wins.
///
/// A release build looks only in the `cores` folder of the app resources (passed by the Tauri
/// shell). A debug build (or the `dev-core-paths` feature) also looks in `EMUDOME_CORES_DIR`
/// (first), the `cores` folder next to the executable, `cores/` in the working directory and
/// every folder above it (the repo root), and the system's libretro folders. Why: `cores.rs`.
pub fn core_search_dirs(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let debug = DEV_CORE_PATHS;
    cores::search_dirs(cores::SearchPlaces {
        debug,
        env_dir: debug
            .then(|| std::env::var_os("EMUDOME_CORES_DIR"))
            .flatten()
            .map(PathBuf::from),
        exe_dir: std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf)),
        resource_dir: resource_dir.map(Path::to_path_buf),
        cwd: std::env::current_dir().ok(),
        system_dirs: imp::extra_core_dirs(),
    })
}

/// Finds the core file by searching the candidate folders ([`core_search_dirs`]). A release
/// build accepts the file only if its sha256 is the one `cores.lock` pins for this platform
/// (when the lock pins one). Where the system loads libraries by name (Android: the `.so`
/// packaged in the APK) and no folder holds the file, the bare file name is returned for the
/// dynamic loader.
pub fn find_core(core: &str, resource_dir: Option<&Path>) -> Result<PathBuf, CoreLookupError> {
    let name = core_file_name(core);
    let verify = imp::CORES_LOCK_PLATFORM
        .filter(|_| !DEV_CORE_PATHS)
        .map(|platform| (cores::cores_lock(), platform));
    cores::find_in(&core_search_dirs(resource_dir), &name, core, verify).or_else(|e| match e {
        CoreLookupError::NotFound => imp::packaged_core(&name).ok_or(CoreLookupError::NotFound),
        CoreLookupError::Mismatch { .. } => Err(e),
    })
}

/// True where the native video cannot draw on Tauri's window and gets a surface of
/// its own from the system instead (Android: a `SurfaceView` under the WebView),
/// handed over through [`attach_external_surface`].
pub const EXTERNAL_SURFACE: bool = imp::EXTERNAL_SURFACE;

/// True on phones (Android), false on the desktop: decides which settings exist here
/// (`Platform` in the settings schema) and, later, the phone interface.
pub const PHONE: bool = imp::PHONE;

/// A drawing surface handed over by the system outside Tauri's window.
/// It keeps the system object alive until dropped.
pub struct ExternalSurface {
    window: imp::SurfaceWindow,
    width: u32,
    height: u32,
}

impl ExternalSurface {
    /// Size in physical pixels when it was handed over.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl HasWindowHandle for ExternalSurface {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.window.window_handle()
    }
}

/// The system's display handle: wgpu's GLES backend (EGL) needs it to create the surface.
impl HasDisplayHandle for ExternalSurface {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        imp::surface_display_handle(&self.window)
    }
}

/// Life cycle of the external surface.
pub enum SurfaceEvent {
    /// A new surface to draw on.
    Ready(ExternalSurface),
    /// The current surface changed size (physical pixels).
    Resized { width: u32, height: u32 },
    /// The surface is going away: the listener must return only once it is no
    /// longer used (the system destroys it right after).
    Lost,
}

/// Puts the external surface under `webview` (made transparent) and reports its life
/// cycle to `on_event`. Does nothing where [`EXTERNAL_SURFACE`] is false.
pub fn attach_external_surface<R: tauri::Runtime>(
    webview: &tauri::Webview<R>,
    on_event: impl Fn(SurfaceEvent) + Send + Sync + 'static,
) {
    imp::attach_external_surface(webview, Box::new(on_event));
}

/// How the main window starts ([`create_window`]).
pub struct WindowOptions {
    /// False in bench mode: the window appears without taking the focus.
    pub focused: bool,
    /// The webview's profile folder, when the data folder is not the default one
    /// (`EMUDOME_DATA_DIR`); `None` leaves Tauri's default.
    pub webview_data_dir: Option<PathBuf>,
}

/// Creates the main window and the interface's webview. Tauri's API differs by form factor:
///
/// - desktop: a bare window with the transparent "ui" child webview on top, so the native
///   video can draw on the window underneath (Tauri's `unstable` feature);
/// - Android: the single webview window the system gives; the native video gets a
///   surface of its own ([`attach_external_surface`]).
pub fn create_window(
    app: &tauri::App,
    options: &WindowOptions,
) -> tauri::Result<(tauri::Window, tauri::Webview)> {
    imp::create_window(app, options)
}

/// Asks the user for one file, offering only the extensions `extensions` (listed as `filter`).
/// `None` when the user cancels, or where no file-path picker exists yet (Android: logged).
pub fn pick_file(app: &tauri::AppHandle, filter: &str, extensions: &[&str]) -> Option<PathBuf> {
    imp::pick_file(app, filter, extensions)
}

/// Puts `window` in fullscreen or takes it out. Android has no such call (Tauri's mobile window):
/// there the app already fills the screen, so it does nothing and succeeds.
pub fn set_fullscreen(window: &tauri::Window, on: bool) -> tauri::Result<()> {
    imp::set_fullscreen(window, on)
}

/// Listener of the external surface, boxed for the per-OS modules.
pub(crate) type SurfaceListener = Box<dyn Fn(SurfaceEvent) + Send + Sync>;
