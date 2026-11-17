//! Which interface theme to use: Scuro (dark), Nero (black) or Chiaro (light).
//!
//! Until the settings engine exists the theme follows the operating system (dark → Scuro,
//! light → Chiaro), and `EMUDOME_THEME=dark|black|light` overrides it for testing.
//! The interface reads the system preference (`prefers-color-scheme`, which also follows
//! live changes) and asks here; the answer goes on `<html data-theme>` (app/src/theme.ts).

use serde::Serialize;

/// The interface theme. Mirror: `Theme` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Black,
    Light,
}

impl Theme {
    /// `dark`, `black` or `light`, any case, surrounding spaces ignored.
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "black" => Some(Self::Black),
            "light" => Some(Self::Light),
            _ => None,
        }
    }
}

/// The theme for an override value (the `EMUDOME_THEME` variable, if set) and the system's
/// preference. An override that is not a theme name is ignored (and logged).
pub fn resolve(override_value: Option<&str>, system_dark: bool) -> Theme {
    if let Some(value) = override_value {
        match Theme::parse(value) {
            Some(theme) => return theme,
            None => log::warn!("EMUDOME_THEME={value:?} is not dark, black or light: ignored"),
        }
    }
    if system_dark {
        Theme::Dark
    } else {
        Theme::Light
    }
}

/// The theme to show, given whether the system is in dark mode.
#[tauri::command]
pub fn theme(system_dark: bool) -> Theme {
    resolve(std::env::var("EMUDOME_THEME").ok().as_deref(), system_dark)
}

#[cfg(test)]
mod tests {
    use super::{resolve, Theme};

    #[test]
    fn follows_the_system_without_override() {
        assert_eq!(resolve(None, true), Theme::Dark);
        assert_eq!(resolve(None, false), Theme::Light);
    }

    #[test]
    fn override_wins_over_the_system() {
        assert_eq!(resolve(Some("black"), true), Theme::Black);
        assert_eq!(resolve(Some("black"), false), Theme::Black);
        assert_eq!(resolve(Some("light"), true), Theme::Light);
        assert_eq!(resolve(Some("dark"), false), Theme::Dark);
    }

    #[test]
    fn override_ignores_case_and_spaces() {
        assert_eq!(resolve(Some(" Light "), true), Theme::Light);
        assert_eq!(resolve(Some("BLACK"), false), Theme::Black);
    }

    #[test]
    fn unknown_or_empty_override_falls_back_to_the_system() {
        assert_eq!(resolve(Some("sepia"), true), Theme::Dark);
        assert_eq!(resolve(Some(""), false), Theme::Light);
        // Italian names are not accepted: the variable takes the English ones.
        assert_eq!(resolve(Some("nero"), false), Theme::Light);
    }

    #[test]
    fn serializes_as_the_names_the_interface_uses() {
        let names =
            [Theme::Dark, Theme::Black, Theme::Light].map(|t| serde_json::to_string(&t).ok());
        assert_eq!(
            names,
            [
                Some("\"dark\"".into()),
                Some("\"black\"".into()),
                Some("\"light\"".into())
            ]
        );
    }
}
