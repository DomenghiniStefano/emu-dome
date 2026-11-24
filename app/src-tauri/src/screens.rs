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
//! `screens.*` setting, `video.aspect` or `video.scale` changes, so the editor's live preview is a
//! Session-level `screens.custom`. `video.aspect` and `video.scale` go through the same layout,
//! so a one-screen game is stretched and sized alike in both renderers and in the touch mapping
//! (`video.scale`'s default is the device's: whole scales on a computer, as large as fits on a
//! phone).

use std::sync::Arc;

use emudome_core::core_profile::{CoreId, Lid, LID_PRESS};
use emudome_core::input::Source;
use emudome_core::screens::{
    Area, CustomLayout, CustomScreen, Inset, Layout, LayoutId, Placement, ScreenSession,
    ScreenSettings, ScreenStatus, Split,
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
    console: ConsoleId,
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

/// The `screens.*` keys, `video.aspect` and `video.scale`: a change to any of them is read again.
fn is_screen_key(key: &str) -> bool {
    key.starts_with("screens.")
        || key == keys::VIDEO_ASPECT.as_str()
        || key == keys::VIDEO_SCALE.as_str()
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
        *state.0.lock() = Some(Open {
            core,
            console,
            session,
        });
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
    if let Some(renderer) = app.try_state::<AppState>().and_then(|s| s.renderer.clone()) {
        renderer.set_layout(current(app).0);
    }
    emit_status(app);
}

/// The status now, on `emu://screens`.
fn emit_status(app: &AppHandle) {
    if let Err(e) = app.emit(crate::events::SCREENS, current(app).1) {
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

/// What L3 + R3 did to a two-screen game's layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cycled {
    /// The next layout of the cycle is shown.
    Moved,
    /// Nothing changed, nothing to say (a one-screen game, a cycle of one).
    Same,
    /// The phone's placement holds the screens: the cycle waits, and the interface says why.
    Held,
}

impl Changed for Cycled {
    fn changed(&self) -> bool {
        *self == Cycled::Moved
    }
}

/// L3 + R3 on `s`: the next layout of the cycle, unless the phone's placement holds the screens.
fn cycle(s: &mut ScreenSession) -> Cycled {
    let status = s.status();
    if status.dual_screen && status.phone_placement {
        return Cycled::Held;
    }
    if s.next_layout() {
        Cycled::Moved
    } else {
        Cycled::Same
    }
}

/// L3 + R3. True: the phone's placement holds the screens, so the cycle did nothing and the
/// interface is told why (`emu://hotkey` `layoutHeld`).
pub fn next_layout_hotkey(app: &AppHandle) -> bool {
    change(app, cycle) == Some(Cycled::Held)
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

/// The C09 editor's «Parti da»: where `layout` puts the screens of a `console` game in an area of
/// `width` × `height` (the whole window), as a custom layout, with the gap and the ratio `scope`
/// resolves. Composed by `emudome_core::screens` like the game itself, so the editor decides no
/// geometry. Empty for a one-screen console.
#[tauri::command]
pub fn screens_preset_custom(
    app: AppHandle,
    scope: Scope,
    console: ConsoleId,
    layout: LayoutId,
    width: f32,
    height: f32,
) -> CustomLayout {
    let settings = ScreenSettings::from_settings(crate::settings::resolver(&app, &scope));
    let split = CoreId::for_console(console).profile().split;
    CustomLayout::preset(&settings, layout, split, width, height)
}

/// Where the touch editor starts the screens of a `console` game in `area` when no placement of
/// the phone's own is saved. Over the open game (`scope` the session, the game of that console):
/// where it shows them now (`ScreenSession::phone_start`: the layout cycled to, swapped, the frame
/// at its real size). Otherwise with the settings `scope` resolves, as a game would open
/// (`emudome_core::screens::phone_start`).
#[tauri::command]
pub fn screens_phone_start(
    app: AppHandle,
    scope: Scope,
    console: ConsoleId,
    area: PointerArea,
) -> Vec<CustomScreen> {
    if matches!(scope, Scope::Session) {
        if let Some(start) = open_phone_start(&app, console, area.area()) {
            return start;
        }
    }
    let settings = ScreenSettings::from_settings(crate::settings::resolver(&app, &scope));
    emudome_core::screens::phone_start(&settings, console, area.area())
}

/// The open game's start for the touch editor; `None` with no game of `console` open. The frame's
/// screens as the core hands them over, the console's 1× frame before the first one.
fn open_phone_start(app: &AppHandle, console: ConsoleId, area: Area) -> Option<Vec<CustomScreen>> {
    // The frame read first, never under the screens' lock.
    let frame = shared(app)
        .map(|s| {
            let f = s.frame.lock();
            (f.width, f.height, f.split)
        })
        .filter(|(w, h, _)| *w > 0 && *h > 0);
    let state = app.try_state::<ScreensState>()?;
    let open = state.0.lock();
    let open = open.as_ref().filter(|o| o.console == console)?;
    let sources = match frame {
        Some((width, height, split)) => split.screens(width, height),
        None => {
            let (width, height) = emudome_core::screens::console_frame(console);
            open.core.profile().split.screens(width, height)
        }
    };
    Some(open.session.phone_start(&sources, area))
}

/// What one `screens_set_phone` call hands the native renderer, in one message: the layout when
/// the placement changed it, and the inset the placement is fitted in.
#[derive(Debug)]
struct PhoneView {
    layout: Option<Layout>,
    inset: Inset,
}

/// The phone's placement set on the open game's screens (`None`: no game), with `inset`. A
/// placement that is not the game's screens is refused whole: neither it nor its inset applies.
fn place_phone(
    session: Option<&mut ScreenSession>,
    screens: Vec<CustomScreen>,
    inset: Inset,
) -> Result<PhoneView, ErrorCode> {
    let Some(session) = session else {
        return Ok(PhoneView {
            layout: None,
            inset,
        });
    };
    let changed =
        session
            .set_phone_placement(screens)
            .ok_or_else(|| ErrorCode::SettingInvalidValue {
                key: keys::TOUCH_SCREENS.as_str().to_owned(),
            })?;
    Ok(PhoneView {
        layout: changed.then(|| session.layout()),
        inset,
    })
}

/// Sets (or, empty, withdraws) the phone's own placement of the open game's screens, and the
/// edges kept clear of the game it is fitted in (as `set_video_inset`): the touch layer's saved
/// one, or the editor's while it is open. The two reach the renderer as one message, so no frame
/// pairs the new placement with the old inset. No game: the inset only. A placement that is not
/// the game's screens is refused with `settingInvalidValue`, its inset with it.
#[tauri::command]
pub fn screens_set_phone(
    app: AppHandle,
    screens: Vec<CustomScreen>,
    inset: Inset,
) -> Result<ScreenStatus, ErrorCode> {
    let view = match app.try_state::<ScreensState>() {
        Some(state) => {
            let mut open = state.0.lock();
            place_phone(open.as_mut().map(|o| &mut o.session), screens, inset)?
        }
        None => place_phone(None, screens, inset)?,
    };
    let renderer = app.try_state::<AppState>().and_then(|s| s.renderer.clone());
    match view.layout {
        Some(layout) => {
            if let Some(renderer) = renderer {
                renderer.set_view(layout, view.inset);
            }
            emit_status(&app);
        }
        None => {
            if let Some(renderer) = renderer {
                renderer.set_inset(view.inset);
            }
        }
    }
    Ok(current(&app).1)
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

#[cfg(test)]
mod tests {
    use emudome_core::screens::{Align, ScreenId};

    use super::*;

    fn main_at(y: f32) -> Vec<CustomScreen> {
        vec![CustomScreen {
            id: ScreenId::Main,
            x: 0.0,
            y,
            w: 1.0,
            h: 0.4,
        }]
    }

    fn band(bottom: f32) -> Inset {
        Inset {
            bottom,
            align: Align::Bottom,
            ..Inset::NONE
        }
    }

    #[test]
    fn a_new_placement_reaches_the_renderer_with_its_inset() {
        let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
        let view = place_phone(Some(&mut s), main_at(0.1), band(0.3));
        let Ok(view) = view else {
            panic!("refused: {view:?}");
        };
        assert_eq!(view.layout, Some(s.layout()));
        assert_eq!(view.inset, band(0.3));
        assert!(s.status().phone_placement);
    }

    #[test]
    fn the_same_placement_sends_the_inset_alone() {
        let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
        assert!(place_phone(Some(&mut s), main_at(0.1), band(0.3)).is_ok());
        let view = place_phone(Some(&mut s), main_at(0.1), band(0.2));
        let Ok(view) = view else {
            panic!("refused: {view:?}");
        };
        assert_eq!(view.layout, None);
        assert_eq!(view.inset, band(0.2));
    }

    #[test]
    fn with_no_game_only_the_inset_applies() {
        let view = place_phone(None, main_at(0.1), band(0.3));
        let Ok(view) = view else {
            panic!("refused: {view:?}");
        };
        assert_eq!(view.layout, None);
        assert_eq!(view.inset, band(0.3));
    }

    #[test]
    fn a_placement_of_other_screens_is_refused_whole() {
        let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
        let view = place_phone(Some(&mut s), main_at(0.1), band(0.3));
        assert!(matches!(
            view,
            Err(ErrorCode::SettingInvalidValue { key }) if key == keys::TOUCH_SCREENS.as_str()
        ));
        assert!(!s.status().phone_placement);
    }

    fn two_screens() -> Vec<CustomScreen> {
        [(ScreenId::Top, 0.0), (ScreenId::Bottom, 0.5)]
            .into_iter()
            .map(|(id, y)| CustomScreen {
                id,
                x: 0.0,
                y,
                w: 1.0,
                h: 0.4,
            })
            .collect()
    }

    #[test]
    fn the_layout_cycle_moves_on_with_no_placement() {
        let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
        let before = s.status().layout;
        assert_eq!(cycle(&mut s), Cycled::Moved);
        assert_ne!(s.status().layout, before);
    }

    #[test]
    fn the_layout_cycle_says_it_waits_under_the_phones_placement() {
        let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
        assert_eq!(s.set_phone_placement(two_screens()), Some(true));
        let before = s.status().layout;
        assert_eq!(cycle(&mut s), Cycled::Held);
        assert_eq!(s.status().layout, before);
    }

    #[test]
    fn the_layout_cycle_says_nothing_for_a_one_screen_game() {
        let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
        assert_eq!(s.set_phone_placement(main_at(0.1)), Some(true));
        assert_eq!(cycle(&mut s), Cycled::Same);
    }
}
