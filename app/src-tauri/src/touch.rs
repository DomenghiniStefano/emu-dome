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

/// The layouts saved for `console` on this device (`touch.tedit`, per device at the console's
/// level), with the controls its games have. A stored value that no longer reads (another
/// version's) counts as none: the default layouts are drawn, nothing is written.
#[tauri::command]
pub(crate) fn touch_layouts(app: AppHandle, console: ConsoleId) -> Result<Setup, ErrorCode> {
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let resolved = state
        .0
        .lock()
        .resolve(keys::TOUCH_LAYOUTS.as_str(), &Scope::Console { console })?;
    let items = match resolved.value {
        Some(Value::List(items)) => items,
        Some(Value::Bool(_) | Value::Int(_) | Value::Text(_)) | None => Vec::new(),
    };
    Ok(Setup::new(
        console,
        Layouts::from_items(&items).unwrap_or_default(),
    ))
}

/// C06's «Salva»: `layout` becomes `console`'s layout for `orientation` (the other orientation
/// kept), and that orientation's choice (`touch.tport` / `touch.tland`) turns to `custom` for the
/// console, so the game shows it at once. A layout with a control the console's games do not
/// have, or otherwise invalid, is refused with `settingInvalidValue`.
#[tauri::command]
pub(crate) fn touch_set_layout(
    app: AppHandle,
    console: ConsoleId,
    orientation: Orientation,
    layout: Layout,
) -> Result<(), ErrorCode> {
    let key = keys::TOUCH_LAYOUTS.as_str();
    if layout.controls.is_empty() || !layout.fits(console) {
        return Err(ErrorCode::SettingInvalidValue {
            key: key.to_owned(),
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
        let stored = match settings.resolve(key, &scope)?.value {
            Some(Value::List(items)) => Layouts::from_items(&items).unwrap_or_default(),
            Some(Value::Bool(_) | Value::Int(_) | Value::Text(_)) | None => Layouts::default(),
        };
        let items = stored.with(orientation, layout).to_items();
        let level = settings.set(key, &scope, Value::List(items))?;
        settings.set(choice, &scope, Value::Text("custom".to_owned()))?;
        level
    };
    crate::settings::emit_changed(&app, &[key, choice], level);
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
