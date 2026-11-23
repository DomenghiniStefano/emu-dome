//! EmuDome updating itself (Trello #44): `emudome_core::updates::UpdateFlow` carried out.
//!
//! One thread, `emudome-updates`, does the slow steps (asking the release server, downloading,
//! starting the installer: `emudome_platform::updater`) one at a time, and wakes once a minute
//! for the automatic check of `general.updates`; the commands only change the flow and wake it.
//! Every change sends the whole `UpdateStatus` on `updates://changed`. A version ready to install
//! is also recorded as a notification (`Notice::UpdateReady`, once per version per run). X06
//! (the interface) shows it outside the game only; installing refuses while a game is open
//! (`updateGameOpen`): the restart never cuts a game's saves.

use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::Duration;

use emudome_core::notifications::Notice;
use emudome_core::settings::{keys, Scope};
use emudome_core::updates::{UpdateAction, UpdateFlow, UpdatePolicy, UpdateStatus};
use emudome_core::ErrorCode;
use emudome_platform::updater::{self, DownloadedUpdate, FoundUpdate, UpdateError};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::events;

/// How often the thread looks at the clock for the automatic check.
const TICK: Duration = Duration::from_secs(60);

struct Inner {
    flow: UpdateFlow,
    /// The version the server announced, and its download once done.
    found: Option<FoundUpdate>,
    downloaded: Option<DownloadedUpdate>,
    /// The version last recorded as a notification.
    noticed: Option<String>,
}

enum Msg {
    /// Look at the clock and the policy now.
    Wake,
    Run(UpdateAction),
}

/// The updater's state, managed by Tauri.
#[derive(Clone)]
pub struct Updates {
    inner: Arc<Mutex<Inner>>,
    tx: Sender<Msg>,
}

/// Starts the updater thread (once, in setup) and returns its state.
pub fn start(app: &AppHandle) -> Updates {
    let flow = UpdateFlow::new(updater::CONFIGURED, policy(app));
    let (tx, rx) = channel();
    let updates = Updates {
        inner: Arc::new(Mutex::new(Inner {
            flow,
            found: None,
            downloaded: None,
            noticed: None,
        })),
        tx,
    };
    if updater::CONFIGURED {
        let (u, a) = (updates.clone(), app.clone());
        if let Err(e) = std::thread::Builder::new()
            .name("emudome-updates".into())
            .spawn(move || run(&a, &u, &rx))
        {
            log::error!("updates thread not started: {e}");
        }
    } else {
        log::info!("updates: this build cannot update itself (no update key)");
    }
    updates
}

/// `general.updates` as this device resolves it.
fn policy(app: &AppHandle) -> UpdatePolicy {
    let id = keys::Reader::new(crate::settings::resolver(app, &Scope::Global))
        .text(keys::GENERAL_UPDATES);
    UpdatePolicy::from_id(&id).unwrap_or_default()
}

/// A setting changed: the thread reads `general.updates` again.
pub fn settings_changed(app: &AppHandle, changed: &[&str]) {
    if changed.contains(&keys::GENERAL_UPDATES.as_str()) {
        if let Some(u) = app.try_state::<Updates>() {
            u.wake(Msg::Wake);
        }
    }
}

fn now_ms() -> u64 {
    crate::unix_ms(std::time::SystemTime::now()).unwrap_or(0)
}

impl Updates {
    fn wake(&self, msg: Msg) {
        // The thread is gone only in a build that cannot update: the flow already says so.
        if self.tx.send(msg).is_err() {
            log::debug!("updates thread not running");
        }
    }

    fn status(&self) -> UpdateStatus {
        self.inner.lock().flow.status()
    }

    /// Changes the flow, tells the interface, and hands the action it returns to the thread.
    fn change(
        &self,
        app: &AppHandle,
        f: impl FnOnce(&mut Inner) -> Result<Option<UpdateAction>, ErrorCode>,
    ) -> Result<UpdateStatus, ErrorCode> {
        let action = f(&mut self.inner.lock())?;
        let status = self.changed(app);
        if let Some(action) = action {
            self.wake(Msg::Run(action));
        }
        Ok(status)
    }

    /// Sends the status, and records a newly offered version as a notification.
    fn changed(&self, app: &AppHandle) -> UpdateStatus {
        let (status, notice) = {
            let mut inner = self.inner.lock();
            let offered = inner.flow.offered().map(str::to_owned);
            let notice = offered.filter(|v| inner.noticed.as_ref() != Some(v));
            if let Some(v) = &notice {
                inner.noticed = Some(v.clone());
            }
            (inner.flow.status(), notice)
        };
        if let Some(version) = notice {
            crate::notifications::record(app, Notice::UpdateReady { version });
        }
        if let Err(e) = app.emit(events::UPDATES_CHANGED, status.clone()) {
            log::warn!("{} not delivered: {e}", events::UPDATES_CHANGED);
        }
        status
    }
}

fn run(app: &AppHandle, updates: &Updates, rx: &Receiver<Msg>) {
    loop {
        let due = {
            let mut inner = updates.inner.lock();
            inner.flow.set_policy(policy(app));
            inner.flow.tick(now_ms())
        };
        if due.is_some() {
            updates.changed(app);
        }
        carry_out_all(app, updates, due);
        match rx.recv_timeout(TICK) {
            Ok(Msg::Wake) | Err(RecvTimeoutError::Timeout) => {}
            Ok(Msg::Run(action)) => carry_out_all(app, updates, Some(action)),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// `first`, then whatever the flow asks after it, telling the interface after each step.
fn carry_out_all(app: &AppHandle, updates: &Updates, first: Option<UpdateAction>) {
    let mut next = first;
    while let Some(action) = next.take() {
        next = carry_out(app, updates, action);
        updates.changed(app);
    }
}

/// The code of a step that failed: no updater as itself, any other failure as `failed` (its
/// text is in the log, written by the caller).
fn code_of(e: UpdateError, failed: ErrorCode) -> ErrorCode {
    match e {
        UpdateError::Unavailable => ErrorCode::UpdateUnavailable,
        UpdateError::Failed(_) => failed,
    }
}

/// One slow step, on the updater thread; returns what the flow asks next.
fn carry_out(app: &AppHandle, updates: &Updates, action: UpdateAction) -> Option<UpdateAction> {
    match action {
        UpdateAction::Check => {
            let result = updater::check_update(app);
            let mut inner = updates.inner.lock();
            let found = match result {
                Ok(found) => {
                    let version = found.as_ref().map(|f| f.version().to_owned());
                    if let Some(v) = &version {
                        log::info!("update: version {v} is available");
                    }
                    inner.found = found;
                    inner.downloaded = None;
                    Ok(version)
                }
                Err(e) => {
                    log::warn!("update check failed: {e:?}");
                    Err(code_of(e, ErrorCode::UpdateCheckFailed))
                }
            };
            inner.flow.checked(found, now_ms())
        }
        UpdateAction::Download { version } => {
            // The download runs without the lock: the commands still answer meanwhile.
            let found = updates.inner.lock().found.take();
            let Some(found) = found else {
                return updates
                    .inner
                    .lock()
                    .flow
                    .downloaded(version, Err(ErrorCode::UpdateNotReady));
            };
            let result = updater::download_update(&found);
            let mut inner = updates.inner.lock();
            inner.found = Some(found);
            let result = match result {
                Ok(bytes) => {
                    log::info!("update: version {version} downloaded and verified");
                    inner.downloaded = Some(bytes);
                    Ok(())
                }
                Err(e) => {
                    log::warn!("update download failed: {e:?}");
                    Err(code_of(e, ErrorCode::UpdateDownloadFailed))
                }
            };
            inner.flow.downloaded(version, result)
        }
        UpdateAction::Install { version } => {
            let game_open = app
                .try_state::<crate::AppState>()
                .is_some_and(|s| s.games.is_open());
            let mut inner = updates.inner.lock();
            let error = if game_open {
                ErrorCode::UpdateGameOpen
            } else {
                match (&inner.found, &inner.downloaded) {
                    (Some(found), Some(bytes)) => {
                        log::info!("update: installing version {version}, EmuDome restarts");
                        // Returns only when it failed (Windows exits for the installer).
                        let e = match updater::install_update(app, found, bytes) {
                            Ok(()) => return None,
                            Err(e) => e,
                        };
                        log::warn!("update install failed: {e:?}");
                        code_of(e, ErrorCode::UpdateInstallFailed)
                    }
                    _ => ErrorCode::UpdateNotReady,
                }
            };
            inner.flow.install_failed(version, error.clone());
            drop(inner);
            if let Err(e) = app.emit(events::SESSION, emudome_core::Event::Error { error }) {
                log::warn!("update error not delivered: {e}");
            }
            None
        }
    }
}

/// Where the update stands (`about.upd`, X06).
#[tauri::command]
pub fn updates_status(state: tauri::State<'_, Updates>) -> UpdateStatus {
    state.status()
}

/// «Controlla ora»: one check now, whatever `general.updates` says.
#[tauri::command]
pub fn updates_check(
    app: AppHandle,
    state: tauri::State<'_, Updates>,
) -> Result<UpdateStatus, ErrorCode> {
    state.change(&app, |inner| inner.flow.check_now(now_ms()))
}

/// X06 «Più tardi»: the card hides for this version until EmuDome starts again.
#[tauri::command]
pub fn updates_later(app: AppHandle, state: tauri::State<'_, Updates>) -> UpdateStatus {
    state
        .change(&app, |inner| {
            inner.flow.later();
            Ok(None)
        })
        .unwrap_or_else(|_| state.status())
}

/// X06 «Riavvia e aggiorna»: installs the downloaded version (downloads it first with «ask
/// first») and restarts. Refused while a game is open.
#[tauri::command]
pub fn updates_install(
    app: AppHandle,
    state: tauri::State<'_, Updates>,
    games: tauri::State<'_, crate::AppState>,
) -> Result<UpdateStatus, ErrorCode> {
    if games.games.is_open() {
        return Err(ErrorCode::UpdateGameOpen);
    }
    state.change(&app, |inner| inner.flow.install())
}
