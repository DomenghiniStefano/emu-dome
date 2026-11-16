//! C callbacks the core calls during `retro_run` and while loading.
//!
//! libretro passes no context pointer, so the context lives in a
//! `thread_local`: everything happens on the emulation thread.

use std::cell::RefCell;
use std::ffi::{c_void, CString};
use std::os::raw::{c_char, c_uint};
use std::sync::Arc;

use libretro_sys::*;

use crate::session::Shared;

pub(crate) struct CallbackCtx {
    pub shared: Arc<Shared>,
    pub pixel_format: u32,
    pub system_dir: CString,
    pub save_dir: CString,
    /// When false, the produced audio is dropped (e.g. in fast forward).
    pub audio_enabled: bool,
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
        // Accepted and ignored for now: button descriptors, core variables.
        RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS | RETRO_ENVIRONMENT_SET_VARIABLES => true,
        // No variable changed: the core uses its defaults.
        // The settings engine will handle them in Phase 1.
        RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
            if !data.is_null() {
                *(data as *mut bool) = false;
            }
            true
        }
        RETRO_ENVIRONMENT_GET_VARIABLE => false,
        _ => false,
    }
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
        c.shared
            .frame
            .lock()
            .write_from_core(data as *const u8, width, height, pitch, fmt);
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
    _index: c_uint,
    id: c_uint,
) -> i16 {
    if port != 0 || device != RETRO_DEVICE_JOYPAD {
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
