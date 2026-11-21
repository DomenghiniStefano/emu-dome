//! Hardware rendering (`SET_HW_RENDER` and the calls around it): a pluggable handler.
//!
//! The environment callback hands every hardware-rendering call to the session's
//! [`HwRenderHandler`], so the libretro side (accepting a context, answering the Vulkan interface
//! and the context negotiation) can be written and tested apart from the host's other answers.
//! Today's handler is [`SoftwareOnly`]: it refuses every request, as the host did before the
//! seam (the cores then render in software: melonDS DS falls back by itself, Azahar is forced to
//! its software renderer by `CoreProfile::forced_options`). The 3DS work (#46 / #16) plugs a
//! Vulkan handler in here, its GPU side in the app's `gpu_interop.rs`, and gives its frames to the
//! session's [`crate::video_sink::VideoSink`] as [`GpuFrame`]s.
//!
//! A handler runs inside the environment / video callbacks with the session's callback context
//! held: it must not call into the core from them (`context_reset` and `context_destroy` are run
//! by the session on the emulation thread, outside any callback).

use std::os::raw::{c_uint, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use libretro_sys::{
    retro_hw_render_callback,
    RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
    RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
    RETRO_ENVIRONMENT_SET_HW_RENDER, RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE,
    RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT,
};

use crate::video_sink::GpuFrame;

/// The libretro side of hardware rendering for one session.
pub trait HwRenderHandler: Send {
    /// `SET_HW_RENDER`: true accepts the core's request (the handler completes the struct's
    /// frontend fields), false refuses it and the core must render in software.
    ///
    /// # Safety
    /// `request` is the core's pointer, possibly null.
    unsafe fn set_hw_render(&mut self, request: *mut retro_hw_render_callback) -> bool;

    /// The other hardware-rendering calls ([`is_hw_command`], `SET_HW_RENDER` excluded), by their
    /// raw command number (experimental flag included). The answer, true when handled.
    ///
    /// # Safety
    /// `data` is the core's pointer for that command, possibly null.
    unsafe fn environment(&mut self, cmd: c_uint, data: *mut c_void) -> bool;

    /// `video_refresh` with `RETRO_HW_FRAME_BUFFER_VALID`: the frame the core just drew on the
    /// GPU, `None` when there is none to give (it is then dropped).
    fn hw_frame(&mut self, width: u32, height: u32) -> Option<GpuFrame>;
}

/// Whether `raw` (the command as the core sent it) is a hardware-rendering call. Compared raw:
/// `SET_HW_SHARED_CONTEXT` is 44 with the experimental flag, and 44 without it is
/// `SET_SERIALIZATION_QUIRKS`.
pub fn is_hw_command(raw: c_uint) -> bool {
    matches!(
        raw,
        RETRO_ENVIRONMENT_SET_HW_RENDER
            | RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE
            | RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE
            | RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT
            | RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER
            | RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT
    )
}

/// Refuses hardware rendering: every call answers false, as before the seam.
#[derive(Default)]
pub struct SoftwareOnly {
    /// The core's request was logged (once a session).
    logged: AtomicBool,
}

impl HwRenderHandler for SoftwareOnly {
    unsafe fn set_hw_render(&mut self, request: *mut retro_hw_render_callback) -> bool {
        if !request.is_null() && !self.logged.swap(true, Ordering::Relaxed) {
            let r = &*request;
            log::info!(
                "core asks SET_HW_RENDER context {} {}.{}: refused (software only)",
                r.context_type,
                r.version_major,
                r.version_minor
            );
        }
        false
    }

    unsafe fn environment(&mut self, _cmd: c_uint, _data: *mut c_void) -> bool {
        false
    }

    fn hw_frame(&mut self, _width: u32, _height: u32) -> Option<GpuFrame> {
        None
    }
}

#[cfg(test)]
mod tests {
    use libretro_sys::{
        retro_hw_render_callback, RETRO_ENVIRONMENT_EXPERIMENTAL,
        RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER, RETRO_ENVIRONMENT_SET_HW_RENDER,
        RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT, RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
        RETRO_HW_CONTEXT_VULKAN,
    };

    use super::{is_hw_command, HwRenderHandler, SoftwareOnly};

    #[test]
    fn the_hardware_calls_are_told_apart_from_serialization_quirks() {
        assert!(is_hw_command(RETRO_ENVIRONMENT_SET_HW_RENDER));
        assert!(is_hw_command(RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT));
        assert!(is_hw_command(RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER));
        assert!(!is_hw_command(RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS));
        assert_eq!(
            RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT & !RETRO_ENVIRONMENT_EXPERIMENTAL,
            RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS
        );
    }

    #[test]
    fn software_only_refuses_everything() {
        let mut h = SoftwareOnly::default();
        let mut request = retro_hw_render_callback {
            context_type: RETRO_HW_CONTEXT_VULKAN,
            context_reset: None,
            get_current_framebuffer: None,
            get_proc_address: None,
            depth: false,
            stencil: false,
            bottom_left_origin: false,
            version_major: 1,
            version_minor: 3,
            cache_context: false,
            context_destroy: None,
            debug_context: false,
        };
        assert!(!unsafe { h.set_hw_render(&mut request) });
        assert!(!unsafe { h.set_hw_render(std::ptr::null_mut()) });
        let mut preferred = 0u32;
        assert!(!unsafe {
            h.environment(
                RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
                &mut preferred as *mut u32 as *mut _,
            )
        });
        assert!(h.hw_frame(400, 480).is_none());
    }
}
