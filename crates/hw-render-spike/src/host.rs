//! A minimal libretro host for the spike: loads the core, answers the environment (core options
//! from the command line, `SET_HW_RENDER` for Vulkan, the Vulkan interfaces) and records what
//! the core does with them. Everything the core reports lands in [`STATE`].
//!
//! Not the shell's host (`emudome-core`): the spike stays out of it on purpose, so the Phase 3
//! integration can choose its own shape from what is measured here.

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_uint};
use std::path::Path;

use anyhow::{bail, Context, Result};
use ash::vk;
use libloading::Library;
use libretro_sys::symbols::{
    RetroApiVersion, RetroDeinit, RetroGetSystemAvInfo, RetroGetSystemInfo, RetroInit,
    RetroLoadGame, RetroRun, RetroSetAudioSample, RetroSetAudioSampleBatch, RetroSetEnvironment,
    RetroSetInputPoll, RetroSetInputState, RetroSetVideoRefresh, RetroUnloadGame,
};
use libretro_sys::{
    retro_game_info, retro_hw_render_callback, retro_system_av_info, retro_system_info,
    retro_variable, RETRO_API_VERSION, RETRO_ENVIRONMENT_EXPERIMENTAL,
    RETRO_ENVIRONMENT_GET_CAN_DUPE, RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION,
    RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
    RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE, RETRO_ENVIRONMENT_GET_INPUT_BITMASKS,
    RETRO_ENVIRONMENT_GET_LOG_INTERFACE, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
    RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY, RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
    RETRO_ENVIRONMENT_GET_VARIABLE, RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL, RETRO_ENVIRONMENT_SET_GEOMETRY,
    RETRO_ENVIRONMENT_SET_HW_RENDER, RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE,
    RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT, RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
    RETRO_ENVIRONMENT_SET_MESSAGE, RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
    RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO, RETRO_ENVIRONMENT_SET_VARIABLES, RETRO_HW_CONTEXT_VULKAN,
    RETRO_HW_FRAME_BUFFER_VALID, RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN,
};
use parking_lot::Mutex;

use crate::vk_iface::{RetroHwRenderInterfaceVulkan, RetroNegotiationVulkanV1, RetroVulkanImage};

/// Raw pointers the core gave us; only touched from the thread that runs the core, or under
/// the lock.
pub struct HostState {
    /// Core options answered to `GET_VARIABLE` (`--opt key=value`).
    pub options: Vec<(String, CString)>,
    pub system_dir: Option<CString>,
    pub save_dir: Option<CString>,
    /// Every `SET_HW_RENDER` the core asked for (context type, version), accepted or not.
    pub hw_requests: Vec<(c_uint, c_uint, c_uint)>,
    /// The accepted `SET_HW_RENDER` (Vulkan only).
    pub hw: Option<retro_hw_render_callback>,
    pub negotiation: *const RetroNegotiationVulkanV1,
    /// What `GET_HW_RENDER_INTERFACE` answers; set by the frontend before `context_reset`.
    pub iface: Option<Box<RetroHwRenderInterfaceVulkan>>,
    /// The last `set_image`.
    pub image: Option<CoreImage>,
    pub set_image_calls: u64,
    /// The last `video_refresh`.
    pub last_frame: FrameKind,
    pub frames: u64,
    pub messages: Vec<String>,
    pub core_log: bool,
}

// SAFETY: the raw pointers are the core's static data and our own boxed interface; the spike
// drives the core from one thread and only reads them there.
unsafe impl Send for HostState {}

#[derive(Clone, Copy, Debug)]
pub struct CoreImage {
    pub image: vk::Image,
    pub layout: vk::ImageLayout,
    pub format: vk::Format,
    pub semaphores: u32,
    pub src_queue_family: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    None,
    /// `RETRO_HW_FRAME_BUFFER_VALID`: the frame is in the image of the last `set_image`.
    Hardware {
        width: u32,
        height: u32,
    },
    /// A software frame in memory (the core did not use the hardware context).
    Software {
        width: u32,
        height: u32,
    },
    /// `NULL`: the previous frame again.
    Dupe,
}

pub static STATE: Mutex<HostState> = parking_lot::const_mutex(HostState {
    options: Vec::new(),
    system_dir: None,
    save_dir: None,
    hw_requests: Vec::new(),
    hw: None,
    negotiation: std::ptr::null(),
    iface: None,
    image: None,
    set_image_calls: 0,
    last_frame: FrameKind::None,
    frames: 0,
    messages: Vec::new(),
    core_log: false,
});

/// The queue lock of the Vulkan interface: the core takes it around its own `vkQueueSubmit`,
/// the frontend around wgpu's. A raw mutex because lock and unlock arrive as two C calls.
pub static QUEUE_LOCK: parking_lot::RawMutex =
    <parking_lot::RawMutex as parking_lot::lock_api::RawMutex>::INIT;

pub struct Core {
    pub run: RetroRun,
    pub info_name: String,
    pub info_version: String,
    pub av_info: retro_system_av_info,
    unload_game: RetroUnloadGame,
    deinit: RetroDeinit,
    game_loaded: bool,
    lib: Library,
}

macro_rules! sym {
    ($lib:expr, $ty:ty, $name:literal) => {{
        let s: libloading::Symbol<$ty> = $lib
            .get(concat!($name, "\0").as_bytes())
            .with_context(|| format!("the core does not export {}", $name))?;
        *s
    }};
}

impl Core {
    /// # Safety
    /// Loads and runs native code: the file must be a libretro core.
    pub unsafe fn load(path: &Path) -> Result<Self> {
        let lib = Library::new(path).with_context(|| format!("cannot load {}", path.display()))?;
        let api: RetroApiVersion = sym!(lib, RetroApiVersion, "retro_api_version");
        if api() != RETRO_API_VERSION {
            bail!("unsupported libretro API version {}", api());
        }
        let set_env: RetroSetEnvironment = sym!(lib, RetroSetEnvironment, "retro_set_environment");
        let set_video: RetroSetVideoRefresh =
            sym!(lib, RetroSetVideoRefresh, "retro_set_video_refresh");
        let set_audio: RetroSetAudioSample =
            sym!(lib, RetroSetAudioSample, "retro_set_audio_sample");
        let set_audio_batch: RetroSetAudioSampleBatch = sym!(
            lib,
            RetroSetAudioSampleBatch,
            "retro_set_audio_sample_batch"
        );
        let set_poll: RetroSetInputPoll = sym!(lib, RetroSetInputPoll, "retro_set_input_poll");
        let set_state: RetroSetInputState = sym!(lib, RetroSetInputState, "retro_set_input_state");
        let init: RetroInit = sym!(lib, RetroInit, "retro_init");
        let get_info: RetroGetSystemInfo = sym!(lib, RetroGetSystemInfo, "retro_get_system_info");
        set_env(environment);
        set_video(video_refresh);
        set_audio(audio_sample);
        set_audio_batch(audio_sample_batch);
        set_poll(input_poll);
        set_state(input_state);
        init();
        let mut info: retro_system_info = std::mem::zeroed();
        get_info(&mut info);
        let s = |p: *const c_char| {
            if p.is_null() {
                String::new()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        Ok(Self {
            run: sym!(lib, RetroRun, "retro_run"),
            info_name: s(info.library_name),
            info_version: s(info.library_version),
            av_info: retro_system_av_info::default(),
            unload_game: sym!(lib, RetroUnloadGame, "retro_unload_game"),
            deinit: sym!(lib, RetroDeinit, "retro_deinit"),
            game_loaded: false,
            lib,
        })
    }

    /// # Safety
    /// Runs the core's loader.
    pub unsafe fn load_game(&mut self, game: &Path) -> Result<()> {
        let data =
            std::fs::read(game).with_context(|| format!("cannot read {}", game.display()))?;
        let path = CString::new(game.to_string_lossy().as_bytes())?;
        let info = retro_game_info {
            path: path.as_ptr(),
            data: data.as_ptr() as *const c_void,
            size: data.len(),
            meta: std::ptr::null(),
        };
        let load: RetroLoadGame = sym!(self.lib, RetroLoadGame, "retro_load_game");
        if !load(&info) {
            bail!("the core refused the content {}", game.display());
        }
        self.game_loaded = true;
        let av: RetroGetSystemAvInfo =
            sym!(self.lib, RetroGetSystemAvInfo, "retro_get_system_av_info");
        av(&mut self.av_info);
        Ok(())
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        // SAFETY: the library is still loaded (`lib` drops after this body).
        unsafe {
            if self.game_loaded {
                (self.unload_game)();
            }
            (self.deinit)();
        }
    }
}

#[repr(C)]
struct RetroMessage {
    msg: *const c_char,
    frames: c_uint,
}

#[repr(C)]
struct RetroLogCallback {
    log: unsafe extern "C" fn(level: c_uint, fmt: *const c_char),
}

/// The core's log callback is variadic in C; reading only the two fixed arguments is sound on
/// the C calling conventions in use (the varargs follow them). The format string is printed
/// as is, placeholders included: enough to follow what the core does.
unsafe extern "C" fn core_log(level: c_uint, fmt: *const c_char) {
    if fmt.is_null() || !STATE.lock().core_log {
        return;
    }
    let text = CStr::from_ptr(fmt).to_string_lossy();
    eprint!("[core log {level}] {text}");
    if !text.ends_with('\n') {
        eprintln!();
    }
}

fn masked(cmd: c_uint) -> c_uint {
    cmd & !RETRO_ENVIRONMENT_EXPERIMENTAL
}

unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    let cmd = masked(cmd);
    if cmd == RETRO_ENVIRONMENT_GET_CAN_DUPE {
        if !data.is_null() {
            *(data as *mut bool) = true;
        }
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_SET_PIXEL_FORMAT {
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY || cmd == RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY
    {
        let st = STATE.lock();
        let dir = if cmd == RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY {
            &st.system_dir
        } else {
            &st.save_dir
        };
        return match dir {
            Some(d) if !data.is_null() => {
                *(data as *mut *const c_char) = d.as_ptr();
                true
            }
            Some(_) | None => false,
        };
    }
    if cmd == masked(RETRO_ENVIRONMENT_GET_INPUT_BITMASKS) {
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION {
        if data.is_null() {
            return false;
        }
        *(data as *mut c_uint) = 2;
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE {
        if !data.is_null() {
            *(data as *mut bool) = false;
        }
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_VARIABLE {
        let var = data as *mut retro_variable;
        if var.is_null() || (*var).key.is_null() {
            return false;
        }
        let key = CStr::from_ptr((*var).key).to_string_lossy();
        let st = STATE.lock();
        return match st.options.iter().find(|(k, _)| *k == key) {
            Some((_, v)) => {
                (*var).value = v.as_ptr();
                true
            }
            None => {
                (*var).value = std::ptr::null();
                false
            }
        };
    }
    if cmd == RETRO_ENVIRONMENT_SET_MESSAGE {
        let m = data as *const RetroMessage;
        if !m.is_null() && !(*m).msg.is_null() {
            let text = CStr::from_ptr((*m).msg).to_string_lossy().into_owned();
            eprintln!("[core message] {text}");
            STATE.lock().messages.push(text);
        }
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_LOG_INTERFACE {
        if data.is_null() {
            return false;
        }
        (*(data as *mut RetroLogCallback)).log = core_log;
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_SET_HW_RENDER {
        let cb = data as *mut retro_hw_render_callback;
        if cb.is_null() {
            return false;
        }
        let req = &mut *cb;
        let mut st = STATE.lock();
        st.hw_requests
            .push((req.context_type, req.version_major, req.version_minor));
        if req.context_type != RETRO_HW_CONTEXT_VULKAN {
            // The spike has no GL context: refuse, as a host without that API must.
            return false;
        }
        // Vulkan uses neither field; set them so a core never calls a null pointer.
        req.get_current_framebuffer = Some(no_framebuffer);
        req.get_proc_address = Some(no_proc_address);
        st.hw = Some(*req);
        return true;
    }
    if cmd == RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER {
        if data.is_null() {
            return false;
        }
        *(data as *mut c_uint) = RETRO_HW_CONTEXT_VULKAN;
        return true;
    }
    if cmd == masked(RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE) {
        let iface = data as *const RetroNegotiationVulkanV1;
        if iface.is_null()
            || (*iface).interface_type != RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN
        {
            return false;
        }
        STATE.lock().negotiation = iface;
        return true;
    }
    if cmd == masked(RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT) {
        // retro_hw_render_context_negotiation_interface { type, version }: the frontend writes
        // the highest version it supports for that type. The spike supports version 1.
        let q = data as *mut [c_uint; 2];
        if q.is_null() {
            return false;
        }
        if (*q)[0] == RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN {
            (*q)[1] = 1;
        } else {
            (*q)[1] = 0;
        }
        return true;
    }
    if cmd == masked(RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE) {
        let out = data as *mut *const RetroHwRenderInterfaceVulkan;
        if out.is_null() {
            return false;
        }
        let st = STATE.lock();
        return match &st.iface {
            Some(b) => {
                *out = &**b;
                true
            }
            None => false,
        };
    }
    if cmd == masked(RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT) {
        return true;
    }
    // Option declarations, input descriptors, geometry: accepted and ignored (unanswered options
    // keep the core's defaults). Everything else: refused.
    [
        RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
        RETRO_ENVIRONMENT_SET_VARIABLES,
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS,
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL,
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY,
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2,
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL,
        RETRO_ENVIRONMENT_SET_GEOMETRY,
        RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO,
    ]
    .contains(&cmd)
}

unsafe extern "C" fn no_framebuffer() -> usize {
    0
}

unsafe extern "C" fn no_proc_address(
    _sym: *const c_char,
) -> Option<libretro_sys::retro_proc_address_t> {
    None
}

unsafe extern "C" fn video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    _pitch: usize,
) {
    let kind = if data == RETRO_HW_FRAME_BUFFER_VALID {
        FrameKind::Hardware { width, height }
    } else if data.is_null() {
        FrameKind::Dupe
    } else {
        FrameKind::Software { width, height }
    };
    let mut st = STATE.lock();
    st.last_frame = kind;
    st.frames += 1;
}

unsafe extern "C" fn audio_sample(_l: i16, _r: i16) {}
unsafe extern "C" fn audio_sample_batch(_data: *const i16, frames: usize) -> usize {
    frames
}
unsafe extern "C" fn input_poll() {}
unsafe extern "C" fn input_state(
    _port: c_uint,
    _device: c_uint,
    _index: c_uint,
    _id: c_uint,
) -> i16 {
    0
}

// --- retro_hw_render_interface_vulkan callbacks -------------------------------------------

pub unsafe extern "C" fn set_image(
    _handle: *mut c_void,
    image: *const RetroVulkanImage,
    num_semaphores: u32,
    _semaphores: *const vk::Semaphore,
    src_queue_family: u32,
) {
    let mut st = STATE.lock();
    st.set_image_calls += 1;
    if image.is_null() {
        st.image = None;
        return;
    }
    let img = &*image;
    st.image = Some(CoreImage {
        image: img.create_info.image,
        layout: img.image_layout,
        format: img.create_info.format,
        semaphores: num_semaphores,
        src_queue_family,
    });
}

/// One sync index: the frontend waits for its own GPU work on the image before the next
/// `retro_run`, so the core can always reuse index 0.
pub unsafe extern "C" fn get_sync_index(_handle: *mut c_void) -> u32 {
    0
}

pub unsafe extern "C" fn get_sync_index_mask(_handle: *mut c_void) -> u32 {
    1
}

pub unsafe extern "C" fn set_command_buffers(
    _handle: *mut c_void,
    num: u32,
    _cmd: *const vk::CommandBuffer,
) {
    // Azahar submits its own command buffers; a core that hands them over is not supported
    // by the spike, and says so.
    eprintln!("[spike] set_command_buffers({num}) called: not supported by the spike");
}

pub unsafe extern "C" fn wait_sync_index(_handle: *mut c_void) {}

pub unsafe extern "C" fn lock_queue(_handle: *mut c_void) {
    parking_lot::lock_api::RawMutex::lock(&QUEUE_LOCK);
}

pub unsafe extern "C" fn unlock_queue(_handle: *mut c_void) {
    parking_lot::lock_api::RawMutex::unlock(&QUEUE_LOCK);
}

pub unsafe extern "C" fn set_signal_semaphore(_handle: *mut c_void, _semaphore: vk::Semaphore) {
    eprintln!("[spike] set_signal_semaphore called: not supported by the spike");
}
