//! The open game's screens, carried out (Trello #28). What the user does with them is pure and
//! tested in `emudome_core::screens` ([`ScreenSession`]: the layout shown, the screens swapped,
//! the lid); here it reaches the parts that act on it:
//!
//! - the native renderer draws the composed placements (`video::Renderer::set_layout`);
//! - the webview canvas asks for them ([`screens_placements`]) and draws what Rust composed;
//! - the touch screen maps through the same layout ([`layout`], the `pointer` command);
//! - the interface gets every change as a [`ScreenStatus`] on `emu://screens`;
//! - the lid is pressed on the core as its own chord (`core_profile::Lid`), never faked.
//!
//! The settings come from the open game's session («in prova» included), read again whenever a
//! `screens.*` setting changes, so the editor's live preview is a Session-level `screens.custom`.

use std::sync::Arc;

use emudome_core::core_profile::{CoreId, Lid, LID_PRESS};
use emudome_core::input::Source;
use emudome_core::screens::{
    CustomLayout, Layout, LayoutId, Placement, ScreenSession, ScreenSettings, ScreenStatus, Split,
};
use emudome_core::settings::{keys, ConsoleId, Scope, Value};
use emudome_core::{ErrorCode, Shared};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use crate::commands::PointerArea;
use crate::settings::SettingsState;
use crate::AppState;

/// The open game's screens; `None` with no game.
#[derive(Default)]
pub struct ScreensState(Mutex<Option<Open>>);

struct Open {
    core: CoreId,
    session: ScreenSession,
}

/// What the webview canvas draws (`screens_placements`): the frame those placements were
/// composed for, and each screen's rectangles. TypeScript: `ScreenPlacements` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub(crate) struct ScreenPlacements {
    /// The frame's size; another size (a new frame) needs the placements asked for again.
    frame_width: u32,
    frame_height: u32,
    /// Drawn in order: the last one in front.
    placements: Vec<Placement>,
}

/// The `screens.*` keys: a change to any of them is read again.
fn is_screen_key(key: &str) -> bool {
    key.starts_with("screens.")
}

/// The `screens.*` settings seen from the open game's session (else «Tutti i giochi»).
fn read_settings(app: &AppHandle) -> ScreenSettings {
    let session = app
        .try_state::<SettingsState>()
        .is_some_and(|s| s.0.lock().session_scope().is_some());
    let scope = if session {
        Scope::Session
    } else {
        Scope::Global
    };
    ScreenSettings::from_settings(crate::settings::resolver(app, &scope))
}

/// A game opened on `core` (its settings session already begun): its screens start from its
/// settings.
pub fn started(app: &AppHandle, core: CoreId, console: ConsoleId) {
    let profile = core.profile();
    let dual = profile.split != Split::Whole;
    log::debug!("screens: {console:?} on {core}, two screens: {dual}");
    let session = ScreenSession::new(read_settings(app), dual, profile.lid.supported());
    if let Some(state) = app.try_state::<ScreensState>() {
        *state.0.lock() = Some(Open { core, session });
    }
    publish(app);
}

/// The game closed.
pub fn stopped(app: &AppHandle) {
    if let Some(state) = app.try_state::<ScreensState>() {
        *state.0.lock() = None;
    }
    publish(app);
}

/// Settings changed: the open game follows when a `screens.*` one did.
pub fn settings_changed(app: &AppHandle, keys: &[&str]) {
    if !keys.iter().any(|k| is_screen_key(k)) {
        return;
    }
    let settings = read_settings(app);
    // No game: nothing to recompose (`None`).
    change(app, |s| {
        s.settings_changed(settings);
        true
    });
}

/// Applies `f` to the open game's screens and, when it changed something, sends the result to
/// the renderer and the interface. `None` with no game.
fn change<R>(app: &AppHandle, f: impl FnOnce(&mut ScreenSession) -> R) -> Option<R>
where
    R: Changed,
{
    let state = app.try_state::<ScreensState>()?;
    let result = {
        let mut open = state.0.lock();
        f(&mut open.as_mut()?.session)
    };
    if result.changed() {
        publish(app);
    }
    Some(result)
}

/// Whether a [`ScreenSession`] call changed something.
trait Changed {
    fn changed(&self) -> bool;
}

impl Changed for bool {
    fn changed(&self) -> bool {
        *self
    }
}

impl Changed for Option<bool> {
    fn changed(&self) -> bool {
        *self == Some(true)
    }
}

/// The layout and status now, sent to the native renderer and on `emu://screens`.
fn publish(app: &AppHandle) {
    let (layout, status) = current(app);
    if let Some(renderer) = app.try_state::<AppState>().and_then(|s| s.renderer.clone()) {
        renderer.set_layout(layout);
    }
    if let Err(e) = app.emit(crate::events::SCREENS, status) {
        log::warn!("emu://screens not delivered: {e}");
    }
}

/// The layout to compose and the status, for the open game (the defaults without one).
fn current(app: &AppHandle) -> (Layout, ScreenStatus) {
    app.try_state::<ScreensState>()
        .and_then(|s| {
            s.0.lock()
                .as_ref()
                .map(|o| (o.session.layout(), o.session.status()))
        })
        .unwrap_or_default()
}

/// The layout the touch screen maps through now (the `pointer` command).
pub fn layout(app: &AppHandle) -> Layout {
    current(app).0
}

/// The open game's shared state, for pressing the lid.
fn shared(app: &AppHandle) -> Option<Arc<Shared>> {
    app.try_state::<AppState>()?.games.current().load_full()
}

/// Presses the core's lid chord for [`LID_PRESS`], then lets it go (on a thread of its own: the
/// caller never waits).
fn press_lid(app: &AppHandle, lid: Lid) {
    let Lid::Chord { hold, press } = lid else {
        return;
    };
    let Some(shared) = shared(app) else {
        return;
    };
    shared.input.set(Source::Shell, hold, true);
    shared.input.set(Source::Shell, press, true);
    let spawned = std::thread::Builder::new()
        .name("emudome-lid".into())
        .spawn(move || {
            std::thread::sleep(LID_PRESS);
            shared.input.release(Source::Shell);
        });
    if let Err(e) = spawned {
        log::warn!("lid chord not released by its thread: {e}");
    }
}

/// Flips the lid of the open game; `Err(LidUnsupported)` when its core cannot.
fn toggle_lid(app: &AppHandle) -> Result<ScreenStatus, ErrorCode> {
    let state = app.try_state::<ScreensState>().ok_or(ErrorCode::NoGame)?;
    let lid = {
        let mut open = state.0.lock();
        let open = open.as_mut().ok_or(ErrorCode::NoGame)?;
        if !open.session.toggle_lid() {
            return Err(ErrorCode::LidUnsupported);
        }
        open.core.profile().lid
    };
    press_lid(app, lid);
    publish(app);
    Ok(current(app).1)
}

// ---- The shortcuts (`hotkeys.rs`): nothing with a one-screen game, as `Hotkey::status` says ----

/// L3 + Y went down (`pressed`) or up.
pub fn swap_hotkey(app: &AppHandle, pressed: bool) {
    change(app, |s| {
        if pressed {
            s.swap_pressed()
        } else {
            s.swap_released()
        }
    });
}

/// L3 + R3.
pub fn next_layout_hotkey(app: &AppHandle) {
    change(app, ScreenSession::next_layout);
}

/// L3 + Select. A core without a lid: logged, the interface already shows the lid disabled.
pub fn lid_hotkey(app: &AppHandle) {
    if let Err(e) = toggle_lid(app) {
        log::info!("lid shortcut ignored: {e:?}");
    }
}

// ---- Commands ----

/// The open game's screens now (the default status, `dualScreen` false, without a game).
#[tauri::command]
pub fn screens_status(app: AppHandle) -> ScreenStatus {
    current(&app).1
}

/// The next layout of the cycle (quick menu G09, like L3 + R3).
#[tauri::command]
pub fn screens_next_layout(app: AppHandle) -> Result<ScreenStatus, ErrorCode> {
    change(&app, ScreenSession::next_layout).ok_or(ErrorCode::NoGame)?;
    Ok(current(&app).1)
}

/// Shows `layout`, in the cycle or not (quick menu G09, the editor's preview of `custom`).
#[tauri::command]
pub fn screens_set_layout(app: AppHandle, layout: LayoutId) -> Result<ScreenStatus, ErrorCode> {
    change(&app, |s| s.set_layout(layout))
        .ok_or(ErrorCode::NoGame)?
        .ok_or_else(|| ErrorCode::ScreenLayoutUnavailable {
            layout: layout.id().to_owned(),
        })?;
    Ok(current(&app).1)
}

/// Swaps the screens (quick menu › Strumenti «Scambia schermi»): a toggle whatever
/// `screens.swap` says.
#[tauri::command]
pub fn screens_swap(app: AppHandle) -> Result<ScreenStatus, ErrorCode> {
    change(&app, ScreenSession::toggle_swap).ok_or(ErrorCode::NoGame)?;
    Ok(current(&app).1)
}

/// Closes or opens the lid (quick menu › Strumenti, like L3 + Select).
#[tauri::command]
pub fn screens_toggle_lid(app: AppHandle) -> Result<ScreenStatus, ErrorCode> {
    toggle_lid(&app)
}

/// The placements of the open game's frame in `area` (the webview canvas: CSS pixels of the
/// window, the menu's inset), composed by Rust as the native renderer composes them. Empty with
/// no game or no frame yet.
#[tauri::command]
pub fn screens_placements(app: AppHandle, area: PointerArea) -> ScreenPlacements {
    let frame = shared(&app).map(|s| {
        let f = s.frame.lock();
        (f.width, f.height, f.split)
    });
    let Some((width, height, split)) = frame.filter(|(w, h, _)| *w > 0 && *h > 0) else {
        return ScreenPlacements {
            frame_width: 0,
            frame_height: 0,
            placements: Vec::new(),
        };
    };
    let placements =
        emudome_core::screens::compose(&layout(&app), &split.screens(width, height), area.area());
    ScreenPlacements {
        frame_width: width,
        frame_height: height,
        placements,
    }
}

/// The custom layout (`screens.custom`) as `scope` resolves it: empty when none is saved there or
/// above. The C09 editor starts from it.
#[tauri::command]
pub fn screens_custom(app: AppHandle, scope: Scope) -> Result<CustomLayout, ErrorCode> {
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let resolved = state
        .0
        .lock()
        .resolve(keys::SCREENS_CUSTOM.as_str(), &scope)?;
    let items = match resolved.value {
        Some(Value::List(items)) => items,
        Some(Value::Bool(_) | Value::Int(_) | Value::Text(_)) | None => Vec::new(),
    };
    Ok(CustomLayout::from_items(&items).unwrap_or_default())
}

/// Stores the custom layout at `scope`: `console` or `game` to save it, `session` to preview it
/// live «in prova» (then committed or dropped with the quick menu's session, like any setting).
/// Removing it is `settings_reset` of `screens.custom`. An invalid layout is refused with
/// `settingInvalidValue`.
#[tauri::command]
pub fn screens_set_custom(
    app: AppHandle,
    scope: Scope,
    layout: CustomLayout,
) -> Result<(), ErrorCode> {
    let key = keys::SCREENS_CUSTOM.as_str();
    if !layout.is_valid() {
        return Err(ErrorCode::SettingInvalidValue {
            key: key.to_owned(),
        });
    }
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let level = state
        .0
        .lock()
        .set(key, &scope, Value::List(layout.to_items()))?;
    crate::settings::emit_changed(&app, &[key], level);
    Ok(())
}
