//! Everything that changes from one operating system to another lives here.
//!
//! Project rule: no `#[cfg(target_os)]` outside this crate.
//! Each system has its own module; macOS exists as a stub but is not in the
//! build pipeline (signing and notarization need the Apple license).

use std::path::{Path, PathBuf};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

mod azahar;
pub mod cloud;
mod content;
mod cores;
mod cpu;
mod paths;

pub use content::is_content_uri;
pub use cores::CoreLookupError;
pub use paths::{long_paths_enabled, long_paths_policy, path_limits, PathLimits};

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

/// The folders Azahar writes its whole user tree (emulated NAND, SD card, states) into when it
/// cannot reach the save directory EmuDome gives it: its own default user folder on this system,
/// every candidate, the one it would pick first first (Trello #97; how they were read from
/// Azahar's source: `azahar.rs`). Only to look at: EmuDome never writes or deletes there.
pub fn azahar_user_dirs() -> Vec<PathBuf> {
    azahar::user_dirs(imp::AZAHAR_LAYOUT, &azahar::Places::from_env())
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

/// Whether a window event says the app is in front again after [`is_suspend`]: Android's
/// `onResume` (Tauri's `WindowEvent::Resumed`, which skips the first one, at creation). The
/// desktop has no such event: the page's focus says it there.
pub fn is_resume(event: &tauri::WindowEvent) -> bool {
    #[cfg(target_os = "android")]
    {
        matches!(event, tauri::WindowEvent::Resumed)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = event;
        false
    }
}

/// The game to open at startup: the first non-flag argument on the desktop
/// (`emudome game.gba`); on Android the `game` extra of the launch intent
/// (`adb shell am start -n <package>/.MainActivity --es game /path/game.gba`), else the
/// `content://` document of an «Open with» (`ACTION_VIEW`), which [`open_content`] opens.
pub fn startup_game() -> Option<String> {
    imp::startup_game()
}

/// The app identifier before 2026-11-21 (Trello #90; now `it.domehub.emudome`, in
/// `tauri.conf.json`). The system named the data folders after it.
pub const OLD_IDENTIFIER: &str = "com.domenghini.emudome";

/// The app's two data folders: the synced one and the per-device one (one folder on Linux).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataDirs {
    pub data: PathBuf,
    pub local: PathBuf,
}

/// The data folders of [`OLD_IDENTIFIER`], given the current ones (`app_data_dir` and
/// `app_local_data_dir`): on the desktop the same folders under the old name, side by side.
/// `None` on Android: the old identifier is another package, whose private storage the new one
/// cannot read (the data of the old app does not carry over there).
pub fn old_data_dirs(data: &Path, local: &Path) -> Option<DataDirs> {
    imp::old_data_dirs(data, local)
}

/// Folders at the top of the data folders that hold the webview's own profile, not the user's
/// data: the one-time copy to the new identifier leaves them (Windows: WebView2's `EBWebView`).
pub const WEBVIEW_PROFILE_DIRS: &[&str] = imp::WEBVIEW_PROFILE_DIRS;

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

/// A `content://` document opened for a core ([`open_content`]): a path with the document's name
/// and extension that reads it in this process, valid while this value lives. Keep it until the
/// core is done with the game; dropping it closes the document and removes the path.
pub struct ContentFile {
    path: PathBuf,
    name: Option<String>,
    /// The descriptor and the link, released on drop.
    _hold: Box<dyn Send>,
}

impl ContentFile {
    /// The path the core and EmuDome read the game from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The document's display name (its file name, as the provider tells it).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Why a `content://` document could not be opened ([`open_content`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    /// This system has no content documents (the desktop).
    Unsupported,
    /// The provider refused it (permission gone, document deleted) or it could not be read.
    Unreadable(String),
    /// Not a file on the device (a stream from a cloud provider): it cannot be read again from
    /// the start, as cores and EmuDome do.
    NotLocal,
}

/// Opens the `content://` document `uri` for reading, without copying it (`content.rs` says how):
/// the link it reads through goes under `cache_dir` (the app's private cache). Blocks on the
/// provider: never call it on the main thread. Takes the persistable read permission when the
/// system granted one (a document picked with `ACTION_OPEN_DOCUMENT`), so that game opens again
/// after a restart. [`ContentError::Unsupported`] on the desktop.
pub fn open_content(uri: &str, cache_dir: &Path) -> Result<ContentFile, ContentError> {
    imp::open_content(uri, cache_dir)
}

/// Connects the app to what the system asks outside the page, whatever the video path. Android:
/// `on_back` is called for every press of the back button or gesture (on the UI thread: it must
/// return quickly; the system's own back no longer runs), and [`startup_game`] learns the launch
/// intent's game. The desktop has no back button (its back is the keyboard's Esc and the
/// controller's B): nothing is kept there.
pub fn attach_system<R: tauri::Runtime>(
    webview: &tauri::Webview<R>,
    on_back: impl Fn() + Send + Sync + 'static,
) {
    imp::attach_system(webview, Box::new(on_back));
}

/// Listener of the external surface, boxed for the per-OS modules.
pub(crate) type SurfaceListener = Box<dyn Fn(SurfaceEvent) + Send + Sync>;

/// Listener of the back button, boxed for the per-OS modules.
pub(crate) type BackListener = Box<dyn Fn() + Send + Sync>;
