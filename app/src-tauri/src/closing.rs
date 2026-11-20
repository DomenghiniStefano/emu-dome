//! The close flow carried out (Trello #86): closing the window, quitting, closing the game and
//! opening another one wait for the open game's exit writes on a worker thread, never in the
//! event loop, and tell the interface each step on `emu://closing` (`close_flow::Progress`).
//!
//! The decisions are `emudome_core::close_flow` (pure, tested there); this module only does what
//! it returns. Locks: `flow` may be held while `session` is taken, never the other way round
//! (the worker drops `session` before feeding the flow).

use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use emudome_core::close_flow::{Action, CloseFlow, Input, Purpose, SAVED_VISIBLE};
use emudome_core::{Closed, ErrorCode, Session, CLOSE_TIMEOUT};
use parking_lot::{Condvar, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;

/// How often the worker looks at the game's thread between its timeouts: also the longest
/// «Close anyway» waits for the worker to hand the session over.
const POLL: Duration = Duration::from_millis(100);

/// The game being closed, held here while its thread is waited for.
struct Closing {
    session: Session,
    /// Its core's name, for `coreNeedsRestart` if it is let go.
    core: &'static str,
}

/// The close flow's state in the app (a field of `AppState`).
pub struct Closer {
    flow: Mutex<CloseFlow>,
    /// Woken when a close finishes: the commands waiting for the game to be closed.
    finished: Condvar,
    /// The session whose thread the worker waits for; taken by «Close anyway» and by the last
    /// resort on `RunEvent::Exit`.
    closing: Mutex<Option<Closing>>,
    /// When the current wait began (`WaitAgain` starts it again).
    wait_from: Mutex<Instant>,
    /// The exit code asked for (`app.exit(code)`, the bench's), used once the game is closed.
    exit_code: AtomicI32,
}

impl Default for Closer {
    fn default() -> Self {
        Self {
            flow: Mutex::new(CloseFlow::new()),
            finished: Condvar::new(),
            closing: Mutex::new(None),
            wait_from: Mutex::new(Instant::now()),
            exit_code: AtomicI32::new(0),
        }
    }
}

/// The window's X or a quit (`ExitRequested`, with its exit code if any). Returns whether the
/// event may go on; otherwise the caller prevents it and the app exits when the game is closed.
pub fn request_quit(app: &AppHandle, state: &AppState, code: Option<i32>) -> bool {
    if let Some(code) = code {
        state.closer.exit_code.store(code, Ordering::SeqCst);
    }
    let game_open = state.session.lock().is_some();
    feed(
        app,
        state,
        Input::Request {
            purpose: Purpose::Quit,
            game_open,
        },
    )
}

/// Closes the open game (back to the library, or before opening another one) and waits until
/// it is closed, the user's «Wait» / «Close anyway» included. `Err(AppClosing)`: the app is
/// exiting meanwhile. Blocks: never call it from the event loop's thread (the commands that
/// do are `async`).
pub fn close_game(app: &AppHandle, state: &AppState) -> Result<(), ErrorCode> {
    let game_open = state.session.lock().is_some();
    feed(
        app,
        state,
        Input::Request {
            purpose: Purpose::Game,
            game_open,
        },
    );
    let mut flow = state.closer.flow.lock();
    while flow.busy() {
        state.closer.finished.wait(&mut flow);
    }
    if flow.exiting() {
        return Err(ErrorCode::AppClosing);
    }
    Ok(())
}

/// «Wait», after a timeout.
#[tauri::command]
pub fn close_wait(app: AppHandle, state: tauri::State<'_, AppState>) {
    feed(&app, &state, Input::Wait);
}

/// «Close anyway», after a timeout (the game is let go) or a failed write.
#[tauri::command]
pub fn close_anyway(app: AppHandle, state: tauri::State<'_, AppState>) {
    feed(&app, &state, Input::CloseAnyway);
}

/// `RunEvent::Exit`, which cannot be prevented: whatever is still open closes now, in the event
/// loop, bounded by `CLOSE_TIMEOUT` (the last resort; every other way out went through the flow).
pub fn close_now(app: &AppHandle, state: &AppState) {
    crate::close_current(app, state);
    let closing = state.closer.closing.lock().take();
    if let Some(closing) = closing {
        if closing.session.close(CLOSE_TIMEOUT) == Closed::Abandoned {
            log::warn!("the closing game did not end within {CLOSE_TIMEOUT:?}: let go");
        }
        state.game_path.lock().take();
    }
}

/// One step of the flow and its actions. Returns whether a request may go on (`Proceed`).
fn feed(app: &AppHandle, state: &AppState, input: Input) -> bool {
    let closer = &state.closer;
    let mut flow = closer.flow.lock();
    let mut proceed = false;
    for action in flow.step(input) {
        match action {
            Action::Proceed => proceed = true,
            Action::Show(progress) => {
                if let Err(e) = app.emit("emu://closing", &progress) {
                    log::warn!("closing progress not sent to the interface: {e}");
                }
            }
            Action::StartClose => start(app, state),
            Action::WaitAgain => *closer.wait_from.lock() = Instant::now(),
            Action::HoldSaved => after(app, SAVED_VISIBLE, Input::SavedShown),
            Action::Abandon => abandon(app, state),
            Action::Finish(purpose) => {
                closer.finished.notify_all();
                match purpose {
                    Purpose::Quit => app.exit(closer.exit_code.load(Ordering::SeqCst)),
                    Purpose::Game => {}
                }
            }
        }
    }
    proceed
}

/// Feeds `input` after `delay`, on a blocking thread of the async runtime.
fn after(app: &AppHandle, delay: Duration, input: Input) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(delay);
        feed(&app, &app.state::<AppState>(), input);
    });
}

/// Takes the open game, asks its thread to stop and waits for it on a worker.
fn start(app: &AppHandle, state: &AppState) {
    let (session, core) = crate::release_game(app, state);
    let Some(session) = session else {
        // Gone in between: nothing to wait for.
        state.game_path.lock().take();
        after(app, Duration::ZERO, Input::Written);
        return;
    };
    session.stop();
    *state.closer.wait_from.lock() = Instant::now();
    *state.closer.closing.lock() = Some(Closing { session, core });
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || worker(&app));
}

/// The worker: waits for the closing game's thread, `POLL` at a time, and reports a timeout
/// each time `CLOSE_TIMEOUT` passes since the wait began.
fn worker(app: &AppHandle) {
    let state = app.state::<AppState>();
    loop {
        let written = {
            let mut closing = state.closer.closing.lock();
            let Some(current) = closing.as_mut() else {
                // Let go («Close anyway») or closed by the last resort: nothing to report.
                return;
            };
            let written = current.session.wait(POLL);
            if written.is_some() {
                closing.take();
            }
            written
        };
        match written {
            Some(written) => {
                // Cleared after the thread ended: the Saves screen refuses this game's files
                // until then.
                state.game_path.lock().take();
                let input = match written {
                    Ok(()) => Input::Written,
                    Err(error) => {
                        log::warn!("the closing game's saves were not all written: {error:?}");
                        Input::Failed(error)
                    }
                };
                feed(app, &state, input);
                return;
            }
            None => {
                if state.closer.wait_from.lock().elapsed() >= CLOSE_TIMEOUT {
                    feed(app, &state, Input::TimedOut);
                }
            }
        }
    }
}

/// «Close anyway» after a timeout: the closing game's thread is let go (it writes nothing more,
/// its core is refused until EmuDome restarts).
fn abandon(app: &AppHandle, state: &AppState) {
    let closing = state.closer.closing.lock().take();
    if let Some(closing) = closing {
        log::warn!("closing abandoned by the user: the last progress may be lost");
        closing.session.abandon();
        state.game_path.lock().take();
        crate::say_core_needs_restart(app, closing.core);
    }
}
