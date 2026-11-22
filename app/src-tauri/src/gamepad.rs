//! The controller thread: reads the pads through [`emudome_platform::pads::PadSource`] (SDL3 on
//! the desktop, Android's own input events on Android), lets
//! [`emudome_core::hotkeys::Router`] decide what the buttons mean, and carries out the result.
//! Everything here is the same on every system; only the source differs.
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
//! - `emu://gamepad-state`: a [`PadStates`], while the interface watches (`gamepad_watch`);
//! - the router's actions (`emu://menu`, `emu://nav`, `emu://hotkey`) through the shortcuts'
//!   runner shared with the keyboard (`hotkeys.rs`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use emudome_core::bindings::{Family, PadConfig, PadMap, Pos};
use emudome_core::hotkeys::{PadInput, Router};
use emudome_core::input::{Source, Sticks};
use emudome_core::notifications::Notice;
use emudome_core::ErrorCode;
use emudome_platform::pads::{PadButton, PadDevice, PadEvent, PadKind, PadReading, PadSource};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::health::{Part, SharedHealth};
use crate::hotkeys::{HotkeyRunner, Origin};
use crate::pads::{self, PadInfo, PadState, PadStates, PadsState};
use crate::{events, CurrentShared};

/// The source's buttons by position (the Guide button is [`PadInput::guide`], not a position).
const MAP: &[(PadButton, Pos)] = &[
    (PadButton::East, Pos::East),
    (PadButton::South, Pos::South),
    (PadButton::North, Pos::North),
    (PadButton::West, Pos::West),
    (PadButton::L1, Pos::L1),
    (PadButton::R1, Pos::R1),
    (PadButton::Back, Pos::Back),
    (PadButton::Start, Pos::Start),
    (PadButton::Up, Pos::Up),
    (PadButton::Down, Pos::Down),
    (PadButton::Left, Pos::Left),
    (PadButton::Right, Pos::Right),
    (PadButton::L3, Pos::L3),
    (PadButton::R3, Pos::R3),
];

/// How often the controllers are read.
const POLL: Duration = Duration::from_millis(4);

/// `emu://gamepad-state` at most this often (a stick moves at every poll).
const STATE_EVERY: Duration = Duration::from_millis(16);

/// A connected controller.
struct Pad {
    info: PadInfo,
    /// How its positions give the game its buttons (from `pad.remap` for its family).
    map: PadMap,
}

impl Pad {
    fn log_name(&self) -> &str {
        self.info.name.as_deref().unwrap_or("(unnamed)")
    }
}

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

/// One poll of one pad.
#[derive(Debug, PartialEq)]
struct Reading {
    /// Positions held: buttons and triggers past the threshold, plus the left stick as d-pad.
    input: PadInput,
    /// The same without the stick, for the tester.
    held: u16,
    axes: [i16; 6],
}

/// What the router reads of one pad's state.
fn reading(raw: &PadReading, config: &PadConfig) -> Reading {
    let mut held = MAP
        .iter()
        .filter(|(button, _)| raw.buttons & button.bit() != 0)
        .fold(0, |held, (_, pos)| held | pos.bit());
    let axes = raw.axes;
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
            guide: raw.buttons & PadButton::Guide.bit() != 0,
            // The right stick as directions too: the interface's own steps (`Nav::Stick*`).
            stick: config.stick_dpad_mask(axes[2], axes[3]),
        },
        held,
        axes,
    }
}

/// The remap family of a pad's kind.
fn family(kind: PadKind) -> Family {
    match kind {
        PadKind::Xbox => Family::Xbox,
        PadKind::PlayStation => Family::PlayStation,
        PadKind::Nintendo => Family::Switch,
        PadKind::Other => Family::Generic,
    }
}

/// What the interface knows of a pad.
fn info(device: &PadDevice) -> PadInfo {
    PadInfo {
        id: device.id,
        name: device.name.clone(),
        family: family(device.kind),
        vendor: device.vendor,
        product: device.product,
        battery: device.battery,
    }
}

/// Starts the controller thread. A thread that cannot start, a source that does not open, or a
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
    let mut source = match PadSource::open() {
        Ok(source) => source,
        Err(detail) => {
            log::warn!("controllers unavailable ({}): {detail}", PadSource::BACKEND);
            health.set(
                Part::Controllers,
                Some(ErrorCode::ControllersUnavailable { detail }),
            );
            return;
        }
    };
    log::info!("controllers: {} ready", PadSource::BACKEND);

    let mut pads: Vec<Pad> = Vec::new();
    let mut config = PadConfig::default();
    // The generation the config was read at; `None` until the settings are open.
    let mut config_at: Option<u64> = None;
    let mut router = Router::new(config.bindings());
    let mut last_state: Option<PadStates> = None;
    let mut last_state_at = Instant::now();
    let mut events_in = Vec::new();
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

        source.poll(&mut events_in);
        for event in events_in.drain(..) {
            match event {
                PadEvent::Added { device, at_start } => {
                    let pad = Pad {
                        info: info(&device),
                        map: config.pad_map(family(device.kind)),
                    };
                    log::info!(
                        "controller connected: {} (type {}, family {}, vid {:04x}, pid {:04x}, battery {:?})",
                        pad.log_name(),
                        device.type_name,
                        pad.info.family.id(),
                        pad.info.vendor,
                        pad.info.product,
                        pad.info.battery,
                    );
                    emit(&app, events::GAMEPAD_CONNECTED, pad.info.clone());
                    // The controllers already there at start arrive as «added» too: only the ones
                    // connected afterwards become a notification (X03, #32).
                    if !at_start {
                        crate::notifications::record(
                            &app,
                            Notice::ControllerConnected {
                                name: pad.info.name.clone().unwrap_or_default(),
                            },
                        );
                    }
                    shared_pads.list.lock().push(pad.info.clone());
                    pads.push(pad);
                }
                PadEvent::Battery { id, battery } => {
                    if let Some(pad) = pads.iter_mut().find(|p| p.info.id == id) {
                        if pad.info.battery != battery {
                            pad.info.battery = battery;
                            for listed in shared_pads.list.lock().iter_mut() {
                                if listed.id == pad.info.id {
                                    listed.battery = battery;
                                }
                            }
                            emit(&app, events::GAMEPAD_CHANGED, pad.info.clone());
                        }
                    }
                }
                PadEvent::Removed(id) => {
                    if let Some(i) = pads.iter().position(|p| p.info.id == id) {
                        let pad = pads.remove(i);
                        log::info!("controller disconnected: {}", pad.log_name());
                        shared_pads.list.lock().retain(|p| p.id != pad.info.id);
                        emit(&app, events::GAMEPAD_DISCONNECTED, pad.info.clone());
                    }
                }
            }
        }

        let readings: Vec<Reading> = pads
            .iter()
            .map(|p| reading(&source.read(p.info.id).unwrap_or_default(), &config))
            .collect();
        let input = readings
            .iter()
            .fold(PadInput::default(), |all, r| PadInput {
                buttons: all.buttons | r.input.buttons,
                guide: all.guide || r.input.guide,
                stick: all.stick | r.input.stick,
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
                emit(&app, events::GAMEPAD_STATE, states.clone());
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

#[cfg(test)]
mod tests {
    use emudome_core::bindings::{Family, PadConfig, Pos};
    use emudome_core::hotkeys::PadInput;
    use emudome_platform::pads::{PadButton, PadKind, PadReading};

    use super::{family, reading, Reading, MAP};

    #[test]
    fn every_position_but_the_triggers_has_a_button_and_guide_has_none() {
        for pos in Pos::ALL {
            let mapped = MAP.iter().filter(|(_, p)| *p == pos).count();
            let expected = usize::from(!matches!(pos, Pos::L2 | Pos::R2));
            assert_eq!(mapped, expected, "{pos:?}");
        }
        for button in PadButton::ALL {
            let mapped = MAP.iter().filter(|(b, _)| *b == button).count();
            assert_eq!(
                mapped,
                usize::from(button != PadButton::Guide),
                "{button:?}"
            );
        }
    }

    #[test]
    fn kinds_give_their_remap_family() {
        assert_eq!(family(PadKind::Xbox), Family::Xbox);
        assert_eq!(family(PadKind::PlayStation), Family::PlayStation);
        assert_eq!(family(PadKind::Nintendo), Family::Switch);
        assert_eq!(family(PadKind::Other), Family::Generic);
    }

    #[test]
    fn a_reading_folds_triggers_guide_and_sticks_as_before() {
        let config = PadConfig::default();
        let threshold = config.trigger_threshold();
        let raw = PadReading {
            buttons: PadButton::South.bit() | PadButton::Guide.bit(),
            axes: [32767, 0, 0, -32768, threshold.saturating_add(1), threshold],
        };
        let r = reading(&raw, &config);
        let held = Pos::South.bit() | Pos::L2.bit();
        assert_eq!(
            r,
            Reading {
                input: PadInput {
                    buttons: held | config.stick_dpad_mask(32767, 0),
                    guide: true,
                    stick: config.stick_dpad_mask(0, -32768),
                },
                held,
                axes: raw.axes,
            }
        );
        assert_ne!(r.input.stick, 0, "the right stick up is a step");
    }
}
