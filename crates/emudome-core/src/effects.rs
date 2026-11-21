//! Screen effects: what the `fx.*` settings ask the native renderer to draw.
//!
//! `fx.fxpick` names a built-in effect (a `.slangp` preset shipped with the app, see
//! `app/src-tauri/effects/`) or `file`, the user's own preset chosen in `fx.fxfile`;
//! `fx.fxint` mixes the effect over the plain image and `fx.fxbright` brightens the effect.
//! Pure: the renderer loads the preset, this module only decides which one and how strong.

use std::path::PathBuf;

use crate::settings::keys::{self, Reader};
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

/// The effect the resolved `fx.*` values ask for (`read`: a resolved value; a missing or
/// mistyped one is the catalogue's default); `None` = draw the plain image (no effect, strength
/// 0, or `file` with no file chosen).
pub fn from_settings(read: impl Fn(&str) -> Option<Value>) -> Option<EffectChoice> {
    let settings = Reader::new(read);
    let pick = settings.text(keys::FX_PICK);
    let source = if pick == "file" {
        let path = settings.text(keys::FX_FILE);
        let path = path.trim();
        if path.is_empty() {
            return None;
        }
        EffectSource::File(PathBuf::from(path))
    } else {
        EffectSource::Builtin(BUILTIN.iter().copied().find(|id| *id == pick)?)
    };
    let intensity = percent(settings.int(keys::FX_INTENSITY));
    if intensity <= 0.0 {
        return None;
    }
    Some(EffectChoice {
        source,
        intensity,
        brightness: percent(settings.int(keys::FX_BRIGHTNESS)),
    })
}

/// A slider percentage (held to its range by [`Reader::int`]) as a factor. At most a few
/// hundred: exact in an f32.
fn percent(p: i64) -> f32 {
    p as f32 / 100.0
}

#[cfg(test)]
mod tests {
    use super::{from_settings, EffectChoice, EffectSource, BUILTIN};
    use crate::settings::keys;
    use crate::settings::{Kind, Value};
    use std::path::PathBuf;

    fn text(s: &str) -> Value {
        Value::Text(s.into())
    }

    #[test]
    fn every_built_in_effect_is_an_option_of_fxpick_and_back() {
        let Kind::Choice { options, .. } = keys::FX_PICK.entry().kind else {
            panic!("fx.fxpick is a choice");
        };
        let mut expected: Vec<&str> = vec!["none"];
        expected.extend(BUILTIN);
        expected.push("file");
        assert_eq!(options, expected.as_slice());
    }

    /// `from_settings` with these `fx.*` values (`None`: not set).
    fn fx(
        pick: Option<&str>,
        intensity: Option<i64>,
        brightness: Option<i64>,
        file: Option<&str>,
    ) -> Option<EffectChoice> {
        from_settings(|key| match key {
            "fx.fxpick" => pick.map(text),
            "fx.fxint" => intensity.map(Value::Int),
            "fx.fxbright" => brightness.map(Value::Int),
            "fx.fxfile" => file.map(text),
            _ => None,
        })
    }

    #[test]
    fn none_and_unknown_picks_draw_the_plain_image() {
        assert_eq!(fx(None, None, None, None), None);
        assert_eq!(fx(Some("none"), None, None, None), None);
        assert_eq!(fx(Some("vhs"), None, None, None), None);
    }

    #[test]
    fn a_built_in_pick_with_the_defaults() {
        assert_eq!(
            fx(Some("crt"), None, None, None),
            Some(EffectChoice {
                source: EffectSource::Builtin("crt"),
                intensity: 0.6,
                brightness: 1.1,
            })
        );
    }

    #[test]
    fn strength_and_brightness_are_percentages_held_to_their_sliders() {
        let c = fx(Some("lcd"), Some(250), Some(80), None).expect("an effect");
        assert!((c.intensity - 1.0).abs() < f32::EPSILON);
        assert!((c.brightness - 0.8).abs() < f32::EPSILON);
        let c = fx(Some("lcd"), None, Some(400), None).expect("an effect");
        assert!((c.brightness - 1.5).abs() < f32::EPSILON);
        // Strength 0 shows nothing of the effect: no effect at all.
        assert_eq!(fx(Some("lcd"), Some(0), None, None), None);
    }

    #[test]
    fn the_file_option_uses_the_chosen_file_or_nothing() {
        assert_eq!(
            fx(Some("file"), None, None, Some(" C:/effects/my.slangp ")).map(|c| c.source),
            Some(EffectSource::File(PathBuf::from("C:/effects/my.slangp")))
        );
        assert_eq!(fx(Some("file"), None, None, Some("")), None);
        assert_eq!(fx(Some("file"), None, None, None), None);
    }
}
