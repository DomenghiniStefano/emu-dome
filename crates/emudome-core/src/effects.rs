//! Screen effects: what the `fx.*` settings ask the native renderer to draw.
//!
//! `fx.fxpick` names a built-in effect (a `.slangp` preset shipped with the app, see
//! `app/src-tauri/effects/`) or `file`, the user's own preset chosen in `fx.fxfile`;
//! `fx.fxint` mixes the effect over the plain image and `fx.fxbright` brightens the effect.
//! Pure: the renderer loads the preset, this module only decides which one and how strong.
//! [`Smoothing`] is `video.filter`, the plain image's smoothing under any effect.

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

/// `video.filter`: how the plain image is smoothed when the game is enlarged (both video paths;
/// a screen effect is drawn over it as before).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Smoothing {
    /// `none`: sharp pixels (nearest).
    None,
    /// `light`: the pixels enlarged by the largest whole scale that fits, then smoothed for the
    /// rest («sharp bilinear»): even pixels at any scale, soft only at their edges.
    Light,
    /// `soft`: smoothed all over (bilinear).
    Soft,
}

impl Smoothing {
    /// Its `video.filter` option id.
    pub fn id(self) -> &'static str {
        match self {
            Smoothing::None => "none",
            Smoothing::Light => "light",
            Smoothing::Soft => "soft",
        }
    }

    /// The smoothing the resolved `video.filter` asks for (a missing, mistyped or unknown value
    /// is the catalogue's default).
    pub fn from_settings(read: impl Fn(&str) -> Option<Value>) -> Self {
        let settings = Reader::new(read);
        let of = |id: &str| {
            [Smoothing::None, Smoothing::Light, Smoothing::Soft]
                .into_iter()
                .find(|s| s.id() == id)
        };
        of(&settings.text(keys::VIDEO_FILTER))
            .or_else(|| of(&Reader::new(|_| None).text(keys::VIDEO_FILTER)))
            .unwrap_or(Smoothing::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::{from_settings, EffectChoice, EffectSource, Smoothing, BUILTIN};
    use crate::settings::keys;
    use crate::settings::{Kind, Value};
    use std::path::PathBuf;

    #[test]
    fn every_filter_option_is_a_smoothing_and_back() {
        let Kind::Choice { options, .. } = keys::VIDEO_FILTER.entry().kind else {
            panic!("video.filter is a choice");
        };
        let ids: Vec<&str> = [Smoothing::None, Smoothing::Light, Smoothing::Soft]
            .iter()
            .map(|s| s.id())
            .collect();
        assert_eq!(options, ids.as_slice());
        for id in options {
            let s = Smoothing::from_settings(|_| Some(text(id)));
            assert_eq!(s.id(), *id);
        }
    }

    #[test]
    fn a_missing_or_unknown_filter_is_the_default_light() {
        assert_eq!(Smoothing::from_settings(|_| None), Smoothing::Light);
        assert_eq!(
            Smoothing::from_settings(|_| Some(text("blur"))),
            Smoothing::Light
        );
        assert_eq!(
            Smoothing::from_settings(|_| Some(Value::Int(1))),
            Smoothing::Light
        );
    }

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
