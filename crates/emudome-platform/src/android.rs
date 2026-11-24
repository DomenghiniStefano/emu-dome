//! Android.
//!
//! - Cores: `.so` files packaged in the APK (`cores/android/<abi>/`, a jniLibs folder of
//!   the Gradle library in `android/`), loaded by file name through the dynamic loader.
//! - Video: a `SurfaceView` under the transparent WebView (Kotlin class
//!   `it.domehub.emudome.platform.NativeSurface`, in `android/`). Its `Surface` reaches
//!   Rust through JNI and becomes an `ANativeWindow` for wgpu.
//! - Logs: logcat, tag `EmuDome`.
//! - The system's requests (Kotlin class `it.domehub.emudome.platform.SystemBridge`): the back
//!   button reaches the listener of [`crate::listen_back`]; the game of «Open with» and the
//!   `content://` games ([`crate::open_content`], `content.rs`), the system's folder and file
//!   pickers and the listing of a picked folder (Storage Access Framework, Trello #47).
//! - Controllers (Kotlin class `it.domehub.emudome.platform.GamepadInput`, attached with the
//!   system bridge): `pads/android.rs`.
use std::ffi::c_void;
use std::fs;
use std::os::fd::{FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Once};
use std::time::Duration;

use jni::objects::{GlobalRef, JClass, JObject, JObjectArray, JString, JValue};
use jni::sys::jint;
use jni::{JNIEnv, JavaVM, NativeMethod};
use ndk::native_window::NativeWindow;
use parking_lot::{Condvar, Mutex};
use raw_window_handle::{DisplayHandle, HandleError};

use crate::content::{fd_path, link_dir, link_name, parse_tree_children, LINKS_DIR};
use crate::{
    BackListener, ContentError, ContentFile, DataDirs, ExternalSurface, PickedDocument,
    SurfaceEvent, SurfaceListener, TreeEntry, WindowMode, WindowOptions, WindowRoom,
};

pub const OS_NAME: &str = "Android";

/// Packaged as `lib<core>_libretro_android.so`: the `lib` prefix is what Android
/// expects of a native library in the APK.
pub fn core_file_name(core: &str) -> String {
    format!("lib{core}_libretro_android.so")
}

/// The cores are loaded by name from the APK (signed as a whole): no checksum of our own.
pub const CORES_LOCK_PLATFORM: Option<&str> = None;

/// The platform key of `cores.lock` the APK's cores come from (the Gradle build packs them).
pub const CORES_PLATFORM: Option<&str> = if cfg!(target_arch = "aarch64") {
    Some("android-arm64-v8a")
} else {
    None
};

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// The dynamic loader finds the APK's libraries by name.
pub fn packaged_core(name: &str) -> Option<PathBuf> {
    Some(PathBuf::from(name))
}

/// The old identifier is another package: its private storage (`/data/user/0/<package>/`) is
/// readable by that package only, so the new one cannot copy from it. The old app's data does
/// not carry over on Android (it stays with the old app until that is uninstalled).
pub fn old_data_dirs(_data: &Path, _local: &Path) -> Option<DataDirs> {
    None
}

pub const WEBVIEW_PROFILE_DIRS: &[&str] = &[];

/// Azahar's default user folder: its libretro build on Android (`HAVE_LIBRETRO_VFS`) takes the
/// generic POSIX branch, the working folder then the XDG data folder. Not checked on a device:
/// an app process may have no `HOME`, and then only the working folder is looked at.
pub const AZAHAR_LAYOUT: crate::azahar::Layout = crate::azahar::Layout::Xdg;

/// Game drawn natively under the transparent webview, on the `SurfaceView`.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
pub const EXTERNAL_SURFACE: bool = true;
pub const PHONE: bool = true;

/// The user's game folders are picked trees (`content://`): no file manager is opened on them.
pub const OPEN_FOLDER: bool = false;

pub fn open_folder(_folder: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no file manager is opened on Android",
    ))
}

pub fn init_logging() {
    let logger = android_logger::AndroidLogger::new(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("EmuDome"),
    );
    // Also into the log file (`log_file.rs`), beside logcat.
    if log::set_boxed_logger(Box::new(crate::log_file::Tee::new(Box::new(logger)))).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

/// The launch intent's game (`SystemBridge.startupGame`: the `game` extra, else the document of
/// «Open with»), read when the system bridge is attached ([`attach_system`]).
/// `None` inside means "not read yet".
static STARTUP_GAME: Mutex<Option<Option<String>>> = Mutex::new(None);
static STARTUP_READ: Condvar = Condvar::new();

pub fn startup_game() -> Option<String> {
    let mut game = STARTUP_GAME.lock();
    if game.is_none() {
        // The intent is read on the UI thread right after the webview is created:
        // the page normally asks later, this only covers a slow start.
        STARTUP_READ.wait_for(&mut game, Duration::from_secs(2));
    }
    game.clone().flatten()
}

fn set_startup_game(game: Option<String>) {
    *STARTUP_GAME.lock() = Some(game);
    STARTUP_READ.notify_all();
}

pub type SurfaceWindow = NativeWindow;

/// Android has a single, implicit display.
pub fn surface_display_handle(_window: &SurfaceWindow) -> Result<DisplayHandle<'_>, HandleError> {
    Ok(DisplayHandle::android())
}

/// Who receives the surface events, and whether a surface is current. The listener is called
/// after the lock is let go (a clone of the `Arc`): `Lost` blocks until the renderer is done with
/// the surface, and nothing may wait on this lock meanwhile.
struct SurfaceState {
    listener: Option<Arc<dyn Fn(SurfaceEvent) + Send + Sync>>,
    current: bool,
}

static SURFACE: Mutex<SurfaceState> = Mutex::new(SurfaceState {
    listener: None,
    current: false,
});

const NATIVE_SURFACE_CLASS: &str = "it.domehub.emudome.platform.NativeSurface";

pub fn attach_external_surface<R: tauri::Runtime>(
    webview: &tauri::Webview<R>,
    on_event: SurfaceListener,
) {
    SURFACE.lock().listener = Some(Arc::from(on_event));
    let result = webview.with_webview(|platform| {
        platform.jni_handle().exec(|env, activity, webview| {
            if let Err(e) = attach_jni(env, activity, webview) {
                log::error!("native surface not attached: {e}");
                clear_exception(env);
            }
        })
    });
    if let Err(e) = result {
        log::error!("webview not reachable for the native surface: {e}");
    }
}

/// Attaches `SystemBridge` on the UI thread (the back button to `on_back`, the launch intent's
/// game for [`startup_game`], the documents). Whatever the video path: the webview mode has no
/// native surface but has the back button.
pub fn attach_system<R: tauri::Runtime>(webview: &tauri::Webview<R>, on_back: BackListener) {
    *BACK.lock() = Some(Arc::from(on_back));
    let result = webview.with_webview(|platform| {
        platform.jni_handle().exec(|env, activity, _webview| {
            let loader = env
                .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
                .and_then(|loader| loader.l());
            let loader = match loader {
                Ok(loader) => loader,
                Err(e) => {
                    log::error!("no class loader for the system bridge and the controllers: {e}");
                    clear_exception(env);
                    set_startup_game(None);
                    crate::pads::android::attach_failed(e.to_string());
                    return;
                }
            };
            match attach_bridge(env, activity, &loader) {
                Ok(game) => set_startup_game(game),
                Err(e) => {
                    log::error!("system bridge not attached (back button, documents): {e}");
                    clear_exception(env);
                    set_startup_game(None);
                }
            }
            // The controllers on their own: a failure there leaves the back button working.
            crate::pads::android::attach(env, activity, &loader);
        })
    });
    if let Err(e) = result {
        log::error!("webview not reachable for the system bridge: {e}");
        set_startup_game(None);
        crate::pads::android::attach_failed(e.to_string());
    }
}

/// A pending Java exception is printed to logcat and cleared, so the next JNI call works.
pub(crate) fn clear_exception(env: &mut JNIEnv) {
    if env.exception_check().unwrap_or(false) {
        // Best effort: nothing more can be done about an exception that cannot be printed.
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
}

/// An app class, through the activity's class loader (FindClass would use the system one when
/// called from native code).
pub(crate) fn load_class<'local>(
    env: &mut JNIEnv<'local>,
    loader: &JObject,
    name: &str,
) -> jni::errors::Result<JClass<'local>> {
    let name = env.new_string(name)?;
    Ok(env
        .call_method(
            loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?
        .into())
}

/// Runs on the UI thread: registers the native methods of `NativeSurface` and asks it to put the
/// `SurfaceView` under the WebView.
fn attach_jni(env: &mut JNIEnv, activity: &JObject, webview: &JObject) -> jni::errors::Result<()> {
    let loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;
    let class = load_class(env, &loader, NATIVE_SURFACE_CLASS)?;
    env.register_native_methods(
        &class,
        &[
            NativeMethod {
                name: "nativeSurfaceChanged".into(),
                sig: "(Landroid/view/Surface;II)V".into(),
                fn_ptr: native_surface_changed as *mut c_void,
            },
            NativeMethod {
                name: "nativeSurfaceDestroyed".into(),
                sig: "()V".into(),
                fn_ptr: native_surface_destroyed as *mut c_void,
            },
        ],
    )?;
    env.call_static_method(
        &class,
        "attach",
        "(Landroid/app/Activity;Landroid/webkit/WebView;)V",
        &[JValue::Object(activity), JValue::Object(webview)],
    )?;
    log::info!("native surface: SurfaceView under the WebView");
    Ok(())
}

// ---- SystemBridge: the back button, the launch intent's game, `content://` documents ----

const SYSTEM_BRIDGE_CLASS: &str = "it.domehub.emudome.platform.SystemBridge";

/// What the document calls need from any thread: the VM, the activity (a `Context`) and the
/// bridge class (loaded once through the app's class loader, which a native thread cannot reach).
struct Java {
    vm: JavaVM,
    activity: GlobalRef,
    bridge: GlobalRef,
}

static JAVA: Mutex<Option<Arc<Java>>> = Mutex::new(None);

/// Who receives the back button; called after the lock is let go (a clone of the `Arc`).
static BACK: Mutex<Option<Arc<dyn Fn() + Send + Sync>>> = Mutex::new(None);

/// Registers the bridge's native method, keeps what [`with_java`] needs, adds the back callback,
/// and returns the launch intent's game.
fn attach_bridge(
    env: &mut JNIEnv,
    activity: &JObject,
    loader: &JObject,
) -> jni::errors::Result<Option<String>> {
    let class = load_class(env, loader, SYSTEM_BRIDGE_CLASS)?;
    env.register_native_methods(
        &class,
        &[
            NativeMethod {
                name: "nativeBackPressed".into(),
                sig: "()V".into(),
                fn_ptr: native_back_pressed as *mut c_void,
            },
            NativeMethod {
                name: "nativeDocumentPicked".into(),
                sig: "(Ljava/lang/String;Ljava/lang/String;)V".into(),
                fn_ptr: native_document_picked as *mut c_void,
            },
        ],
    )?;
    *JAVA.lock() = Some(Arc::new(Java {
        vm: env.get_java_vm()?,
        activity: env.new_global_ref(activity)?,
        bridge: env.new_global_ref(&class)?,
    }));
    env.call_static_method(
        &class,
        "attach",
        "(Landroid/app/Activity;)V",
        &[JValue::Object(activity)],
    )?;
    let game = env
        .call_static_method(
            &class,
            "startupGame",
            "(Landroid/app/Activity;)Ljava/lang/String;",
            &[JValue::Object(activity)],
        )?
        .l()?;
    optional_string(env, game)
}

/// A Java `String` that may be null.
fn optional_string(env: &mut JNIEnv, value: JObject) -> jni::errors::Result<Option<String>> {
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(env.get_string(&JString::from(value))?.into()))
}

/// `SystemBridge.nativeBackPressed`, on the UI thread.
extern "system" fn native_back_pressed(_env: JNIEnv, _class: JClass) {
    let listener = BACK.lock().clone();
    match listener {
        Some(on_back) => on_back(),
        None => log::info!("back pressed with no listener: ignored"),
    }
}

/// `SystemBridge.moveToBack`: the task behind the others, the app still running (back at the
/// shell's root, Trello #29).
pub fn move_to_background() -> bool {
    let moved = with_java("move to the background", |env, activity, bridge| {
        env.call_static_method(
            bridge,
            "moveToBack",
            "(Landroid/app/Activity;)V",
            &[JValue::Object(activity)],
        )
        .map(|_| ())
    });
    match moved {
        Ok(()) => true,
        Err(e) => {
            log::warn!("{e}");
            false
        }
    }
}

/// `SystemBridge.setImmersive`: the system bars hidden while a game is open, shown in the shell
/// (Trello #131).
pub fn set_immersive(on: bool) {
    let result = with_java("system bars", |env, activity, bridge| {
        env.call_static_method(
            bridge,
            "setImmersive",
            "(Landroid/app/Activity;Z)V",
            &[JValue::Object(activity), JValue::Bool(u8::from(on))],
        )
        .map(|_| ())
    });
    match result {
        Ok(()) => log::info!(
            "android system bars {}",
            if on { "hidden" } else { "shown" }
        ),
        Err(e) => log::warn!("{e}"),
    }
}

/// Runs `call` with a JNI environment on this thread (attached for the call when it is not a
/// Java thread), the activity and the bridge class. A Java exception is cleared and returned as
/// the error, prefixed with `what`.
fn with_java<T>(
    what: &str,
    call: impl FnOnce(&mut JNIEnv, &JObject, &JClass) -> jni::errors::Result<T>,
) -> Result<T, String> {
    let java = JAVA
        .lock()
        .clone()
        .ok_or_else(|| format!("{what}: the activity is not attached yet"))?;
    let mut env = java
        .vm
        .attach_current_thread()
        .map_err(|e| format!("{what}: {e}"))?;
    let bridge: &JClass = java.bridge.as_obj().into();
    let result = call(&mut env, java.activity.as_obj(), bridge);
    if result.is_err() {
        clear_exception(&mut env);
    }
    result.map_err(|e| format!("{what}: {e}"))
}

/// The links of the documents a killed process left are removed once, at the first open.
static LINKS_CLEANED: Once = Once::new();

pub fn open_content(uri: &str, cache_dir: &Path) -> Result<ContentFile, ContentError> {
    let (fd, name) = with_java("document", |env, activity, bridge| {
        let juri = env.new_string(uri)?;
        let args = [JValue::Object(activity), JValue::Object(&juri)];
        let fd = env
            .call_static_method(
                bridge,
                "openDocument",
                "(Landroid/content/Context;Ljava/lang/String;)I",
                &args,
            )?
            .i()?;
        let name = env
            .call_static_method(
                bridge,
                "displayName",
                "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;",
                &args,
            )?
            .l()?;
        Ok((fd, optional_string(env, name)?))
    })
    .map_err(ContentError::Unreadable)?;
    if fd < 0 {
        return Err(ContentError::Unreadable(format!(
            "the provider did not open {uri} (logcat, tag EmuDome)"
        )));
    }
    // SAFETY: `detachFd` handed the descriptor over: nothing else owns or closes it.
    let file = fs::File::from(unsafe { OwnedFd::from_raw_fd(fd) });
    let meta = file
        .metadata()
        .map_err(|e| ContentError::Unreadable(format!("{uri}: {e}")))?;
    if !meta.is_file() {
        return Err(ContentError::NotLocal);
    }
    // EmuDome reads the descriptor itself; only a core that opens the game by itself uses the
    // path, which works only where the system lets the descriptor be opened again (Android 17
    // refuses it for the Downloads provider: `Permission denied`).
    let source = fd_path(fd);
    let reopenable = match fs::File::open(&source) {
        Ok(_) => true,
        Err(e) => {
            log::info!(
                "document {uri}: {} cannot be opened again ({e}): read through its descriptor",
                source.display()
            );
            false
        }
    };

    LINKS_CLEANED.call_once(|| {
        // Only links live there (removed, never followed); a missing folder is the usual case.
        let leftovers = cache_dir.join(LINKS_DIR);
        if let Err(e) = fs::remove_dir_all(&leftovers) {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("old document links in {}: {e}", leftovers.display());
            }
        }
    });
    let dir = link_dir(cache_dir, fd);
    let link = dir.join(link_name(name.as_deref()));
    fs::create_dir_all(&dir)
        .and_then(|()| std::os::unix::fs::symlink(&source, &link))
        .map_err(|e| ContentError::Unreadable(format!("{}: {e}", link.display())))?;
    log::info!(
        "document {uri} opened: name {name:?}, {} bytes, fd {fd}, reopenable by path {reopenable}",
        meta.len()
    );
    Ok(ContentFile {
        path: link.clone(),
        name,
        _hold: Box::new(OpenDocument { link, dir }),
        file,
        reopenable,
    })
}

/// One pick at a time: the sender of the pick waiting for `nativeDocumentPicked`. A new pick
/// replaces it (the old one's receiver then sees a cancel).
static PICKED: Mutex<Option<std::sync::mpsc::Sender<Option<PickedDocument>>>> = Mutex::new(None);

/// How long a pick waits for the user before it counts as cancelled (the thread is let go).
const PICK_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// `SystemBridge.nativeDocumentPicked`: a picker's answer, on a background thread.
extern "system" fn native_document_picked(
    mut env: JNIEnv,
    _class: JClass,
    uri: JObject,
    name: JObject,
) {
    let read =
        optional_string(&mut env, uri).and_then(|uri| Ok((uri, optional_string(&mut env, name)?)));
    let picked = match read {
        Ok((Some(uri), name)) => Some(PickedDocument { uri, name }),
        Ok((None, _)) => None,
        Err(e) => {
            log::warn!("picked document not read: {e}");
            clear_exception(&mut env);
            None
        }
    };
    match PICKED.lock().take() {
        Some(tx) => {
            // The receiver may have timed out: nobody to tell then.
            let _ = tx.send(picked);
        }
        None => log::warn!("a picker answered with no pick waiting: {picked:?}"),
    }
}

/// Shows a picker (`SystemBridge.<method>(Activity)`) and waits for its answer.
fn pick(what: &str, method: &str) -> Result<Option<PickedDocument>, ContentError> {
    let (tx, rx) = std::sync::mpsc::channel();
    *PICKED.lock() = Some(tx);
    with_java(what, |env, activity, bridge| {
        env.call_static_method(
            bridge,
            method,
            "(Landroid/app/Activity;)V",
            &[JValue::Object(activity)],
        )
        .map(|_| ())
    })
    .map_err(ContentError::Unreadable)?;
    match rx.recv_timeout(PICK_TIMEOUT) {
        Ok(picked) => Ok(picked),
        Err(e) => {
            log::warn!("{what}: no answer ({e}), taken as cancelled");
            Ok(None)
        }
    }
}

pub const DOCUMENT_PICKERS: bool = true;

pub fn pick_document_tree() -> Result<Option<PickedDocument>, ContentError> {
    let picked = pick("folder picker", "pickTree")?;
    match &picked {
        Some(p) => log::info!("android folder picked: {:?} {}", p.name, p.uri),
        None => log::info!("android folder picker cancelled"),
    }
    Ok(picked)
}

pub fn pick_document() -> Result<Option<PickedDocument>, ContentError> {
    let picked = pick("file picker", "pickDocument")?;
    match &picked {
        Some(p) => log::info!("android file picked: {:?} {}", p.name, p.uri),
        None => log::info!("android file picker cancelled"),
    }
    Ok(picked)
}

pub fn list_document_tree(tree: &str, dir: Option<&str>) -> Result<Vec<TreeEntry>, ContentError> {
    let fields = with_java("folder listing", |env, activity, bridge| {
        let jtree = env.new_string(tree)?;
        let jdir = match dir {
            Some(dir) => JObject::from(env.new_string(dir)?),
            None => JObject::null(),
        };
        let answer = env
            .call_static_method(
                bridge,
                "listTree",
                "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)[Ljava/lang/String;",
                &[
                    JValue::Object(activity),
                    JValue::Object(&jtree),
                    JValue::Object(&jdir),
                ],
            )?
            .l()?;
        if answer.is_null() {
            return Ok(None);
        }
        let array = JObjectArray::from(answer);
        let len = env.get_array_length(&array)?;
        let mut fields = Vec::with_capacity(usize::try_from(len).unwrap_or_default());
        for i in 0..len {
            let element = JString::from(env.get_object_array_element(&array, i)?);
            let text: String = env.get_string(&element)?.into();
            // One local reference per element: freed now, a folder may hold thousands.
            env.delete_local_ref(element)?;
            fields.push(text);
        }
        Ok(Some(fields))
    })
    .map_err(ContentError::Unreadable)?;
    match fields {
        Some(fields) => Ok(parse_tree_children(&fields)),
        None => Err(ContentError::Unreadable(format!(
            "{tree}: the folder could not be listed (permission given back, or the folder is \
             gone; logcat, tag EmuDome)"
        ))),
    }
}

/// `SystemBridge.<method>(Context, String): String?`, logged on failure.
fn query_string(what: &str, method: &str, uri: &str) -> Option<String> {
    let result = with_java(what, |env, activity, bridge| {
        let juri = env.new_string(uri)?;
        let name = env
            .call_static_method(
                bridge,
                method,
                "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(activity), JValue::Object(&juri)],
            )?
            .l()?;
        optional_string(env, name)
    });
    result.unwrap_or_else(|e| {
        log::warn!("{e}");
        None
    })
}

pub fn document_tree_name(tree: &str) -> Option<String> {
    query_string("folder name", "treeName", tree)
}

pub fn document_name(uri: &str) -> Option<String> {
    query_string("document name", "displayName", uri)
}

pub fn document_tree_permitted(tree: &str) -> Option<bool> {
    let result = with_java("folder permission", |env, activity, bridge| {
        let jtree = env.new_string(tree)?;
        env.call_static_method(
            bridge,
            "treePermitted",
            "(Landroid/content/Context;Ljava/lang/String;)Z",
            &[JValue::Object(activity), JValue::Object(&jtree)],
        )?
        .z()
    });
    result.map_err(|e| log::warn!("{e}")).ok()
}

pub fn release_document_tree(tree: &str) {
    let result = with_java("folder permission", |env, activity, bridge| {
        let jtree = env.new_string(tree)?;
        env.call_static_method(
            bridge,
            "releaseTree",
            "(Landroid/content/Context;Ljava/lang/String;)V",
            &[JValue::Object(activity), JValue::Object(&jtree)],
        )
        .map(|_| ())
    });
    match result {
        Ok(()) => log::info!("android folder permission given back: {tree}"),
        Err(e) => log::warn!("{e}"),
    }
}

/// How long a touch control's press vibrates.
const HAPTIC_MS: i64 = 18;

/// `SystemBridge.vibrate`: a one-shot vibration, its amplitude (1–255) from `strength` (1–100).
pub fn haptic(strength: u8) {
    let amplitude = (i32::from(strength.clamp(1, 100)) * 255 + 99) / 100;
    let result = with_java("vibration", |env, activity, bridge| {
        env.call_static_method(
            bridge,
            "vibrate",
            "(Landroid/content/Context;JI)V",
            &[
                JValue::Object(activity),
                JValue::Long(HAPTIC_MS),
                JValue::Int(amplitude),
            ],
        )?;
        Ok(())
    });
    if let Err(e) = result {
        log::warn!("{e}");
    }
}

/// The link of a document opened for a game: removed before the descriptor closes (the
/// `ContentFile` drops it first), so a new descriptor with the same number never finds it.
struct OpenDocument {
    link: PathBuf,
    dir: PathBuf,
}

impl Drop for OpenDocument {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.link) {
            log::warn!("document link {}: {e}", self.link.display());
        }
        // Empty now; if it is not, the next process's first open removes it.
        if let Err(e) = fs::remove_dir(&self.dir) {
            log::warn!("document folder {}: {e}", self.dir.display());
        }
    }
}

/// `SurfaceHolder.Callback.surfaceChanged`: the first call after a creation hands
/// the surface over, the next ones are resizes.
extern "system" fn native_surface_changed(
    env: JNIEnv,
    _class: JClass,
    surface: JObject,
    width: jint,
    height: jint,
) {
    let (width, height) = (width.max(1) as u32, height.max(1) as u32);
    let (event, listener) = {
        let mut state = SURFACE.lock();
        let event = if state.current {
            SurfaceEvent::Resized { width, height }
        } else {
            // SAFETY: `env` and `surface` are the valid JNI arguments of this call;
            // `from_surface` takes its own reference to the ANativeWindow.
            let window = unsafe { NativeWindow::from_surface(env.get_raw(), surface.as_raw()) };
            let Some(window) = window else {
                log::error!("ANativeWindow_fromSurface failed");
                return;
            };
            state.current = true;
            log::info!("native surface ready: {width}x{height}");
            SurfaceEvent::Ready(ExternalSurface {
                window,
                width,
                height,
            })
        };
        (event, state.listener.clone())
    };
    if let Some(listener) = listener {
        listener(event);
    }
}

/// `SurfaceHolder.Callback.surfaceDestroyed`: blocks until the renderer let go of it (Android
/// destroys the surface when this returns, so the GPU must be done with it first).
extern "system" fn native_surface_destroyed(_env: JNIEnv, _class: JClass) {
    let listener = {
        let mut state = SURFACE.lock();
        if !state.current {
            return;
        }
        state.current = false;
        state.listener.clone()
    };
    log::info!("native surface lost");
    if let Some(listener) = listener {
        listener(SurfaceEvent::Lost);
    }
}

/// `options.focused` is not used: Tauri's mobile builder has no `focused`, the activity's
/// window is the one in front (bench mode is a desktop tool).
pub fn create_window(
    app: &tauri::App,
    options: &WindowOptions,
) -> tauri::Result<(tauri::Window, tauri::Webview)> {
    let mut builder =
        tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
            .transparent(true);
    if let Some(dir) = &options.webview_data_dir {
        builder = builder.data_directory(dir.clone());
    }
    let main = builder.build()?;
    let ui: &tauri::Webview = main.as_ref();
    Ok((ui.window(), ui.clone()))
}

/// No file-path picker on Android: games and folders come from the document pickers as
/// `content://` URIs ([`pick_document`], [`pick_document_tree`]); this path picker (screen effect
/// presets) has none yet.
pub fn pick_file(_app: &tauri::AppHandle, _filter: &str, _extensions: &[&str]) -> Option<PathBuf> {
    log::warn!("file picker not available on this platform yet");
    None
}

pub fn save_file(
    _app: &tauri::AppHandle,
    _name: &str,
    _filter: &str,
    _extensions: &[&str],
) -> Option<PathBuf> {
    log::warn!("save dialog not available on this platform yet");
    None
}

/// Tauri's mobile window has no fullscreen call; the app already fills the screen.
pub fn set_fullscreen(_window: &tauri::Window, _on: bool) -> tauri::Result<()> {
    Ok(())
}

/// The activity's window fills the screen: it has no size of its own to choose.
pub fn window_room(_window: &tauri::Window) -> Option<WindowRoom> {
    None
}

pub fn set_window_mode(_window: &tauri::Window, _mode: WindowMode) -> tauri::Result<()> {
    Ok(())
}

pub fn set_window_inner_size(_window: &tauri::Window, _size: (u32, u32)) -> tauri::Result<()> {
    Ok(())
}
