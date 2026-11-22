//! Android's controllers (Trello #47): the JNI side of `GamepadInput.kt`, and the [`Source`] the
//! app's controller thread reads.
//!
//! Kotlin wraps the activity window's `Window.Callback`, so it sees every `KeyEvent` and
//! `MotionEvent` before the views (the WebView would eat the d-pad and the face buttons): those
//! of a gamepad or joystick device go to the native methods here, everything else (touch, the
//! phone's own back key, the keyboard, IME) goes on to the system untouched. A key EmuDome does
//! not read is handed back too ([`super::android_map::key_target`]). Devices come and go through
//! `InputManager.InputDeviceListener`.
//!
//! The native methods run on the UI thread and only write [`PADS`] (a short lock); the
//! controller thread polls it every few milliseconds, as it polls SDL on the desktop.
//!
//! Logs (logcat, tag `EmuDome`): `android pad connected: <name> <vid>:<pid> ...` and
//! `android pad disconnected` at info; every key and every coarse change of a pad's reading at
//! info in a debug build (the APK the owner tests), at debug in a release one.

use std::ffi::c_void;
use std::time::Duration;

use jni::objects::{JClass, JFloatArray, JObject, JString, JValue};
use jni::sys::{jboolean, jint, JNI_FALSE, JNI_TRUE};
use jni::{JNIEnv, NativeMethod};
use parking_lot::{Condvar, Mutex};

use super::android_map::{self, AxisLayout, Pads, AXES};
use super::{PadDevice, PadEvent, PadReading};

pub const NAME: &str = "Android input";

const GAMEPAD_INPUT_CLASS: &str = "it.domehub.emudome.platform.GamepadInput";

/// The connected pads, written by the JNI calls, read by [`Source`].
static PADS: Mutex<Pads> = Mutex::new(Pads::new());

/// How attaching `GamepadInput` went: `None` until it ran (on the UI thread, after the webview).
static ATTACHED: Mutex<Option<Result<(), String>>> = Mutex::new(None);
static ATTACH_DONE: Condvar = Condvar::new();

/// A line per key and per coarse change: visible in a debug build's logcat, hidden in a release.
macro_rules! pad_trace {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            log::info!($($arg)*);
        } else {
            log::debug!($($arg)*);
        }
    };
}

/// Registers `GamepadInput`'s native methods and attaches it to the activity (its window's
/// callback, the device listener, the pads already connected). On the UI thread, from
/// `attach_system`. A failure is logged and kept for [`Source::open`].
pub(crate) fn attach(env: &mut JNIEnv, activity: &JObject, loader: &JObject) {
    let result = attach_jni(env, activity, loader).map_err(|e| e.to_string());
    if let Err(e) = &result {
        log::error!("controllers not attached: {e}");
        crate::android::clear_exception(env);
    }
    *ATTACHED.lock() = Some(result);
    ATTACH_DONE.notify_all();
}

/// The activity could not be reached at all: [`Source::open`] says why.
pub(crate) fn attach_failed(detail: String) {
    *ATTACHED.lock() = Some(Err(detail));
    ATTACH_DONE.notify_all();
}

fn attach_jni(env: &mut JNIEnv, activity: &JObject, loader: &JObject) -> jni::errors::Result<()> {
    let class = crate::android::load_class(env, loader, GAMEPAD_INPUT_CLASS)?;
    env.register_native_methods(
        &class,
        &[
            NativeMethod {
                name: "nativePadAdded".into(),
                sig: "(ILjava/lang/String;Ljava/lang/String;II[FIZ)V".into(),
                fn_ptr: native_pad_added as *mut c_void,
            },
            NativeMethod {
                name: "nativePadRemoved".into(),
                sig: "(I)V".into(),
                fn_ptr: native_pad_removed as *mut c_void,
            },
            NativeMethod {
                name: "nativePadKey".into(),
                sig: "(IIZ)Z".into(),
                fn_ptr: native_pad_key as *mut c_void,
            },
            NativeMethod {
                name: "nativePadMotion".into(),
                sig: "(I[F)Z".into(),
                fn_ptr: native_pad_motion as *mut c_void,
            },
        ],
    )?;
    let len = i32::try_from(AXES.len()).unwrap_or(i32::MAX);
    let axes = env.new_int_array(len)?;
    env.set_int_array_region(&axes, 0, &AXES)?;
    env.call_static_method(
        &class,
        "attach",
        "(Landroid/app/Activity;[I)V",
        &[JValue::Object(activity), JValue::Object(&axes)],
    )?;
    log::info!("controllers: Android input attached");
    Ok(())
}

/// Android's controllers for the controller thread.
pub struct Source;

impl Source {
    /// Waits a little for `GamepadInput` to be attached (the UI thread does it right after the
    /// webview, usually before this): `Err` when attaching failed. A slow attach is not an
    /// error: the pads arrive when it is done.
    pub fn open() -> Result<Source, String> {
        let mut attached = ATTACHED.lock();
        if attached.is_none() {
            ATTACH_DONE.wait_for(&mut attached, Duration::from_secs(2));
        }
        match attached.as_ref() {
            Some(Err(e)) => Err(e.clone()),
            Some(Ok(())) => Ok(Source),
            None => {
                log::warn!("controllers: Android input not attached yet, reading it when it is");
                Ok(Source)
            }
        }
    }

    pub fn poll(&mut self, events: &mut Vec<PadEvent>) {
        PADS.lock().take_events(events);
    }

    pub fn read(&mut self, id: u32) -> Option<PadReading> {
        PADS.lock().read(id)
    }
}

/// A Java `String` that may be null; an unreadable one is `None`.
fn optional_string(env: &mut JNIEnv, value: &JString) -> Option<String> {
    if value.is_null() {
        return None;
    }
    env.get_string(value).ok().map(Into::into)
}

/// Android device ids are positive; a negative one (a virtual device) is not a pad.
fn pad_id(id: jint) -> Option<u32> {
    u32::try_from(id).ok()
}

/// `GamepadInput.nativePadAdded`: a pad connected, or there at attach (`at_start`). `ranges`:
/// min and max of each of [`AXES`] on the device, NaN for an axis it lacks. `battery`: 0–100, or
/// -1 when not told.
extern "system" fn native_pad_added(
    mut env: JNIEnv,
    _class: JClass,
    id: jint,
    name: JString,
    descriptor: JString,
    vendor: jint,
    product: jint,
    ranges: JFloatArray,
    battery: jint,
    at_start: jboolean,
) {
    let Some(id) = pad_id(id) else {
        return;
    };
    let name = optional_string(&mut env, &name).filter(|n| !n.is_empty());
    let descriptor = optional_string(&mut env, &descriptor).unwrap_or_default();
    let mut flat = [f32::NAN; AXES.len() * 2];
    if let Err(e) = env.get_float_array_region(&ranges, 0, &mut flat) {
        log::warn!("android pad {id}: motion ranges not read: {e}");
        crate::android::clear_exception(&mut env);
    }
    let layout = AxisLayout::from_ranges(&android_map::ranges(&flat));
    // USB ids are 16 bits; Android hands them as ints.
    let vendor = u16::try_from(vendor).unwrap_or(0);
    let product = u16::try_from(product).unwrap_or(0);
    let device = PadDevice {
        id,
        kind: android_map::kind(vendor, name.as_deref()),
        name,
        type_name: "android".to_owned(),
        vendor,
        product,
        battery: android_map::battery(battery),
    };
    log::info!(
        "android pad connected: {} {vendor:04x}:{product:04x} (id {id}, descriptor {descriptor}, \
         kind {:?}, right stick {:?}, triggers {:?}, battery {:?}, at start {})",
        device.name.as_deref().unwrap_or("(unnamed)"),
        device.kind,
        layout.right.map(|(x, y)| (AXES[x], AXES[y])),
        layout.triggers.map(|(l, r)| (AXES[l], AXES[r])),
        device.battery,
        at_start != JNI_FALSE,
    );
    PADS.lock().add(device, layout, at_start != JNI_FALSE);
}

/// `GamepadInput.nativePadRemoved`.
extern "system" fn native_pad_removed(_env: JNIEnv, _class: JClass, id: jint) {
    let Some(id) = pad_id(id) else {
        return;
    };
    if PADS.lock().remove(id) {
        log::info!("android pad disconnected: id {id}");
    }
}

/// `GamepadInput.nativePadKey`: `true` when EmuDome reads the key (Kotlin consumes the event).
extern "system" fn native_pad_key(
    _env: JNIEnv,
    _class: JClass,
    id: jint,
    code: jint,
    down: jboolean,
) -> jboolean {
    let Some(id) = pad_id(id) else {
        return JNI_FALSE;
    };
    let down = down != JNI_FALSE;
    let read = PADS.lock().key(id, code, down);
    pad_trace!(
        "android pad {id}: key {code} {} ({})",
        if down { "down" } else { "up" },
        if read { "read" } else { "left to the system" }
    );
    if read {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `GamepadInput.nativePadMotion`: the values of [`AXES`]; `true` when the pad is known (Kotlin
/// consumes the event, so the system makes no d-pad keys of the stick).
extern "system" fn native_pad_motion(
    mut env: JNIEnv,
    _class: JClass,
    id: jint,
    values: JFloatArray,
) -> jboolean {
    let Some(id) = pad_id(id) else {
        return JNI_FALSE;
    };
    let mut buf = [0f32; AXES.len()];
    if let Err(e) = env.get_float_array_region(&values, 0, &mut buf) {
        log::warn!("android pad {id}: axes not read: {e}");
        crate::android::clear_exception(&mut env);
        return JNI_FALSE;
    }
    let (known, change) = {
        let mut pads = PADS.lock();
        let known = pads.motion(id, &buf);
        (known, pads.coarse_change(id))
    };
    if let Some(r) = change {
        pad_trace!(
            "android pad {id}: buttons {:#06x} axes {:?}",
            r.buttons,
            r.axes
        );
    }
    if known {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
