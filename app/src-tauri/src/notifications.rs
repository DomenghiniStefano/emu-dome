//! Tauri side of the notification centre (X03, Trello #32, `emudome_core::notifications`): the
//! store on disk, its commands, and `notifications://changed` with the whole list after every
//! change.
//!
//! The file is `<app local data>/notifications.json`: per device, never synced, like
//! `local-settings/device.json`. Recording happens off the thread that saw the event (the
//! controllers' thread must not wait for the disk); the commands run on a blocking thread.
//! What records a notification today: a controller connected while EmuDome runs (gamepad.rs).
//! Sync done, cover not found and update ready have no event yet.

use std::sync::Arc;

use emudome_core::notifications::{Notice, Notification, Store};
use emudome_core::ErrorCode;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::events;

/// The store, shared by the commands and the recorders.
#[derive(Clone)]
pub struct Notifications(Arc<Mutex<Option<Store>>>);

/// Opens the store (no data folder: nothing is kept, every call answers an empty list).
pub fn open(app: &AppHandle) -> Notifications {
    let store = crate::dirs::local_or_log(app, "notifications")
        .map(|dir| Store::open(dir.join("notifications.json")));
    Notifications(Arc::new(Mutex::new(store)))
}

impl Notifications {
    /// Runs `f` on the store; without a data folder, `none` is the answer.
    fn with<R>(
        &self,
        none: R,
        f: impl FnOnce(&mut Store) -> Result<R, ErrorCode>,
    ) -> Result<R, ErrorCode> {
        match self.0.lock().as_mut() {
            Some(store) => f(store),
            None => Ok(none),
        }
    }

    fn list(&self) -> Result<Vec<Notification>, ErrorCode> {
        self.with(Vec::new(), Store::list)
    }
}

/// The whole list, newest first, to the interface.
fn changed(app: &AppHandle, state: &Notifications) {
    match state.list() {
        Ok(list) => {
            if let Err(e) = app.emit(events::NOTIFICATIONS_CHANGED, list) {
                log::warn!("{} not delivered: {e}", events::NOTIFICATIONS_CHANGED);
            }
        }
        Err(e) => log::warn!("notifications not listed: {e:?}"),
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Records `notice` on a blocking thread and tells the interface. A failure is logged: the
/// short toast the interface showed for the same event already told the user.
#[allow(dead_code, reason = "called by gamepad.rs, desktop only (SDL3)")]
pub fn record(app: &AppHandle, notice: Notice) {
    let Some(state) = app.try_state::<Notifications>().map(|s| s.inner().clone()) else {
        return;
    };
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        match state.with(None, |s| s.record(notice, now_ms()).map(Some)) {
            Ok(_) => changed(&app, &state),
            Err(e) => log::warn!("notification not recorded: {e:?}"),
        }
    });
}

/// Every notification, newest first.
#[tauri::command]
pub async fn notifications_list(app: AppHandle) -> Result<Vec<Notification>, ErrorCode> {
    crate::blocking(app, |app| app.state::<Notifications>().list()).await?
}

/// Marks one read.
#[tauri::command]
pub async fn notifications_mark_read(app: AppHandle, id: u64) -> Result<(), ErrorCode> {
    crate::blocking(app, move |app| {
        let state: State<Notifications> = app.state();
        if state.with(false, |s| s.mark_read(id))? {
            changed(app, &state);
        }
        Ok(())
    })
    .await?
}

/// «Segna tutte come lette».
#[tauri::command]
pub async fn notifications_mark_all_read(app: AppHandle) -> Result<(), ErrorCode> {
    crate::blocking(app, |app| {
        let state: State<Notifications> = app.state();
        if state.with(false, Store::mark_all_read)? {
            changed(app, &state);
        }
        Ok(())
    })
    .await?
}
