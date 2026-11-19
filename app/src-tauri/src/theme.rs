//! Which interface theme to use: Scuro (dark), Nero (black) or Chiaro (light).
//!
//! In order: `EMUDOME_THEME=dark|black|light` (a test override), then the `general.theme`
//! setting (`system`, `dark`, `black`, `light`), then, for `system`, the operating system
//! (dark → Scuro, light → Chiaro). The interface reads the system preference
//! (`prefers-color-scheme`, which also follows live changes) and asks here again on every change
//! of it or of the setting; the answer goes on `<html data-theme>` (app/src/lib/theme.ts).

use serde::Serialize;

use crate::settings::{theme_setting, SettingsState};

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

/// The theme for an override value (the `EMUDOME_THEME` variable, if set), the `general.theme`
/// setting and the system's preference. An override that is not a theme name is ignored (and
/// logged); the setting `system` (or none) follows the system.
pub fn resolve(override_value: Option<&str>, setting: Option<&str>, system_dark: bool) -> Theme {
    if let Some(value) = override_value {
        match Theme::parse(value) {
            Some(theme) => return theme,
            None => log::warn!("EMUDOME_THEME={value:?} is not dark, black or light: ignored"),
        }
    }
    if let Some(theme) = setting.and_then(Theme::parse) {
        return theme;
    }
    if system_dark {
        Theme::Dark
    } else {
        Theme::Light
    }
}

/// The theme to show, given whether the system is in dark mode.
#[tauri::command]
pub fn theme(settings: tauri::State<'_, SettingsState>, system_dark: bool) -> Theme {
    resolve(
        std::env::var("EMUDOME_THEME").ok().as_deref(),
        theme_setting(&settings).as_deref(),
        system_dark,
    )
}

#[cfg(test)]
mod tests {
    use super::{resolve, Theme};

    #[test]
    fn follows_the_system_without_override() {
        assert_eq!(resolve(None, None, true), Theme::Dark);
        assert_eq!(resolve(None, None, false), Theme::Light);
    }

    #[test]
    fn override_wins_over_the_system() {
        assert_eq!(resolve(Some("black"), None, true), Theme::Black);
        assert_eq!(resolve(Some("black"), None, false), Theme::Black);
        assert_eq!(resolve(Some("light"), None, true), Theme::Light);
        assert_eq!(resolve(Some("dark"), None, false), Theme::Dark);
    }

    #[test]
    fn override_ignores_case_and_spaces() {
        assert_eq!(resolve(Some(" Light "), None, true), Theme::Light);
        assert_eq!(resolve(Some("BLACK"), None, false), Theme::Black);
    }

    #[test]
    fn unknown_or_empty_override_falls_back_to_the_system() {
        assert_eq!(resolve(Some("sepia"), None, true), Theme::Dark);
        assert_eq!(resolve(Some(""), None, false), Theme::Light);
        // Italian names are not accepted: the variable takes the English ones.
        assert_eq!(resolve(Some("nero"), None, false), Theme::Light);
    }

    #[test]
    fn the_setting_wins_over_the_system() {
        assert_eq!(resolve(None, Some("black"), false), Theme::Black);
        assert_eq!(resolve(None, Some("light"), true), Theme::Light);
        assert_eq!(resolve(None, Some("dark"), false), Theme::Dark);
    }

    #[test]
    fn the_setting_system_follows_the_system() {
        assert_eq!(resolve(None, Some("system"), true), Theme::Dark);
        assert_eq!(resolve(None, Some("system"), false), Theme::Light);
    }

    #[test]
    fn the_override_wins_over_the_setting() {
        assert_eq!(resolve(Some("light"), Some("black"), true), Theme::Light);
        // An invalid override falls back to the setting, not straight to the system.
        assert_eq!(resolve(Some("sepia"), Some("black"), true), Theme::Black);
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
