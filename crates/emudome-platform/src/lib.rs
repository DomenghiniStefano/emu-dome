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
pub mod log_file;
pub mod pads;
mod paths;
mod storage;
pub mod updater;

pub use content::is_content_uri;
pub use cores::{shipped_cores, CoreLookupError, ShippedCore};
pub use paths::{long_paths_enabled, long_paths_policy, path_limits, PathLimits};
pub use storage::{disk_space, DiskSpace};

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

/// Leaves to the background, as the system's back does at an app's first screen (Android's
/// back at the shell's root, Trello #29): the app keeps running, its task goes behind the
/// others. `true` when the system did it; `false` on the desktop, which has no such place, and
/// on Android when the activity could not be reached (logged).
pub fn move_to_background() -> bool {
    imp::move_to_background()
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
/// dynamic loader, if the build ships that core for this platform.
///
/// A core no folder holds and this build does not ship ([`core_shipped`]) is
/// [`CoreLookupError::NotShipped`] where no development place could hold it (a release, or the
/// package's cores on Android): its console is not available on this device yet (#127).
pub fn find_core(core: &str, resource_dir: Option<&Path>) -> Result<PathBuf, CoreLookupError> {
    let name = core_file_name(core);
    let verify = imp::CORES_LOCK_PLATFORM
        .filter(|_| !DEV_CORE_PATHS)
        .map(|platform| (cores::cores_lock(), platform));
    cores::find_in(&core_search_dirs(resource_dir), &name, core, verify).or_else(|e| match e {
        CoreLookupError::NotFound => {
            let from_package = imp::packaged_core(&name);
            if !core_shipped(core) && (!DEV_CORE_PATHS || from_package.is_some()) {
                return Err(CoreLookupError::NotShipped);
            }
            from_package.ok_or(CoreLookupError::NotFound)
        }
        CoreLookupError::Mismatch { .. } | CoreLookupError::NotShipped => Err(e),
    })
}

/// Whether this build ships `core` for this platform (`cores.lock`: `ship` on and a build for
/// [`CORES_PLATFORM`]). False on a platform the lock has no key for.
pub fn core_shipped(core: &str) -> bool {
    imp::CORES_PLATFORM.is_some_and(|platform| cores::ships(cores::cores_lock(), core, platform))
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

/// Asks the user where to save a file, proposing `name` and offering the extensions
/// `extensions` (listed as `filter`). `None` when the user cancels, or where no save dialog exists
/// yet (Android: logged; the system's document picker comes later).
pub fn save_file(
    app: &tauri::AppHandle,
    name: &str,
    filter: &str,
    extensions: &[&str],
) -> Option<PathBuf> {
    imp::save_file(app, name, filter, extensions)
}

/// The platform key of `cores.lock` this build ships cores for (`windows-x86_64`,
/// `linux-x86_64`, `android-arm64-v8a`); `None` where none is packaged (macOS, another CPU).
pub const CORES_PLATFORM: Option<&str> = imp::CORES_PLATFORM;

/// Puts `window` in fullscreen or takes it out. Android has no such call (Tauri's mobile window):
/// there the app already fills the screen, so it does nothing and succeeds.
pub fn set_fullscreen(window: &tauri::Window, on: bool) -> tauri::Result<()> {
    imp::set_fullscreen(window, on)
}

/// The main window's size when EmuDome starts, and its smallest, in logical pixels (desktop).
pub const WINDOW_SIZE: (f64, f64) = (1280.0, 800.0);
pub const WINDOW_MIN_SIZE: (f64, f64) = (640.0, 480.0);

/// How the window shows (`video.window`, desktop only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    /// A normal window with its frame.
    Windowed,
    /// The system's fullscreen (as F11).
    Fullscreen,
    /// No frame, covering the whole monitor, without the system's fullscreen state.
    Borderless,
}

/// How large the window's inside may be on its monitor now, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowRoom {
    /// The monitor's work area less the window's frame.
    pub max_inner: (u32, u32),
    /// [`WINDOW_MIN_SIZE`] at the monitor's scale.
    pub min_inner: (u32, u32),
    /// The monitor's scale factor (physical pixels per logical one).
    pub scale: f64,
    pub maximized: bool,
    /// In the system's fullscreen (`video.window`, or F11).
    pub fullscreen: bool,
}

/// Where the window can grow; `None` where it cannot be resized (Android) or the monitor is not
/// known.
pub fn window_room(window: &tauri::Window) -> Option<WindowRoom> {
    imp::window_room(window)
}

/// Shows the window as `mode` says. Android: nothing (the app fills the screen).
pub fn set_window_mode(window: &tauri::Window, mode: WindowMode) -> tauri::Result<()> {
    imp::set_window_mode(window, mode)
}

/// Gives the window's inside `size` physical pixels (un-maximizing it), moved back into its
/// monitor's work area when it would stick out. Android: nothing.
pub fn set_window_inner_size(window: &tauri::Window, size: (u32, u32)) -> tauri::Result<()> {
    imp::set_window_inner_size(window, size)
}

/// A `content://` document opened for a core ([`open_content`]): its descriptor (read through
/// [`ContentFile::reader`]) and a path with the document's name and extension, valid while this
/// value lives. Keep it until the core is done with the game; dropping it closes the document and
/// removes the path.
pub struct ContentFile {
    path: PathBuf,
    name: Option<String>,
    /// The link, removed on drop (declared before `file`: dropped first, so a new descriptor
    /// with the same number never finds it).
    _hold: Box<dyn Send>,
    /// The provider's descriptor, read through duplicates of it ([`ContentFile::reader`]).
    file: std::fs::File,
    /// Whether [`ContentFile::path`] can be opened (Android 17 refuses it for some providers).
    reopenable: bool,
}

impl ContentFile {
    /// A path named after the document (its name and extension: the console is chosen by it).
    /// It opens the document only when [`ContentFile::reopenable`]: read it with
    /// [`ContentFile::reader`] instead.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The document's display name (its file name, as the provider tells it).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The document for reading, at its start: a duplicate of the provider's descriptor. The
    /// duplicates share one position, so read them one at a time (each call seeks to the start).
    pub fn reader(&self) -> std::io::Result<std::fs::File> {
        use std::io::{Seek, SeekFrom};
        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(0))?;
        Ok(file)
    }

    /// Whether [`ContentFile::path`] opens the document (what a core that reads the file by
    /// itself needs). False where the system refuses to open the descriptor again by its path.
    pub fn reopenable(&self) -> bool {
        self.reopenable
    }
}

/// One child of a folder of a picked document tree ([`list_document_tree`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    /// The child's document URI: how it is opened ([`open_content`]) and listed (a folder).
    pub uri: String,
    /// Its display name (file name with extension).
    pub name: String,
    /// Bytes; 0 when the provider does not say.
    pub size: u64,
    /// Last modified, Unix ms; 0 when the provider does not say.
    pub mtime: u64,
    pub dir: bool,
}

/// What the system's picker gave ([`pick_document_tree`], [`pick_document`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedDocument {
    /// The `content://` URI of the tree or document: what is stored, never turned into a path.
    pub uri: String,
    /// Its display name, as the provider tells it.
    pub name: Option<String>,
}

/// True where library folders and game files are picked through the system's document pickers
/// and are `content://` URIs (Android's Storage Access Framework, Trello #47); false where they
/// are paths from the file dialog (the desktop).
pub const DOCUMENT_PICKERS: bool = imp::DOCUMENT_PICKERS;

/// Shows the system's folder picker (Android: `ACTION_OPEN_DOCUMENT_TREE`, the read permission
/// persisted across restarts) and waits for the answer: `Ok(None)` when cancelled. Blocks until
/// the user answers: never call it on the main thread. [`ContentError::Unsupported`] on the
/// desktop.
pub fn pick_document_tree() -> Result<Option<PickedDocument>, ContentError> {
    imp::pick_document_tree()
}

/// Shows the system's file picker for one document (Android: `ACTION_OPEN_DOCUMENT`, every file
/// offered: ROMs have no registered MIME type; the read permission persisted). Blocks until the
/// user answers. [`ContentError::Unsupported`] on the desktop.
pub fn pick_document() -> Result<Option<PickedDocument>, ContentError> {
    imp::pick_document()
}

/// The children of one folder of the picked tree `tree`: its root when `dir` is `None`, else the
/// folder whose document URI (a [`TreeEntry::uri`]) is `dir`. One call into the system per
/// folder. [`ContentError::Unreadable`] when it cannot be listed (permission revoked, folder
/// gone). Blocks on the provider.
pub fn list_document_tree(tree: &str, dir: Option<&str>) -> Result<Vec<TreeEntry>, ContentError> {
    imp::list_document_tree(tree, dir)
}

/// The display name of the picked tree `tree` (its root folder's name); `None` when it cannot be
/// read. Blocks on the provider.
pub fn document_tree_name(tree: &str) -> Option<String> {
    imp::document_tree_name(tree)
}

/// Gives back the persisted read permission of the tree `tree` (a folder removed from the
/// library). Nothing on the desktop.
pub fn release_document_tree(tree: &str) {
    imp::release_document_tree(tree);
}

/// The display name of the `content://` document `uri` (its file name, as the provider tells
/// it); `None` when it cannot be read, and on the desktop. Blocks on the provider.
pub fn document_name(uri: &str) -> Option<String> {
    imp::document_name(uri)
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
/// the link named after it goes under `cache_dir` (the app's private cache). Blocks on the
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

/// A short vibration for a touch control's press (Trello #29, `touch.thapt`), `strength` 1–100.
/// Android: the system vibrator, its amplitude from `strength` (the device's own haptics setting
/// still applies); a device without one, or a failure, only logs. The desktop has no vibrator:
/// nothing happens. Returns at once (the vibration plays by itself).
pub fn haptic(strength: u8) {
    imp::haptic(strength);
}

/// Listener of the external surface, boxed for the per-OS modules.
pub(crate) type SurfaceListener = Box<dyn Fn(SurfaceEvent) + Send + Sync>;

/// Listener of the back button, boxed for the per-OS modules.
pub(crate) type BackListener = Box<dyn Fn() + Send + Sync>;
