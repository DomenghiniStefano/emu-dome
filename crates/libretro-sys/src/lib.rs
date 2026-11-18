//! libretro API types and constants (the subset EmuDome uses).
//!
//! Reference: the libretro project's `libretro.h`. Only the parts the shell
//! needs are here; more are added as they are needed.
#![allow(non_camel_case_types, dead_code)]

use std::os::raw::{c_char, c_uint, c_void};

pub const RETRO_API_VERSION: c_uint = 1;

// Devices
pub const RETRO_DEVICE_NONE: c_uint = 0;
pub const RETRO_DEVICE_JOYPAD: c_uint = 1;
pub const RETRO_DEVICE_POINTER: c_uint = 6;

// RetroPad buttons
pub const RETRO_DEVICE_ID_JOYPAD_B: c_uint = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: c_uint = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: c_uint = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: c_uint = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: c_uint = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: c_uint = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: c_uint = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: c_uint = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: c_uint = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: c_uint = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: c_uint = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: c_uint = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: c_uint = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: c_uint = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: c_uint = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: c_uint = 15;
pub const RETRO_DEVICE_ID_JOYPAD_MASK: c_uint = 256;

// Memory
pub const RETRO_MEMORY_SAVE_RAM: c_uint = 0;
pub const RETRO_MEMORY_RTC: c_uint = 1;
pub const RETRO_MEMORY_SYSTEM_RAM: c_uint = 2;
pub const RETRO_MEMORY_VIDEO_RAM: c_uint = 3;

// Pixel formats
pub const RETRO_PIXEL_FORMAT_0RGB1555: c_uint = 0;
pub const RETRO_PIXEL_FORMAT_XRGB8888: c_uint = 1;
pub const RETRO_PIXEL_FORMAT_RGB565: c_uint = 2;

// Environment commands (only the handled ones)
pub const RETRO_ENVIRONMENT_EXPERIMENTAL: c_uint = 0x10000;
pub const RETRO_ENVIRONMENT_GET_CAN_DUPE: c_uint = 3;
pub const RETRO_ENVIRONMENT_SET_MESSAGE: c_uint = 6;
pub const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: c_uint = 9;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;
pub const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: c_uint = 11;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: c_uint = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: c_uint = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: c_uint = 17;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME: c_uint = 18;
pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: c_uint = 27;
pub const RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY: c_uint = 30;
pub const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: c_uint = 31;
pub const RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO: c_uint = 32;
pub const RETRO_ENVIRONMENT_SET_GEOMETRY: c_uint = 37;
pub const RETRO_ENVIRONMENT_GET_LANGUAGE: c_uint = 39;
pub const RETRO_ENVIRONMENT_GET_INPUT_BITMASKS: c_uint = 51 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION: c_uint = 52;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS: c_uint = 53;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL: c_uint = 54;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY: c_uint = 55;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2: c_uint = 67;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL: c_uint = 68;

// Hardware rendering (`SET_HW_RENDER` and its interfaces). The experimental ones carry the flag,
// like `GET_INPUT_BITMASKS`; a host that masks the flag off compares against `x & !EXPERIMENTAL`.
pub const RETRO_ENVIRONMENT_SET_HW_RENDER: c_uint = 14;
pub const RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE: c_uint = 41 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE: c_uint =
    43 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT: c_uint = 44 | RETRO_ENVIRONMENT_EXPERIMENTAL;
pub const RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER: c_uint = 56;
pub const RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT: c_uint =
    73 | RETRO_ENVIRONMENT_EXPERIMENTAL;

/// `retro_hw_context_type`.
pub const RETRO_HW_CONTEXT_NONE: c_uint = 0;
pub const RETRO_HW_CONTEXT_OPENGL: c_uint = 1;
pub const RETRO_HW_CONTEXT_OPENGLES2: c_uint = 2;
pub const RETRO_HW_CONTEXT_OPENGL_CORE: c_uint = 3;
pub const RETRO_HW_CONTEXT_OPENGLES3: c_uint = 4;
pub const RETRO_HW_CONTEXT_OPENGLES_VERSION: c_uint = 5;
pub const RETRO_HW_CONTEXT_VULKAN: c_uint = 6;
pub const RETRO_HW_CONTEXT_D3D11: c_uint = 7;
pub const RETRO_HW_CONTEXT_D3D10: c_uint = 8;
pub const RETRO_HW_CONTEXT_D3D12: c_uint = 9;
pub const RETRO_HW_CONTEXT_D3D9: c_uint = 10;

/// `retro_hw_render_interface_type` / `retro_hw_render_context_negotiation_interface_type`.
pub const RETRO_HW_RENDER_INTERFACE_VULKAN: c_uint = 0;
pub const RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN: c_uint = 0;

/// The `data` a hardware-rendering core passes to `video_refresh` when its frame is in the
/// render target (the GL framebuffer, or the Vulkan image given to `set_image`).
pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

pub type retro_hw_context_reset_t = unsafe extern "C" fn();
pub type retro_hw_get_current_framebuffer_t = unsafe extern "C" fn() -> usize;
pub type retro_proc_address_t = unsafe extern "C" fn();
pub type retro_hw_get_proc_address_t =
    unsafe extern "C" fn(sym: *const c_char) -> Option<retro_proc_address_t>;

/// `SET_HW_RENDER`: filled by the core (type, version, callbacks), completed by the frontend
/// (`get_current_framebuffer`, `get_proc_address`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_hw_render_callback {
    pub context_type: c_uint,
    pub context_reset: Option<retro_hw_context_reset_t>,
    pub get_current_framebuffer: Option<retro_hw_get_current_framebuffer_t>,
    pub get_proc_address: Option<retro_hw_get_proc_address_t>,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: c_uint,
    pub version_minor: c_uint,
    pub cache_context: bool,
    pub context_destroy: Option<retro_hw_context_reset_t>,
    pub debug_context: bool,
}

/// Size of the fixed `values` array of a core option definition (v1 and v2).
pub const RETRO_NUM_CORE_OPTION_VALUES_MAX: usize = 128;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_system_info {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct retro_game_geometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct retro_system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct retro_system_av_info {
    pub geometry: retro_game_geometry,
    pub timing: retro_system_timing,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_game_info {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

/// One accepted value of a core option; the list ends with a null `value`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_option_value {
    pub value: *const c_char,
    pub label: *const c_char,
}

/// Core options v1 (`SET_CORE_OPTIONS`): an array ending with a null `key`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_option_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
    pub values: [retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

/// Core options v1 with translations (`SET_CORE_OPTIONS_INTL`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_options_intl {
    pub us: *const retro_core_option_definition,
    pub local: *const retro_core_option_definition,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_option_v2_category {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
}

/// Core options v2: an array ending with a null `key`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_option_v2_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub desc_categorized: *const c_char,
    pub info: *const c_char,
    pub info_categorized: *const c_char,
    pub category_key: *const c_char,
    pub values: [retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

/// Core options v2 (`SET_CORE_OPTIONS_V2`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_options_v2 {
    pub categories: *const retro_core_option_v2_category,
    pub definitions: *const retro_core_option_v2_definition,
}

/// Core options v2 with translations (`SET_CORE_OPTIONS_V2_INTL`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_core_options_v2_intl {
    pub us: *const retro_core_options_v2,
    pub local: *const retro_core_options_v2,
}

pub type retro_environment_t = unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;
pub type retro_video_refresh_t =
    unsafe extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize);
pub type retro_audio_sample_t = unsafe extern "C" fn(left: i16, right: i16);
pub type retro_audio_sample_batch_t =
    unsafe extern "C" fn(data: *const i16, frames: usize) -> usize;
pub type retro_input_poll_t = unsafe extern "C" fn();
pub type retro_input_state_t =
    unsafe extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;

/// The functions every libretro core exports, loaded by name.
pub mod symbols {
    use super::{
        c_uint, c_void, retro_audio_sample_batch_t, retro_audio_sample_t, retro_environment_t,
        retro_game_info, retro_input_poll_t, retro_input_state_t, retro_system_av_info,
        retro_system_info, retro_video_refresh_t,
    };
    pub type RetroInit = unsafe extern "C" fn();
    pub type RetroDeinit = unsafe extern "C" fn();
    pub type RetroApiVersion = unsafe extern "C" fn() -> c_uint;
    pub type RetroGetSystemInfo = unsafe extern "C" fn(info: *mut retro_system_info);
    pub type RetroGetSystemAvInfo = unsafe extern "C" fn(info: *mut retro_system_av_info);
    pub type RetroSetEnvironment = unsafe extern "C" fn(cb: retro_environment_t);
    pub type RetroSetVideoRefresh = unsafe extern "C" fn(cb: retro_video_refresh_t);
    pub type RetroSetAudioSample = unsafe extern "C" fn(cb: retro_audio_sample_t);
    pub type RetroSetAudioSampleBatch = unsafe extern "C" fn(cb: retro_audio_sample_batch_t);
    pub type RetroSetInputPoll = unsafe extern "C" fn(cb: retro_input_poll_t);
    pub type RetroSetInputState = unsafe extern "C" fn(cb: retro_input_state_t);
    pub type RetroSetControllerPortDevice = unsafe extern "C" fn(port: c_uint, device: c_uint);
    pub type RetroReset = unsafe extern "C" fn();
    pub type RetroRun = unsafe extern "C" fn();
    pub type RetroSerializeSize = unsafe extern "C" fn() -> usize;
    pub type RetroSerialize = unsafe extern "C" fn(data: *mut c_void, size: usize) -> bool;
    pub type RetroUnserialize = unsafe extern "C" fn(data: *const c_void, size: usize) -> bool;
    pub type RetroLoadGame = unsafe extern "C" fn(game: *const retro_game_info) -> bool;
    pub type RetroUnloadGame = unsafe extern "C" fn();
    pub type RetroGetMemoryData = unsafe extern "C" fn(id: c_uint) -> *mut c_void;
    pub type RetroGetMemorySize = unsafe extern "C" fn(id: c_uint) -> usize;
}
