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
//! The router works on **positions**, not on what the game sees: every bit is the RetroPad
//! button that sits at that place on a Nintendo pad (A = right face button, B = bottom one,
//! see [`crate::bindings::Pos`]). Combos and menu navigation therefore never move when the
//! game's buttons are remapped; the shell turns [`Routed::game_mask`] into the game's buttons
//! with each pad's [`crate::bindings::PadMap`].
//!
//! The default combo table is [`DEFAULT_COMBOS`] with the modifier [`DEFAULT_MODIFIER`];
//! the settings (`pad.combos`, `pad.mod`, `pad.confirm`, `pad.repeat`) build other
//! [`Bindings`] through [`crate::bindings::PadConfig`].

use std::time::{Duration, Instant};

use ts_rs::TS;

use crate::input::Button;

/// A held direction repeats its navigation step after this delay...
pub const REPEAT_DELAY: Duration = Duration::from_millis(400);
/// ...and then every this often, while it stays held.
pub const REPEAT_INTERVAL: Duration = Duration::from_millis(120);

/// One navigation step for the interface (payload of `emu://nav`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "NavAction")]
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
    /// X: the focused item's other actions (a cover's game details, like a right click).
    More,
}

impl Nav {
    fn repeats(self) -> bool {
        match self {
            Nav::Up | Nav::Down | Nav::Left | Nav::Right => true,
            Nav::Confirm | Nav::Back | Nav::Prev | Nav::Next | Nav::More => false,
        }
    }
}

/// Buttons that navigate the interface while it has the input, confirm and back aside
/// (they follow [`Bindings::confirm`]).
const NAV_BUTTONS: &[(Button, Nav)] = &[
    (Button::Up, Nav::Up),
    (Button::Down, Nav::Down),
    (Button::Left, Nav::Left),
    (Button::Right, Nav::Right),
    (Button::L, Nav::Prev),
    (Button::R, Nav::Next),
    (Button::X, Nav::More),
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
    /// Slow motion (`speed.slow`) on/off.
    SlowMotionToggle,
    /// Rewind while the button stays held (with `a11y.toggle`, on at one press and off at the
    /// next: the shell decides, the router always sends press and release).
    Rewind,
    /// Pause on/off, without opening the menu.
    Pause,
    /// Take a screenshot (PNG next to the saves).
    Screenshot,
    /// FPS overlay on/off.
    FpsOverlay,
    /// Swap the two screens ([`Status::DualScreen`]). A hold: the router sends press and release,
    /// the shell decides (`screens.swap`: toggled at each press, or swapped while held).
    SwapScreens,
    /// Next screen layout ([`Status::DualScreen`]).
    NextLayout,
    /// Close / open the lid, also from the quick menu ([`Status::DualScreen`]).
    Lid,
}

/// Whether a hotkey does something yet. TypeScript: `HotkeyStatus` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "HotkeyStatus")]
pub enum Status {
    /// It works.
    Active,
    /// Its feature does not exist yet: the combo is listed and reserved, and does nothing.
    Later,
    /// It works with Nintendo DS and 3DS games only (Trello #28): with any other game it does
    /// nothing.
    DualScreen,
}

impl Hotkey {
    /// Every hotkey, in the order of the shortcut lists (C01, C03; the G12 overlay later).
    pub const ALL: [Hotkey; 15] = [
        Hotkey::ToggleMenu,
        Hotkey::SaveState,
        Hotkey::LoadState,
        Hotkey::NextSlot,
        Hotkey::PreviousSlot,
        Hotkey::FastForwardHold,
        Hotkey::FastForwardToggle,
        Hotkey::SlowMotionToggle,
        Hotkey::Rewind,
        Hotkey::Pause,
        Hotkey::Screenshot,
        Hotkey::FpsOverlay,
        Hotkey::SwapScreens,
        Hotkey::NextLayout,
        Hotkey::Lid,
    ];

    /// The stable id in `pad.combos` values and in the interface (`controls.hotkey.<id>`).
    pub fn id(self) -> &'static str {
        match self {
            Hotkey::ToggleMenu => "menu",
            Hotkey::SaveState => "save",
            Hotkey::LoadState => "load",
            Hotkey::NextSlot => "next_slot",
            Hotkey::PreviousSlot => "prev_slot",
            Hotkey::FastForwardHold => "ff_hold",
            Hotkey::FastForwardToggle => "ff_toggle",
            Hotkey::SlowMotionToggle => "slow",
            Hotkey::Rewind => "rewind",
            Hotkey::Pause => "pause",
            Hotkey::Screenshot => "screenshot",
            Hotkey::FpsOverlay => "fps",
            Hotkey::SwapScreens => "swap",
            Hotkey::NextLayout => "layout",
            Hotkey::Lid => "lid",
        }
    }

    pub fn from_id(id: &str) -> Option<Hotkey> {
        Hotkey::ALL.into_iter().find(|h| h.id() == id)
    }

    /// Whether the hotkey does something yet. The router never fires the [`Status::Later`] ones.
    pub fn status(self) -> Status {
        match self {
            Hotkey::ToggleMenu
            | Hotkey::SaveState
            | Hotkey::LoadState
            | Hotkey::NextSlot
            | Hotkey::PreviousSlot
            | Hotkey::FastForwardHold
            | Hotkey::FastForwardToggle
            | Hotkey::SlowMotionToggle
            | Hotkey::Pause
            | Hotkey::FpsOverlay
            | Hotkey::Rewind
            | Hotkey::Screenshot => Status::Active,
            Hotkey::SwapScreens | Hotkey::NextLayout | Hotkey::Lid => Status::DualScreen,
        }
    }

    /// Listed among «Scorciatoie da ricordare» (C01, board Controlli2): every hotkey but the
    /// screen layout, which only C03 lists. The previous / next slot pair is one line there.
    pub fn to_remember(self) -> bool {
        self != Hotkey::NextLayout
    }

    /// Whether the action lasts while the button is held (and then gets a release).
    pub fn is_hold(self) -> bool {
        match self {
            Hotkey::FastForwardHold | Hotkey::Rewind | Hotkey::SwapScreens => true,
            Hotkey::ToggleMenu
            | Hotkey::SaveState
            | Hotkey::LoadState
            | Hotkey::NextSlot
            | Hotkey::PreviousSlot
            | Hotkey::FastForwardToggle
            | Hotkey::SlowMotionToggle
            | Hotkey::Pause
            | Hotkey::Screenshot
            | Hotkey::FpsOverlay
            | Hotkey::NextLayout
            | Hotkey::Lid => false,
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

/// Default combos, modifier + button → action, in [`Hotkey::ALL`] order. Each fires once per
/// press of the button; the rows whose [`Hotkey::status`] is [`Status::Later`] are reserved and
/// do nothing ([`Status::DualScreen`] ones act on DS and 3DS games only). The Guide button alone also toggles the menu (it is not a RetroPad button, see
/// [`PadInput`]). Fullscreen has no combo: keyboard F11 and the quick menu only.
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
    (Button::L2, Hotkey::Rewind),
    (Button::A, Hotkey::Pause),
    (Button::B, Hotkey::Screenshot),
    (Button::X, Hotkey::FpsOverlay),
    (Button::Y, Hotkey::SwapScreens),
    (Button::R3, Hotkey::NextLayout),
    (Button::Select, Hotkey::Lid),
];

/// The quick save slots the current slot moves through (L3 + d-pad left / right, keyboard
/// F2 / F3), wrapping around.
pub const FIRST_SLOT: u8 = 1;
pub const LAST_SLOT: u8 = 9;

/// The current slot after one step (`forward`: next, else previous), wrapping between
/// [`FIRST_SLOT`] and [`LAST_SLOT`]. From a slot out of that range the next is the first and
/// the previous the last.
pub fn step_slot(slot: u8, forward: bool) -> u8 {
    let in_range = (FIRST_SLOT..=LAST_SLOT).contains(&slot);
    match (forward, in_range) {
        (true, true) if slot < LAST_SLOT => slot + 1,
        (false, true) if slot > FIRST_SLOT => slot - 1,
        (true, true | false) => FIRST_SLOT,
        (false, true | false) => LAST_SLOT,
    }
}

/// The controller bindings the router follows, built from the settings by
/// [`crate::bindings::PadConfig::bindings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bindings {
    pub modifier: Button,
    pub combos: Vec<(Button, Hotkey)>,
    /// The position that confirms in the menus (`pad.confirm`): [`Button::A`] (right, like
    /// Nintendo) or [`Button::B`] (bottom, like Xbox). The other one goes back.
    pub confirm: Button,
    /// Menu key repeat (`pad.repeat`): first repeat after `repeat_delay`, then every
    /// `repeat_interval`.
    pub repeat_delay: Duration,
    pub repeat_interval: Duration,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            modifier: DEFAULT_MODIFIER,
            combos: DEFAULT_COMBOS.to_vec(),
            confirm: Button::A,
            repeat_delay: REPEAT_DELAY,
            repeat_interval: REPEAT_INTERVAL,
        }
    }
}

impl Bindings {
    /// The navigation step of a position while the interface has the input.
    fn nav(&self, button: Button) -> Option<Nav> {
        if button == self.confirm {
            return Some(Nav::Confirm);
        }
        let back = if self.confirm == Button::B {
            Button::A
        } else {
            Button::B
        };
        if button == back {
            return Some(Nav::Back);
        }
        NAV_BUTTONS
            .iter()
            .find(|(b, _)| *b == button)
            .map(|(_, nav)| *nav)
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
    /// The game has the controllers' input (no menu, no combo modifier held): the analog sticks
    /// reach the core only then, centred otherwise.
    pub game_has_input: bool,
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

    /// New bindings from the settings; the routing state carries on.
    pub fn set_bindings(&mut self, bindings: Bindings) {
        self.bindings = bindings;
    }

    /// A poll the router must not act on (the Controls section is capturing a button):
    /// nothing fires, the game gets nothing, and the buttons held now stay away from the game
    /// and from the menu until released, so the button just assigned does not also click.
    pub fn absorb(&mut self, input: PadInput) {
        self.suppressed |= input.buttons;
        self.repeat = None;
        self.prev = input;
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
                // A reserved row (feature still to come) does nothing: its button only stays
                // away from the game, like any button held with the modifier.
                let reserved = hotkey.status() == Status::Later;
                if pressed & button.bit() == 0
                    || reserved
                    || (ui_has_input && !hotkey.works_in_ui())
                {
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
            // A button absorbed during a capture does not navigate until released.
            let pressed = pressed & !self.suppressed;
            for button in Button::ALL {
                let Some(nav) = self.bindings.nav(button) else {
                    continue;
                };
                if pressed & button.bit() != 0 {
                    actions.push(Action::Nav(nav));
                    if nav.repeats() {
                        self.repeat = Some((button, nav, now + self.bindings.repeat_delay));
                    }
                }
            }
            if let Some((button, nav, next)) = self.repeat {
                if held & button.bit() == 0 {
                    self.repeat = None;
                } else if now >= next {
                    actions.push(Action::Nav(nav));
                    // A late poll does not fire a burst to catch up.
                    let on_time = next + self.bindings.repeat_interval;
                    let after = if on_time > now {
                        on_time
                    } else {
                        now + self.bindings.repeat_interval
                    };
                    self.repeat = Some((button, nav, after));
                }
            }
        }

        self.prev = input;
        let game_has_input = !(ui_has_input || modifier_held);
        let game_mask = if game_has_input {
            held & !self.suppressed & !modifier
        } else {
            0
        };
        Routed {
            game_mask,
            game_has_input,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        step_slot, Action, Bindings, Hotkey, Nav, PadInput, Router, Status, DEFAULT_COMBOS,
        FIRST_SLOT, LAST_SLOT, REPEAT_DELAY, REPEAT_INTERVAL,
    };
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
        assert!(out.game_has_input);
    }

    #[test]
    fn the_sticks_are_the_games_only_without_menu_or_modifier() {
        let t = Instant::now();
        assert!(!router().step(pad(&[]), true, t).game_has_input);
        assert!(!router().step(pad(&[Button::L3]), false, t).game_has_input);
        assert!(router().step(pad(&[]), false, t).game_has_input);
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
            assert_eq!(hotkey.status(), Status::Active, "{hotkey:?}");
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
        r.step(pad(&[]), true, t);
        // X: the focused item's other actions; it does not repeat while held.
        let out = r.step(pad(&[Button::X]), true, t);
        assert_eq!(out.actions, vec![Action::Nav(Nav::More)]);
        assert_eq!(out.game_mask, 0);
        assert!(r
            .step(pad(&[Button::X]), true, t + REPEAT_DELAY)
            .actions
            .is_empty());
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

    #[test]
    fn confirm_at_the_bottom_swaps_confirm_and_back() {
        let mut r = Router::new(Bindings {
            confirm: Button::B,
            ..Bindings::default()
        });
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::B]), true, t).actions,
            vec![Action::Nav(Nav::Confirm)]
        );
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::A]), true, t).actions,
            vec![Action::Nav(Nav::Back)]
        );
    }

    #[test]
    fn the_repeat_follows_the_bindings() {
        let delay = Duration::from_millis(250);
        let interval = Duration::from_millis(70);
        let mut r = Router::new(Bindings {
            repeat_delay: delay,
            repeat_interval: interval,
            ..Bindings::default()
        });
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        r.step(pad(&[Button::Up]), true, t);
        assert_eq!(
            r.step(pad(&[Button::Up]), true, t + delay).actions,
            vec![Action::Nav(Nav::Up)]
        );
        assert_eq!(
            r.step(pad(&[Button::Up]), true, t + delay + interval)
                .actions,
            vec![Action::Nav(Nav::Up)]
        );
    }

    #[test]
    fn a_button_absorbed_by_a_capture_neither_navigates_nor_plays_until_released() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[]), true, t);
        // Captured while held: the capture ends with B still down.
        r.absorb(pad(&[Button::B]));
        assert!(r.step(pad(&[Button::B]), true, t).actions.is_empty());
        r.step(pad(&[]), true, t);
        assert_eq!(
            r.step(pad(&[Button::B]), true, t).actions,
            vec![Action::Nav(Nav::Back)]
        );
        // In game too.
        r.absorb(pad(&[Button::A]));
        assert_eq!(r.step(pad(&[Button::A]), false, t).game_mask, 0);
        r.step(pad(&[]), false, t);
        assert_eq!(
            r.step(pad(&[Button::A]), false, t).game_mask,
            Button::A.bit()
        );
    }

    #[test]
    fn the_table_lists_every_hotkey_once_in_the_order_of_the_lists() {
        let order: Vec<Hotkey> = DEFAULT_COMBOS.iter().map(|(_, h)| *h).collect();
        assert_eq!(order, Hotkey::ALL);
        for (i, (button, _)) in DEFAULT_COMBOS.iter().enumerate() {
            assert!(
                DEFAULT_COMBOS[..i].iter().all(|(b, _)| b != button),
                "{button:?} twice"
            );
        }
    }

    #[test]
    fn the_dual_screen_rows_are_the_screens_and_the_lid() {
        let dual = [
            (Button::Y, Hotkey::SwapScreens, Status::DualScreen),
            (Button::R3, Hotkey::NextLayout, Status::DualScreen),
            (Button::Select, Hotkey::Lid, Status::DualScreen),
        ];
        for (button, hotkey, status) in dual {
            assert!(DEFAULT_COMBOS.contains(&(button, hotkey)), "{hotkey:?}");
            assert_eq!(hotkey.status(), status, "{hotkey:?}");
        }
        let active = Hotkey::ALL
            .iter()
            .filter(|h| h.status() == Status::Active)
            .count();
        assert_eq!(active, 12);
        assert!(Hotkey::ALL.iter().all(|h| h.status() != Status::Later));
    }

    #[test]
    fn rewind_lasts_while_l2_is_held_and_the_screenshot_fires_once() {
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::L2]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::Rewind)]);
        assert_eq!(out.game_mask, 0);
        // L3 let go first: still rewinding, L2 still away from the game.
        let out = r.step(pad(&[Button::L2]), false, t);
        assert!(out.actions.is_empty());
        assert_eq!(out.game_mask, 0);
        let out = r.step(pad(&[]), false, t);
        assert_eq!(out.actions, vec![Action::Release(Hotkey::Rewind)]);

        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::B]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::Screenshot)]);
        assert!(r
            .step(pad(&[Button::L3, Button::B]), false, t)
            .actions
            .is_empty());
        assert!(r.step(pad(&[Button::L3]), false, t).actions.is_empty());
        // Not from the menu: only the menu combo works there.
        let out = r.step(pad(&[Button::L3, Button::B]), true, t);
        assert!(out.actions.is_empty());
    }

    #[test]
    fn the_dual_screen_combos_fire_and_keep_their_button_from_the_game() {
        // The shell does nothing with them for a one-screen game (app/src-tauri/src/screens.rs).
        for (button, hotkey) in [
            (Button::R3, Hotkey::NextLayout),
            (Button::Select, Hotkey::Lid),
        ] {
            let mut r = router();
            let t = Instant::now();
            r.step(pad(&[Button::L3]), false, t);
            let out = r.step(pad(&[Button::L3, button]), false, t);
            assert_eq!(out.actions, vec![Action::Press(hotkey)], "{button:?}");
            assert_eq!(out.game_mask, 0, "{button:?}");
            // Released with the modifier gone: no release action (not a hold), and it reaches
            // the game only after a new press.
            let out = r.step(pad(&[button]), false, t);
            assert!(out.actions.is_empty(), "{button:?}");
            assert_eq!(out.game_mask, 0, "{button:?}");
            let out = r.step(pad(&[]), false, t);
            assert!(out.actions.is_empty(), "{button:?}");
        }
        // Swap is a hold: its release follows Y, for `screens.swap` = hold.
        let mut r = router();
        let t = Instant::now();
        r.step(pad(&[Button::L3]), false, t);
        let out = r.step(pad(&[Button::L3, Button::Y]), false, t);
        assert_eq!(out.actions, vec![Action::Press(Hotkey::SwapScreens)]);
        let out = r.step(pad(&[Button::Y]), false, t);
        assert!(out.actions.is_empty());
        let out = r.step(pad(&[]), false, t);
        assert_eq!(out.actions, vec![Action::Release(Hotkey::SwapScreens)]);
        // Not from the menu.
        r.step(pad(&[Button::L3]), true, t);
        assert!(r
            .step(pad(&[Button::L3, Button::R3]), true, t)
            .actions
            .is_empty());
    }

    #[test]
    fn c01_remembers_every_shortcut_but_the_screen_layout() {
        let listed = Hotkey::ALL.iter().filter(|h| h.to_remember()).count();
        // 14 hotkeys, the slot pair on one line: the board's 13 rows.
        assert_eq!(listed, 14);
        assert!(!Hotkey::NextLayout.to_remember());
    }

    #[test]
    fn the_slot_steps_and_wraps() {
        assert_eq!(step_slot(1, true), 2);
        assert_eq!(step_slot(2, false), 1);
        assert_eq!(step_slot(LAST_SLOT, true), FIRST_SLOT);
        assert_eq!(step_slot(FIRST_SLOT, false), LAST_SLOT);
        // Out of range (the auto slot, a stale value): the first forward, the last backward.
        assert_eq!(step_slot(0, true), FIRST_SLOT);
        assert_eq!(step_slot(0, false), LAST_SLOT);
        assert_eq!(step_slot(200, true), FIRST_SLOT);
    }

    #[test]
    fn hotkey_ids_round_trip() {
        for h in Hotkey::ALL {
            assert_eq!(Hotkey::from_id(h.id()), Some(h));
        }
        for b in Button::ALL {
            assert_eq!(Button::from_id(b.id()), Some(b));
        }
    }
}
