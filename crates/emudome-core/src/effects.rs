//! Screen effects: what the `fx.*` settings ask the native renderer to draw.
//!
//! `fx.fxpick` names a built-in effect (a `.slangp` preset shipped with the app, see
//! `app/src-tauri/effects/`) or `file`, the user's own preset chosen in `fx.fxfile`;
//! `fx.fxint` mixes the effect over the plain image and `fx.fxbright` brightens the effect.
//! Pure: the renderer loads the preset, this module only decides which one and how strong.

use std::path::PathBuf;

use crate::settings::Value;

/// The built-in effects, by their `fx.fxpick` option id; each is `<id>.slangp`.
pub const BUILTIN: [&str; 5] = ["lcd", "lcd_grid", "crt", "crt_curved", "soft_pixels"];

/// Where the preset comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectSource {
    /// One of [`BUILTIN`].
    Builtin(&'static str),
    /// A `.slangp` the user chose (`fx.fxfile`).
    File(PathBuf),
}

/// An effect to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectChoice {
    pub source: EffectSource,
    /// 0..=1: how much of the effect is mixed over the plain image (`fx.fxint`).
    pub intensity: f32,
    /// Multiplies the effect's colours (`fx.fxbright`, 1 = unchanged).
    pub brightness: f32,
}

/// Defaults of `fx.fxint` and `fx.fxbright` when a value is missing (the catalogue's).
const INTENSITY_DEFAULT: i64 = 60;
const BRIGHTNESS_DEFAULT: i64 = 110;

fn percent(value: Option<&Value>, default: i64, max: i64) -> f32 {
    let p = match value {
        Some(Value::Int(p)) => *p,
        Some(Value::Bool(_) | Value::Text(_) | Value::List(_)) | None => default,
    };
    // At most a few hundred: exact in an f32.
    p.clamp(0, max) as f32 / 100.0
}

/// The effect the resolved `fx.*` values ask for; `None` = draw the plain image (no effect,
/// strength 0, or `file` with no file chosen).
pub fn from_settings(
    pick: Option<&Value>,
    intensity: Option<&Value>,
    brightness: Option<&Value>,
    file: Option<&Value>,
) -> Option<EffectChoice> {
    let Some(Value::Text(pick)) = pick else {
        return None;
    };
    let source = if pick == "file" {
        match file {
            Some(Value::Text(path)) if !path.trim().is_empty() => {
                EffectSource::File(PathBuf::from(path.trim()))
            }
            Some(Value::Text(_) | Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => {
                return None
            }
        }
    } else {
        EffectSource::Builtin(BUILTIN.iter().copied().find(|id| id == pick)?)
    };
    let intensity = percent(intensity, INTENSITY_DEFAULT, 100);
    if intensity <= 0.0 {
        return None;
    }
    Some(EffectChoice {
        source,
        intensity,
        brightness: percent(brightness, BRIGHTNESS_DEFAULT, 400),
    })
}

#[cfg(test)]
mod tests {
    use super::{from_settings, EffectChoice, EffectSource, BUILTIN};
    use crate::settings::{entry, Kind, Value};
    use std::path::PathBuf;

    fn text(s: &str) -> Value {
        Value::Text(s.into())
    }

    #[test]
    fn every_built_in_effect_is_an_option_of_fxpick_and_back() {
        let pick = entry("fx.fxpick").expect("fx.fxpick in the catalogue");
        let Kind::Choice { options, .. } = pick.kind else {
            panic!("fx.fxpick is a choice");
        };
        let mut expected: Vec<&str> = vec!["none"];
        expected.extend(BUILTIN);
        expected.push("file");
        assert_eq!(options, expected.as_slice());
    }

    #[test]
    fn none_and_unknown_picks_draw_the_plain_image() {
        assert_eq!(from_settings(None, None, None, None), None);
        assert_eq!(from_settings(Some(&text("none")), None, None, None), None);
        assert_eq!(from_settings(Some(&text("vhs")), None, None, None), None);
    }

    #[test]
    fn a_built_in_pick_with_the_defaults() {
        assert_eq!(
            from_settings(Some(&text("crt")), None, None, None),
            Some(EffectChoice {
                source: EffectSource::Builtin("crt"),
                intensity: 0.6,
                brightness: 1.1,
            })
        );
    }

    #[test]
    fn strength_and_brightness_are_percentages_clamped() {
        let c = from_settings(
            Some(&text("lcd")),
            Some(&Value::Int(250)),
            Some(&Value::Int(80)),
            None,
        )
        .expect("an effect");
        assert!((c.intensity - 1.0).abs() < f32::EPSILON);
        assert!((c.brightness - 0.8).abs() < f32::EPSILON);
        // Strength 0 shows nothing of the effect: no effect at all.
        assert_eq!(
            from_settings(Some(&text("lcd")), Some(&Value::Int(0)), None, None),
            None
        );
    }

    #[test]
    fn the_file_option_uses_the_chosen_file_or_nothing() {
        let path = text(" C:/effects/my.slangp ");
        assert_eq!(
            from_settings(Some(&text("file")), None, None, Some(&path)).map(|c| c.source),
            Some(EffectSource::File(PathBuf::from("C:/effects/my.slangp")))
        );
        assert_eq!(
            from_settings(Some(&text("file")), None, None, Some(&text(""))),
            None
        );
        assert_eq!(from_settings(Some(&text("file")), None, None, None), None);
    }
}
