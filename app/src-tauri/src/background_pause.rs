//! `general.bgpause`: the game pauses while EmuDome is not in front (the page lost the focus on
//! the desktop, Android sent the app to the background) and resumes when it comes back, only if
//! the pause was this one: a pause of the user's (the quick menu, the resume question, a pause
//! asked by the interface meanwhile) stays. The decision is [`BackgroundPause`], pure and tested;
//! [`focus_changed`] carries it out.
//!
//! Where the focus is seen: the desktop's page (`window_focus`, from its `blur` / `focus` /
//! `visibilitychange`), because the window's own `WindowEvent::Focused` also flips when the
//! child webview takes the keyboard focus and may not fire while it has it; Android's
//! `Suspended` / `Resumed` (`emudome_platform::is_suspend` / `is_resume`).
//!
//! Blind spot: a pause toggled with the controller's shortcut (L3 + Start) while EmuDome is in
//! the background is not seen; coming back then resumes the game.

use emudome_core::settings::{keys, Scope};
use emudome_core::Command;
use tauri::{AppHandle, Emitter, Manager};

use crate::hotkeys::HotkeyEvent;
use crate::AppState;

/// The open game as the decision sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameNow {
    pub open: bool,
    pub paused: bool,
    pub menu_open: bool,
}

/// What to do to the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Pause,
    Resume,
    Nothing,
}

/// Whether the game is paused because EmuDome went to the background.
#[derive(Debug, Default)]
pub struct BackgroundPause {
    ours: bool,
}

impl BackgroundPause {
    /// EmuDome lost the focus: a running game pauses, with `general.bgpause` on (`enabled`). A
    /// game already paused (the menu, the resume question, the user) is left as it is.
    pub fn lost(&mut self, enabled: bool, game: GameNow) -> Step {
        if enabled && game.open && !game.paused && !game.menu_open && !self.ours {
            self.ours = true;
            Step::Pause
        } else {
            Step::Nothing
        }
    }

    /// EmuDome is in front again: the game resumes if the pause was ours and the menu is not
    /// open. The game's own pause flag is not read here: it follows the command a moment late.
    pub fn back(&mut self, game: GameNow) -> Step {
        if std::mem::take(&mut self.ours) && game.open && !game.menu_open {
            Step::Resume
        } else {
            Step::Nothing
        }
    }

    /// The interface paused or resumed the game: the pause is the user's now.
    pub fn forget(&mut self) {
        self.ours = false;
    }
}

/// EmuDome went to the background (`focused` false) or came back.
pub fn focus_changed(app: &AppHandle, focused: bool) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if !focused {
        // Keys let go elsewhere never reach the page (#74).
        state.release_keyboard();
    }
    let shared = state.games.current().load_full();
    let game = GameNow {
        open: shared.is_some(),
        paused: shared.as_ref().is_some_and(|s| s.stats.lock().paused),
        menu_open: state.menu_open.load(std::sync::atomic::Ordering::Relaxed),
    };
    let step = if focused {
        state.background.lock().back(game)
    } else {
        let enabled = keys::Reader::new(crate::settings::resolver(app, &Scope::Global))
            .bool(keys::GENERAL_BACKGROUND_PAUSE);
        state.background.lock().lost(enabled, game)
    };
    let paused = match step {
        Step::Pause => true,
        Step::Resume => false,
        Step::Nothing => return,
    };
    if state.games.send(Command::SetPaused(paused)).is_err() {
        return;
    }
    log::info!(
        "{}: the game is {}",
        if focused {
            "in front"
        } else {
            "in the background"
        },
        if paused { "paused" } else { "resumed" }
    );
    // The interface follows as for the pause shortcut (its watchdog does not take a pause for
    // a hang).
    if let Err(e) = app.emit(crate::events::HOTKEY, HotkeyEvent::Paused { paused }) {
        log::warn!("pause not sent to the interface: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{BackgroundPause, GameNow, Step};

    const PLAYING: GameNow = GameNow {
        open: true,
        paused: false,
        menu_open: false,
    };

    #[test]
    fn a_running_game_pauses_in_the_background_and_resumes_in_front() {
        let mut bg = BackgroundPause::default();
        assert_eq!(bg.lost(true, PLAYING), Step::Pause);
        // A second «lost» (the page's blur and Android's pause both arrive) changes nothing.
        let paused = GameNow {
            paused: true,
            ..PLAYING
        };
        assert_eq!(bg.lost(true, paused), Step::Nothing);
        assert_eq!(bg.back(paused), Step::Resume);
        // Once.
        assert_eq!(bg.back(PLAYING), Step::Nothing);
    }

    #[test]
    fn it_resumes_even_before_the_game_says_it_paused() {
        // The pause command is carried out by the emulation thread a moment later.
        let mut bg = BackgroundPause::default();
        assert_eq!(bg.lost(true, PLAYING), Step::Pause);
        assert_eq!(bg.back(PLAYING), Step::Resume);
    }

    #[test]
    fn the_users_pause_stays() {
        let mut bg = BackgroundPause::default();
        // Paused already (the resume question, the pause shortcut): not ours.
        let paused = GameNow {
            paused: true,
            ..PLAYING
        };
        assert_eq!(bg.lost(true, paused), Step::Nothing);
        assert_eq!(bg.back(paused), Step::Nothing);
        // The quick menu open: it paused the game itself.
        let menu = GameNow {
            menu_open: true,
            ..paused
        };
        assert_eq!(bg.lost(true, menu), Step::Nothing);
        assert_eq!(bg.back(menu), Step::Nothing);
        // Ours, then the interface paused or resumed meanwhile: the pause is the user's.
        assert_eq!(bg.lost(true, PLAYING), Step::Pause);
        bg.forget();
        assert_eq!(bg.back(paused), Step::Nothing);
        // Ours, then the menu opened before EmuDome came back: the menu keeps it.
        assert_eq!(bg.lost(true, PLAYING), Step::Pause);
        assert_eq!(bg.back(menu), Step::Nothing);
    }

    #[test]
    fn nothing_happens_with_the_setting_off_or_no_game() {
        let mut bg = BackgroundPause::default();
        assert_eq!(bg.lost(false, PLAYING), Step::Nothing);
        assert_eq!(bg.back(PLAYING), Step::Nothing);
        let none = GameNow {
            open: false,
            ..PLAYING
        };
        assert_eq!(bg.lost(true, none), Step::Nothing);
        assert_eq!(bg.back(none), Step::Nothing);
    }
}
