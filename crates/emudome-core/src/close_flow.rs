//! Closing a game whose exit writes must reach the disk before anything else happens (Trello
//! #86): the window's X, quitting, closing the game to the library, opening another one.
//!
//! A pure state machine: [`CloseFlow::step`] takes what happened and returns what to do. The
//! shell (`app/src-tauri/src/closing.rs`) carries the actions out: it stops the session, waits
//! for its thread on a worker (never in the event loop), tells the interface each [`Progress`]
//! (`emu://closing`) and exits the app or lets the waiting command return.
//!
//! ```text
//! Idle ──request──▶ Saving ──written──▶ Saved ──after SAVED_VISIBLE──▶ finish (exit / back)
//!                    │  ▲                 ▲
//!          timed out │  │ wait            │ written (late)
//!                    ▼  │                 │
//!                   Stuck ────────────────┘
//!                    │ close anyway: let the session go, finish
//! Saving / Stuck ──failed──▶ Failed ──close anyway──▶ finish
//! ```
//!
//! A request while a close runs starts nothing: the X pressed twice is ignored, and a quit
//! arriving while a game closes to the library turns that close into a quit.

use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

use crate::ErrorCode;

/// How long «Saved» stays on screen before the app exits (or the game goes), so it is read.
pub const SAVED_VISIBLE: Duration = Duration::from_millis(600);

/// What the close is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "ClosingPurpose")]
pub enum Purpose {
    /// The game closes and the app stays: back to the library, or another game opens.
    Game,
    /// The window closes or the app quits: it exits once the game is closed.
    Quit,
}

/// What the interface shows while a game closes (`emu://closing`'s `phase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "ClosingPhase")]
pub enum Phase {
    /// The game is writing its saves: «Saving, don't turn off the device».
    Saving,
    /// Everything was written: «Saved», shortly before the app exits.
    Saved,
    /// The saves are taking longer than [`crate::CLOSE_TIMEOUT`]: wait, or close anyway.
    Timeout,
    /// A save could not be written (`error`): close anyway, the latest progress is lost.
    Failed,
    /// The close is over (the game is gone; with [`Purpose::Quit`] the app exits now).
    Closed,
}

/// One step of the close for the interface: the payload of `emu://closing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "ClosingProgress")]
pub struct Progress {
    pub phase: Phase,
    pub purpose: Purpose,
    /// What failed, with [`Phase::Failed`].
    pub error: Option<ErrorCode>,
}

/// What happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    /// The window's X, a quit, closing the game or opening another one. `game_open`: whether a
    /// game is open now (nothing to wait for otherwise).
    Request { purpose: Purpose, game_open: bool },
    /// The game's thread ended with every exit write done.
    Written,
    /// The game's thread ended, and this write failed.
    Failed(ErrorCode),
    /// The thread is still writing after [`crate::CLOSE_TIMEOUT`].
    TimedOut,
    /// «Saved» has been on screen for [`SAVED_VISIBLE`].
    SavedShown,
    /// The user chose «Wait».
    Wait,
    /// The user chose «Close anyway».
    CloseAnyway,
}

/// What to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The request needs nothing from the flow: let the window close or the app exit, or return.
    Proceed,
    /// Tell the interface (`emu://closing`).
    Show(Progress),
    /// Take the open game, ask its thread to stop and wait for it on a worker.
    StartClose,
    /// Wait up to [`crate::CLOSE_TIMEOUT`] again before the next [`Input::TimedOut`].
    WaitAgain,
    /// Feed [`Input::SavedShown`] after [`SAVED_VISIBLE`].
    HoldSaved,
    /// Let the session's thread go: it writes nothing more, its core is refused until restart.
    Abandon,
    /// The close is over: [`Purpose::Quit`] exits the app, [`Purpose::Game`] lets the commands
    /// waiting for it return.
    Finish(Purpose),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Idle,
    Saving(Purpose),
    Saved(Purpose),
    Stuck(Purpose),
    Failed(Purpose, ErrorCode),
    /// The app is exiting: every further request lets it.
    Exiting,
}

/// The close flow (module docs). Starts idle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseFlow {
    state: State,
}

impl Default for CloseFlow {
    fn default() -> Self {
        Self::new()
    }
}

fn show(phase: Phase, purpose: Purpose, error: Option<ErrorCode>) -> Action {
    Action::Show(Progress {
        phase,
        purpose,
        error,
    })
}

impl CloseFlow {
    pub fn new() -> Self {
        Self { state: State::Idle }
    }

    /// Whether a close is running (requested, not finished): commands that need the game closed
    /// wait while it is.
    pub fn busy(&self) -> bool {
        match self.state {
            State::Saving(_) | State::Saved(_) | State::Stuck(_) | State::Failed(..) => true,
            State::Idle | State::Exiting => false,
        }
    }

    /// Whether the app is exiting: a game must not open any more.
    pub fn exiting(&self) -> bool {
        self.state == State::Exiting
    }

    /// One step: the new state is kept, the actions are returned in the order to carry them out.
    pub fn step(&mut self, input: Input) -> Vec<Action> {
        let state = std::mem::replace(&mut self.state, State::Idle);
        let (next, actions) = transition(state, input);
        self.state = next;
        actions
    }
}

/// The state after a close ends.
fn ended(purpose: Purpose) -> State {
    match purpose {
        Purpose::Game => State::Idle,
        Purpose::Quit => State::Exiting,
    }
}

fn finish(purpose: Purpose) -> Vec<Action> {
    vec![show(Phase::Closed, purpose, None), Action::Finish(purpose)]
}

fn transition(state: State, input: Input) -> (State, Vec<Action>) {
    match (state, input) {
        // A request with nothing to close, or while the app exits: let it through.
        (
            State::Idle,
            Input::Request {
                game_open: false, ..
            },
        ) => (State::Idle, vec![Action::Proceed]),
        (State::Exiting, Input::Request { .. }) => (State::Exiting, vec![Action::Proceed]),
        (
            State::Idle,
            Input::Request {
                purpose,
                game_open: true,
            },
        ) => (
            State::Saving(purpose),
            vec![show(Phase::Saving, purpose, None), Action::StartClose],
        ),

        // A quit while a game closes to the library: the same close, then the app exits.
        (
            State::Saving(Purpose::Game),
            Input::Request {
                purpose: Purpose::Quit,
                ..
            },
        ) => (
            State::Saving(Purpose::Quit),
            vec![show(Phase::Saving, Purpose::Quit, None)],
        ),
        (
            State::Saved(Purpose::Game),
            Input::Request {
                purpose: Purpose::Quit,
                ..
            },
        ) => (
            State::Saved(Purpose::Quit),
            vec![show(Phase::Saved, Purpose::Quit, None)],
        ),
        (
            State::Stuck(Purpose::Game),
            Input::Request {
                purpose: Purpose::Quit,
                ..
            },
        ) => (
            State::Stuck(Purpose::Quit),
            vec![show(Phase::Timeout, Purpose::Quit, None)],
        ),
        (
            State::Failed(Purpose::Game, error),
            Input::Request {
                purpose: Purpose::Quit,
                ..
            },
        ) => (
            State::Failed(Purpose::Quit, error.clone()),
            vec![show(Phase::Failed, Purpose::Quit, Some(error))],
        ),

        // The thread ended (late, from Stuck, too).
        (State::Saving(purpose) | State::Stuck(purpose), Input::Written) => (
            State::Saved(purpose),
            vec![show(Phase::Saved, purpose, None), Action::HoldSaved],
        ),
        (State::Saving(purpose) | State::Stuck(purpose), Input::Failed(error)) => (
            State::Failed(purpose, error.clone()),
            vec![show(Phase::Failed, purpose, Some(error))],
        ),
        (State::Saving(purpose), Input::TimedOut) => (
            State::Stuck(purpose),
            vec![show(Phase::Timeout, purpose, None)],
        ),
        (State::Saved(purpose), Input::SavedShown) => (ended(purpose), finish(purpose)),

        // The user's choice.
        (State::Stuck(purpose), Input::Wait) => (
            State::Saving(purpose),
            vec![show(Phase::Saving, purpose, None), Action::WaitAgain],
        ),
        (State::Stuck(purpose), Input::CloseAnyway) => {
            let mut actions = vec![Action::Abandon];
            actions.extend(finish(purpose));
            (ended(purpose), actions)
        }
        (State::Failed(purpose, _), Input::CloseAnyway) => (ended(purpose), finish(purpose)),

        // Anything else changes nothing: a second request while saving (the X pressed twice),
        // a choice with no question asked, a late timer.
        (state, _) => (state, Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, CloseFlow, Input, Phase, Progress, Purpose};
    use crate::ErrorCode;

    fn shown(phase: Phase, purpose: Purpose) -> Action {
        Action::Show(Progress {
            phase,
            purpose,
            error: None,
        })
    }

    fn request(purpose: Purpose) -> Input {
        Input::Request {
            purpose,
            game_open: true,
        }
    }

    fn disk_full() -> ErrorCode {
        ErrorCode::SaveFileFailed {
            cause: crate::IoCause::Other,
        }
    }

    /// A flow already saving for `purpose`.
    fn saving(purpose: Purpose) -> CloseFlow {
        let mut flow = CloseFlow::new();
        flow.step(request(purpose));
        flow
    }

    #[test]
    fn with_no_game_open_the_app_closes_at_once() {
        let mut flow = CloseFlow::new();
        let actions = flow.step(Input::Request {
            purpose: Purpose::Quit,
            game_open: false,
        });
        assert_eq!(actions, vec![Action::Proceed]);
        assert!(!flow.busy());
    }

    #[test]
    fn a_quit_saves_shows_saved_then_exits() {
        let mut flow = CloseFlow::new();
        assert_eq!(
            flow.step(request(Purpose::Quit)),
            vec![shown(Phase::Saving, Purpose::Quit), Action::StartClose]
        );
        assert!(flow.busy());
        assert_eq!(
            flow.step(Input::Written),
            vec![shown(Phase::Saved, Purpose::Quit), Action::HoldSaved]
        );
        assert_eq!(
            flow.step(Input::SavedShown),
            vec![
                shown(Phase::Closed, Purpose::Quit),
                Action::Finish(Purpose::Quit)
            ]
        );
        assert!(flow.exiting());
        // The exit that follows (app.exit → ExitRequested, then Exit) goes through.
        assert_eq!(flow.step(request(Purpose::Quit)), vec![Action::Proceed]);
    }

    #[test]
    fn a_second_request_while_saving_starts_nothing() {
        let mut flow = saving(Purpose::Quit);
        assert_eq!(flow.step(request(Purpose::Quit)), Vec::new());
        assert_eq!(flow.step(request(Purpose::Game)), Vec::new());
        assert!(flow.busy());
        // Still the one close: its end goes on as before.
        assert_eq!(
            flow.step(Input::Written),
            vec![shown(Phase::Saved, Purpose::Quit), Action::HoldSaved]
        );
        assert_eq!(flow.step(request(Purpose::Quit)), Vec::new());
    }

    #[test]
    fn a_timeout_asks_and_wait_goes_back_to_saving() {
        let mut flow = saving(Purpose::Quit);
        assert_eq!(
            flow.step(Input::TimedOut),
            vec![shown(Phase::Timeout, Purpose::Quit)]
        );
        // A second timeout while the question is up changes nothing.
        assert_eq!(flow.step(Input::TimedOut), Vec::new());
        assert_eq!(
            flow.step(Input::Wait),
            vec![shown(Phase::Saving, Purpose::Quit), Action::WaitAgain]
        );
        assert_eq!(
            flow.step(Input::Written),
            vec![shown(Phase::Saved, Purpose::Quit), Action::HoldSaved]
        );
    }

    #[test]
    fn close_anyway_after_a_timeout_lets_the_game_go_and_exits() {
        let mut flow = saving(Purpose::Quit);
        flow.step(Input::TimedOut);
        assert_eq!(
            flow.step(Input::CloseAnyway),
            vec![
                Action::Abandon,
                shown(Phase::Closed, Purpose::Quit),
                Action::Finish(Purpose::Quit)
            ]
        );
        assert!(flow.exiting());
    }

    #[test]
    fn writes_ending_while_the_question_is_up_still_show_saved() {
        let mut flow = saving(Purpose::Quit);
        flow.step(Input::TimedOut);
        assert_eq!(
            flow.step(Input::Written),
            vec![shown(Phase::Saved, Purpose::Quit), Action::HoldSaved]
        );
        // The question is gone: a late «Close anyway» does not let anything go.
        assert_eq!(flow.step(Input::CloseAnyway), Vec::new());
    }

    #[test]
    fn a_failed_write_asks_and_only_close_anyway_ends_it() {
        let mut flow = saving(Purpose::Quit);
        assert_eq!(
            flow.step(Input::Failed(disk_full())),
            vec![Action::Show(Progress {
                phase: Phase::Failed,
                purpose: Purpose::Quit,
                error: Some(disk_full()),
            })]
        );
        // Nothing is left to wait for.
        assert_eq!(flow.step(Input::Wait), Vec::new());
        assert_eq!(flow.step(request(Purpose::Quit)), Vec::new());
        assert_eq!(
            flow.step(Input::CloseAnyway),
            vec![
                shown(Phase::Closed, Purpose::Quit),
                Action::Finish(Purpose::Quit)
            ]
        );
    }

    #[test]
    fn closing_a_game_ends_idle_and_can_close_the_next_one() {
        let mut flow = saving(Purpose::Game);
        flow.step(Input::Written);
        assert_eq!(
            flow.step(Input::SavedShown),
            vec![
                shown(Phase::Closed, Purpose::Game),
                Action::Finish(Purpose::Game)
            ]
        );
        assert!(!flow.busy());
        assert!(!flow.exiting());
        assert_eq!(
            flow.step(request(Purpose::Game)),
            vec![shown(Phase::Saving, Purpose::Game), Action::StartClose]
        );
    }

    #[test]
    fn a_quit_while_a_game_closes_turns_it_into_a_quit() {
        let mut flow = saving(Purpose::Game);
        assert_eq!(
            flow.step(request(Purpose::Quit)),
            vec![shown(Phase::Saving, Purpose::Quit)]
        );
        flow.step(Input::Written);
        assert_eq!(
            flow.step(Input::SavedShown),
            vec![
                shown(Phase::Closed, Purpose::Quit),
                Action::Finish(Purpose::Quit)
            ]
        );
        assert!(flow.exiting());

        // The same from the question and from a failure, which keeps its error.
        let mut stuck = saving(Purpose::Game);
        stuck.step(Input::TimedOut);
        assert_eq!(
            stuck.step(request(Purpose::Quit)),
            vec![shown(Phase::Timeout, Purpose::Quit)]
        );
        let mut failed = saving(Purpose::Game);
        failed.step(Input::Failed(disk_full()));
        assert_eq!(
            failed.step(request(Purpose::Quit)),
            vec![Action::Show(Progress {
                phase: Phase::Failed,
                purpose: Purpose::Quit,
                error: Some(disk_full()),
            })]
        );
    }

    #[test]
    fn choices_and_timers_with_nothing_running_change_nothing() {
        let mut flow = CloseFlow::new();
        for input in [
            Input::Wait,
            Input::CloseAnyway,
            Input::TimedOut,
            Input::SavedShown,
            Input::Written,
        ] {
            assert_eq!(flow.step(input), Vec::new());
        }
        assert_eq!(flow, CloseFlow::new());
    }

    #[test]
    fn progress_is_sent_as_codes() {
        let progress = Progress {
            phase: Phase::Timeout,
            purpose: Purpose::Quit,
            error: None,
        };
        assert_eq!(
            serde_json::to_value(progress).unwrap(),
            serde_json::json!({ "phase": "timeout", "purpose": "quit", "error": null })
        );
    }
}
