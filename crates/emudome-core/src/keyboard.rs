//! Keyboard routing in game (Trello #81): the keyboard's fixed shortcuts and the keys of the
//! game's buttons, decided here as the controller's are in [`crate::hotkeys`].
//!
//! The webview has the focus, so it receives the keys and forwards them raw (`KeyboardEvent.code`,
//! down / up, repeat) to Rust; [`KeyRouter::key`] turns each into the game's button and the
//! shell's [`Action`]s, the same actions the controller combos give, carried out by the same code
//! in the shell. What stays in the interface: the menu key (Esc: open the quick menu, or back in
//! it), fullscreen (F11) and moving the focus in menus (arrows, Enter).
//!
//! The table is [`KEYBOARD_SHORTCUTS`], also generated for the interface's lists
//! (`app/src/generated/keyboard-shortcuts.json`, C01 / C03) and written in docs/combo-tasti.md,
//! section 5. A key does what the controller hotkey with the same id does, plus fullscreen, which
//! only the keyboard (and the quick menu) has.

use crate::hotkeys::{Action, Hotkey, Status};
use crate::input::Button;

/// What a key does: a controller hotkey's action, or fullscreen (keyboard only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Hotkey(Hotkey),
    Fullscreen,
}

impl KeyAction {
    /// The id the interface shows it by: the hotkey's id, or `fullscreen`.
    pub fn id(self) -> &'static str {
        match self {
            KeyAction::Hotkey(h) => h.id(),
            KeyAction::Fullscreen => "fullscreen",
        }
    }

    pub fn status(self) -> Status {
        match self {
            KeyAction::Hotkey(h) => h.status(),
            KeyAction::Fullscreen => Status::Active,
        }
    }
}

/// One fixed key of the keyboard (`code` is `KeyboardEvent.code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyShortcut {
    pub code: &'static str,
    pub action: KeyAction,
}

/// Esc menu · F1 save · F4 load · F2 / F3 previous / next slot · Tab fast forward (held, or
/// toggled with `speed.ffmode`, [`KeyRouter::set_fast_forward_toggle`]) ·
/// Backspace rewind (held; toggled with `a11y.toggle`) · F9 FPS · F11 full screen · F12
/// screenshot. F1 and F4 act on the current slot, the one F2 / F3 and L3 + ← → move.
pub const KEYBOARD_SHORTCUTS: &[KeyShortcut] = &[
    KeyShortcut {
        code: "Escape",
        action: KeyAction::Hotkey(Hotkey::ToggleMenu),
    },
    KeyShortcut {
        code: "F1",
        action: KeyAction::Hotkey(Hotkey::SaveState),
    },
    KeyShortcut {
        code: "F4",
        action: KeyAction::Hotkey(Hotkey::LoadState),
    },
    KeyShortcut {
        code: "F2",
        action: KeyAction::Hotkey(Hotkey::PreviousSlot),
    },
    KeyShortcut {
        code: "F3",
        action: KeyAction::Hotkey(Hotkey::NextSlot),
    },
    KeyShortcut {
        code: "Tab",
        action: KeyAction::Hotkey(Hotkey::FastForwardHold),
    },
    // Select's default key moved from Backspace to the right Shift (bindings.rs, Trello #42).
    KeyShortcut {
        code: "Backspace",
        action: KeyAction::Hotkey(Hotkey::Rewind),
    },
    KeyShortcut {
        code: "F9",
        action: KeyAction::Hotkey(Hotkey::FpsOverlay),
    },
    KeyShortcut {
        code: "F11",
        action: KeyAction::Fullscreen,
    },
    KeyShortcut {
        code: "F12",
        action: KeyAction::Hotkey(Hotkey::Screenshot),
    },
];

/// The working shortcut of a key, if it has one.
pub fn shortcut(code: &str) -> Option<KeyAction> {
    KEYBOARD_SHORTCUTS
        .iter()
        .find(|k| k.code == code && k.action.status() == Status::Active)
        .map(|k| k.action)
}

/// What one key event does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyRouted {
    /// A game button pressed or released.
    pub button: Option<(Button, bool)>,
    pub actions: Vec<Action>,
}

/// A hold hotkey in progress and the key that started it.
#[derive(Debug)]
struct Hold {
    code: String,
    hotkey: Hotkey,
    /// Started in toggle mode (`speed.ffmode` = `toggle`): it ends at the key's next press,
    /// not at its release.
    latched: bool,
}

/// The keyboard's routing state: the hold hotkeys whose key is down (or latched on), and how
/// the fast forward key works.
#[derive(Debug, Default)]
pub struct KeyRouter {
    holds: Vec<Hold>,
    fast_forward_toggle: bool,
}

impl KeyRouter {
    /// `speed.ffmode`: `true` (toggle) makes the fast forward key turn it on at one press and
    /// off at the next; `false` (hold, the default) lasts while the key is held. A fast forward
    /// already latched on still ends at the key's next press after a change.
    pub fn set_fast_forward_toggle(&mut self, toggle: bool) {
        self.fast_forward_toggle = toggle;
    }

    /// One key event. `button` is the game button the key gives (the keyboard's row of
    /// `pad.remap`); `game_has_keys`: a game is open and the quick menu closed.
    ///
    /// - The menu key and fullscreen are the interface's: nothing here.
    /// - In game: a shortcut fires on its first press (not on the key's repeats), a hold hotkey
    ///   ends on its release; any other key is the game's button (repeats ignored). In toggle
    ///   mode the fast forward key's press starts the hold and its next press ends it (the
    ///   same [`Action::Press`] / [`Action::Release`] pair, so the shell carries both modes out
    ///   alike).
    /// - Menu open or no game: nothing fires; a key let go still lets go of its game button and
    ///   ends its hold, so nothing stays held after resuming (Trello #74).
    pub fn key(
        &mut self,
        code: &str,
        pressed: bool,
        repeat: bool,
        button: Option<Button>,
        game_has_keys: bool,
    ) -> KeyRouted {
        let mut routed = KeyRouted::default();
        if !pressed {
            if let Some(i) = self.holds.iter().position(|h| h.code == code && !h.latched) {
                let hold = self.holds.remove(i);
                routed.actions.push(Action::Release(hold.hotkey));
                return routed;
            }
        }
        let action = shortcut(code);
        match action {
            Some(KeyAction::Fullscreen | KeyAction::Hotkey(Hotkey::ToggleMenu)) => return routed,
            Some(KeyAction::Hotkey(hotkey)) => {
                if game_has_keys && pressed && !repeat {
                    if let Some(i) = self.holds.iter().position(|h| h.code == code && h.latched) {
                        let hold = self.holds.remove(i);
                        routed.actions.push(Action::Release(hold.hotkey));
                        return routed;
                    }
                    routed.actions.push(Action::Press(hotkey));
                    if hotkey.is_hold() {
                        self.holds.push(Hold {
                            code: code.to_owned(),
                            hotkey,
                            latched: self.fast_forward_toggle && hotkey == Hotkey::FastForwardHold,
                        });
                    }
                }
                return routed;
            }
            None => {}
        }
        if let Some(button) = button {
            if !pressed || (game_has_keys && !repeat) {
                routed.button = Some((button, pressed));
            }
        }
        routed
    }

    /// Every key is let go where the game cannot see it (the menu opened, the window lost the
    /// focus): the holds end, a fast forward latched on in toggle mode too. The game's buttons
    /// are let go by the shell.
    pub fn release_all(&mut self) -> Vec<Action> {
        self.holds
            .drain(..)
            .map(|hold| Action::Release(hold.hotkey))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{shortcut, KeyAction, KeyRouted, KeyRouter, KEYBOARD_SHORTCUTS};
    use crate::hotkeys::{Action, Hotkey};
    use crate::input::Button;

    const A: Option<Button> = Some(Button::A);

    #[test]
    fn the_table_has_one_action_per_key_and_every_hotkey_id_is_known() {
        for (i, k) in KEYBOARD_SHORTCUTS.iter().enumerate() {
            assert!(
                KEYBOARD_SHORTCUTS[i + 1..].iter().all(|o| o.code != k.code),
                "{} twice",
                k.code
            );
            if let KeyAction::Hotkey(h) = k.action {
                assert_eq!(Hotkey::from_id(h.id()), Some(h));
            }
        }
        assert_eq!(shortcut("F11"), Some(KeyAction::Fullscreen));
        assert_eq!(shortcut("KeyX"), None);
    }

    #[test]
    fn a_game_key_is_the_games_button_without_its_repeats() {
        let mut r = KeyRouter::default();
        assert_eq!(
            r.key("KeyX", true, false, A, true).button,
            Some((Button::A, true))
        );
        assert_eq!(r.key("KeyX", true, true, A, true), KeyRouted::default());
        assert_eq!(
            r.key("KeyX", false, false, A, true).button,
            Some((Button::A, false))
        );
        assert_eq!(r.key("KeyK", true, false, None, true), KeyRouted::default());
    }

    #[test]
    fn a_shortcut_fires_once_per_press_and_a_hold_ends_on_release() {
        let mut r = KeyRouter::default();
        let press = r.key("F1", true, false, None, true);
        assert_eq!(press.actions, [Action::Press(Hotkey::SaveState)]);
        assert!(r.key("F1", true, true, None, true).actions.is_empty());
        assert!(r.key("F1", false, false, None, true).actions.is_empty());

        assert_eq!(
            r.key("Tab", true, false, None, true).actions,
            [Action::Press(Hotkey::FastForwardHold)]
        );
        assert!(r.key("Tab", true, true, None, true).actions.is_empty());
        assert_eq!(
            r.key("Tab", false, false, None, true).actions,
            [Action::Release(Hotkey::FastForwardHold)]
        );
    }

    #[test]
    fn a_shortcut_wins_over_a_game_key_on_the_same_key() {
        let mut r = KeyRouter::default();
        let routed = r.key("F1", true, false, A, true);
        assert_eq!(routed.button, None);
        assert_eq!(routed.actions, [Action::Press(Hotkey::SaveState)]);
    }

    #[test]
    fn the_menu_key_and_fullscreen_are_the_interfaces() {
        let mut r = KeyRouter::default();
        assert_eq!(
            r.key("Escape", true, false, None, true),
            KeyRouted::default()
        );
        assert_eq!(r.key("F11", true, false, None, true), KeyRouted::default());
    }

    #[test]
    fn in_the_menu_nothing_fires_but_releases_still_reach_the_game() {
        let mut r = KeyRouter::default();
        assert_eq!(r.key("KeyX", true, false, A, false), KeyRouted::default());
        assert!(r.key("F1", true, false, None, false).actions.is_empty());
        // A key held into the menu is let go there (#74).
        assert_eq!(
            r.key("KeyX", false, false, A, false).button,
            Some((Button::A, false))
        );
        // Backspace held, the menu opens, Backspace let go: the rewind it held ends.
        let _ = r.key("Backspace", true, false, None, true);
        assert_eq!(
            r.key("Backspace", false, false, None, false).actions,
            [Action::Release(Hotkey::Rewind)]
        );
    }

    #[test]
    fn letting_go_of_everything_ends_the_holds_once() {
        let mut r = KeyRouter::default();
        let _ = r.key("Tab", true, false, None, true);
        let _ = r.key("Backspace", true, false, None, true);
        assert_eq!(
            r.release_all(),
            [
                Action::Release(Hotkey::FastForwardHold),
                Action::Release(Hotkey::Rewind)
            ]
        );
        assert!(r.release_all().is_empty());
        // The key's own release afterwards does nothing more.
        assert!(r.key("Tab", false, false, None, true).actions.is_empty());
    }

    #[test]
    fn in_toggle_mode_tab_turns_fast_forward_on_and_its_next_press_off() {
        let mut r = KeyRouter::default();
        r.set_fast_forward_toggle(true);
        let on = [Action::Press(Hotkey::FastForwardHold)];
        let off = [Action::Release(Hotkey::FastForwardHold)];
        assert_eq!(r.key("Tab", true, false, None, true).actions, on);
        // Repeats and the release do nothing: it stays on.
        assert!(r.key("Tab", true, true, None, true).actions.is_empty());
        assert!(r.key("Tab", false, false, None, true).actions.is_empty());
        assert_eq!(r.key("Tab", true, false, None, true).actions, off);
        assert!(r.key("Tab", false, false, None, true).actions.is_empty());
        // And on again.
        assert_eq!(r.key("Tab", true, false, None, true).actions, on);
    }

    #[test]
    fn in_toggle_mode_only_fast_forward_latches() {
        let mut r = KeyRouter::default();
        r.set_fast_forward_toggle(true);
        let _ = r.key("Backspace", true, false, None, true);
        assert_eq!(
            r.key("Backspace", false, false, None, true).actions,
            [Action::Release(Hotkey::Rewind)]
        );
    }

    #[test]
    fn a_latched_fast_forward_ends_when_the_menu_opens_or_the_focus_goes() {
        let mut r = KeyRouter::default();
        r.set_fast_forward_toggle(true);
        let _ = r.key("Tab", true, false, None, true);
        let _ = r.key("Tab", false, false, None, true);
        assert_eq!(r.release_all(), [Action::Release(Hotkey::FastForwardHold)]);
        // Back in game, the next press turns it on again (not off).
        assert_eq!(
            r.key("Tab", true, false, None, true).actions,
            [Action::Press(Hotkey::FastForwardHold)]
        );
        // In the menu a press does nothing.
        let _ = r.release_all();
        assert!(r.key("Tab", true, false, None, false).actions.is_empty());
    }

    #[test]
    fn a_latched_fast_forward_still_ends_at_the_next_press_after_switching_to_hold() {
        let mut r = KeyRouter::default();
        r.set_fast_forward_toggle(true);
        let _ = r.key("Tab", true, false, None, true);
        let _ = r.key("Tab", false, false, None, true);
        r.set_fast_forward_toggle(false);
        assert_eq!(
            r.key("Tab", true, false, None, true).actions,
            [Action::Release(Hotkey::FastForwardHold)]
        );
        // From now on it holds.
        let _ = r.key("Tab", false, false, None, true);
        assert_eq!(
            r.key("Tab", true, false, None, true).actions,
            [Action::Press(Hotkey::FastForwardHold)]
        );
        assert_eq!(
            r.key("Tab", false, false, None, true).actions,
            [Action::Release(Hotkey::FastForwardHold)]
        );
    }
}
