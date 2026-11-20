//! Android.
//!
//! - Cores: `.so` files packaged in the APK (`cores/android/<abi>/`, a jniLibs folder of
//!   the Gradle library in `android/`), loaded by file name through the dynamic loader.
//! - Video: a `SurfaceView` under the transparent WebView (Kotlin class
//!   `com.domenghini.emudome.platform.NativeSurface`, in `android/`). Its `Surface` reaches
//!   Rust through JNI and becomes an `ANativeWindow` for wgpu.
//! - Logs: logcat, tag `EmuDome`.
use std::ffi::c_void;
use std::path::PathBuf;
use std::time::Duration;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jint;
use jni::{JNIEnv, NativeMethod};
use ndk::native_window::NativeWindow;
use parking_lot::{Condvar, Mutex};
use raw_window_handle::{DisplayHandle, HandleError};

use crate::{ExternalSurface, SurfaceEvent, SurfaceListener, WindowOptions};

pub const OS_NAME: &str = "Android";

/// Packaged as `lib<core>_libretro_android.so`: the `lib` prefix is what Android
/// expects of a native library in the APK.
pub fn core_file_name(core: &str) -> String {
    format!("lib{core}_libretro_android.so")
}

pub fn extra_core_dirs() -> Vec<PathBuf> {
    Vec::new()
}

/// The dynamic loader finds the APK's libraries by name.
pub fn packaged_core(name: &str) -> Option<PathBuf> {
    Some(PathBuf::from(name))
}

/// Game drawn natively under the transparent webview, on the `SurfaceView`.
pub const NATIVE_VIDEO_DEFAULT: bool = true;
pub const EXTERNAL_SURFACE: bool = true;
pub const PHONE: bool = true;

pub fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("EmuDome"),
    );
}

/// The `game` extra of the launch intent, read when the surface is attached.
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

/// Who receives the surface events, and whether a surface is current.
struct SurfaceState {
    listener: Option<SurfaceListener>,
    current: bool,
}

static SURFACE: Mutex<SurfaceState> = Mutex::new(SurfaceState {
    listener: None,
    current: false,
});

const NATIVE_SURFACE_CLASS: &str = "com.domenghini.emudome.platform.NativeSurface";

pub fn attach_external_surface<R: tauri::Runtime>(
    webview: &tauri::Webview<R>,
    on_event: SurfaceListener,
) {
    SURFACE.lock().listener = Some(on_event);
    let result = webview.with_webview(|platform| {
        platform.jni_handle().exec(|env, activity, webview| {
            if let Err(e) = attach_jni(env, activity, webview) {
                log::error!("native surface not attached: {e}");
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_describe();
                    let _ = env.exception_clear();
                }
                set_startup_game(None);
            }
        })
    });
    if let Err(e) = result {
        log::error!("webview not reachable for the native surface: {e}");
        set_startup_game(None);
    }
}

/// Runs on the UI thread: reads the launch intent, registers the native methods of
/// `NativeSurface` and asks it to put the `SurfaceView` under the WebView.
fn attach_jni(env: &mut JNIEnv, activity: &JObject, webview: &JObject) -> jni::errors::Result<()> {
    set_startup_game(intent_game(env, activity)?);

    // App classes are found through the activity's class loader (FindClass would use
    // the system one when called from native code).
    let loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;
    let name = env.new_string(NATIVE_SURFACE_CLASS)?;
    let class: JClass = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?
        .into();
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

fn intent_game(env: &mut JNIEnv, activity: &JObject) -> jni::errors::Result<Option<String>> {
    let intent = env
        .call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])?
        .l()?;
    if intent.is_null() {
        return Ok(None);
    }
    let key = env.new_string("game")?;
    let value = env
        .call_method(
            &intent,
            "getStringExtra",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&key)],
        )?
        .l()?;
    if value.is_null() {
        return Ok(None);
    }
    let value = JString::from(value);
    let game: String = env.get_string(&value)?.into();
    Ok(Some(game))
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
    if let Some(listener) = &state.listener {
        listener(event);
    }
}

/// `SurfaceHolder.Callback.surfaceDestroyed`: blocks until the renderer let go of it.
extern "system" fn native_surface_destroyed(_env: JNIEnv, _class: JClass) {
    let mut state = SURFACE.lock();
    if !state.current {
        return;
    }
    state.current = false;
    log::info!("native surface lost");
    if let Some(listener) = &state.listener {
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

/// No file-path picker on Android yet (TODO with the SAF).
pub fn pick_file(_app: &tauri::AppHandle, _filter: &str, _extensions: &[&str]) -> Option<PathBuf> {
    log::warn!("file picker not available on this platform yet");
    None
}
