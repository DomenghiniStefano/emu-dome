//! The desktop's controllers through SDL3: only the gamepad subsystem, no SDL window.
//!
//! SDL recognises Xbox, PS4/PS5, Switch Pro, the pads in its database and the Steam Controller,
//! read directly through HIDAPI without Steam running. Every pad comes with SDL's standard
//! layout, so its buttons are already positions.

use std::ffi::{c_char, c_int, CStr};

use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{
    SDL_Event, SDL_PollEvent, SDL_EVENT_GAMEPAD_ADDED, SDL_EVENT_GAMEPAD_REMOVED,
    SDL_EVENT_JOYSTICK_BATTERY_UPDATED,
};
use sdl3_sys::gamepad::{
    SDL_CloseGamepad, SDL_Gamepad, SDL_GamepadAxis, SDL_GamepadButton, SDL_GamepadType,
    SDL_GetGamepadAxis, SDL_GetGamepadButton, SDL_GetGamepadName, SDL_GetGamepadPowerInfo,
    SDL_GetGamepadProduct, SDL_GetGamepadStringForType, SDL_GetGamepadType, SDL_GetGamepadVendor,
    SDL_GetGamepads, SDL_OpenGamepad, SDL_GAMEPAD_AXIS_LEFTX, SDL_GAMEPAD_AXIS_LEFTY,
    SDL_GAMEPAD_AXIS_LEFT_TRIGGER, SDL_GAMEPAD_AXIS_RIGHTX, SDL_GAMEPAD_AXIS_RIGHTY,
    SDL_GAMEPAD_AXIS_RIGHT_TRIGGER, SDL_GAMEPAD_BUTTON_BACK, SDL_GAMEPAD_BUTTON_DPAD_DOWN,
    SDL_GAMEPAD_BUTTON_DPAD_LEFT, SDL_GAMEPAD_BUTTON_DPAD_RIGHT, SDL_GAMEPAD_BUTTON_DPAD_UP,
    SDL_GAMEPAD_BUTTON_EAST, SDL_GAMEPAD_BUTTON_GUIDE, SDL_GAMEPAD_BUTTON_LEFT_SHOULDER,
    SDL_GAMEPAD_BUTTON_LEFT_STICK, SDL_GAMEPAD_BUTTON_NORTH, SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER,
    SDL_GAMEPAD_BUTTON_RIGHT_STICK, SDL_GAMEPAD_BUTTON_SOUTH, SDL_GAMEPAD_BUTTON_START,
    SDL_GAMEPAD_BUTTON_WEST, SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT,
    SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR, SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT,
    SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO, SDL_GAMEPAD_TYPE_PS3, SDL_GAMEPAD_TYPE_PS4,
    SDL_GAMEPAD_TYPE_PS5, SDL_GAMEPAD_TYPE_STEAM, SDL_GAMEPAD_TYPE_XBOX360,
    SDL_GAMEPAD_TYPE_XBOXONE,
};
use sdl3_sys::hints::{
    SDL_SetHint, SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, SDL_HINT_JOYSTICK_HIDAPI_STEAM,
};
use sdl3_sys::init::{SDL_Init, SDL_INIT_GAMEPAD};
use sdl3_sys::joystick::SDL_JoystickID;
use sdl3_sys::power::{
    SDL_PowerState, SDL_POWERSTATE_CHARGED, SDL_POWERSTATE_CHARGING, SDL_POWERSTATE_ON_BATTERY,
};
use sdl3_sys::stdinc::SDL_free;

use super::{PadButton, PadDevice, PadEvent, PadKind, PadReading};

pub const NAME: &str = "SDL3";

/// SDL's buttons by position.
const MAP: &[(SDL_GamepadButton, PadButton)] = &[
    (SDL_GAMEPAD_BUTTON_EAST, PadButton::East),
    (SDL_GAMEPAD_BUTTON_SOUTH, PadButton::South),
    (SDL_GAMEPAD_BUTTON_NORTH, PadButton::North),
    (SDL_GAMEPAD_BUTTON_WEST, PadButton::West),
    (SDL_GAMEPAD_BUTTON_LEFT_SHOULDER, PadButton::L1),
    (SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER, PadButton::R1),
    (SDL_GAMEPAD_BUTTON_BACK, PadButton::Back),
    (SDL_GAMEPAD_BUTTON_START, PadButton::Start),
    (SDL_GAMEPAD_BUTTON_GUIDE, PadButton::Guide),
    (SDL_GAMEPAD_BUTTON_DPAD_UP, PadButton::Up),
    (SDL_GAMEPAD_BUTTON_DPAD_DOWN, PadButton::Down),
    (SDL_GAMEPAD_BUTTON_DPAD_LEFT, PadButton::Left),
    (SDL_GAMEPAD_BUTTON_DPAD_RIGHT, PadButton::Right),
    (SDL_GAMEPAD_BUTTON_LEFT_STICK, PadButton::L3),
    (SDL_GAMEPAD_BUTTON_RIGHT_STICK, PadButton::R3),
];

/// The axes in `PadReading::axes` order.
const AXES: [SDL_GamepadAxis; 6] = [
    SDL_GAMEPAD_AXIS_LEFTX,
    SDL_GAMEPAD_AXIS_LEFTY,
    SDL_GAMEPAD_AXIS_RIGHTX,
    SDL_GAMEPAD_AXIS_RIGHTY,
    SDL_GAMEPAD_AXIS_LEFT_TRIGGER,
    SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
];

/// SDL, initialised on the thread that made it; the open pads' pointers stay valid until
/// `SDL_CloseGamepad`.
pub struct Source {
    pads: Vec<(u32, *mut SDL_Gamepad)>,
    /// The instance ids connected when SDL started: their «added» events are `at_start`.
    at_start: Vec<u32>,
    event: SDL_Event,
}

impl Source {
    pub fn open() -> Result<Source, String> {
        // SAFETY: SDL is initialised and used by this thread only; hints must be set before
        // SDL_Init.
        unsafe {
            // The focus is in WebView2, which is another process: without this SDL would ignore
            // the buttons as if the window were in the background.
            SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
            // Steam Controller read directly, without Steam.
            SDL_SetHint(SDL_HINT_JOYSTICK_HIDAPI_STEAM, c"1".as_ptr());
            if !SDL_Init(SDL_INIT_GAMEPAD) {
                return Err(text(SDL_GetError()));
            }
        }
        Ok(Source {
            pads: Vec::new(),
            at_start: connected_ids(),
            event: SDL_Event::default(),
        })
    }

    pub fn poll(&mut self, events: &mut Vec<PadEvent>) {
        // SAFETY: SDL_PollEvent writes a valid SDL_Event; the union field read matches the
        // event's type.
        while unsafe { SDL_PollEvent(&mut self.event) } {
            let kind = unsafe { self.event.r#type };
            if kind == SDL_EVENT_GAMEPAD_ADDED {
                let id = unsafe { self.event.gdevice.which };
                if let Some((raw, device)) = open(id) {
                    let at_start = match self.at_start.iter().position(|&s| s == id.0) {
                        Some(i) => {
                            self.at_start.swap_remove(i);
                            true
                        }
                        None => false,
                    };
                    self.pads.push((id.0, raw));
                    events.push(PadEvent::Added { device, at_start });
                }
            } else if kind == SDL_EVENT_JOYSTICK_BATTERY_UPDATED {
                let (id, state, percent) = unsafe {
                    (
                        self.event.jbattery.which,
                        self.event.jbattery.state,
                        self.event.jbattery.percent,
                    )
                };
                if self.pads.iter().any(|(p, _)| *p == id.0) {
                    events.push(PadEvent::Battery {
                        id: id.0,
                        battery: battery(state, percent),
                    });
                }
            } else if kind == SDL_EVENT_GAMEPAD_REMOVED {
                let id = unsafe { self.event.gdevice.which };
                if let Some(i) = self.pads.iter().position(|(p, _)| *p == id.0) {
                    let (_, raw) = self.pads.remove(i);
                    // SAFETY: opened by `open`, closed only once.
                    unsafe { SDL_CloseGamepad(raw) };
                    events.push(PadEvent::Removed(id.0));
                }
            }
        }
    }

    pub fn read(&mut self, id: u32) -> Option<PadReading> {
        let (_, raw) = self.pads.iter().find(|(p, _)| *p == id)?;
        let mut reading = PadReading::default();
        // SAFETY: `raw` is open while the pad is in the list.
        unsafe {
            for (button, pos) in MAP {
                if SDL_GetGamepadButton(*raw, *button) {
                    reading.buttons |= pos.bit();
                }
            }
            for (slot, axis) in reading.axes.iter_mut().zip(AXES) {
                *slot = SDL_GetGamepadAxis(*raw, axis);
            }
        }
        Some(reading)
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        for (_, raw) in self.pads.drain(..) {
            // SAFETY: opened by `open`, closed only once, on SDL's thread.
            unsafe { SDL_CloseGamepad(raw) };
        }
    }
}

/// The maker's layout of an SDL controller type.
fn kind(sdl_type: SDL_GamepadType) -> PadKind {
    if sdl_type == SDL_GAMEPAD_TYPE_XBOX360
        || sdl_type == SDL_GAMEPAD_TYPE_XBOXONE
        || sdl_type == SDL_GAMEPAD_TYPE_STEAM
    {
        PadKind::Xbox
    } else if sdl_type == SDL_GAMEPAD_TYPE_PS3
        || sdl_type == SDL_GAMEPAD_TYPE_PS4
        || sdl_type == SDL_GAMEPAD_TYPE_PS5
    {
        PadKind::PlayStation
    } else if sdl_type == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO
        || sdl_type == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR
        || sdl_type == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT
        || sdl_type == SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT
    {
        PadKind::Nintendo
    } else {
        PadKind::Other
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

/// The instance ids of the controllers connected now.
fn connected_ids() -> Vec<u32> {
    let mut count: c_int = 0;
    // SAFETY: SDL calls on the thread that initialised it; the array holds `count` ids and is
    // freed once, with SDL_free.
    unsafe {
        let ids = SDL_GetGamepads(&mut count);
        if ids.is_null() {
            return Vec::new();
        }
        let len = usize::try_from(count).unwrap_or(0);
        let list = std::slice::from_raw_parts(ids, len)
            .iter()
            .map(|id| id.0)
            .collect();
        SDL_free(ids.cast());
        list
    }
}

/// Opens the controller: its pointer and what the app shows and logs of it (name, type, USB
/// ids, battery), so whoever tests can tell exactly which model is connected.
fn open(id: SDL_JoystickID) -> Option<(*mut SDL_Gamepad, PadDevice)> {
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
        let mut percent: c_int = -1;
        let power = SDL_GetGamepadPowerInfo(raw, &mut percent);
        let device = PadDevice {
            id: id.0,
            name,
            kind: kind(sdl_type),
            type_name: text(SDL_GetGamepadStringForType(sdl_type)),
            vendor: SDL_GetGamepadVendor(raw),
            product: SDL_GetGamepadProduct(raw),
            battery: battery(power, percent),
        };
        Some((raw, device))
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
