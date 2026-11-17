//! Controller routing: what the pad's buttons do at each poll.
//!
//! The same buttons mean three different things depending on the moment:
//!
//! - **in game**, they are the RetroPad the core reads;
//! - **with the modifier held** (L3 by default), they are combos ("hotkeys") for the shell:
//!   quick save, slots, speed, FPS overlay, pause, quick menu;
//! - **when the interface has the input** (quick menu open, or no game running), the d-pad
//!   and A/B move the focus in the interface instead of reaching the game.
//!
//! [`Router::step`] decides it as a pure function of (previous state, buttons now, time),
//! so it is tested without SDL or a window. The shell reads the pads, calls `step` and
//! dispatches the [`Action`]s; the game gets [`Routed::game_mask`].
//!
//! The combo table is [`DEFAULT_COMBOS`] with the modifier [`DEFAULT_MODIFIER`]: the
//! Phase 1 settings engine will make it remappable by building a different [`Bindings`].

use std::time::{Duration, Instant};

use crate::input::Button;

/// A held direction repeats its navigation step after this delay...
pub const REPEAT_DELAY: Duration = Duration::from_millis(400);
/// ...and then every this often, while it stays held.
pub const REPEAT_INTERVAL: Duration = Duration::from_millis(120);

/// One navigation step for the interface (payload of `emu://nav`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    /// L1: the previous tab or level (the settings scope bar, TV mode tabs).
    Prev,
    /// R1: the next tab or level.
    Next,
}

impl Nav {
    fn repeats(self) -> bool {
        match self {
            Nav::Up | Nav::Down | Nav::Left | Nav::Right => true,
            Nav::Confirm | Nav::Back | Nav::Prev | Nav::Next => false,
        }
    }
}

/// Buttons that navigate the interface while it has the input.
const NAV_BUTTONS: &[(Button, Nav)] = &[
    (Button::Up, Nav::Up),
    (Button::Down, Nav::Down),
    (Button::Left, Nav::Left),
    (Button::Right, Nav::Right),
    (Button::A, Nav::Confirm),
    (Button::B, Nav::Back),
    (Button::L, Nav::Prev),
    (Button::R, Nav::Next),
];

/// A shell feature reachable from the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hotkey {
    /// Open or close the quick menu.
    ToggleMenu,
    /// Save to the current slot.
    SaveState,
    /// Load from the current slot.
    LoadState,
    NextSlot,
    PreviousSlot,
    /// Fast forward while the button stays held: a [`Action::Release`] follows.
    FastForwardHold,
    FastForwardToggle,
    /// Slow motion (0.5×) on/off.
    SlowMotionToggle,
    /// FPS overlay on/off.
    FpsOverlay,
    /// Pause on/off, without opening the menu.
    Pause,
}

impl Hotkey {
    /// Whether the action lasts while the button is held (and then gets a release).
    pub fn is_hold(self) -> bool {
        match self {
            Hotkey::FastForwardHold => true,
            Hotkey::ToggleMenu
            | Hotkey::SaveState
            | Hotkey::LoadState
            | Hotkey::NextSlot
            | Hotkey::PreviousSlot
            | Hotkey::FastForwardToggle
            | Hotkey::SlowMotionToggle
            | Hotkey::FpsOverlay
            | Hotkey::Pause => false,
        }
    }

    /// Whether it works while the interface has the input. Only the menu toggle does:
    /// the other combos act on the running game.
    fn works_in_ui(self) -> bool {
        self == Hotkey::ToggleMenu
    }
}

/// The combo modifier: held, it turns the other buttons into hotkeys.
/// L3 because GBA, NDS and 3DS games never use a stick click (docs/combo-tasti.md).
pub const DEFAULT_MODIFIER: Button = Button::L3;

/// Default combos, modifier + button → action. Each fires once per press of the button.
/// The Guide button alone also toggles the menu (it is not a RetroPad button, see [`PadInput`]).
pub const DEFAULT_COMBOS: &[(Button, Hotkey)] = &[
    // Fallback for when Steam or the Xbox Game Bar take the Guide button.
    (Button::Start, Hotkey::ToggleMenu),
    (Button::R, Hotkey::SaveState),
    (Button::L, Hotkey::LoadState),
    (Button::Right, Hotkey::NextSlot),
    (Button::Left, Hotkey::PreviousSlot),
    (Button::R2, Hotkey::FastForwardHold),
    (Button::Up, Hotkey::FastForwardToggle),
    (Button::Down, Hotkey::SlowMotionToggle),
    (Button::X, Hotkey::FpsOverlay),
    (Button::A, Hotkey::Pause),
    // TODO(phase 1): (Button::L2, Rewind) held, when rewind exists.
    // TODO: (Button::B, Screenshot), when screenshots exist.
    // TODO(phase 2): (Button::Y, SwapScreens) and (Button::R3, NextLayout), NDS/3DS.
    // TODO: fullscreen on/off: the research had it on L3, which is now the modifier;
    //       its button is still to choose (Select is free).
];

/// The controller bindings the router follows. Today only the defaults exist;
/// the settings engine (Phase 1) will build other ones.
#[derive(Debug, Clone)]
pub struct Bindings {
    pub modifier: Button,
    pub combos: Vec<(Button, Hotkey)>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            modifier: DEFAULT_MODIFIER,
            combos: DEFAULT_COMBOS.to_vec(),
        }
    }
}

/// The controllers read at one poll, all pads merged.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PadInput {
    /// RetroPad mask ([`Button::bit`]), triggers and stick directions already folded in.
    pub buttons: u16,
    /// The Guide / Home button.
    pub guide: bool,
}

/// Something the shell has to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Press(Hotkey),
    /// End of a hold hotkey ([`Hotkey::is_hold`]): its button was released.
    Release(Hotkey),
    Nav(Nav),
}

/// Result of one [`Router::step`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Routed {
    /// What the core sees from the controllers.
    pub game_mask: u16,
    pub actions: Vec<Action>,
}

/// The routing state between two polls.
#[derive(Debug, Default)]
pub struct Router {
    bindings: Bindings,
    prev: PadInput,
    /// Buttons kept away from the game until released: those used in a combo, and those
    /// held when the input moved between game and interface (the A that confirmed
    /// "Resume" must not reach the game).
    suppressed: u16,
    /// Hold hotkeys in progress, with their button.
    holds: Vec<(Button, Hotkey)>,
    /// The held direction, and when it fires again.
    repeat: Option<(Button, Nav, Instant)>,
    ui_had_input: bool,
}

impl Router {
    pub fn new(bindings: Bindings) -> Self {
        Self {
            bindings,
            ..Self::default()
        }
    }

    /// Routes one poll. `ui_has_input` is true while the interface takes the controller
    /// (quick menu open, or no game running).
    pub fn step(&mut self, input: PadInput, ui_has_input: bool, now: Instant) -> Routed {
        let held = input.buttons;
        let pressed = held & !self.prev.buttons;
        let modifier = self.bindings.modifier.bit();
        let modifier_held = held & modifier != 0;
        let mut actions = Vec::new();

        self.suppressed &= held;
        if ui_has_input != self.ui_had_input {
            self.suppressed |= held;
            self.repeat = None;
            self.ui_had_input = ui_has_input;
        }

        // Hold hotkeys end when their button is released, whatever else happens.
        self.holds.retain(|(button, hotkey)| {
            let still = held & button.bit() != 0;
            if !still {
                actions.push(Action::Release(*hotkey));
            }
            still
        });

        if input.guide && !self.prev.guide {
            actions.push(Action::Press(Hotkey::ToggleMenu));
        }

        if modifier_held {
            for &(button, hotkey) in &self.bindings.combos {
                if pressed & button.bit() == 0 || (ui_has_input && !hotkey.works_in_ui()) {
                    continue;
                }
                actions.push(Action::Press(hotkey));
                if hotkey.is_hold() {
                    self.holds.push((button, hotkey));
                }
            }
            // Whatever is held with the modifier stays away from the game until released.
            self.suppressed |= held & !modifier;
            self.repeat = None;
        } else if ui_has_input {
            for &(button, nav) in NAV_BUTTONS {
                if pressed & button.bit() != 0 {
                    actions.push(Action::Nav(nav));
                    if nav.repeats() {
                        self.repeat = Some((button, nav, now + REPEAT_DELAY));
                    }
                }
            }
            if let Some((button, nav, next)) = self.repeat {
                if held & button.bit() == 0 {
                    self.repeat = None;
                } else if now >= next {
                    actions.push(Action::Nav(nav));
                    // A late poll does not fire a burst to catch up.
                    let on_time = next + REPEAT_INTERVAL;
                    let after = if on_time > now {
                        on_time
                    } else {
                        now + REPEAT_INTERVAL
                    };
                    self.repeat = Some((button, nav, after));
                }
            }
        }

        self.prev = input;
        let game_mask = if ui_has_input || modifier_held {
            0
        } else {
            held & !self.suppressed & !modifier
        };
        Routed { game_mask, actions }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, Bindings, Hotkey, Nav, PadInput, Router, REPEAT_DELAY, REPEAT_INTERVAL};
    use crate::input::Button;
    use std::time::{Duration, Instant};

    fn pad(buttons: &[Button]) -> PadInput {
        PadInput {
            buttons: buttons.iter().fold(0, |m, b| m | b.bit()),
            guide: false,
        }
    }

    fn router() -> Router {
        Router::new(Bindings::default())
    }

    #[test]
    fn in_game_buttons_reach_the_game() {
        let mut r = router();
        let out = r.step(pad(&[Button::A, Button::Up]), false, Instant::now());
        assert_eq!(out.game_mask, Button::A.bit() | Button::Up.bit());
        assert!(out.actions.is_empty());
    }

    #[test]
    fn the_modifier_never_reaches_the_game() {
        let mut r = router();
        let t = Instant::now();
        let out = r.step(pad(&[Button::L3]), false, t);
        assert_eq!(out.game_mask, 0);
        assert!(out.actions.is_empty());
        let out = r.step(pad(&[]), false, t);
        assert_eq!(out.game_mask, 0);
        assert!(out.actions.is_empty());
    }

    #[test]
    fn a_combo_fires_once_per_press_and_blocks_the_game() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::R]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::SaveState)]);
        assert_eq!(out.game_mask, 0);
        // Still held: nothing more.
        let out = r.step(pad(&[Button::L3, Button::R]), false, t);
        assert!(out.actions.is_empty());
        // Released and pressed again: fires again.
        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::R]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::SaveState)]);
    }

    #[test]
    fn a_combo_button_stays_away_from_the_game_after_the_modifier_is_released() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), false, t);
        r.step(pad(&[Button::L3, Button::R]), false, t);
        let out = r.step(pad(&[Button::R]), false, t);
        assert_eq!(out.game_mask, 0);
        assert!(out.actions.is_empty());
        r.step(pad(&[]), false, t);
        let out = r.step(pad(&[Button::R]), false, t);
        assert_eq!(out.game_mask, Button::R.bit());
    }

    #[test]
    fn a_button_held_before_the_modifier_does_not_fire() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::R]), false, t);
        let out = r.step(pad(&[Button::R, Button::L3]), false, t);
        assert!(out.actions.is_empty());
        assert_eq!(out.game_mask, 0);
    }

    #[test]
    fn every_default_combo_maps_to_its_action() {
        let expected = [
            (Button::Start, Hotkey::ToggleMenu),
            (Button::R, Hotkey::SaveState),
            (Button::L, Hotkey::LoadState),
            (Button::Right, Hotkey::NextSlot),
            (Button::Left, Hotkey::PreviousSlot),
            (Button::R2, Hotkey::FastForwardHold),
            (Button::Up, Hotkey::FastForwardToggle),
            (Button::Down, Hotkey::SlowMotionToggle),
            (Button::X, Hotkey::FpsOverlay),
            (Button::A, Hotkey::Pause),
        ];
        for (button, hotkey) in expected {
            let mut r = router();
            let t = Instant::now();
            r.step(pad(&[Button::L3]), false, t);
            let out = r.step(pad(&[Button::L3, button]), false, t);
            assert_eq!(out.actions, vec![Action::Press(hotkey)], "{button:?}");
        }
    }

    #[test]
    fn fast_forward_hold_releases_with_its_button_even_without_the_modifier() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::R2]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::FastForwardHold)]);
        let out = r.step(pad(&[Button::R2]), false, t);
        assert!(out.actions.is_empty());
        let out = r.step(pad(&[]), false, t);
        assert_eq!(out.actions, vec![Action::Release(Hotkey::FastForwardHold)]);
    }

    #[test]
    fn guide_toggles_the_menu_on_the_press_edge() {
        let mut r = router();
        let t = Instant::now();
        let guide = PadInput {
            buttons: 0,
            guide: true,
        };
        assert_eq!(
            r.step(guide, false, t).actions,
            vec![Action::Press(Hotkey::ToggleMenu)]
        );
        assert!(r.step(guide, false, t).actions.is_empty());
        r.step(PadInput::default(), true, t);
        assert_eq!(
            r.step(guide, true, t).actions,
            vec![Action::Press(Hotkey::ToggleMenu)]
        );
    }

    #[test]
    fn in_the_ui_the_pad_navigates_and_the_game_gets_nothing() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        let out = r.step(pad(&[Button::Down]), true, t);
        assert_eq!(out.actions, vec![Action::Nav(Nav::Down)]);
        assert_eq!(out.game_mask, 0);
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::A]), true, t).actions,
            vec![Action::Nav(Nav::Confirm)]
        );
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::B]), true, t).actions,
            vec![Action::Nav(Nav::Back)]
        );
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::L]), true, t).actions,
            vec![Action::Nav(Nav::Prev)]
        );
        r.step(pad(&[]), true, t);
        let out = r.step(pad(&[Button::R]), true, t);
        assert_eq!(out.actions, vec![Action::Nav(Nav::Next)]);
        assert_eq!(out.game_mask, 0);
    }

    #[test]
    fn in_the_ui_only_the_menu_combo_works() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), true, t);
        assert!(r
            .step(pad(&[Button::L3, Button::R]), true, t)
            .actions
            .is_empty());
        assert_eq!(
            r.step(pad(&[Button::L3, Button::R, Button::Start]), true, t)
                .actions,
            vec![Action::Press(Hotkey::ToggleMenu)]
        );
    }

    #[test]
    fn a_held_direction_repeats_after_the_delay() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        let navs =
            |r: &mut Router, at: Duration| r.step(pad(&[Button::Right]), true, t + at).actions;
        assert_eq!(navs(&mut r, Duration::ZERO), vec![Action::Nav(Nav::Right)]);
        assert!(navs(&mut r, REPEAT_DELAY - Duration::from_millis(1)).is_empty());
        assert_eq!(navs(&mut r, REPEAT_DELAY), vec![Action::Nav(Nav::Right)]);
        assert!(navs(&mut r, REPEAT_DELAY + REPEAT_INTERVAL / 2).is_empty());
        assert_eq!(
            navs(&mut r, REPEAT_DELAY + REPEAT_INTERVAL),
            vec![Action::Nav(Nav::Right)]
        );
        // Released: no more repeats.
        r.step(pad(&[]), true, t + REPEAT_DELAY * 2);
        assert!(r
            .step(pad(&[]), true, t + REPEAT_DELAY * 10)
            .actions
            .is_empty());
    }

    #[test]
    fn a_late_poll_fires_one_repeat_not_a_burst() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        r.step(pad(&[Button::Up]), true, t);
        let late = t + REPEAT_DELAY + REPEAT_INTERVAL * 10;
        assert_eq!(
            r.step(pad(&[Button::Up]), true, late).actions,
            vec![Action::Nav(Nav::Up)]
        );
        assert!(r
            .step(pad(&[Button::Up]), true, late + REPEAT_INTERVAL / 2)
            .actions
            .is_empty());
    }

    #[test]
    fn confirm_and_back_do_not_repeat() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        r.step(pad(&[Button::A]), true, t);
        assert!(r
            .step(pad(&[Button::A]), true, t + REPEAT_DELAY * 3)
            .actions
            .is_empty());
    }

    #[test]
    fn the_button_that_closes_the_menu_does_not_reach_the_game() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        r.step(pad(&[Button::A]), true, t);
        // The interface closed the menu while A is still held.
        let out = r.step(pad(&[Button::A, Button::Up]), false, t);
        assert_eq!(out.game_mask, 0);
        let out = r.step(pad(&[Button::A, Button::Up]), false, t);
        assert_eq!(out.game_mask, 0);
        let out = r.step(pad(&[Button::A]), false, t);
        assert_eq!(out.game_mask, 0);
        r.step(pad(&[]), false, t);
        assert_eq!(
            r.step(pad(&[Button::A]), false, t).game_mask,
            Button::A.bit()
        );
    }

    #[test]
    fn opening_the_ui_stops_a_direction_repeat_and_closing_it_does_not_navigate() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        r.step(pad(&[Button::Down]), true, t);
        let out = r.step(pad(&[Button::Down]), false, t + REPEAT_DELAY);
        assert!(out.actions.is_empty());
        assert_eq!(out.game_mask, 0);
    }
}
