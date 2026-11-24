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

/// The choice key of `orientation` (`touch.tport` / `touch.tland`).
fn choice_key(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Portrait => keys::TOUCH_PORTRAIT.as_str(),
        Orientation::Landscape => keys::TOUCH_LANDSCAPE.as_str(),
    }
}

/// What one «Salva» stores: `stored` with each layout given in its orientation's place (one not
/// given stays as stored), and the choice keys of the orientations given. A layout with no
/// controls, or not `console`'s, refuses the whole save with `settingInvalidValue`.
fn saved_layouts(
    stored: Layouts,
    console: ConsoleId,
    portrait: Option<Layout>,
    landscape: Option<Layout>,
) -> Result<(Layouts, Vec<&'static str>), ErrorCode> {
    let given: Vec<(Orientation, Layout)> = [
        (Orientation::Portrait, portrait),
        (Orientation::Landscape, landscape),
    ]
    .into_iter()
    .filter_map(|(orientation, layout)| layout.map(|l| (orientation, l)))
    .collect();
    if given
        .iter()
        .any(|(_, l)| l.controls.is_empty() || !l.fits(console))
    {
        return Err(ErrorCode::SettingInvalidValue {
            key: keys::TOUCH_LAYOUTS.as_str().to_owned(),
        });
    }
    let choices = given.iter().map(|(o, _)| choice_key(*o)).collect();
    let saved = given
        .into_iter()
        .fold(stored, |layouts, (o, l)| layouts.with(o, l));
    Ok((saved, choices))
}

/// C06's «Salva»: each layout given (controls and screens) becomes `console`'s layout for its
/// orientation (an orientation not given is kept as stored), and each given orientation's choice
/// (`touch.tport` / `touch.tland`) turns to `custom` for the console, so the game shows it at
/// once; all in one write. An empty `screens` clears that orientation's placement. One layout with
/// a control or screen the console's games do not have, or otherwise invalid, refuses the whole
/// save with `settingInvalidValue`.
#[tauri::command]
pub(crate) fn touch_set_layouts(
    app: AppHandle,
    console: ConsoleId,
    portrait: Option<Layout>,
    landscape: Option<Layout>,
) -> Result<(), ErrorCode> {
    let tedit = keys::TOUCH_LAYOUTS.as_str();
    let tscreens = keys::TOUCH_SCREENS.as_str();
    let scope = Scope::Console { console };
    let state = app.try_state::<SettingsState>().ok_or(ErrorCode::NoGame)?;
    let (level, choices) = {
        let mut settings = state.0.lock();
        let stored = Layouts::read(
            &stored_items(&mut settings, tedit, &scope)?,
            &stored_items(&mut settings, tscreens, &scope)?,
        );
        let (saved, choices) = saved_layouts(stored, console, portrait, landscape)?;
        let values: Vec<(&str, Value)> = [
            (tedit, Value::List(saved.to_items())),
            (tscreens, Value::List(saved.screen_items())),
        ]
        .into_iter()
        .chain(
            choices
                .iter()
                .map(|key| (*key, Value::Text("custom".to_owned()))),
        )
        .collect();
        (settings.set_many(&scope, &values)?, choices)
    };
    let written: Vec<&str> = [tedit, tscreens].into_iter().chain(choices).collect();
    crate::settings::emit_changed(&app, &written, level);
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

#[cfg(test)]
mod tests {
    use emudome_core::touch::{Behaviour, Control, ControlId};

    use super::*;

    fn layout(x: f32) -> Layout {
        let control = |id, x| Control {
            id,
            x,
            y: 0.5,
            size: 100,
            opacity: 100,
            behaviour: Behaviour::Tap,
        };
        Layout {
            controls: vec![control(ControlId::Dpad, x), control(ControlId::Menu, 0.5)],
            screens: Vec::new(),
        }
    }

    #[test]
    fn both_orientations_are_saved_and_made_custom() {
        let stored = Layouts::default().with(Orientation::Portrait, layout(0.1));
        let Ok((saved, choices)) =
            saved_layouts(stored, ConsoleId::Gba, Some(layout(0.2)), Some(layout(0.3)))
        else {
            panic!("refused");
        };
        assert_eq!(saved.portrait, layout(0.2));
        assert_eq!(saved.landscape, layout(0.3));
        assert_eq!(
            choices,
            [
                keys::TOUCH_PORTRAIT.as_str(),
                keys::TOUCH_LANDSCAPE.as_str()
            ]
        );
    }

    #[test]
    fn saving_one_leaves_the_other_and_its_choice_alone() {
        let stored = Layouts::default().with(Orientation::Portrait, layout(0.1));
        let Ok((saved, choices)) = saved_layouts(stored, ConsoleId::Gba, None, Some(layout(0.3)))
        else {
            panic!("refused");
        };
        assert_eq!(saved.portrait, layout(0.1));
        assert_eq!(saved.landscape, layout(0.3));
        assert_eq!(choices, [keys::TOUCH_LANDSCAPE.as_str()]);
    }

    #[test]
    fn an_invalid_layout_refuses_the_whole_save() {
        // The 3DS Circle Pad is not a GBA control; an empty layout is not a save either.
        let mut foreign = layout(0.2);
        foreign.controls[0].id = ControlId::CirclePad;
        for bad in [foreign, Layout::default()] {
            let refused = saved_layouts(
                Layouts::default(),
                ConsoleId::Gba,
                Some(layout(0.2)),
                Some(bad),
            );
            assert!(matches!(
                refused,
                Err(ErrorCode::SettingInvalidValue { key }) if key == keys::TOUCH_LAYOUTS.as_str()
            ));
        }
    }
}
