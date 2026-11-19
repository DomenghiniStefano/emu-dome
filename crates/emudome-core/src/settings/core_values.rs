//! Settings that become core options: entries of the Console options page (T13) that a core
//! reads through `GET_VARIABLE`, turned into that core's keys and values. Pure: the caller
//! resolves each key for the game being opened.
//!
//! For now the Game Boy / Game Boy Color entries (mGBA). The option keys and values are the
//! ones mGBA's libretro core declares (checked against the core in `cores/`); a value the
//! core does not accept falls back to its default with a warning (`core_options.rs`).
//! TODO(settings engine): the GBA (`consoles.gbacc`, `gbablend`, `solar`) and `cores.*`
//! entries.

use super::{ConsoleId, Value};

/// The core options the settings ask for, for a game of `console`. `value` resolves a
/// settings key for that game (None when it cannot be read: the core's default stays).
/// Options are read by the core when the game starts (mGBA marks these «Restart»).
pub fn core_options(
    console: ConsoleId,
    value: impl Fn(&str) -> Option<Value>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    match console {
        ConsoleId::Gb | ConsoleId::Gbc => gb(&value, &mut out),
        // melonDS DS: its own module (`core_values_nds.rs`).
        ConsoleId::Nds => super::core_values_nds::nds(&value, &mut out),
        ConsoleId::Gba | ConsoleId::N3ds => {}
    }
    out
}

fn text(value: &impl Fn(&str) -> Option<Value>, key: &str) -> Option<String> {
    match value(key) {
        Some(Value::Text(t)) => Some(t),
        Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => None,
    }
}

fn push(out: &mut Vec<(String, String)>, key: &str, value: &str) {
    out.push((key.to_owned(), value.to_owned()));
}

/// `consoles.gbmodel`, `consoles.gbpal`, `consoles.sgb` → mGBA.
fn gb(value: &impl Fn(&str) -> Option<Value>, out: &mut Vec<(String, String)>) {
    let model = match text(value, "consoles.gbmodel").as_deref() {
        Some("auto") => Some("Autodetect"),
        Some("gb") => Some("Game Boy"),
        Some("gbc") => Some("Game Boy Color"),
        Some("sgb") => Some("Super Game Boy"),
        Some(_) | None => None,
    };
    if let Some(model) = model {
        push(out, "mgba_gb_model", model);
    }

    // The palette of black-and-white games. «gbc» = the colours a Game Boy Color gives the
    // games it knows (mGBA's hardware presets); a game it does not know keeps the core's
    // default palette.
    let palette = match text(value, "consoles.gbpal").as_deref() {
        Some("green") => Some(("DMG Green", "0")),
        Some("pocket") => Some(("GB Pocket", "0")),
        Some("light") => Some(("GB Light", "0")),
        Some("gbc") => Some(("", "1")),
        Some(_) | None => None,
    };
    if let Some((colors, preset)) = palette {
        if !colors.is_empty() {
            push(out, "mgba_gb_colors", colors);
        }
        push(out, "mgba_gb_colors_preset", preset);
    }

    match value("consoles.sgb") {
        Some(Value::Bool(on)) => push(out, "mgba_sgb_borders", if on { "ON" } else { "OFF" }),
        Some(Value::Int(_) | Value::Text(_) | Value::List(_)) | None => {}
    }
}
