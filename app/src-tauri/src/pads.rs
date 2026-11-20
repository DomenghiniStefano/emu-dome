//! What the interface knows about the controllers (Controls section, C01–C04), on every
//! platform: the connected pads, the live state for the tester and the capture field, and the
//! bindings resolved from the settings. The SDL thread that fills it is `gamepad.rs`
//! (desktop only: on Android the list stays empty for now).
//!
//! Commands (called through `app/src/emu.ts` only): `gamepad_list`, `gamepad_watch`,
//! `gamepad_capture`, `controls_bindings`.
//! Events: `emu://gamepad-connected` / `emu://gamepad-disconnected` (a [`PadInfo`]),
//! `emu://gamepad-changed` (a [`PadInfo`] whose battery changed),
//! `emu://pad-state` (a [`PadStates`], only while the interface watches).

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

use emudome_core::bindings::{default_combo, Family, PadConfig, Pos};
use emudome_core::hotkeys::{self, Hotkey, Status};
use emudome_core::settings::{Scope, Value};
use emudome_core::ErrorCode;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::settings::SettingsState;

/// A connected controller. Payload of `emu://gamepad-connected|disconnected|changed`.
/// Mirror: `PadInfo` in app/src/emu.ts. `name` is `None` when SDL knows none.
#[derive(Clone, Debug, Serialize)]
pub struct PadInfo {
    pub id: u32,
    pub name: Option<String>,
    pub family: Family,
    pub vendor: u16,
    pub product: u16,
    /// Battery charge left, 0..=100 (SDL's estimate); `None` when the controller has no battery
    /// (wired) or does not say.
    pub battery: Option<u8>,
}

/// One pad's live state for the tester and the capture field. Mirror: `PadState`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PadState {
    pub id: u32,
    /// Positions held: buttons, and the triggers past `pad.trig` (the stick is in `axes`).
    pub held: Vec<Pos>,
    /// Left x, left y, right x, right y (-32768..=32767, y positive downwards),
    /// left trigger, right trigger (0..=32767).
    pub axes: [i16; 6],
}

/// Payload of `emu://pad-state`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[allow(dead_code, reason = "used by gamepad.rs, desktop only (SDL3)")]
pub struct PadStates {
    pub pads: Vec<PadState>,
}

/// Shared between the commands and the controller thread.
#[derive(Default)]
pub struct Pads {
    pub list: Mutex<Vec<PadInfo>>,
    /// The interface shows the tester: send `emu://pad-state`.
    pub watch: AtomicBool,
    /// The interface is capturing a button: the router acts on nothing (no game, no menu).
    pub capture: AtomicBool,
    /// The current quick save slot: L3 + R1 / L3 + L1 and F1 / F4 save to and load from it,
    /// L3 + d-pad left / right and F2 / F3 move it (0 until first moved: slot 1).
    slot: AtomicU8,
}

impl Pads {
    /// The current quick save slot.
    pub fn slot(&self) -> u8 {
        let slot = self.slot.load(Ordering::Relaxed);
        if (hotkeys::FIRST_SLOT..=hotkeys::LAST_SLOT).contains(&slot) {
            slot
        } else {
            hotkeys::FIRST_SLOT
        }
    }

    /// Moves the current slot one step (`forward`: next), wrapping; answers the new slot.
    pub fn step_slot(&self, forward: bool) -> u8 {
        let slot = hotkeys::step_slot(self.slot(), forward);
        self.slot.store(slot, Ordering::Relaxed);
        slot
    }
}

pub type PadsState = Arc<Pads>;

/// Bumped at every settings change: the controller thread reloads its [`PadConfig`].
static CONFIG_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn settings_changed() {
    CONFIG_GENERATION.fetch_add(1, Ordering::Relaxed);
}

#[allow(dead_code, reason = "used by gamepad.rs, desktop only (SDL3)")]
pub fn config_generation() -> u64 {
    CONFIG_GENERATION.load(Ordering::Relaxed)
}

/// The controller settings in force now: the open game's (with its changes «in prova»), or
/// «Tutti i giochi» without a game. `None` while the settings are not open yet.
#[allow(dead_code, reason = "used by gamepad.rs, desktop only (SDL3)")]
pub fn current_config(app: &AppHandle) -> Option<PadConfig> {
    let state = app.try_state::<SettingsState>()?;
    // `resolve` needs `&mut` and `from_settings` takes an `Fn`: borrow the lock per call.
    let settings = std::cell::RefCell::new(state.0.lock());
    let scope = if settings.borrow().session_scope().is_some() {
        Scope::Session
    } else {
        Scope::Global
    };
    Some(PadConfig::from_settings(|key| {
        let resolved = settings.borrow_mut().resolve(key, &scope);
        match resolved {
            Ok(r) => r.value,
            Err(e) => {
                log::warn!("{key} unavailable: {e:?}");
                None
            }
        }
    }))
}

/// The connected controllers, in connection order.
#[tauri::command]
pub fn gamepad_list(pads: State<'_, PadsState>) -> Vec<PadInfo> {
    pads.list.lock().clone()
}

/// While on, `emu://pad-state` carries every change of the controllers (tester, capture).
#[tauri::command]
pub fn gamepad_watch(pads: State<'_, PadsState>, on: bool) {
    pads.watch.store(on, Ordering::Relaxed);
}

/// The current quick save slot (the quick menu opens on it, F1 / F4 use it).
#[tauri::command]
pub fn current_slot(pads: State<'_, PadsState>) -> u8 {
    pads.slot()
}

/// Keyboard F2 / F3: the previous / next slot, the same one L3 + d-pad left / right moves.
/// Answers the new slot (the interface shows «Slot N»).
#[tauri::command]
pub fn step_slot(pads: State<'_, PadsState>, forward: bool) -> u8 {
    pads.step_slot(forward)
}

/// While on, the controllers neither play nor navigate: the Controls section is waiting for
/// the button to assign. The button held when it ends does nothing until released.
#[tauri::command]
pub fn gamepad_capture(pads: State<'_, PadsState>, on: bool) {
    pads.capture.store(on, Ordering::Relaxed);
}

/// One game button in the remap table. Mirror: `RemapRow` in app/src/emu.ts.
#[derive(Debug, Serialize)]
pub struct RemapRow {
    button: &'static str,
    /// A position id (controllers) or a key code (keyboard); `None` = nothing.
    source: Option<String>,
    default: Option<String>,
    changed: bool,
}

/// One shortcut, in [`Hotkey::ALL`] order. Mirror: `ComboRow` in app/src/emu.ts.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboRow {
    hotkey: &'static str,
    /// Position pressed with the shortcut button; `None` = no shortcut.
    pos: Option<Pos>,
    default: Option<Pos>,
    /// Whether it works yet; reserved rows are listed and do nothing.
    status: Status,
    /// Listed among «Scorciatoie da ricordare» (C01).
    to_remember: bool,
}

/// Mirror: `ControlsBindings` in app/src/emu.ts.
#[derive(Debug, Serialize)]
pub struct ControlsBindings {
    remap: Vec<RemapRow>,
    combos: Vec<ComboRow>,
    /// The shortcut button (`pad.mod`).
    modifier: Pos,
}

/// The bindings of a controller family (or the keyboard) seen from a settings scope:
/// resolved with the defaults, so the interface never repeats the default tables.
#[tauri::command]
pub fn controls_bindings(
    state: State<'_, SettingsState>,
    family: Family,
    scope: Scope,
) -> Result<ControlsBindings, ErrorCode> {
    let mut settings = state.0.lock();
    let mut values = Vec::new();
    for key in [
        "pad.ab",
        "pad.mod",
        "pad.combos",
        "pad.remap",
        "pad.dead",
        "pad.stickdpad",
        "pad.trig",
        "pad.confirm",
        "pad.repeat",
    ] {
        values.push((key, settings.resolve(key, &scope)?.value));
    }
    let get = |key: &str| -> Option<Value> {
        values
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.clone())
    };
    let config = PadConfig::from_settings(get);
    let defaults = PadConfig {
        remap: Vec::new(),
        ..config.clone()
    }
    .remap_table(family);
    let remap = config
        .remap_table(family)
        .into_iter()
        .zip(defaults)
        .map(|((button, source, changed), (_, default, _))| RemapRow {
            button: button.id(),
            source,
            default,
            changed,
        })
        .collect();
    let combos = config
        .combo_table()
        .into_iter()
        .map(|(hotkey, pos): (Hotkey, Option<Pos>)| ComboRow {
            hotkey: hotkey.id(),
            pos,
            default: default_combo(hotkey),
            status: hotkey.status(),
            to_remember: hotkey.to_remember(),
        })
        .collect();
    Ok(ControlsBindings {
        remap,
        combos,
        modifier: config.modifier,
    })
}
