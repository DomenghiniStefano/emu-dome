//! C callbacks the core calls during `retro_run` and while loading.
//!
//! libretro passes no context pointer, so the context lives in a
//! `thread_local`: everything happens on the emulation thread.

use std::cell::RefCell;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_uint};
use std::sync::Arc;

use libretro_sys::{
    retro_core_option_definition, retro_core_option_v2_definition, retro_core_option_value,
    retro_core_options_intl, retro_core_options_v2, retro_core_options_v2_intl,
    retro_game_geometry, retro_system_av_info, retro_variable, RETRO_DEVICE_ID_JOYPAD_MASK,
    RETRO_DEVICE_ID_POINTER_COUNT, RETRO_DEVICE_ID_POINTER_PRESSED, RETRO_DEVICE_ID_POINTER_X,
    RETRO_DEVICE_ID_POINTER_Y, RETRO_DEVICE_JOYPAD, RETRO_DEVICE_POINTER,
    RETRO_ENVIRONMENT_EXPERIMENTAL, RETRO_ENVIRONMENT_GET_CAN_DUPE,
    RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION, RETRO_ENVIRONMENT_GET_INPUT_BITMASKS,
    RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES, RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
    RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY, RETRO_ENVIRONMENT_GET_USERNAME,
    RETRO_ENVIRONMENT_GET_VARIABLE, RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL, RETRO_ENVIRONMENT_SET_GEOMETRY,
    RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS, RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
    RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO, RETRO_ENVIRONMENT_SET_VARIABLES,
    RETRO_NUM_CORE_OPTION_VALUES_MAX, RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565,
    RETRO_PIXEL_FORMAT_XRGB8888,
};

use crate::core_options::{CoreOptions, OptionDef};
use crate::screens::Split;
use crate::session::Shared;

pub(crate) struct CallbackCtx {
    pub shared: Arc<Shared>,
    pub pixel_format: u32,
    pub system_dir: CString,
    pub save_dir: CString,
    /// When false, the produced audio is dropped (e.g. in fast forward).
    pub audio_enabled: bool,
    /// Answers to `GET_VARIABLE`: the core's definitions and the shell's values.
    pub options: CoreOptions,
    /// How the core lays its screens out in the frame (recorded on every frame).
    pub split: Split,
    /// The answer to `GET_USERNAME` (melonDS DS: the DS user name); `None`: not answered.
    pub username: Option<CString>,
}

thread_local! {
    static CTX: RefCell<Option<CallbackCtx>> = const { RefCell::new(None) };
}

pub(crate) fn install(ctx: CallbackCtx) {
    CTX.with(|c| *c.borrow_mut() = Some(ctx));
}

pub(crate) fn clear() {
    CTX.with(|c| *c.borrow_mut() = None);
}

pub(crate) fn set_audio_enabled(enabled: bool) {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            ctx.audio_enabled = enabled;
        }
    });
}

/// Changes a core option while the game runs (seen at the core's next `GET_VARIABLE_UPDATE`).
pub(crate) fn set_core_option(key: String, value: String) {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            ctx.options.set(key, value);
        }
    });
}

fn with_ctx<R>(f: impl FnOnce(&mut CallbackCtx) -> R) -> Option<R> {
    CTX.with(|c| c.borrow_mut().as_mut().map(f))
}

pub(crate) unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    let cmd = cmd & !RETRO_ENVIRONMENT_EXPERIMENTAL;
    match cmd {
        RETRO_ENVIRONMENT_GET_CAN_DUPE => {
            if !data.is_null() {
                *(data as *mut bool) = true;
            }
            true
        }
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            let fmt = *(data as *const u32);
            match fmt {
                RETRO_PIXEL_FORMAT_0RGB1555
                | RETRO_PIXEL_FORMAT_XRGB8888
                | RETRO_PIXEL_FORMAT_RGB565 => with_ctx(|c| c.pixel_format = fmt).is_some(),
                _ => false,
            }
        }
        RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => with_ctx(|c| {
            *(data as *mut *const c_char) = c.system_dir.as_ptr();
        })
        .is_some(),
        RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => with_ctx(|c| {
            *(data as *mut *const c_char) = c.save_dir.as_ptr();
        })
        .is_some(),
        RETRO_ENVIRONMENT_GET_USERNAME => {
            if data.is_null() {
                return false;
            }
            let name = with_ctx(|c| c.username.as_ref().map(|n| n.as_ptr())).flatten();
            match name {
                Some(p) => {
                    *(data as *mut *const c_char) = p;
                    true
                }
                None => false,
            }
        }
        // The devices the host feeds: RetroPad and the touch-screen pointer.
        RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES => {
            if data.is_null() {
                return false;
            }
            *(data as *mut u64) = (1 << RETRO_DEVICE_JOYPAD) | (1 << RETRO_DEVICE_POINTER);
            true
        }
        // Buttons can also be read as a single mask (faster for the core).
        x if x == (RETRO_ENVIRONMENT_GET_INPUT_BITMASKS & !RETRO_ENVIRONMENT_EXPERIMENTAL) => true,
        RETRO_ENVIRONMENT_SET_GEOMETRY => {
            let geo = *(data as *const retro_game_geometry);
            with_ctx(|c| c.shared.stats.lock().av_info.geometry = geo);
            true
        }
        RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO => {
            let av = *(data as *const retro_system_av_info);
            with_ctx(|c| {
                c.shared.stats.lock().av_info = av;
                c.shared.audio.lock().set_rate(av.timing.sample_rate);
            });
            true
        }
        // Accepted and ignored for now: button descriptors, how options are shown.
        RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY => {
            true
        }
        // Core options: the core declares them in one of four layouts, the shell answers
        // GET_VARIABLE (core_options.rs). Version 2 lets the cores send their richest layout.
        RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => {
            if data.is_null() {
                return false;
            }
            *(data as *mut c_uint) = 2;
            true
        }
        RETRO_ENVIRONMENT_SET_VARIABLES => define(variables_v0(data as *const retro_variable)),
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS => {
            define(definitions_v1(data as *const retro_core_option_definition))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL => {
            let intl = data as *const retro_core_options_intl;
            !intl.is_null() && define(definitions_v1((*intl).us))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 => {
            let opts = data as *const retro_core_options_v2;
            !opts.is_null() && define(definitions_v2((*opts).definitions))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
            let intl = data as *const retro_core_options_v2_intl;
            if intl.is_null() || (*intl).us.is_null() {
                return false;
            }
            define(definitions_v2((*(*intl).us).definitions))
        }
        RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
            if data.is_null() {
                return false;
            }
            *(data as *mut bool) = with_ctx(|c| c.options.take_updated()).unwrap_or(false);
            true
        }
        RETRO_ENVIRONMENT_GET_VARIABLE => {
            let var = data as *mut retro_variable;
            if var.is_null() || (*var).key.is_null() {
                return false;
            }
            let key = CStr::from_ptr((*var).key).to_string_lossy();
            // The pointer stays valid while the answer does not change (core_options.rs).
            let value = with_ctx(|c| c.options.answer(&key).map(CStr::as_ptr)).flatten();
            match value {
                Some(p) => {
                    (*var).value = p;
                    true
                }
                None => {
                    (*var).value = std::ptr::null();
                    false
                }
            }
        }
        _ => false,
    }
}

/// Stores the definitions the core declared; false when there is no session to store them in.
fn define(defs: Vec<OptionDef>) -> bool {
    log::debug!("core declared {} options", defs.len());
    with_ctx(|c| c.options.define(defs)).is_some()
}

unsafe fn c_string(p: *const c_char) -> Option<String> {
    (!p.is_null()).then(|| CStr::from_ptr(p).to_string_lossy().into_owned())
}

/// `SET_VARIABLES`: an array of `{ key, "Description; a|b|c" }` ending with a null key.
unsafe fn variables_v0(mut var: *const retro_variable) -> Vec<OptionDef> {
    let mut defs = Vec::new();
    while !var.is_null() && !(*var).key.is_null() {
        if let (Some(key), Some(raw)) = (c_string((*var).key), c_string((*var).value)) {
            defs.extend(OptionDef::from_variable(&key, &raw));
        }
        var = var.add(1);
    }
    defs
}

unsafe fn values(
    list: &[retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX],
) -> Vec<String> {
    list.iter().map_while(|v| c_string(v.value)).collect()
}

/// `SET_CORE_OPTIONS` (v1): an array of definitions ending with a null key.
unsafe fn definitions_v1(mut d: *const retro_core_option_definition) -> Vec<OptionDef> {
    let mut defs = Vec::new();
    while !d.is_null() && !(*d).key.is_null() {
        if let Some(key) = c_string((*d).key) {
            defs.push(OptionDef::new(
                key,
                values(&(*d).values),
                c_string((*d).default_value),
            ));
        }
        d = d.add(1);
    }
    defs
}

/// `SET_CORE_OPTIONS_V2`: same as v1 with categories, which the shell ignores for now.
unsafe fn definitions_v2(mut d: *const retro_core_option_v2_definition) -> Vec<OptionDef> {
    let mut defs = Vec::new();
    while !d.is_null() && !(*d).key.is_null() {
        if let Some(key) = c_string((*d).key) {
            defs.push(OptionDef::new(
                key,
                values(&(*d).values),
                c_string((*d).default_value),
            ));
        }
        d = d.add(1);
    }
    defs
}

pub(crate) unsafe extern "C" fn video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if data.is_null() {
        return; // duplicated frame: the previous one stays
    }
    with_ctx(|c| {
        let fmt = c.pixel_format;
        let mut frame = c.shared.frame.lock();
        frame.write_from_core(data as *const u8, width, height, pitch, fmt);
        frame.split = c.split;
    });
}

pub(crate) unsafe extern "C" fn audio_sample(left: i16, right: i16) {
    with_ctx(|c| {
        if c.audio_enabled {
            c.shared.audio.lock().push(&[left, right]);
        }
    });
}

pub(crate) unsafe extern "C" fn audio_sample_batch(data: *const i16, frames: usize) -> usize {
    with_ctx(|c| {
        if c.audio_enabled && !data.is_null() {
            let samples = std::slice::from_raw_parts(data, frames * 2);
            c.shared.audio.lock().push(samples);
        }
    });
    frames
}

pub(crate) unsafe extern "C" fn input_poll() {}

pub(crate) unsafe extern "C" fn input_state(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    if port != 0 {
        return 0;
    }
    if device == RETRO_DEVICE_POINTER {
        // One pointer (index 0): the touch screen.
        if index != 0 {
            return 0;
        }
        return with_ctx(|c| {
            let p = c.shared.input.pointer();
            match id {
                RETRO_DEVICE_ID_POINTER_X => p.x,
                RETRO_DEVICE_ID_POINTER_Y => p.y,
                RETRO_DEVICE_ID_POINTER_PRESSED => i16::from(p.pressed),
                RETRO_DEVICE_ID_POINTER_COUNT => i16::from(p.pressed),
                _ => 0,
            }
        })
        .unwrap_or(0);
    }
    if device != RETRO_DEVICE_JOYPAD {
        return 0;
    }
    with_ctx(|c| {
        let mask = c.shared.input.mask();
        if id == RETRO_DEVICE_ID_JOYPAD_MASK {
            mask as i16
        } else if id < 16 {
            ((mask >> id) & 1) as i16
        } else {
            0
        }
    })
    .unwrap_or(0)
}
