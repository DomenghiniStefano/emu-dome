//! melonDS DS's core options from the settings: the NDS console options (`consoles.dsmode`,
//! `dslang`, `dsname`, `slot2`) and melonDS DS's own entries (`cores.mdsrender`, `mdsboot`).
//! Pure, like `core_values.rs`, which calls [`nds`] for a DS game. Option keys and values as
//! melonDS DS 1.4.0 declares them (`src/libretro/config/constants.hpp`); a value the core does
//! not accept falls back to its default with a warning (`core_options.rs`).
//!
//! The options that hold the screens where the shell cuts them are not settings: they are
//! forced, `crate::nds::FORCED_OPTIONS`.

use super::Value;

fn text(value: &impl Fn(&str) -> Option<Value>, key: &str) -> Option<String> {
    match value(key) {
        Some(Value::Text(t)) => Some(t),
        Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => None,
    }
}

fn push(out: &mut Vec<(String, String)>, key: &str, value: &str) {
    out.push((key.to_owned(), value.to_owned()));
}

/// The core options of a DS game.
pub fn nds(value: &impl Fn(&str) -> Option<Value>, out: &mut Vec<(String, String)>) {
    // DSi mode needs the DSi's own BIOS and NAND, which have no built-in replacement: with the
    // built-in system files forced (`crate::nds`), «dsi» does not boot until `bios.bdsi` is
    // wired. TODO(bios.*).
    let mode = match text(value, "consoles.dsmode").as_deref() {
        Some("auto") => Some("auto"),
        Some("ds") => Some("ds"),
        Some("dsi") => Some("dsi"),
        Some(_) | None => None,
    };
    if let Some(mode) = mode {
        push(out, "melonds_console_mode", mode);
    }

    // Same ids on both sides.
    let language = text(value, "consoles.dslang")
        .filter(|l| matches!(l.as_str(), "ja" | "en" | "fr" | "de" | "it" | "es"));
    if let Some(language) = language {
        push(out, "melonds_firmware_language", &language);
    }

    // The core takes the name from the frontend (`GET_USERNAME`, see [`username`]); with none,
    // it guesses the OS user, which is what an empty `consoles.dsname` means.
    push(out, "melonds_firmware_username", "guess_username");

    let slot2 = match text(value, "consoles.slot2").as_deref() {
        // Nothing in the slot: «auto» puts nothing there when no GBA game is loaded.
        Some("none") => Some("auto"),
        Some("rumble") => Some("rumble-pak"),
        Some("memory") => Some("expansion-pak"),
        // TODO(#45): load the GBA game into slot 2 (melonDS DS's subsystem content).
        Some("gba_game") => Some("auto"),
        Some(_) | None => None,
    };
    if let Some(slot2) = slot2 {
        push(out, "melonds_slot2_device", slot2);
    }

    // OpenGL needs a GL context the host does not offer (Trello #16): melonDS DS then falls back
    // to software by itself (measured, docs/feasibility.md).
    let render = match text(value, "cores.mdsrender").as_deref() {
        Some("software") => Some("software"),
        Some("opengl") => Some("opengl"),
        Some(_) | None => None,
    };
    if let Some(render) = render {
        push(out, "melonds_render_mode", render);
    }

    // «menu» boots through the DS menu, which needs the real firmware: with the built-in one
    // melonDS DS boots the game directly anyway.
    let boot = match text(value, "cores.mdsboot").as_deref() {
        Some("direct") => Some("direct"),
        Some("menu") => Some("native"),
        Some(_) | None => None,
    };
    if let Some(boot) = boot {
        push(out, "melonds_boot_mode", boot);
    }
}

/// The DS user name for `GET_USERNAME` (`consoles.dsname`): `None` when empty (the core then
/// uses the OS user name). The DS keeps 10 characters; melonDS DS cuts the rest.
pub fn username(value: impl Fn(&str) -> Option<Value>) -> Option<String> {
    text(&value, "consoles.dsname")
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{nds, username, Value};

    fn options(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        let value = |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| Value::Text(v.clone()))
        };
        let mut out = Vec::new();
        nds(&value, &mut out);
        out
    }

    fn get<'a>(out: &'a [(String, String)], key: &str) -> Option<&'a str> {
        out.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    #[test]
    fn the_catalog_defaults_become_core_values() {
        let out = options(&[
            ("consoles.dsmode", "auto"),
            ("consoles.dslang", "it"),
            ("consoles.slot2", "none"),
            ("cores.mdsrender", "software"),
            ("cores.mdsboot", "direct"),
        ]);
        assert_eq!(get(&out, "melonds_console_mode"), Some("auto"));
        assert_eq!(get(&out, "melonds_firmware_language"), Some("it"));
        assert_eq!(get(&out, "melonds_slot2_device"), Some("auto"));
        assert_eq!(get(&out, "melonds_render_mode"), Some("software"));
        assert_eq!(get(&out, "melonds_boot_mode"), Some("direct"));
        assert_eq!(
            get(&out, "melonds_firmware_username"),
            Some("guess_username")
        );
    }

    #[test]
    fn every_choice_has_a_core_value() {
        for (key, id, option, want) in [
            ("consoles.dsmode", "ds", "melonds_console_mode", "ds"),
            ("consoles.dsmode", "dsi", "melonds_console_mode", "dsi"),
            ("consoles.dslang", "ja", "melonds_firmware_language", "ja"),
            ("consoles.dslang", "es", "melonds_firmware_language", "es"),
            (
                "consoles.slot2",
                "rumble",
                "melonds_slot2_device",
                "rumble-pak",
            ),
            (
                "consoles.slot2",
                "memory",
                "melonds_slot2_device",
                "expansion-pak",
            ),
            ("consoles.slot2", "gba_game", "melonds_slot2_device", "auto"),
            ("cores.mdsrender", "opengl", "melonds_render_mode", "opengl"),
            ("cores.mdsboot", "menu", "melonds_boot_mode", "native"),
        ] {
            assert_eq!(
                get(&options(&[(key, id)]), option),
                Some(want),
                "{key}={id}"
            );
        }
    }

    #[test]
    fn unknown_or_missing_values_leave_the_core_default() {
        let out = options(&[("consoles.dslang", "xx"), ("consoles.slot2", "?")]);
        assert_eq!(get(&out, "melonds_firmware_language"), None);
        assert_eq!(get(&out, "melonds_slot2_device"), None);
        assert_eq!(get(&options(&[]), "melonds_console_mode"), None);
    }

    #[test]
    fn the_user_name_is_answered_only_when_set() {
        let name = |v: &str| {
            let v = v.to_owned();
            username(move |key| (key == "consoles.dsname").then(|| Value::Text(v.clone())))
        };
        assert_eq!(name(""), None);
        assert_eq!(name("   "), None);
        assert_eq!(name(" Stefano "), Some("Stefano".into()));
        assert_eq!(username(|_| None), None);
    }
}
