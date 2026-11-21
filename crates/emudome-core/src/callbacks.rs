//! C callbacks the core calls during `retro_run`, while loading, and from its own threads.
//!
//! libretro passes no context pointer. Each loaded core takes a [`Slot`]: a static context and
//! its own set of callback functions (one copy per slot, [`Callbacks::of`]), so a call from any
//! thread of that core (a multi-threaded core such as Azahar calls from its workers) finds that
//! core's context, and a core that was let go ([`Slot::retire`]) can never reach the context of
//! the next game's core. A callback that finds no context logs it once, with its thread.
//!
//! Every `extern "C"` entry runs under [`guard`]: a panic is caught and logged, never unwound into
//! the core's C code (in builds with `panic = "unwind"`; the release profile aborts on panic).

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_uint};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::ThreadId;

use anyhow::{bail, Result};
use libretro_sys::{
    retro_audio_sample_batch_t, retro_audio_sample_t, retro_core_option_definition,
    retro_core_option_v2_definition, retro_core_option_value, retro_core_options_intl,
    retro_core_options_v2, retro_core_options_v2_intl, retro_environment_t, retro_game_geometry,
    retro_game_info_ext, retro_hw_render_callback, retro_input_poll_t, retro_input_state_t,
    retro_log_callback, retro_message, retro_system_av_info, retro_variable, retro_video_refresh_t,
    RETRO_DEVICE_ANALOG, RETRO_DEVICE_ID_ANALOG_X, RETRO_DEVICE_ID_ANALOG_Y,
    RETRO_DEVICE_ID_JOYPAD_MASK, RETRO_DEVICE_ID_POINTER_COUNT, RETRO_DEVICE_ID_POINTER_PRESSED,
    RETRO_DEVICE_ID_POINTER_X, RETRO_DEVICE_ID_POINTER_Y, RETRO_DEVICE_INDEX_ANALOG_BUTTON,
    RETRO_DEVICE_INDEX_ANALOG_LEFT, RETRO_DEVICE_INDEX_ANALOG_RIGHT, RETRO_DEVICE_JOYPAD,
    RETRO_DEVICE_POINTER, RETRO_ENVIRONMENT_EXPERIMENTAL, RETRO_ENVIRONMENT_GET_CAN_DUPE,
    RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION, RETRO_ENVIRONMENT_GET_GAME_INFO_EXT,
    RETRO_ENVIRONMENT_GET_INPUT_BITMASKS, RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES,
    RETRO_ENVIRONMENT_GET_LANGUAGE, RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
    RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY, RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
    RETRO_ENVIRONMENT_GET_USERNAME, RETRO_ENVIRONMENT_GET_VARIABLE,
    RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE, RETRO_ENVIRONMENT_SET_CORE_OPTIONS,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL,
    RETRO_ENVIRONMENT_SET_GEOMETRY, RETRO_ENVIRONMENT_SET_HW_RENDER,
    RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS, RETRO_ENVIRONMENT_SET_MESSAGE,
    RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
    RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO, RETRO_ENVIRONMENT_SET_VARIABLES,
    RETRO_HW_FRAME_BUFFER_VALID, RETRO_LANGUAGE_ENGLISH, RETRO_LOG_DEBUG, RETRO_LOG_ERROR,
    RETRO_LOG_INFO, RETRO_LOG_WARN, RETRO_NUM_CORE_OPTION_VALUES_MAX, RETRO_PIXEL_FORMAT_0RGB1555,
    RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
    RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE, RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT,
    RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE, RETRO_SERIALIZATION_QUIRK_INCOMPLETE,
    RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE, RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT,
    RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION,
};
use parking_lot::Mutex;

use crate::audio::AudioProducer;
use crate::core::GameStrings;
use crate::core_options::{CoreOptions, OptionDef};
use crate::frame::bytes_per_pixel;
use crate::hw_render::{is_hw_command, HwRenderHandler, SoftwareOnly};
use crate::input::Snapshot;
use crate::screens::Split;
use crate::session::Shared;
use crate::speed_audio::SpeedAudio;
use crate::video_sink::{CpuFrame, SharedFrameSink, VideoFrame, VideoSink};

pub(crate) struct CallbackCtx {
    pub shared: Arc<Shared>,
    pub pixel_format: u32,
    pub system_dir: CString,
    pub save_dir: CString,
    /// When false, the produced audio is dropped (while rewinding).
    pub audio_enabled: bool,
    /// The audio at speeds other than 1× (`audio.ff`).
    pub speed_audio: SpeedAudio,
    /// Scratch buffer for [`SpeedAudio`]'s output.
    pub audio_out: Vec<i16>,
    /// The emulation thread's end of the session's audio ring (no lock shared with the output).
    pub audio: AudioProducer,
    /// Answers to `GET_VARIABLE`: the core's definitions and the shell's values.
    pub options: CoreOptions,
    /// How the core lays its screens out in the frame (recorded on every frame).
    pub split: Split,
    /// The answer to `GET_USERNAME` (melonDS DS: the DS user name); `None`: not answered.
    pub username: Option<CString>,
    /// The emulation thread. A call from another one is the core's own worker: logged once.
    pub emu_thread: ThreadId,
    /// The timing the core set with `SET_SYSTEM_AV_INFO` since the session last looked
    /// ([`Slot::take_av_change`]).
    pub av_change: Option<retro_system_av_info>,
    /// The answer to `GET_GAME_INFO_EXT` (boxed: the core keeps its address), from just before
    /// `retro_load_game` until the slot is released.
    pub game_ext: Option<GameExt>,
    /// The strings `game_ext` points into, once the load is over.
    pub game_strings: Option<GameStrings>,
    /// Where the frames go ([`VideoSink`]): today the RGBA8 frame in [`Shared`].
    pub video: Box<dyn VideoSink>,
    /// The answers to the hardware-rendering calls ([`HwRenderHandler`]): today refused.
    pub hw: Box<dyn HwRenderHandler>,
    /// The input the core reads ([`Snapshot`]), taken at `input_poll` and before each frame.
    pub input: Snapshot,
}

impl CallbackCtx {
    /// A context for a session: everything the core may ask before and while it runs.
    pub(crate) fn new(shared: Arc<Shared>, system_dir: CString, save_dir: CString) -> Self {
        Self {
            shared: shared.clone(),
            pixel_format: RETRO_PIXEL_FORMAT_0RGB1555,
            system_dir,
            save_dir,
            audio_enabled: true,
            speed_audio: SpeedAudio::new(crate::speed_audio::FastAudio::Mute),
            audio_out: Vec::new(),
            audio: shared
                .audio
                .take_producer()
                .unwrap_or_else(AudioProducer::detached),
            options: CoreOptions::default(),
            split: Split::Whole,
            username: None,
            emu_thread: std::thread::current().id(),
            av_change: None,
            game_ext: None,
            game_strings: None,
            video: Box::new(SharedFrameSink::new(shared.clone())),
            hw: Box::new(SoftwareOnly::default()),
            input: Snapshot::default(),
        }
    }
}

/// The boxed answer to `GET_GAME_INFO_EXT`.
pub(crate) struct GameExt(Box<retro_game_info_ext>);

// SAFETY: the pointers inside lead to the game's path strings, owned by the same context
// (`game_strings`) and never written through, and to its bytes only while `retro_load_game` runs
// on the thread that owns them; the context moves between threads only behind its mutex.
unsafe impl Send for GameExt {}

/// How many cores can be loaded at once in one run, counting the ones let go (each keeps its slot
/// until EmuDome exits). One game runs at a time and a core let go is refused until restart, so
/// at most one slot per core file is ever retired.
const SLOT_COUNT: usize = 8;

struct SlotState {
    ctx: Mutex<Option<CallbackCtx>>,
    /// Taken by a [`Slot`]; given back only after its core was unloaded.
    claimed: AtomicBool,
    /// The «no context» line was logged for the current claim.
    missing_logged: AtomicBool,
    /// The «called from the core's own thread» line was logged for the current claim.
    foreign_logged: AtomicBool,
}

impl SlotState {
    const fn new() -> Self {
        Self {
            ctx: parking_lot::const_mutex(None),
            claimed: AtomicBool::new(false),
            missing_logged: AtomicBool::new(false),
            foreign_logged: AtomicBool::new(false),
        }
    }

    /// Runs `f` on the context, when there is one. `what` names the callback for the logs.
    fn with<R>(&self, what: &str, f: impl FnOnce(&mut CallbackCtx) -> R) -> Option<R> {
        let mut guard = self.ctx.lock();
        let Some(ctx) = guard.as_mut() else {
            if !self.missing_logged.swap(true, Ordering::Relaxed) {
                log::warn!(
                    "libretro callback {what} with no session (thread {:?}): answered as unsupported",
                    std::thread::current().id()
                );
            }
            return None;
        };
        let thread = std::thread::current().id();
        if thread != ctx.emu_thread && !self.foreign_logged.swap(true, Ordering::Relaxed) {
            log::info!(
                "the core calls {what} from its own thread {thread:?} (emulation thread {:?})",
                ctx.emu_thread
            );
        }
        Some(f(ctx))
    }
}

static SLOTS: [SlotState; SLOT_COUNT] = [const { SlotState::new() }; SLOT_COUNT];

/// The callback functions of one slot, handed to the core's `retro_set_*`.
#[derive(Clone, Copy)]
pub(crate) struct Callbacks {
    pub environment: retro_environment_t,
    pub video_refresh: retro_video_refresh_t,
    pub audio_sample: retro_audio_sample_t,
    pub audio_sample_batch: retro_audio_sample_batch_t,
    pub input_poll: retro_input_poll_t,
    pub input_state: retro_input_state_t,
}

impl Callbacks {
    const fn of<const S: usize>() -> Self {
        Self {
            environment: environment::<S>,
            video_refresh: video_refresh::<S>,
            audio_sample: audio_sample::<S>,
            audio_sample_batch: audio_sample_batch::<S>,
            input_poll: input_poll::<S>,
            input_state: input_state::<S>,
        }
    }
}

const CALLBACKS: [Callbacks; SLOT_COUNT] = [
    Callbacks::of::<0>(),
    Callbacks::of::<1>(),
    Callbacks::of::<2>(),
    Callbacks::of::<3>(),
    Callbacks::of::<4>(),
    Callbacks::of::<5>(),
    Callbacks::of::<6>(),
    Callbacks::of::<7>(),
];

/// One loaded core's place for its callbacks' context. Given back with [`Slot::release`] once
/// the core is unloaded; a slot dropped otherwise (a core let go, a panic) is retired: its context
/// is cleared and it is never reused, since the core may still be loaded and calling.
pub(crate) struct Slot {
    index: usize,
    released: bool,
}

impl Slot {
    /// Takes a free slot and puts `ctx` in it.
    pub(crate) fn claim(ctx: CallbackCtx) -> Result<Self> {
        for (index, state) in SLOTS.iter().enumerate() {
            if state
                .claimed
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                state.missing_logged.store(false, Ordering::Relaxed);
                state.foreign_logged.store(false, Ordering::Relaxed);
                *state.ctx.lock() = Some(ctx);
                return Ok(Self {
                    index,
                    released: false,
                });
            }
        }
        bail!("no free libretro callback slot: {SLOT_COUNT} cores loaded or let go in this run")
    }

    /// The functions to hand to the core loaded in this slot.
    pub(crate) fn callbacks(&self) -> Callbacks {
        CALLBACKS[self.index]
    }

    fn with<R>(&self, f: impl FnOnce(&mut CallbackCtx) -> R) -> Option<R> {
        SLOTS[self.index].ctx.lock().as_mut().map(f)
    }

    /// When false, the produced audio is dropped (while rewinding).
    pub(crate) fn set_audio_enabled(&self, enabled: bool) {
        self.with(|c| c.audio_enabled = enabled);
    }

    /// The emulation speed for the audio (None: not measured yet, at Max).
    pub(crate) fn set_audio_speed(&self, speed: Option<f64>) {
        self.with(|c| c.speed_audio.set_speed(speed));
    }

    /// What the audio does off 1× (`audio.ff`).
    pub(crate) fn set_fast_audio(&self, policy: crate::speed_audio::FastAudio) {
        self.with(|c| c.speed_audio.set_policy(policy));
    }

    /// Changes a core option while the game runs (seen at the core's next `GET_VARIABLE_UPDATE`).
    pub(crate) fn set_core_option(&self, key: String, value: String) {
        self.with(|c| c.options.set(key, value));
    }

    /// Freezes the input for the next frame (also done at the core's `input_poll`): a core that
    /// reads without polling still sees this frame's input.
    pub(crate) fn refresh_input(&self) {
        self.with(|c| c.input = c.shared.input.snapshot());
    }

    /// The timing the core set with `SET_SYSTEM_AV_INFO` since the last call, if it did.
    pub(crate) fn take_av_change(&self) -> Option<retro_system_av_info> {
        self.with(|c| c.av_change.take()).flatten()
    }

    /// Before `retro_load_game`: the answer to `GET_GAME_INFO_EXT`.
    pub(crate) fn begin_game(&self, ext: retro_game_info_ext) {
        self.with(|c| c.game_ext = Some(GameExt(Box::new(ext))));
    }

    /// After `retro_load_game` (whatever it returned): the game's bytes are dropped, so the answer
    /// no longer points at them; the path strings stay for as long as the core runs.
    pub(crate) fn end_game_load(&self, strings: GameStrings) {
        self.with(|c| {
            if let Some(GameExt(ext)) = c.game_ext.as_mut() {
                ext.data = std::ptr::null();
                ext.size = 0;
            }
            c.game_strings = Some(strings);
        });
    }

    /// The core was unloaded (`retro_deinit` returned): the slot can serve another core.
    pub(crate) fn release(mut self) {
        self.released = true;
    }

    /// The core is left loaded (let go): the slot keeps it forever, with no context, so its
    /// threads, if they wake up, find nothing of the next session.
    pub(crate) fn retire(self) {
        log::warn!("libretro callback slot {} retired", self.index);
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        let state = &SLOTS[self.index];
        *state.ctx.lock() = None;
        if self.released {
            state.claimed.store(false, Ordering::Release);
        }
    }
}

/// Runs a callback's body, catching a panic instead of unwinding into the core's C code.
fn guard<R>(what: &str, fallback: R, f: impl FnOnce() -> R) -> R {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(_) => {
            log::error!("libretro callback {what} panicked: answered as failed");
            fallback
        }
    }
}

/// The core's log lines (`GET_LOG_INTERFACE`), to EmuDome's log under the target `core`.
///
/// The C logger is variadic (`level, fmt, ...`) and stable Rust cannot read the arguments: the
/// format string is logged as it is (`%s`, `%d` left unfilled). Enough to see what the core
/// says and when; the arguments need a C shim, which would put C code in this crate.
unsafe extern "C" fn core_log(level: c_uint, fmt: *const c_char) {
    guard("log", (), || {
        if fmt.is_null() {
            return;
        }
        let text = CStr::from_ptr(fmt).to_string_lossy();
        let text = text.trim_end();
        match level {
            RETRO_LOG_DEBUG => log::debug!(target: "core", "{text}"),
            RETRO_LOG_INFO => log::info!(target: "core", "{text}"),
            RETRO_LOG_WARN => log::warn!(target: "core", "{text}"),
            RETRO_LOG_ERROR => log::error!(target: "core", "{text}"),
            other => log::info!(target: "core", "[level {other}] {text}"),
        }
    });
}

/// The serialization quirks the host knows; the others are zeroed, as libretro asks.
const KNOWN_QUIRKS: u64 = RETRO_SERIALIZATION_QUIRK_INCOMPLETE
    | RETRO_SERIALIZATION_QUIRK_MUST_INITIALIZE
    | RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE
    | RETRO_SERIALIZATION_QUIRK_FRONT_VARIABLE_SIZE
    | RETRO_SERIALIZATION_QUIRK_SINGLE_SESSION
    | RETRO_SERIALIZATION_QUIRK_ENDIAN_DEPENDENT
    | RETRO_SERIALIZATION_QUIRK_PLATFORM_DEPENDENT;

unsafe extern "C" fn environment<const S: usize>(cmd: c_uint, data: *mut c_void) -> bool {
    guard("environment", false, || {
        environment_in(&SLOTS[S], cmd, data)
    })
}

/// Writes `value` through `data` when it is not null; false (unanswered) when it is.
unsafe fn answer<T>(data: *mut c_void, value: T) -> bool {
    if data.is_null() {
        return false;
    }
    *(data as *mut T) = value;
    true
}

/// Reads a `T` from `data` when it is not null.
unsafe fn read<T: Copy>(data: *mut c_void) -> Option<T> {
    (!data.is_null()).then(|| *(data as *const T))
}

unsafe fn environment_in(slot: &SlotState, raw: c_uint, data: *mut c_void) -> bool {
    // Hardware rendering goes to the session's handler (`hw_render`), by the raw number.
    if is_hw_command(raw) {
        return slot
            .with("hardware rendering", |c| {
                if raw == RETRO_ENVIRONMENT_SET_HW_RENDER {
                    c.hw.set_hw_render(data as *mut retro_hw_render_callback)
                } else {
                    c.hw.environment(raw, data)
                }
            })
            .unwrap_or(false);
    }
    // 44 is two commands: `SET_SERIALIZATION_QUIRKS` without the experimental flag,
    // `SET_HW_SHARED_CONTEXT` with it. Told apart before the flag is masked off.
    if raw == RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS {
        let Some(quirks) = read::<u64>(data) else {
            return false;
        };
        let kept = quirks & KNOWN_QUIRKS;
        *(data as *mut u64) = kept;
        log::info!("core serialization quirks: {kept:#x}");
        return slot
            .with("SET_SERIALIZATION_QUIRKS", |c| {
                c.shared.serialization_quirks.store(kept, Ordering::Relaxed);
            })
            .is_some();
    }
    let cmd = raw & !RETRO_ENVIRONMENT_EXPERIMENTAL;
    match cmd {
        RETRO_ENVIRONMENT_GET_CAN_DUPE => answer(data, true),
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => match read::<u32>(data) {
            Some(
                fmt @ (RETRO_PIXEL_FORMAT_0RGB1555
                | RETRO_PIXEL_FORMAT_XRGB8888
                | RETRO_PIXEL_FORMAT_RGB565),
            ) => slot
                .with("SET_PIXEL_FORMAT", |c| c.pixel_format = fmt)
                .is_some(),
            Some(_) | None => false,
        },
        RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => {
            !data.is_null()
                && slot
                    .with("GET_SYSTEM_DIRECTORY", |c| {
                        *(data as *mut *const c_char) = c.system_dir.as_ptr();
                    })
                    .is_some()
        }
        RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => {
            !data.is_null()
                && slot
                    .with("GET_SAVE_DIRECTORY", |c| {
                        *(data as *mut *const c_char) = c.save_dir.as_ptr();
                    })
                    .is_some()
        }
        RETRO_ENVIRONMENT_GET_USERNAME => {
            if data.is_null() {
                return false;
            }
            let name = slot
                .with("GET_USERNAME", |c| c.username.as_ref().map(|n| n.as_ptr()))
                .flatten();
            match name {
                Some(p) => answer(data, p),
                None => false,
            }
        }
        // The cores read the language for their own menus and messages; EmuDome sets the
        // console's language through its options (`consoles.dslang`), so English here, as an
        // unanswered call meant before.
        RETRO_ENVIRONMENT_GET_LANGUAGE => answer::<c_uint>(data, RETRO_LANGUAGE_ENGLISH),
        RETRO_ENVIRONMENT_GET_LOG_INTERFACE => answer(
            data,
            retro_log_callback {
                log: Some(core_log),
            },
        ),
        // Shown nowhere yet (no on-screen messages in game): logged.
        RETRO_ENVIRONMENT_SET_MESSAGE => match read::<retro_message>(data) {
            Some(m) if !m.msg.is_null() => {
                log::info!(target: "core", "message: {}", CStr::from_ptr(m.msg).to_string_lossy());
                true
            }
            Some(_) | None => false,
        },
        RETRO_ENVIRONMENT_GET_GAME_INFO_EXT => {
            if data.is_null() {
                return false;
            }
            let ext = slot
                .with("GET_GAME_INFO_EXT", |c| {
                    c.game_ext
                        .as_ref()
                        .map(|GameExt(e)| &**e as *const retro_game_info_ext)
                })
                .flatten();
            match ext {
                Some(p) => answer(data, p),
                None => false,
            }
        }
        // The devices the host feeds: RetroPad and the touch-screen pointer.
        RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES => answer::<u64>(
            data,
            (1 << RETRO_DEVICE_JOYPAD) | (1 << RETRO_DEVICE_POINTER) | (1 << RETRO_DEVICE_ANALOG),
        ),
        // Buttons can also be read as a single mask (faster for the core).
        x if x == (RETRO_ENVIRONMENT_GET_INPUT_BITMASKS & !RETRO_ENVIRONMENT_EXPERIMENTAL) => true,
        RETRO_ENVIRONMENT_SET_GEOMETRY => match read::<retro_game_geometry>(data) {
            Some(geo) => {
                slot.with("SET_GEOMETRY", |c| {
                    c.shared.stats.lock().av_info.geometry = geo;
                });
                true
            }
            None => false,
        },
        // The session picks the new timing up after the frame (pacing, rewind, the core's
        // `av_info`); the audio queue and the stats change at once.
        RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO => match read::<retro_system_av_info>(data) {
            Some(av) => {
                slot.with("SET_SYSTEM_AV_INFO", |c| {
                    c.shared.stats.lock().av_info = av;
                    c.shared.audio.set_rate(av.timing.sample_rate);
                    c.av_change = Some(av);
                });
                true
            }
            None => false,
        },
        // Accepted and ignored for now: button descriptors, how options are shown.
        RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY => {
            true
        }
        // Core options: the core declares them in one of four layouts, the shell answers
        // GET_VARIABLE (core_options.rs). Version 2 lets the cores send their richest layout.
        RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => answer::<c_uint>(data, 2),
        RETRO_ENVIRONMENT_SET_VARIABLES => {
            define(slot, variables_v0(data as *const retro_variable))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS => define(
            slot,
            definitions(data as *const retro_core_option_definition),
        ),
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL => {
            let intl = data as *const retro_core_options_intl;
            !intl.is_null() && define(slot, definitions((*intl).us))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 => {
            let opts = data as *const retro_core_options_v2;
            !opts.is_null() && define(slot, definitions((*opts).definitions))
        }
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
            let intl = data as *const retro_core_options_v2_intl;
            if intl.is_null() || (*intl).us.is_null() {
                return false;
            }
            define(slot, definitions((*(*intl).us).definitions))
        }
        RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
            if data.is_null() {
                return false;
            }
            let updated = slot
                .with("GET_VARIABLE_UPDATE", |c| c.options.take_updated())
                .unwrap_or(false);
            answer(data, updated)
        }
        RETRO_ENVIRONMENT_GET_VARIABLE => {
            let var = data as *mut retro_variable;
            if var.is_null() || (*var).key.is_null() {
                return false;
            }
            let key = CStr::from_ptr((*var).key).to_string_lossy();
            // The pointer stays valid while the answer does not change (core_options.rs).
            let value = slot
                .with("GET_VARIABLE", |c| c.options.answer(&key).map(CStr::as_ptr))
                .flatten();
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
        other => {
            log::debug!("environment call {other} (raw {raw:#x}) not supported");
            false
        }
    }
}

/// Stores the definitions the core declared; false when there is no session to store them in.
fn define(slot: &SlotState, defs: Vec<OptionDef>) -> bool {
    log::debug!("core declared {} options", defs.len());
    slot.with("SET_CORE_OPTIONS", |c| {
        c.options.define(defs);
        *c.shared.core_options.lock() = c.options.declared();
    })
    .is_some()
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

/// The fields a core option definition has in both layouts: v1 (`SET_CORE_OPTIONS`) and v2
/// (`SET_CORE_OPTIONS_V2`, the same with categories, which the shell ignores for now).
trait OptionDefinition {
    fn key(&self) -> *const c_char;
    fn desc(&self) -> *const c_char;
    fn values(&self) -> &[retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX];
    fn default_value(&self) -> *const c_char;
}

impl OptionDefinition for retro_core_option_definition {
    fn key(&self) -> *const c_char {
        self.key
    }
    fn desc(&self) -> *const c_char {
        self.desc
    }
    fn values(&self) -> &[retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX] {
        &self.values
    }
    fn default_value(&self) -> *const c_char {
        self.default_value
    }
}

impl OptionDefinition for retro_core_option_v2_definition {
    fn key(&self) -> *const c_char {
        self.key
    }
    fn desc(&self) -> *const c_char {
        self.desc
    }
    fn values(&self) -> &[retro_core_option_value; RETRO_NUM_CORE_OPTION_VALUES_MAX] {
        &self.values
    }
    fn default_value(&self) -> *const c_char {
        self.default_value
    }
}

/// An array of definitions (v1 or v2) ending with a null key.
unsafe fn definitions<D: OptionDefinition>(mut d: *const D) -> Vec<OptionDef> {
    let mut defs = Vec::new();
    while !d.is_null() && !(*d).key().is_null() {
        if let Some(key) = c_string((*d).key()) {
            defs.push(
                OptionDef::new(key, values((*d).values()), c_string((*d).default_value()))
                    .with_label(c_string((*d).desc())),
            );
        }
        d = d.add(1);
    }
    defs
}

unsafe extern "C" fn video_refresh<const S: usize>(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    guard("video_refresh", (), || {
        if data.is_null() {
            return; // duplicated frame: the previous one stays
        }
        SLOTS[S].with("video_refresh", |c| {
            let frame = if data == RETRO_HW_FRAME_BUFFER_VALID {
                // The frame is on the GPU: the hardware handler knows where.
                match c.hw.hw_frame(width, height) {
                    Some(gpu) => VideoFrame::Gpu(gpu),
                    None => return,
                }
            } else {
                // `height` rows `pitch` apart, the last one only `width` pixels long.
                let len = (height as usize).saturating_sub(1) * pitch
                    + width as usize * bytes_per_pixel(c.pixel_format);
                VideoFrame::Cpu(CpuFrame {
                    data: std::slice::from_raw_parts(data as *const u8, len),
                    width,
                    height,
                    pitch,
                    format: c.pixel_format,
                })
            };
            c.video.frame(frame, c.split);
        });
    });
}

/// The core's samples to the audio ring, through [`SpeedAudio`] off 1×. The time-stretch runs
/// here, before the push, and the push is lock-free: the audio output never waits for it.
fn push_audio(c: &mut CallbackCtx, samples: &[i16]) {
    if !c.audio_enabled {
        return;
    }
    let ring = &c.shared.audio;
    if c.speed_audio.is_passthrough() {
        c.audio.push(ring, samples);
        return;
    }
    c.audio_out.clear();
    c.speed_audio
        .process(ring.sample_rate(), samples, &mut c.audio_out);
    if !c.audio_out.is_empty() {
        c.audio.push(ring, &c.audio_out);
    }
}

unsafe extern "C" fn audio_sample<const S: usize>(left: i16, right: i16) {
    guard("audio_sample", (), || {
        SLOTS[S].with("audio_sample", |c| push_audio(c, &[left, right]));
    });
}

unsafe extern "C" fn audio_sample_batch<const S: usize>(data: *const i16, frames: usize) -> usize {
    guard("audio_sample_batch", frames, || {
        if !data.is_null() {
            let samples = std::slice::from_raw_parts(data, frames * 2);
            SLOTS[S].with("audio_sample_batch", |c| push_audio(c, samples));
        }
        frames
    })
}

/// The core is about to read its input: it reads this snapshot until the next poll (or the next
/// frame, [`Slot::refresh_input`]), never input changing between two of its reads.
unsafe extern "C" fn input_poll<const S: usize>() {
    guard("input_poll", (), || {
        SLOTS[S].with("input_poll", |c| c.input = c.shared.input.snapshot());
    });
}

unsafe extern "C" fn input_state<const S: usize>(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    guard("input_state", 0, || {
        input_state_in(&SLOTS[S], port, device, index, id)
    })
}

fn input_state_in(
    slot: &SlotState,
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    slot.with("input_state", |c| {
        read_input(&c.input, port, device, index, id)
    })
    .unwrap_or(0)
}

/// One `input_state` answer from the frozen input: port 0 only (one player).
fn read_input(s: &Snapshot, port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16 {
    if port != 0 {
        return 0;
    }
    match device {
        // One pointer (index 0): the touch screen.
        RETRO_DEVICE_POINTER if index == 0 => match id {
            RETRO_DEVICE_ID_POINTER_X => s.pointer.x,
            RETRO_DEVICE_ID_POINTER_Y => s.pointer.y,
            RETRO_DEVICE_ID_POINTER_PRESSED | RETRO_DEVICE_ID_POINTER_COUNT => {
                i16::from(s.pointer.pressed)
            }
            _ => 0,
        },
        RETRO_DEVICE_JOYPAD => {
            if id == RETRO_DEVICE_ID_JOYPAD_MASK {
                s.mask as i16
            } else if id < 16 {
                ((s.mask >> id) & 1) as i16
            } else {
                0
            }
        }
        RETRO_DEVICE_ANALOG => {
            let stick = match index {
                RETRO_DEVICE_INDEX_ANALOG_LEFT => s.sticks.left,
                RETRO_DEVICE_INDEX_ANALOG_RIGHT => s.sticks.right,
                // Analog buttons: the digital ones, fully pressed or not.
                RETRO_DEVICE_INDEX_ANALOG_BUTTON => {
                    return if id < 16 && (s.mask >> id) & 1 != 0 {
                        0x7fff
                    } else {
                        0
                    };
                }
                _ => return 0,
            };
            match id {
                RETRO_DEVICE_ID_ANALOG_X => stick.0,
                RETRO_DEVICE_ID_ANALOG_Y => stick.1,
                _ => 0,
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{c_void, CString};
    use std::sync::Arc;

    use libretro_sys::{
        retro_log_callback, retro_system_av_info, RETRO_ENVIRONMENT_GET_LANGUAGE,
        RETRO_ENVIRONMENT_GET_LOG_INTERFACE, RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
        RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY, RETRO_ENVIRONMENT_SET_GEOMETRY,
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
        RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO, RETRO_LANGUAGE_ENGLISH,
    };

    use super::{CallbackCtx, Slot, SLOTS};
    use crate::session::Shared;

    /// The slots are static: the tests that claim them run one at a time, so a slot freed by one
    /// is not taken by another before it looks.
    static SERIAL: parking_lot::Mutex<()> = parking_lot::const_mutex(());

    fn ctx() -> (Arc<Shared>, CallbackCtx) {
        let shared = Arc::new(Shared::default());
        let ctx = CallbackCtx::new(
            shared.clone(),
            CString::new("sys").unwrap(),
            CString::new("save").unwrap(),
        );
        (shared, ctx)
    }

    /// The tests share the static slots: each claims its own, and they never assume an index.
    #[test]
    fn a_core_thread_reaches_its_slot_and_a_retired_slot_reaches_nothing() {
        let _serial = SERIAL.lock();
        let (shared, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let env = slot.callbacks().environment;
        // From another thread (a core's worker): the context is there.
        let fps = std::thread::spawn(move || {
            let mut av = retro_system_av_info::default();
            av.timing.fps = 30.0;
            av.timing.sample_rate = 32768.0;
            unsafe {
                env(
                    RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO,
                    &mut av as *mut _ as *mut c_void,
                )
            }
        })
        .join()
        .unwrap();
        assert!(fps);
        assert_eq!(shared.stats.lock().av_info.timing.fps, 30.0);
        assert_eq!(slot.take_av_change().map(|av| av.timing.fps), Some(30.0));
        assert!(slot.take_av_change().is_none());

        let index = slot.index;
        slot.retire();
        // Retired: still claimed, no context; the core's calls are answered as unsupported.
        assert!(SLOTS[index]
            .claimed
            .load(std::sync::atomic::Ordering::Acquire));
        let mut xrgb8888 = 1u32;
        assert!(!unsafe {
            env(
                RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
                &mut xrgb8888 as *mut _ as *mut c_void,
            )
        });
    }

    #[test]
    fn a_released_slot_is_free_again() {
        let _serial = SERIAL.lock();
        let (_, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let index = slot.index;
        slot.release();
        assert!(!SLOTS[index]
            .claimed
            .load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn null_data_is_refused_not_read() {
        let _serial = SERIAL.lock();
        let (_, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let env = slot.callbacks().environment;
        for cmd in [
            RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
            RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
            RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
            RETRO_ENVIRONMENT_SET_GEOMETRY,
            RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO,
            RETRO_ENVIRONMENT_GET_LANGUAGE,
            RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
            RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
        ] {
            assert!(!unsafe { env(cmd, std::ptr::null_mut()) }, "cmd {cmd}");
        }
        slot.release();
    }

    #[test]
    fn language_log_and_quirks_are_answered() {
        let _serial = SERIAL.lock();
        let (shared, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let env = slot.callbacks().environment;
        let mut lang = 99u32;
        assert!(unsafe {
            env(
                RETRO_ENVIRONMENT_GET_LANGUAGE,
                &mut lang as *mut _ as *mut c_void,
            )
        });
        assert_eq!(lang, RETRO_LANGUAGE_ENGLISH);

        let mut log = retro_log_callback { log: None };
        assert!(unsafe {
            env(
                RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
                &mut log as *mut _ as *mut c_void,
            )
        });
        let line = CString::new("core says %s\n").unwrap();
        unsafe { (log.log.unwrap())(1, line.as_ptr()) };

        // An unknown quirk bit is zeroed; the known ones are kept and recorded.
        let mut quirks: u64 = (1 << 2) | (1 << 40);
        assert!(unsafe {
            env(
                RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS,
                &mut quirks as *mut _ as *mut c_void,
            )
        });
        assert_eq!(quirks, 1 << 2);
        assert_eq!(
            shared
                .serialization_quirks
                .load(std::sync::atomic::Ordering::Relaxed),
            1 << 2
        );
        slot.release();
    }

    /// A handler that accepts Vulkan and has a GPU frame; a sink that records what it got.
    struct Accepting;
    impl crate::hw_render::HwRenderHandler for Accepting {
        unsafe fn set_hw_render(
            &mut self,
            request: *mut libretro_sys::retro_hw_render_callback,
        ) -> bool {
            !request.is_null()
        }
        unsafe fn environment(&mut self, _cmd: u32, _data: *mut c_void) -> bool {
            true
        }
        fn hw_frame(&mut self, width: u32, height: u32) -> Option<crate::video_sink::GpuFrame> {
            Some(crate::video_sink::GpuFrame {
                width,
                height,
                image: crate::video_sink::GpuImage {
                    handle: 7,
                    format: 0,
                    layout: 0,
                },
                sync: crate::video_sink::GpuSync::default(),
            })
        }
    }

    struct Recording(Arc<parking_lot::Mutex<Vec<String>>>);
    impl crate::video_sink::VideoSink for Recording {
        fn frame(
            &mut self,
            frame: crate::video_sink::VideoFrame<'_>,
            _split: crate::screens::Split,
        ) {
            self.0.lock().push(match frame {
                crate::video_sink::VideoFrame::Cpu(c) => format!("cpu {}x{}", c.width, c.height),
                crate::video_sink::VideoFrame::Gpu(g) => {
                    format!("gpu {}x{} #{}", g.width, g.height, g.image.handle)
                }
            });
        }
    }

    #[test]
    fn hardware_calls_and_frames_go_to_the_sessions_handler_and_sink() {
        let _serial = SERIAL.lock();
        let (shared, mut ctx) = ctx();
        let got = Arc::new(parking_lot::Mutex::new(Vec::new()));
        ctx.video = Box::new(Recording(got.clone()));
        ctx.hw = Box::new(Accepting);
        let slot = Slot::claim(ctx).unwrap();
        let cb = slot.callbacks();
        let mut preferred = 0u32;
        assert!(unsafe {
            (cb.environment)(
                libretro_sys::RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
                &mut preferred as *mut _ as *mut c_void,
            )
        });
        let pixels = [0u8; 4];
        unsafe {
            (cb.video_refresh)(libretro_sys::RETRO_HW_FRAME_BUFFER_VALID, 400, 480, 0);
            (cb.video_refresh)(pixels.as_ptr() as *const c_void, 2, 1, 4);
            (cb.video_refresh)(std::ptr::null(), 2, 1, 4);
        }
        assert_eq!(*got.lock(), ["gpu 400x480 #7", "cpu 2x1"]);
        assert_eq!(
            shared.frame.lock().seq,
            0,
            "the recording sink replaced the default"
        );
        slot.release();
    }

    #[test]
    fn the_default_handler_refuses_hardware_rendering() {
        let _serial = SERIAL.lock();
        let (_, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let env = slot.callbacks().environment;
        let mut preferred = 0u32;
        assert!(!unsafe {
            env(
                libretro_sys::RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
                &mut preferred as *mut _ as *mut c_void,
            )
        });
        slot.release();
    }

    #[test]
    fn input_reads_come_from_the_snapshot_not_the_live_state() {
        use crate::input::{Button, Source, Sticks};
        use libretro_sys::{
            RETRO_DEVICE_ANALOG, RETRO_DEVICE_ID_ANALOG_X, RETRO_DEVICE_ID_ANALOG_Y,
            RETRO_DEVICE_INDEX_ANALOG_BUTTON, RETRO_DEVICE_INDEX_ANALOG_LEFT,
            RETRO_DEVICE_INDEX_ANALOG_RIGHT, RETRO_DEVICE_JOYPAD,
        };
        let _serial = SERIAL.lock();
        let (shared, ctx) = ctx();
        let slot = Slot::claim(ctx).unwrap();
        let cb = slot.callbacks();
        shared.input.set(Source::Gamepad, Button::A, true);
        shared.input.set_sticks(Sticks {
            left: (-20000, 300),
            right: (0, -32768),
        });
        let read = |device, index, id| unsafe { (cb.input_state)(0, device, index, id) };
        let (left, right) = (
            RETRO_DEVICE_INDEX_ANALOG_LEFT,
            RETRO_DEVICE_INDEX_ANALOG_RIGHT,
        );
        let (x, y) = (RETRO_DEVICE_ID_ANALOG_X, RETRO_DEVICE_ID_ANALOG_Y);
        // Not polled yet: the empty snapshot.
        assert_eq!(read(RETRO_DEVICE_JOYPAD, 0, 8), 0);
        unsafe { (cb.input_poll)() };
        assert_eq!(read(RETRO_DEVICE_JOYPAD, 0, 8), 1);
        assert_eq!(read(RETRO_DEVICE_ANALOG, left, x), -20000);
        assert_eq!(read(RETRO_DEVICE_ANALOG, left, y), 300);
        assert_eq!(read(RETRO_DEVICE_ANALOG, right, y), -32768);
        assert_eq!(
            read(RETRO_DEVICE_ANALOG, RETRO_DEVICE_INDEX_ANALOG_BUTTON, 8),
            0x7fff
        );
        // Released mid-frame: the core keeps seeing the polled state until it polls again.
        shared.input.set(Source::Gamepad, Button::A, false);
        assert_eq!(read(RETRO_DEVICE_JOYPAD, 0, 8), 1);
        slot.refresh_input();
        assert_eq!(read(RETRO_DEVICE_JOYPAD, 0, 8), 0);
        // Other ports read nothing.
        assert_eq!(unsafe { (cb.input_state)(1, RETRO_DEVICE_JOYPAD, 0, 8) }, 0);
        slot.release();
    }
}
