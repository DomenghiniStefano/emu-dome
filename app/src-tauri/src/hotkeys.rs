//! Carries out the shortcuts of the controllers and of the keyboard, the same way for both
//! (Trello #81): what the routers decide (`emudome_core::hotkeys::Router` for the pads in
//! `gamepad.rs`, `emudome_core::keyboard::KeyRouter` for the keys here) becomes session
//! `Command`s and events for the interface:
//! - `emu://menu`: open / close the quick menu;
//! - `emu://nav`: a [`NavEvent`], one focus step in the interface (controllers only);
//! - `emu://hotkey`: a [`HotkeyEvent`], what a shortcut changed that the interface shows.
//!
//! A hold shortcut (fast forward, rewind) held from both the keyboard and a pad lasts until both
//! let go: fast forward goes back to the speed from before the first one. With `speed.ffmode` =
//! toggle the keyboard's Tab latches its fast forward hold (`KeyRouter`): its next press, the
//! quick menu or the window losing the focus let go of it.

use std::sync::Arc;

use emudome_core::bindings::Family;
use emudome_core::hotkeys::{Action, Hotkey, Nav};
use emudome_core::input::Button;
use emudome_core::keyboard::KeyRouter;
use emudome_core::{Command, Speed};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::pads::{self, PadsState};
use crate::{events, CurrentShared};

/// The speed the toggles go back to. Fast forward and slow motion come from the settings
/// (`speed.ffval`, `speed.slow`, read into `PadConfig`).
const NORMAL: Speed = Speed::Factor(1.0);

/// Payload of `emu://nav`: `{ action: "up" | "down" | "left" | "right" | "confirm" | "back" }`.
#[derive(Clone, Serialize)]
pub struct NavEvent {
    action: Nav,
}

/// Payload of `emu://hotkey`: what a shortcut changed, for the interface to show. Codes and
/// values only; the interface turns them into text. TypeScript: `HotkeyEvent` (generated).
#[derive(Clone, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export_to = "app/")]
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

/// Where a shortcut came from: a hold lasts while any of them holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Keyboard,
    #[allow(dead_code, reason = "used by gamepad.rs, desktop only (SDL3)")]
    Pads,
}

impl Origin {
    fn bit(self) -> u8 {
        match self {
            Origin::Keyboard => 1,
            Origin::Pads => 2,
        }
    }
}

/// Who holds a hold shortcut now (Tab and L3 + R2 together are one fast forward).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Holders(u8);

impl Holders {
    fn press(&mut self, origin: Origin) {
        self.0 |= origin.bit();
    }

    /// `origin` lets go: true when nobody holds it any more.
    fn release(&mut self, origin: Origin) -> bool {
        self.0 &= !origin.bit();
        self.0 == 0
    }
}

/// The shortcuts' state, shared by the controller thread and the keyboard's command.
pub struct HotkeyRunner(Mutex<Runner>);

struct Runner {
    app: AppHandle,
    current: CurrentShared,
    /// The current slot (F1 / F4, L3 + R1 / L3 + L1 save to and load from it).
    pads: PadsState,
    /// While fast forward is held, the speed to go back to.
    turbo_restore: Option<Speed>,
    /// Who holds fast forward / rewind now.
    turbo_holders: Holders,
    rewind_holders: Holders,
    /// The settings generation the values below were read at.
    config_at: Option<u64>,
    /// `speed.ffval` and `speed.slow` of the open game (or «Tutti i giochi»).
    fast_forward: Speed,
    slow_motion: Speed,
    /// `a11y.toggle`: rewind turns on at one press and off at the next, instead of holding.
    rewind_toggle: bool,
    /// The keyboard's row of `pad.remap`: `KeyboardEvent.code` → game button.
    keymap: Vec<(String, Button)>,
    keys: KeyRouter,
}

impl HotkeyRunner {
    pub fn new(app: AppHandle, current: CurrentShared, pads: PadsState) -> Arc<Self> {
        let config = emudome_core::bindings::PadConfig::default();
        Arc::new(Self(Mutex::new(Runner {
            app,
            current,
            pads,
            turbo_restore: None,
            turbo_holders: Holders::default(),
            rewind_holders: Holders::default(),
            config_at: None,
            fast_forward: config.fast_forward,
            slow_motion: config.slow_motion,
            rewind_toggle: config.hold_toggle,
            keymap: keymap(&config),
            keys: KeyRouter::default(),
        })))
    }

    /// The controller router's actions.
    #[allow(dead_code, reason = "used by gamepad.rs, desktop only (SDL3)")]
    pub fn run(&self, actions: Vec<Action>, from: Origin) {
        if actions.is_empty() {
            return;
        }
        let mut runner = self.0.lock();
        runner.refresh();
        for action in actions {
            runner.run(action, from);
        }
    }

    /// One key event from the interface: its shortcut carried out; the game button it gives
    /// (pressed or released) returned for the caller to set. `game_has_keys`: a game is open
    /// and the quick menu closed.
    pub fn key(
        &self,
        code: &str,
        pressed: bool,
        repeat: bool,
        game_has_keys: bool,
    ) -> Option<(Button, bool)> {
        let mut runner = self.0.lock();
        runner.refresh();
        let button = runner
            .keymap
            .iter()
            .find(|(c, _)| c == code)
            .map(|(_, b)| *b);
        let routed = runner
            .keys
            .key(code, pressed, repeat, button, game_has_keys);
        for action in routed.actions {
            runner.run(action, Origin::Keyboard);
        }
        routed.button
    }

    /// The keys held now are let go where the game cannot see it (the quick menu opened, the
    /// window lost the focus): fast forward and a held rewind of the keyboard end (a rewind
    /// switched on with `a11y.toggle` stays on).
    pub fn release_keys(&self) {
        let mut runner = self.0.lock();
        for action in runner.keys.release_all() {
            runner.run(action, Origin::Keyboard);
        }
    }
}

/// The keyboard's game keys from the controller settings.
fn keymap(config: &emudome_core::bindings::PadConfig) -> Vec<(String, Button)> {
    config
        .remap_table(Family::Keyboard)
        .into_iter()
        .filter_map(|(button, source, _)| Some((source?, button)))
        .collect()
}

impl Runner {
    /// Follows the settings after a change (the open game's, with its changes «in prova»).
    fn refresh(&mut self) {
        let generation = pads::config_generation();
        if self.config_at == Some(generation) {
            return;
        }
        if let Some(config) = pads::current_config(&self.app) {
            self.fast_forward = config.fast_forward;
            self.slow_motion = config.slow_motion;
            self.rewind_toggle = config.hold_toggle;
            self.keymap = keymap(&config);
            self.keys
                .set_fast_forward_toggle(config.fast_forward_toggle);
            self.config_at = Some(generation);
        }
    }

    fn run(&mut self, action: Action, from: Origin) {
        match action {
            Action::Nav(nav) => self.emit(events::NAV, NavEvent { action: nav }),
            Action::Press(hotkey) => self.press(hotkey, from),
            Action::Release(hotkey) => self.release(hotkey, from),
        }
    }

    fn press(&mut self, hotkey: Hotkey, from: Origin) {
        match hotkey {
            Hotkey::ToggleMenu => self.emit(events::MENU, ()),
            Hotkey::SaveState => {
                self.send(Command::SaveState(self.pads.slot()));
            }
            Hotkey::LoadState => {
                self.send(Command::LoadState(self.pads.slot()));
            }
            Hotkey::NextSlot | Hotkey::PreviousSlot => {
                let slot = self.pads.step_slot(hotkey == Hotkey::NextSlot);
                self.emit(events::HOTKEY, HotkeyEvent::Slot { slot });
            }
            Hotkey::FastForwardHold => {
                if self.turbo_restore.is_none() {
                    let before = self.speed();
                    if self.send(Command::SetSpeed(self.fast_forward)) {
                        self.turbo_restore = Some(before);
                        self.emit(events::HOTKEY, HotkeyEvent::Turbo { held: true });
                    }
                }
                self.turbo_holders.press(from);
            }
            Hotkey::FastForwardToggle => self.toggle_speed(self.fast_forward),
            Hotkey::SlowMotionToggle => self.toggle_speed(self.slow_motion),
            Hotkey::FpsOverlay => self.emit(events::HOTKEY, HotkeyEvent::Fps),
            Hotkey::Pause => {
                let paused = !self.paused();
                if self.send(Command::SetPaused(paused)) {
                    self.emit(events::HOTKEY, HotkeyEvent::Paused { paused });
                }
            }
            Hotkey::Rewind => {
                let on = !(self.rewind_toggle && self.rewinding());
                self.send(Command::Rewind(on));
                if !self.rewind_toggle {
                    self.rewind_holders.press(from);
                }
            }
            Hotkey::Screenshot => {
                self.send(Command::Screenshot);
            }
            // Two-screen games only (`Hotkey::status`): nothing with a one-screen game.
            Hotkey::SwapScreens => crate::screens::swap_hotkey(&self.app, true),
            Hotkey::NextLayout => crate::screens::next_layout_hotkey(&self.app),
            Hotkey::Lid => crate::screens::lid_hotkey(&self.app),
        }
    }

    fn release(&mut self, hotkey: Hotkey, from: Origin) {
        match hotkey {
            Hotkey::FastForwardHold => {
                if self.turbo_holders.release(from) {
                    if let Some(before) = self.turbo_restore.take() {
                        self.send(Command::SetSpeed(before));
                        self.emit(events::HOTKEY, HotkeyEvent::Turbo { held: false });
                    }
                }
            }
            Hotkey::Rewind => {
                if !self.rewind_toggle && self.rewind_holders.release(from) {
                    self.send(Command::Rewind(false));
                }
            }
            // `screens.swap` = hold: the screens go back (a toggle ignores it).
            Hotkey::SwapScreens => crate::screens::swap_hotkey(&self.app, false),
            // Press-only hotkeys have nothing to undo.
            Hotkey::ToggleMenu
            | Hotkey::SaveState
            | Hotkey::LoadState
            | Hotkey::NextSlot
            | Hotkey::PreviousSlot
            | Hotkey::FastForwardToggle
            | Hotkey::SlowMotionToggle
            | Hotkey::FpsOverlay
            | Hotkey::Pause
            | Hotkey::Screenshot
            | Hotkey::NextLayout
            | Hotkey::Lid => {}
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
            self.emit(events::HOTKEY, HotkeyEvent::Speed { speed });
        }
    }

    fn speed(&self) -> Speed {
        self.current
            .load()
            .as_ref()
            .and_then(|s| s.stats.lock().speed)
            .unwrap_or(NORMAL)
    }

    fn rewinding(&self) -> bool {
        self.current
            .load()
            .as_ref()
            .is_some_and(|s| s.stats.lock().rewinding)
    }

    fn paused(&self) -> bool {
        self.current
            .load()
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

/// What Android's back button means now: what the controller's buttons already mean, so the
/// interface handles it where it handles them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackRoute {
    /// In game, the quick menu closed: the menu button (`emu://menu`) opens the quick menu.
    Menu,
    /// Anywhere else (the quick menu, the shell, a dialog): B (`emu://nav` back), one level up.
    Nav,
}

pub(crate) fn back_route(game_open: bool, menu_open: bool) -> BackRoute {
    if game_open && !menu_open {
        BackRoute::Menu
    } else {
        BackRoute::Nav
    }
}

/// Android's back button (`emudome_platform::attach_system`), on the UI thread: sent to the
/// interface as the controller's menu button or B ([`back_route`]). The state is read without
/// waiting: `menu_open` is an atomic, the open game a short lock.
pub(crate) fn back_button(app: &AppHandle) {
    use tauri::Manager;
    let Some(state) = app.try_state::<crate::AppState>() else {
        log::info!("back pressed before the app is set up: ignored");
        return;
    };
    let route = back_route(
        state.games.is_open(),
        state.menu_open.load(std::sync::atomic::Ordering::Relaxed),
    );
    let sent = match route {
        BackRoute::Menu => app.emit(events::MENU, ()),
        BackRoute::Nav => app.emit(events::NAV, NavEvent { action: Nav::Back }),
    };
    if let Err(e) = sent {
        log::warn!("back button ({route:?}) not delivered: {e}");
    }
}

/// Android's back at the shell's root, with no section to go back to (Trello #29): the app goes
/// to the background as at any app's first screen, still running. `false` where the system has
/// no background (the desktop) or the activity could not be reached (logged).
#[tauri::command]
pub(crate) fn leave_to_background() -> bool {
    emudome_platform::move_to_background()
}

#[cfg(test)]
mod tests {
    use super::{back_route, BackRoute, Holders, Origin};

    #[test]
    fn back_opens_the_quick_menu_in_game_and_is_b_everywhere_else() {
        assert_eq!(back_route(true, false), BackRoute::Menu);
        // In the quick menu: one level up, and from its main screen the game resumes.
        assert_eq!(back_route(true, true), BackRoute::Nav);
        // The shell (no game): its own back.
        assert_eq!(back_route(false, false), BackRoute::Nav);
        assert_eq!(back_route(false, true), BackRoute::Nav);
    }

    #[test]
    fn tab_and_the_pads_combo_together_are_one_hold() {
        let mut holders = Holders::default();
        holders.press(Origin::Keyboard);
        holders.press(Origin::Pads);
        // Tab let go while L3 + R2 still holds: fast forward goes on.
        assert!(!holders.release(Origin::Keyboard));
        // ...a second Tab release changes nothing...
        assert!(!holders.release(Origin::Keyboard));
        // ...and it ends with the last one.
        assert!(holders.release(Origin::Pads));
    }

    #[test]
    fn a_single_holder_ends_the_hold_on_its_release() {
        let mut holders = Holders::default();
        holders.press(Origin::Keyboard);
        assert!(holders.release(Origin::Keyboard));
        // A release nobody pressed (the hold began elsewhere and ended) still says «free».
        assert!(holders.release(Origin::Pads));
    }
}
