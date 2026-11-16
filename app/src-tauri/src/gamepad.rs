//! Controllers through SDL3 (only the gamepad subsystem, no SDL windows):
//! updates the RetroPad buttons and opens the quick menu.
//!
//! SDL recognises Xbox, PS4/PS5, Switch Pro, the pads in its database and the
//! Steam Controller, read directly through HIDAPI without Steam running.
//!
//! Positional mapping like RetroArch: the controller's right button is A,
//! the bottom one is B. It becomes configurable with the settings engine.
//!
//! Events to the interface: `emu://gamepad-connected` and `emu://gamepad-disconnected`
//! with a [`GamepadEvent`] payload, and `emu://menu` for the Home/Guide button.

use std::ffi::{c_char, CStr};
use std::time::Duration;

use emudome_core::input::{Button, Source};
use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{
    SDL_Event, SDL_PollEvent, SDL_EVENT_GAMEPAD_ADDED, SDL_EVENT_GAMEPAD_BUTTON_DOWN,
    SDL_EVENT_GAMEPAD_REMOVED,
};
use sdl3_sys::gamepad::*;
use sdl3_sys::hints::{
    SDL_SetHint, SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, SDL_HINT_JOYSTICK_HIDAPI_STEAM,
};
use sdl3_sys::init::{SDL_Init, SDL_INIT_GAMEPAD};
use sdl3_sys::joystick::SDL_JoystickID;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::CurrentShared;

const MAP: &[(SDL_GamepadButton, Button)] = &[
    (SDL_GAMEPAD_BUTTON_EAST, Button::A),
    (SDL_GAMEPAD_BUTTON_SOUTH, Button::B),
    (SDL_GAMEPAD_BUTTON_NORTH, Button::X),
    (SDL_GAMEPAD_BUTTON_WEST, Button::Y),
    (SDL_GAMEPAD_BUTTON_LEFT_SHOULDER, Button::L),
    (SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER, Button::R),
    (SDL_GAMEPAD_BUTTON_BACK, Button::Select),
    (SDL_GAMEPAD_BUTTON_START, Button::Start),
    (SDL_GAMEPAD_BUTTON_DPAD_UP, Button::Up),
    (SDL_GAMEPAD_BUTTON_DPAD_DOWN, Button::Down),
    (SDL_GAMEPAD_BUTTON_DPAD_LEFT, Button::Left),
    (SDL_GAMEPAD_BUTTON_DPAD_RIGHT, Button::Right),
];

/// How often the controllers are read.
const POLL: Duration = Duration::from_millis(4);

/// Payload of `emu://gamepad-connected` and `emu://gamepad-disconnected`.
/// `name` is `None` when SDL knows no name for the pad: the interface shows a generic one.
#[derive(Clone, Serialize)]
pub struct GamepadEvent {
    name: Option<String>,
}

/// An open controller. The pointer stays valid until `SDL_CloseGamepad`.
struct Pad {
    id: SDL_JoystickID,
    raw: *mut SDL_Gamepad,
    name: Option<String>,
}

impl Pad {
    fn event(&self) -> GamepadEvent {
        GamepadEvent {
            name: self.name.clone(),
        }
    }

    fn log_name(&self) -> &str {
        self.name.as_deref().unwrap_or("(unnamed)")
    }
}

pub fn spawn(app: AppHandle, current: CurrentShared) {
    std::thread::Builder::new()
        .name("emudome-gamepad".into())
        .spawn(move || run(app, current))
        .expect("controller thread");
}

fn run(app: AppHandle, current: CurrentShared) {
    // SAFETY: SDL is initialised and used by this thread only;
    // hints must be set before SDL_Init.
    unsafe {
        // The focus is in WebView2, which is another process: without this
        // SDL would ignore the buttons as if the window were in the background.
        SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
        // Steam Controller read directly, without Steam.
        SDL_SetHint(SDL_HINT_JOYSTICK_HIDAPI_STEAM, c"1".as_ptr());
        if !SDL_Init(SDL_INIT_GAMEPAD) {
            log::warn!("controllers unavailable: {}", text(SDL_GetError()));
            return;
        }
    }
    log::info!("controllers: SDL3 ready");

    let mut pads: Vec<Pad> = Vec::new();
    let mut event = SDL_Event::default();
    loop {
        // SAFETY: SDL_PollEvent writes a valid SDL_Event; the union field
        // read matches the event's type.
        while unsafe { SDL_PollEvent(&mut event) } {
            let kind = unsafe { event.r#type };
            if kind == SDL_EVENT_GAMEPAD_ADDED {
                let id = unsafe { event.gdevice.which };
                if let Some(pad) = open(id) {
                    let _ = app.emit("emu://gamepad-connected", pad.event());
                    pads.push(pad);
                }
            } else if kind == SDL_EVENT_GAMEPAD_REMOVED {
                let id = unsafe { event.gdevice.which };
                if let Some(i) = pads.iter().position(|p| p.id == id) {
                    let pad = pads.swap_remove(i);
                    log::info!("controller disconnected: {}", pad.log_name());
                    let _ = app.emit("emu://gamepad-disconnected", pad.event());
                    // SAFETY: opened by `open`, closed only once.
                    unsafe { SDL_CloseGamepad(pad.raw) };
                }
            } else if kind == SDL_EVENT_GAMEPAD_BUTTON_DOWN {
                let button = unsafe { event.gbutton.button };
                if i32::from(button) == SDL_GAMEPAD_BUTTON_GUIDE.0 {
                    let _ = app.emit("emu://menu", ());
                }
            }
        }

        let mut mask = 0u16;
        for pad in &pads {
            for (p, b) in MAP {
                // SAFETY: `pad.raw` is open while it is in `pads`.
                if unsafe { SDL_GetGamepadButton(pad.raw, *p) } {
                    mask |= b.bit();
                }
            }
        }
        if let Some(shared) = current.lock().as_ref() {
            shared.input.set_mask(Source::Gamepad, mask);
        }
        std::thread::sleep(POLL);
    }
}

/// Opens the controller and logs its name, type and USB ids, so whoever tests
/// can tell exactly which model is connected.
fn open(id: SDL_JoystickID) -> Option<Pad> {
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
        let kind = text(SDL_GetGamepadStringForType(SDL_GetGamepadType(raw)));
        let pad = Pad { id, raw, name };
        log::info!(
            "controller connected: {} (type {kind}, vid {:04x}, pid {:04x})",
            pad.log_name(),
            SDL_GetGamepadVendor(raw),
            SDL_GetGamepadProduct(raw),
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
