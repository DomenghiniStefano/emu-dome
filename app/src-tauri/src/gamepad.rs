//! Controllers through SDL3 (only the gamepad subsystem, no SDL windows):
//! reads the pads, lets [`emudome_core::hotkeys::Router`] decide what the buttons mean,
//! and carries out the result.
//!
//! SDL recognises Xbox, PS4/PS5, Switch Pro, the pads in its database and the
//! Steam Controller, read directly through HIDAPI without Steam running.
//!
//! Positional mapping like RetroArch: the controller's right button is A,
//! the bottom one is B. Triggers past half travel are L2/R2, the left stick past half
//! travel is the d-pad. It becomes configurable with the settings engine.
//!
//! Routing (pure, in `emudome_core::hotkeys`): in game the buttons go to the core;
//! with L3 held they are combos; while the interface has the input (quick menu open, or
//! no game) the d-pad and A/B become `emu://nav` events and the game gets nothing.
//!
//! Events to the interface:
//! - `emu://gamepad-connected`, `emu://gamepad-disconnected`: a [`GamepadEvent`];
//! - `emu://menu`: open/close the quick menu (Guide, or L3 + Start);
//! - `emu://nav`: a [`NavEvent`], one focus step in the interface;
//! - `emu://hotkey`: a [`HotkeyEvent`], what a combo changed that the interface shows.

use std::ffi::{c_char, CStr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use emudome_core::hotkeys::{Action, Bindings, Hotkey, Nav, PadInput, Router};
use emudome_core::input::{Button, Source};
use emudome_core::{Command, Speed};
use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{
    SDL_Event, SDL_PollEvent, SDL_EVENT_GAMEPAD_ADDED, SDL_EVENT_GAMEPAD_REMOVED,
};
use sdl3_sys::gamepad::{
    SDL_CloseGamepad, SDL_Gamepad, SDL_GamepadAxis, SDL_GamepadButton, SDL_GetGamepadAxis,
    SDL_GetGamepadButton, SDL_GetGamepadName, SDL_GetGamepadProduct, SDL_GetGamepadStringForType,
    SDL_GetGamepadType, SDL_GetGamepadVendor, SDL_OpenGamepad, SDL_GAMEPAD_AXIS_LEFTX,
    SDL_GAMEPAD_AXIS_LEFTY, SDL_GAMEPAD_AXIS_LEFT_TRIGGER, SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
    SDL_GAMEPAD_BUTTON_BACK, SDL_GAMEPAD_BUTTON_DPAD_DOWN, SDL_GAMEPAD_BUTTON_DPAD_LEFT,
    SDL_GAMEPAD_BUTTON_DPAD_RIGHT, SDL_GAMEPAD_BUTTON_DPAD_UP, SDL_GAMEPAD_BUTTON_EAST,
    SDL_GAMEPAD_BUTTON_GUIDE, SDL_GAMEPAD_BUTTON_LEFT_SHOULDER, SDL_GAMEPAD_BUTTON_LEFT_STICK,
    SDL_GAMEPAD_BUTTON_NORTH, SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER, SDL_GAMEPAD_BUTTON_RIGHT_STICK,
    SDL_GAMEPAD_BUTTON_SOUTH, SDL_GAMEPAD_BUTTON_START, SDL_GAMEPAD_BUTTON_WEST,
};
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
    (SDL_GAMEPAD_BUTTON_LEFT_STICK, Button::L3),
    (SDL_GAMEPAD_BUTTON_RIGHT_STICK, Button::R3),
];

/// Axes read as buttons: (axis, button past +half, button past -half).
/// Triggers only go positive; SDL's Y axis is positive downwards.
const AXES: &[(SDL_GamepadAxis, Option<Button>, Option<Button>)] = &[
    (SDL_GAMEPAD_AXIS_LEFT_TRIGGER, Some(Button::L2), None),
    (SDL_GAMEPAD_AXIS_RIGHT_TRIGGER, Some(Button::R2), None),
    (
        SDL_GAMEPAD_AXIS_LEFTX,
        Some(Button::Right),
        Some(Button::Left),
    ),
    (SDL_GAMEPAD_AXIS_LEFTY, Some(Button::Down), Some(Button::Up)),
];

/// Half of an axis' travel (SDL axes go from -32768 to 32767).
const HALF_TRAVEL: i16 = 16384;

/// How often the controllers are read.
const POLL: Duration = Duration::from_millis(4);

/// Save state slots reachable with L3 + d-pad left/right.
const FIRST_SLOT: u8 = 1;
const LAST_SLOT: u8 = 9;

/// Slow motion speed for L3 + d-pad down, and the speed toggles go back to.
const SLOW_MOTION: Speed = Speed::Factor(0.5);
const NORMAL: Speed = Speed::Factor(1.0);

/// Payload of `emu://gamepad-connected` and `emu://gamepad-disconnected`.
/// `name` is `None` when SDL knows no name for the pad: the interface shows a generic one.
#[derive(Clone, Serialize)]
pub struct GamepadEvent {
    name: Option<String>,
}

/// Payload of `emu://nav`: `{ action: "up" | "down" | "left" | "right" | "confirm" | "back" }`.
#[derive(Clone, Serialize)]
pub struct NavEvent {
    action: Nav,
}

/// Payload of `emu://hotkey`: what a combo changed, for the interface to show.
/// Codes and values only; the interface turns them into text.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HotkeyEvent {
    /// FPS overlay on/off.
    Fps,
    /// The current slot changed.
    Slot { slot: u8 },
    /// The game speed changed (fast forward or slow motion toggled).
    Speed { speed: Speed },
    /// Fast forward held (true) or released (false).
    Turbo { held: bool },
    /// Paused or resumed without the menu.
    Paused { paused: bool },
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

    /// The pad's buttons as a RetroPad mask, plus Guide.
    fn read(&self) -> PadInput {
        let mut input = PadInput::default();
        // SAFETY: `self.raw` is open while the pad is in the list.
        unsafe {
            for (p, b) in MAP {
                if SDL_GetGamepadButton(self.raw, *p) {
                    input.buttons |= b.bit();
                }
            }
            for (axis, plus, minus) in AXES {
                let v = SDL_GetGamepadAxis(self.raw, *axis);
                if let Some(b) = plus.filter(|_| v > HALF_TRAVEL) {
                    input.buttons |= b.bit();
                }
                if let Some(b) = minus.filter(|_| v < -HALF_TRAVEL) {
                    input.buttons |= b.bit();
                }
            }
            input.guide = SDL_GetGamepadButton(self.raw, SDL_GAMEPAD_BUTTON_GUIDE);
        }
        input
    }
}

pub fn spawn(app: AppHandle, current: CurrentShared, menu_open: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("emudome-gamepad".into())
        .spawn(move || run(app, current, menu_open))
        .expect("controller thread");
}

fn run(app: AppHandle, current: CurrentShared, menu_open: Arc<AtomicBool>) {
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
    let mut router = Router::new(Bindings::default());
    let mut hotkeys = Hotkeys {
        app,
        current: current.clone(),
        slot: FIRST_SLOT,
        turbo_restore: None,
    };
    let mut event = SDL_Event::default();
    loop {
        // SAFETY: SDL_PollEvent writes a valid SDL_Event; the union field
        // read matches the event's type.
        while unsafe { SDL_PollEvent(&mut event) } {
            let kind = unsafe { event.r#type };
            if kind == SDL_EVENT_GAMEPAD_ADDED {
                let id = unsafe { event.gdevice.which };
                if let Some(pad) = open(id) {
                    hotkeys.emit("emu://gamepad-connected", pad.event());
                    pads.push(pad);
                }
            } else if kind == SDL_EVENT_GAMEPAD_REMOVED {
                let id = unsafe { event.gdevice.which };
                if let Some(i) = pads.iter().position(|p| p.id == id) {
                    let pad = pads.swap_remove(i);
                    log::info!("controller disconnected: {}", pad.log_name());
                    hotkeys.emit("emu://gamepad-disconnected", pad.event());
                    // SAFETY: opened by `open`, closed only once.
                    unsafe { SDL_CloseGamepad(pad.raw) };
                }
            }
        }

        let input = pads
            .iter()
            .map(Pad::read)
            .fold(PadInput::default(), |all, pad| PadInput {
                buttons: all.buttons | pad.buttons,
                guide: all.guide || pad.guide,
            });
        let shared = current.lock().clone();
        // Without a game the interface (start screen) takes the controller.
        let ui_has_input = shared.is_none() || menu_open.load(Ordering::Relaxed);
        let routed = router.step(input, ui_has_input, Instant::now());
        if let Some(shared) = shared {
            shared.input.set_mask(Source::Gamepad, routed.game_mask);
        }
        for action in routed.actions {
            hotkeys.run(action);
        }
        std::thread::sleep(POLL);
    }
}

/// Carries out the router's actions: commands to the session, events to the interface.
struct Hotkeys {
    app: AppHandle,
    current: CurrentShared,
    /// The slot L3 + R1 / L3 + L1 save to and load from.
    slot: u8,
    /// While fast forward is held, the speed to go back to.
    turbo_restore: Option<Speed>,
}

impl Hotkeys {
    fn run(&mut self, action: Action) {
        match action {
            Action::Nav(nav) => self.emit("emu://nav", NavEvent { action: nav }),
            Action::Press(hotkey) => self.press(hotkey),
            Action::Release(hotkey) => self.release(hotkey),
        }
    }

    fn press(&mut self, hotkey: Hotkey) {
        match hotkey {
            Hotkey::ToggleMenu => self.emit("emu://menu", ()),
            Hotkey::SaveState => {
                self.send(Command::SaveState(self.slot));
            }
            Hotkey::LoadState => {
                self.send(Command::LoadState(self.slot));
            }
            Hotkey::NextSlot => {
                self.slot = if self.slot >= LAST_SLOT {
                    FIRST_SLOT
                } else {
                    self.slot + 1
                };
                self.emit("emu://hotkey", HotkeyEvent::Slot { slot: self.slot });
            }
            Hotkey::PreviousSlot => {
                self.slot = if self.slot <= FIRST_SLOT {
                    LAST_SLOT
                } else {
                    self.slot - 1
                };
                self.emit("emu://hotkey", HotkeyEvent::Slot { slot: self.slot });
            }
            Hotkey::FastForwardHold => {
                if self.turbo_restore.is_none() {
                    let before = self.speed();
                    if self.send(Command::SetSpeed(Speed::Max)) {
                        self.turbo_restore = Some(before);
                        self.emit("emu://hotkey", HotkeyEvent::Turbo { held: true });
                    }
                }
            }
            Hotkey::FastForwardToggle => self.toggle_speed(Speed::Max),
            Hotkey::SlowMotionToggle => self.toggle_speed(SLOW_MOTION),
            Hotkey::FpsOverlay => self.emit("emu://hotkey", HotkeyEvent::Fps),
            Hotkey::Pause => {
                let paused = !self.paused();
                if self.send(Command::SetPaused(paused)) {
                    self.emit("emu://hotkey", HotkeyEvent::Paused { paused });
                }
            }
        }
    }

    fn release(&mut self, hotkey: Hotkey) {
        match hotkey {
            Hotkey::FastForwardHold => {
                if let Some(before) = self.turbo_restore.take() {
                    self.send(Command::SetSpeed(before));
                    self.emit("emu://hotkey", HotkeyEvent::Turbo { held: false });
                }
            }
            // Press-only hotkeys have nothing to undo.
            Hotkey::ToggleMenu
            | Hotkey::SaveState
            | Hotkey::LoadState
            | Hotkey::NextSlot
            | Hotkey::PreviousSlot
            | Hotkey::FastForwardToggle
            | Hotkey::SlowMotionToggle
            | Hotkey::FpsOverlay
            | Hotkey::Pause => {}
        }
    }

    /// `target` if the game is not already at it, otherwise back to 1×.
    fn toggle_speed(&mut self, target: Speed) {
        let speed = if self.speed() == target {
            NORMAL
        } else {
            target
        };
        if self.send(Command::SetSpeed(speed)) {
            self.emit("emu://hotkey", HotkeyEvent::Speed { speed });
        }
    }

    fn speed(&self) -> Speed {
        self.current
            .lock()
            .as_ref()
            .and_then(|s| s.stats.lock().speed)
            .unwrap_or(NORMAL)
    }

    fn paused(&self) -> bool {
        self.current
            .lock()
            .as_ref()
            .is_some_and(|s| s.stats.lock().paused)
    }

    /// Sends a command to the running game; false when there is none.
    fn send(&self, cmd: Command) -> bool {
        crate::send_command(&self.app, cmd)
    }

    fn emit<S: Serialize + Clone>(&self, name: &str, payload: S) {
        if let Err(e) = self.app.emit(name, payload) {
            log::warn!("{name} not delivered: {e}");
        }
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
