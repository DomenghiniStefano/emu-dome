//! Console controller test, no windows: initialises only SDL3's gamepad
//! subsystem (with the app's same hints) and prints connections and
//! buttons for 20 seconds.
//!
//! `cargo run -p emudome --example gamepad_probe`

use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::time::{Duration, Instant};

use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{
    SDL_Event, SDL_PollEvent, SDL_EVENT_GAMEPAD_ADDED, SDL_EVENT_GAMEPAD_AXIS_MOTION,
    SDL_EVENT_GAMEPAD_BUTTON_DOWN, SDL_EVENT_GAMEPAD_BUTTON_UP, SDL_EVENT_GAMEPAD_REMOVED,
};
use sdl3_sys::gamepad::*;
use sdl3_sys::hints::{
    SDL_SetHint, SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, SDL_HINT_JOYSTICK_HIDAPI_STEAM,
};
use sdl3_sys::init::{SDL_Init, SDL_Quit, SDL_INIT_GAMEPAD};

const DURATION: Duration = Duration::from_secs(20);

unsafe fn text(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

fn main() {
    // SAFETY: all of SDL stays on this thread.
    unsafe {
        SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
        SDL_SetHint(SDL_HINT_JOYSTICK_HIDAPI_STEAM, c"1".as_ptr());
        if !SDL_Init(SDL_INIT_GAMEPAD) {
            eprintln!("SDL_Init failed: {}", text(SDL_GetError()));
            std::process::exit(1);
        }
        println!(
            "Connect a controller and press its buttons: listening for {} seconds.",
            DURATION.as_secs()
        );

        let mut pads = Vec::new();
        let mut axes = HashMap::new();
        let mut event = SDL_Event::default();
        let start = Instant::now();
        while start.elapsed() < DURATION {
            while SDL_PollEvent(&mut event) {
                let kind = event.r#type;
                if kind == SDL_EVENT_GAMEPAD_ADDED {
                    let id = event.gdevice.which;
                    let pad = SDL_OpenGamepad(id);
                    if pad.is_null() {
                        println!("[{}] cannot be opened: {}", id.0, text(SDL_GetError()));
                        continue;
                    }
                    println!(
                        "[{}] connected: {} | type {} | vid {:04x} pid {:04x}",
                        id.0,
                        text(SDL_GetGamepadName(pad)),
                        text(SDL_GetGamepadStringForType(SDL_GetGamepadType(pad))),
                        SDL_GetGamepadVendor(pad),
                        SDL_GetGamepadProduct(pad),
                    );
                    pads.push(pad);
                } else if kind == SDL_EVENT_GAMEPAD_REMOVED {
                    let id = event.gdevice.which;
                    println!("[{}] disconnected", id.0);
                    if let Some(i) = pads.iter().position(|p| SDL_GetGamepadID(*p) == id) {
                        SDL_CloseGamepad(pads.swap_remove(i));
                    }
                } else if kind == SDL_EVENT_GAMEPAD_BUTTON_DOWN
                    || kind == SDL_EVENT_GAMEPAD_BUTTON_UP
                {
                    let b = event.gbutton;
                    let name = text(SDL_GetGamepadStringForButton(SDL_GamepadButton(i32::from(
                        b.button,
                    ))));
                    let state = if b.down { "pressed" } else { "released" };
                    println!("[{}] {name} {state}", b.which.0);
                } else if kind == SDL_EVENT_GAMEPAD_AXIS_MOTION {
                    let a = event.gaxis;
                    // Only when an axis crosses half travel either way,
                    // so the console does not fill up.
                    let past_half = a.value.unsigned_abs() > 16000;
                    if axes.insert((a.which.0, a.axis), past_half).unwrap_or(false) != past_half {
                        let name = text(SDL_GetGamepadStringForAxis(SDL_GamepadAxis(i32::from(
                            a.axis,
                        ))));
                        println!("[{}] axis {name} = {}", a.which.0, a.value);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(4));
        }
        for pad in pads {
            SDL_CloseGamepad(pad);
        }
        SDL_Quit();
    }
    println!("Done.");
}
