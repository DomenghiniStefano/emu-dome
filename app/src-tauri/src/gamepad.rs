//! Controllers through SDL3 (only the gamepad subsystem, no SDL windows):
//! reads the pads, lets [`emudome_core::hotkeys::Router`] decide what the buttons mean,
//! and carries out the result.
//!
//! SDL recognises Xbox, PS4/PS5, Switch Pro, the pads in its database and the
//! Steam Controller, read directly through HIDAPI without Steam running.
//!
//! Each pad is read by **position** ([`Pos`]: right face button, bottom one...); triggers past
//! `pad.trig` are L2/R2, the left stick past `pad.stickdpad` (outside the `pad.dead` dead zone)
//! is the d-pad. The router works on positions; each pad's [`PadMap`] (`pad.remap` for its
//! family, `pad.ab`) then gives the game its buttons. Everything is read from the settings
//! through [`PadConfig`] and reloaded at every settings change (`pads::settings_changed`).
//!
//! Routing (pure, in `emudome_core::hotkeys`): in game the buttons go to the core;
//! with the shortcut button (L3) held they are combos; while the interface has the input
//! (quick menu open, or no game) the d-pad and A/B become `emu://nav` events and the game
//! gets nothing. While the Controls section captures a button, nothing is routed.
//!
//! Events to the interface:
//! - `emu://gamepad-connected`, `emu://gamepad-disconnected`: a [`PadInfo`];
//! - `emu://gamepad-changed`: a [`PadInfo`] whose battery charge changed;
//! - `emu://pad-state`: a [`PadStates`], while the interface watches (`gamepad_watch`);
//! - the router's actions (`emu://menu`, `emu://nav`, `emu://hotkey`) through the shortcuts'
//!   runner shared with the keyboard (`hotkeys.rs`).

use std::ffi::{c_char, c_int, CStr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use emudome_core::bindings::{Family, PadConfig, PadMap, Pos};
use emudome_core::hotkeys::{PadInput, Router};
use emudome_core::input::{Source, Sticks};
use emudome_core::ErrorCode;
use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{
    SDL_Event, SDL_PollEvent, SDL_EVENT_GAMEPAD_ADDED, SDL_EVENT_GAMEPAD_REMOVED,
    SDL_EVENT_JOYSTICK_BATTERY_UPDATED,
};
use sdl3_sys::gamepad::{
    SDL_CloseGamepad, SDL_Gamepad, SDL_GamepadAxis, SDL_GamepadButton, SDL_GamepadType,
    SDL_GetGamepadAxis, SDL_GetGamepadButton, SDL_GetGamepadName, SDL_GetGamepadPowerInfo,
    SDL_GetGamepadProduct, SDL_GetGamepadStringForType, SDL_GetGamepadType, SDL_GetGamepadVendor,
    SDL_OpenGamepad, SDL_GAMEPAD_AXIS_LEFTX, SDL_GAMEPAD_AXIS_LEFTY, SDL_GAMEPAD_AXIS_LEFT_TRIGGER,
    SDL_GAMEPAD_AXIS_RIGHTX, SDL_GAMEPAD_AXIS_RIGHTY, SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
    SDL_GAMEPAD_BUTTON_BACK, SDL_GAMEPAD_BUTTON_DPAD_DOWN, SDL_GAMEPAD_BUTTON_DPAD_LEFT,
    SDL_GAMEPAD_BUTTON_DPAD_RIGHT, SDL_GAMEPAD_BUTTON_DPAD_UP, SDL_GAMEPAD_BUTTON_EAST,
    SDL_GAMEPAD_BUTTON_GUIDE, SDL_GAMEPAD_BUTTON_LEFT_SHOULDER, SDL_GAMEPAD_BUTTON_LEFT_STICK,
    SDL_GAMEPAD_BUTTON_NORTH, SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER, SDL_GAMEPAD_BUTTON_RIGHT_STICK,
    SDL_GAMEPAD_BUTTON_SOUTH, SDL_GAMEPAD_BUTTON_START, SDL_GAMEPAD_BUTTON_WEST,
    SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT, SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR,
    SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT, SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO,
    SDL_GAMEPAD_TYPE_PS3, SDL_GAMEPAD_TYPE_PS4, SDL_GAMEPAD_TYPE_PS5, SDL_GAMEPAD_TYPE_STEAM,
    SDL_GAMEPAD_TYPE_XBOX360, SDL_GAMEPAD_TYPE_XBOXONE,
};
use sdl3_sys::hints::{
    SDL_SetHint, SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, SDL_HINT_JOYSTICK_HIDAPI_STEAM,
};
use sdl3_sys::init::{SDL_Init, SDL_INIT_GAMEPAD};
use sdl3_sys::joystick::SDL_JoystickID;
use sdl3_sys::power::{
    SDL_PowerState, SDL_POWERSTATE_CHARGED, SDL_POWERSTATE_CHARGING, SDL_POWERSTATE_ON_BATTERY,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::health::{Part, SharedHealth};
use crate::hotkeys::{HotkeyRunner, Origin};
use crate::pads::{self, PadInfo, PadState, PadStates, PadsState};
use crate::CurrentShared;

/// SDL's buttons by position.
const MAP: &[(SDL_GamepadButton, Pos)] = &[
    (SDL_GAMEPAD_BUTTON_EAST, Pos::East),
    (SDL_GAMEPAD_BUTTON_SOUTH, Pos::South),
    (SDL_GAMEPAD_BUTTON_NORTH, Pos::North),
    (SDL_GAMEPAD_BUTTON_WEST, Pos::West),
    (SDL_GAMEPAD_BUTTON_LEFT_SHOULDER, Pos::L1),
    (SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER, Pos::R1),
    (SDL_GAMEPAD_BUTTON_BACK, Pos::Back),
    (SDL_GAMEPAD_BUTTON_START, Pos::Start),
    (SDL_GAMEPAD_BUTTON_DPAD_UP, Pos::Up),
    (SDL_GAMEPAD_BUTTON_DPAD_DOWN, Pos::Down),
    (SDL_GAMEPAD_BUTTON_DPAD_LEFT, Pos::Left),
    (SDL_GAMEPAD_BUTTON_DPAD_RIGHT, Pos::Right),
    (SDL_GAMEPAD_BUTTON_LEFT_STICK, Pos::L3),
    (SDL_GAMEPAD_BUTTON_RIGHT_STICK, Pos::R3),
];

/// The axes in `PadState::axes` order.
const AXES: [SDL_GamepadAxis; 6] = [
    SDL_GAMEPAD_AXIS_LEFTX,
    SDL_GAMEPAD_AXIS_LEFTY,
    SDL_GAMEPAD_AXIS_RIGHTX,
    SDL_GAMEPAD_AXIS_RIGHTY,
    SDL_GAMEPAD_AXIS_LEFT_TRIGGER,
    SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
];

/// How often the controllers are read.
const POLL: Duration = Duration::from_millis(4);

/// `emu://pad-state` at most this often (a stick moves at every poll).
const STATE_EVERY: Duration = Duration::from_millis(16);

/// An open controller. The pointer stays valid until `SDL_CloseGamepad`.
struct Pad {
    info: PadInfo,
    id: SDL_JoystickID,
    raw: *mut SDL_Gamepad,
    /// How its positions give the game its buttons (from `pad.remap` for its family).
    map: PadMap,
}

/// One poll of one pad.
/// The sticks the game gets: the first pad with a stick off centre (several pads are one player).
fn game_sticks(readings: &[Reading]) -> Sticks {
    readings
        .iter()
        .map(|r| Sticks {
            left: (r.axes[0], r.axes[1]),
            right: (r.axes[2], r.axes[3]),
        })
        .find(|s| *s != Sticks::default())
        .unwrap_or_default()
}

struct Reading {
    /// Positions held: buttons and triggers past the threshold, plus the left stick as d-pad.
    input: PadInput,
    /// The same without the stick, for the tester.
    held: u16,
    axes: [i16; 6],
}

impl Pad {
    fn log_name(&self) -> &str {
        self.info.name.as_deref().unwrap_or("(unnamed)")
    }

    fn read(&self, config: &PadConfig) -> Reading {
        let mut held = 0;
        let mut axes = [0i16; 6];
        let guide;
        // SAFETY: `self.raw` is open while the pad is in the list.
        unsafe {
            for (p, pos) in MAP {
                if SDL_GetGamepadButton(self.raw, *p) {
                    held |= pos.bit();
                }
            }
            for (slot, axis) in axes.iter_mut().zip(AXES) {
                *slot = SDL_GetGamepadAxis(self.raw, axis);
            }
            guide = SDL_GetGamepadButton(self.raw, SDL_GAMEPAD_BUTTON_GUIDE);
        }
        let threshold = config.trigger_threshold();
        if axes[4] > threshold {
            held |= Pos::L2.bit();
        }
        if axes[5] > threshold {
            held |= Pos::R2.bit();
        }
        Reading {
            input: PadInput {
                buttons: held | config.stick_dpad_mask(axes[0], axes[1]),
                guide,
            },
            held,
            axes,
        }
    }
}

/// Starts the controller thread. A thread that cannot start, SDL that does not initialise, or a
/// panic on the thread is reported to the interface (`controllersUnavailable`, `threadStopped`).
pub fn spawn(
    app: AppHandle,
    current: CurrentShared,
    menu_open: Arc<AtomicBool>,
    pads: PadsState,
    hotkeys: Arc<HotkeyRunner>,
    health: SharedHealth,
) {
    let thread_health = health.clone();
    let spawned = std::thread::Builder::new()
        .name("emudome-gamepad".into())
        .spawn(move || {
            let reported = thread_health.clone();
            thread_health.guard(Part::Controllers, || {
                run(app, current, menu_open, pads, &hotkeys, &reported);
            });
        });
    if let Err(e) = spawned {
        log::error!("controller thread not started: {e}");
        health.set(
            Part::Controllers,
            Some(ErrorCode::ControllersUnavailable {
                detail: e.to_string(),
            }),
        );
    }
}

fn run(
    app: AppHandle,
    current: CurrentShared,
    menu_open: Arc<AtomicBool>,
    shared_pads: PadsState,
    hotkeys: &HotkeyRunner,
    health: &SharedHealth,
) {
    // SAFETY: SDL is initialised and used by this thread only;
    // hints must be set before SDL_Init.
    unsafe {
        // The focus is in WebView2, which is another process: without this
        // SDL would ignore the buttons as if the window were in the background.
        SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
        // Steam Controller read directly, without Steam.
        SDL_SetHint(SDL_HINT_JOYSTICK_HIDAPI_STEAM, c"1".as_ptr());
        if !SDL_Init(SDL_INIT_GAMEPAD) {
            let detail = text(SDL_GetError());
            log::warn!("controllers unavailable: {detail}");
            health.set(
                Part::Controllers,
                Some(ErrorCode::ControllersUnavailable { detail }),
            );
            return;
        }
    }
    log::info!("controllers: SDL3 ready");

    let mut pads: Vec<Pad> = Vec::new();
    let mut config = PadConfig::default();
    // The generation the config was read at; `None` until the settings are open.
    let mut config_at: Option<u64> = None;
    let mut router = Router::new(config.bindings());
    let mut last_state: Option<PadStates> = None;
    let mut last_state_at = Instant::now();
    let mut event = SDL_Event::default();
    loop {
        let generation = pads::config_generation();
        if config_at != Some(generation) {
            if let Some(fresh) = pads::current_config(&app) {
                config = fresh;
                config_at = Some(generation);
                router.set_bindings(config.bindings());
                for pad in &mut pads {
                    pad.map = config.pad_map(pad.info.family);
                }
            }
        }

        // SAFETY: SDL_PollEvent writes a valid SDL_Event; the union field
        // read matches the event's type.
        while unsafe { SDL_PollEvent(&mut event) } {
            let kind = unsafe { event.r#type };
            if kind == SDL_EVENT_GAMEPAD_ADDED {
                let id = unsafe { event.gdevice.which };
                if let Some(pad) = open(id, &config) {
                    emit(&app, "emu://gamepad-connected", pad.info.clone());
                    shared_pads.list.lock().push(pad.info.clone());
                    pads.push(pad);
                }
            } else if kind == SDL_EVENT_JOYSTICK_BATTERY_UPDATED {
                let (id, state, percent) = unsafe {
                    (
                        event.jbattery.which,
                        event.jbattery.state,
                        event.jbattery.percent,
                    )
                };
                if let Some(pad) = pads.iter_mut().find(|p| p.id == id) {
                    let battery = battery(state, percent);
                    if pad.info.battery != battery {
                        pad.info.battery = battery;
                        for listed in shared_pads.list.lock().iter_mut() {
                            if listed.id == pad.info.id {
                                listed.battery = battery;
                            }
                        }
                        emit(&app, "emu://gamepad-changed", pad.info.clone());
                    }
                }
            } else if kind == SDL_EVENT_GAMEPAD_REMOVED {
                let id = unsafe { event.gdevice.which };
                if let Some(i) = pads.iter().position(|p| p.id == id) {
                    let pad = pads.remove(i);
                    log::info!("controller disconnected: {}", pad.log_name());
                    shared_pads.list.lock().retain(|p| p.id != pad.info.id);
                    emit(&app, "emu://gamepad-disconnected", pad.info.clone());
                    // SAFETY: opened by `open`, closed only once.
                    unsafe { SDL_CloseGamepad(pad.raw) };
                }
            }
        }

        let readings: Vec<Reading> = pads.iter().map(|p| p.read(&config)).collect();
        let input = readings
            .iter()
            .fold(PadInput::default(), |all, r| PadInput {
                buttons: all.buttons | r.input.buttons,
                guide: all.guide || r.input.guide,
            });

        if shared_pads.watch.load(Ordering::Relaxed) {
            let states = PadStates {
                pads: pads
                    .iter()
                    .zip(&readings)
                    .map(|(pad, r)| PadState {
                        id: pad.info.id,
                        held: Pos::in_mask(r.held),
                        axes: r.axes,
                    })
                    .collect(),
            };
            let now = Instant::now();
            if last_state.as_ref() != Some(&states) && now - last_state_at >= STATE_EVERY {
                emit(&app, "emu://pad-state", states.clone());
                last_state = Some(states);
                last_state_at = now;
            }
        } else {
            last_state = None;
        }

        let shared = current.load_full();
        if shared_pads.capture.load(Ordering::Relaxed) {
            // The Controls section is waiting for a button: nothing plays, nothing navigates.
            router.absorb(input);
            if let Some(shared) = shared {
                shared.input.set_mask(Source::Gamepad, 0);
                shared.input.set_sticks(Sticks::default());
            }
            std::thread::sleep(POLL);
            continue;
        }
        // Without a game the interface (start screen) takes the controller.
        let ui_has_input = shared.is_none() || menu_open.load(Ordering::Relaxed);
        let routed = router.step(input, ui_has_input, Instant::now());
        if let Some(shared) = shared {
            // The router lets positions through; each pad turns its own into game buttons.
            let mask = pads.iter().zip(&readings).fold(0, |mask, (pad, r)| {
                mask | pad.map.apply(r.input.buttons & routed.game_mask)
            });
            shared.input.set_mask(Source::Gamepad, mask);
            // The sticks as they are (cores keep their own dead zones), from the first pad that
            // moves one; centred while the menu or a combo has the controller.
            shared.input.set_sticks(if routed.game_has_input {
                game_sticks(&readings)
            } else {
                Sticks::default()
            });
        }
        hotkeys.run(routed.actions, Origin::Pads);
        std::thread::sleep(POLL);
    }
}

fn emit<S: Serialize + Clone>(app: &AppHandle, name: &str, payload: S) {
    if let Err(e) = app.emit(name, payload) {
        log::warn!("{name} not delivered: {e}");
    }
}

/// The remap family of an SDL controller type.
fn family(kind: SDL_GamepadType) -> Family {
    if kind == SDL_GAMEPAD_TYPE_XBOX360
        || kind == SDL_GAMEPAD_TYPE_XBOXONE
        || kind == SDL_GAMEPAD_TYPE_STEAM
    {
        Family::Xbox
    } else if kind == SDL_GAMEPAD_TYPE_PS3
        || kind == SDL_GAMEPAD_TYPE_PS4
        || kind == SDL_GAMEPAD_TYPE_PS5
    {
        Family::PlayStation
    } else if kind == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO
        || kind == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR
        || kind == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT
        || kind == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT
    {
        Family::Switch
    } else {
        Family::Generic
    }
}

/// The charge to show from SDL's power info: a percentage while on battery (charging or not),
/// `None` for wired controllers and when SDL cannot tell.
fn battery(state: SDL_PowerState, percent: c_int) -> Option<u8> {
    let on_battery = state == SDL_POWERSTATE_ON_BATTERY
        || state == SDL_POWERSTATE_CHARGING
        || state == SDL_POWERSTATE_CHARGED;
    if on_battery {
        u8::try_from(percent).ok().filter(|p| *p <= 100)
    } else {
        None
    }
}

/// Opens the controller and logs its name, type and USB ids, so whoever tests
/// can tell exactly which model is connected.
fn open(id: SDL_JoystickID, config: &PadConfig) -> Option<Pad> {
    // SAFETY: SDL calls on the thread that initialised it.
    unsafe {
        let raw = SDL_OpenGamepad(id);
        if raw.is_null() {
            log::warn!(
                "controller {} cannot be opened: {}",
                id.0,
                text(SDL_GetError())
            );
            return None;
        }
        let name = Some(text(SDL_GetGamepadName(raw))).filter(|n| !n.is_empty());
        let sdl_type = SDL_GetGamepadType(raw);
        let kind = text(SDL_GetGamepadStringForType(sdl_type));
        let family = family(sdl_type);
        let mut percent: c_int = -1;
        let power = SDL_GetGamepadPowerInfo(raw, &mut percent);
        let info = PadInfo {
            id: id.0,
            name,
            family,
            vendor: SDL_GetGamepadVendor(raw),
            product: SDL_GetGamepadProduct(raw),
            battery: battery(power, percent),
        };
        let pad = Pad {
            map: config.pad_map(family),
            info,
            id,
            raw,
        };
        log::info!(
            "controller connected: {} (type {kind}, family {}, vid {:04x}, pid {:04x}, battery {:?})",
            pad.log_name(),
            family.id(),
            pad.info.vendor,
            pad.info.product,
            pad.info.battery,
        );
        Some(pad)
    }
}

/// Copies an SDL C string; empty when the pointer is null.
///
/// # Safety
/// `p` must be null or a valid C string.
unsafe fn text(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}
