//! The touch controls (Trello #29): the user's layouts read and saved for one console (the C06
//! editor), the touch Circle Pad, and the vibration of a press. What a layout is, and which
//! controls a console has, is `emudome_core::touch`'s; the default layouts and the drawing are
//! the interface's (`app/src/components/touch/`).

use emudome_core::settings::{keys, ConsoleId, Scope, Value};
use emudome_core::touch::{Layout, Layouts, Orientation, Setup};
use emudome_core::ErrorCode;
use tauri::{AppHandle, Manager};

use crate::settings::SettingsState;
use crate::AppState;

/// The stored list under `key` at `scope`, none when there is no value.
fn stored_items(
    settings: &mut emudome_core::settings::Settings,
    key: &str,
    scope: &Scope,
) -> Result<Vec<String>, ErrorCode> {
    Ok(match settings.resolve(key, scope)?.value {
        Some(Value::List(items)) => items,
        Some(Value::Bool(_) | Value::Int(_) | Value::Text(_)) | None => Vec::new(),
    })
}

/// The layouts saved for `console` on this device (`touch.tedit` and `touch.tscreens`, per device
/// at the console's level), with the controls and screens its games have. A stored value that no
/// longer reads (another version's) counts as none: the default layouts are drawn, nothing is
/// written.
#[tauri::command]
pub(crate) fn touch_layouts(app: AppHandle, console: ConsoleId) -> Result<Setup, ErrorCode> {
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let scope = Scope::Console { console };
    let (tedit, tscreens) = {
        let mut settings = state.0.lock();
        (
            stored_items(&mut settings, keys::TOUCH_LAYOUTS.as_str(), &scope)?,
            stored_items(&mut settings, keys::TOUCH_SCREENS.as_str(), &scope)?,
        )
    };
    Ok(Setup::new(
        console,
        Layouts::read(&tedit, &tscreens).fitting(console),
    ))
}

/// C06's «Salva»: `layout` (controls and screens) becomes `console`'s layout for `orientation`
/// (the other orientation kept), and that orientation's choice (`touch.tport` / `touch.tland`)
/// turns to `custom` for the console, so the game shows it at once; all in one write. An empty
/// `screens` clears that orientation's placement. A layout with a control or screen the console's
/// games do not have, or otherwise invalid, is refused with `settingInvalidValue`.
#[tauri::command]
pub(crate) fn touch_set_layout(
    app: AppHandle,
    console: ConsoleId,
    orientation: Orientation,
    layout: Layout,
) -> Result<(), ErrorCode> {
    let tedit = keys::TOUCH_LAYOUTS.as_str();
    let tscreens = keys::TOUCH_SCREENS.as_str();
    if layout.controls.is_empty() || !layout.fits(console) {
        return Err(ErrorCode::SettingInvalidValue {
            key: tedit.to_owned(),
        });
    }
    let choice = match orientation {
        Orientation::Portrait => keys::TOUCH_PORTRAIT.as_str(),
        Orientation::Landscape => keys::TOUCH_LANDSCAPE.as_str(),
    };
    let scope = Scope::Console { console };
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let level = {
        let mut settings = state.0.lock();
        let stored = Layouts::read(
            &stored_items(&mut settings, tedit, &scope)?,
            &stored_items(&mut settings, tscreens, &scope)?,
        );
        let saved = stored.with(orientation, layout);
        settings.set_many(
            &scope,
            &[
                (tedit, Value::List(saved.to_items())),
                (tscreens, Value::List(saved.screen_items())),
                (choice, Value::Text("custom".to_owned())),
            ],
        )?
    };
    crate::settings::emit_changed(&app, &[tedit, tscreens, choice], level);
    Ok(())
}

/// The touch Circle Pad (3DS) at (`x`, `y`), each -1..1 (y downwards); (0, 0) when let go.
#[tauri::command]
pub(crate) fn touch_stick(state: tauri::State<'_, AppState>, x: f32, y: f32) {
    let axis = |v: f32| {
        let v = if v.is_finite() {
            v.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        (v * f32::from(i16::MAX)).round() as i16
    };
    if let Some(shared) = state.games.current().load().as_ref() {
        shared.input.set_touch_stick(axis(x), axis(y));
    }
}

/// A short vibration for a touch control's press, `strength` 1–100 (`touch.thapt`); nothing
/// where the device cannot vibrate (the desktop) or with 0.
#[tauri::command]
pub(crate) fn haptic(strength: u8) {
    if strength > 0 {
        emudome_platform::haptic(strength.min(100));
    }
}
