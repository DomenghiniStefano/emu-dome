//! Where Azahar writes when it cannot reach the save directory EmuDome gives it (Trello #97).
//!
//! Read in Azahar's source at the pinned version (2126.2, commit 219d743):
//! `src/citra_libretro/core_settings.cpp` `ParseStorageOptions` creates `<save dir>/Azahar/` and,
//! when that fails, keeps "Azahar's default paths": `FileUtil::SetUserPath("")`
//! (`src/common/file_util.cpp`), whose user folder (the emulated NAND, SD card, system data,
//! states) is, by system ([`Layout`]):
//!
//! - Windows: `<executable's folder>\user\` when that folder exists, else
//!   `<Roaming AppData>\Azahar\` (`SHGetKnownFolderPath(FOLDERID_RoamingAppData)`, the folder
//!   `APPDATA` names). The executable is the process's (`GetModuleFileNameW(nullptr)`): EmuDome's.
//! - Linux, and Android (the libretro build defines `HAVE_LIBRETRO_VFS`, which takes the generic
//!   branch): `<working folder>/user/` when it exists, else `$XDG_DATA_HOME/azahar-emu/`, by
//!   default `$HOME/.local/share/azahar-emu/`.
//! - macOS (not compiled): the same XDG folder when one of Azahar's XDG folders exists, else
//!   `~/Library/Application Support/Azahar/`.
//!
//! Every candidate is listed, the one Azahar would pick first: the caller only looks at them.
//! Nothing here creates, writes or deletes anything.

use std::path::{Path, PathBuf};

/// How a system lays out Azahar's default user folder.
// An `allow`, not an `expect`: each system's build constructs its own variant only (the tests
// construct all of them), and which ones go unused changes with the target.
#[allow(
    dead_code,
    reason = "one variant per system; the others are built only by the tests"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Windows,
    /// Linux and Android.
    Xdg,
    MacOs,
}

/// What the folders are made from, read from the process's environment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Places {
    pub exe_dir: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    /// `APPDATA` (Windows' Roaming AppData).
    pub app_data: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl Places {
    pub fn from_env() -> Self {
        Self {
            exe_dir: std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf)),
            cwd: std::env::current_dir().ok(),
            app_data: env_dir("APPDATA"),
            xdg_data_home: env_dir("XDG_DATA_HOME"),
            home: env_dir("HOME"),
        }
    }
}

/// The environment variable as a path, when it is set and not empty.
fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Azahar's own user folders for `layout`, the one it would pick first first.
pub fn user_dirs(layout: Layout, places: &Places) -> Vec<PathBuf> {
    match layout {
        Layout::Windows => {
            let mut dirs: Vec<PathBuf> = places.exe_dir.iter().map(|d| d.join("user")).collect();
            dirs.extend(places.app_data.as_ref().map(|d| d.join("Azahar")));
            dirs
        }
        Layout::Xdg | Layout::MacOs => {
            let mut dirs: Vec<PathBuf> = places.cwd.iter().map(|d| d.join("user")).collect();
            // `XDG_DATA_HOME` wins over `HOME`, as in Azahar's `GetUserDirectory`.
            let data_home = places
                .xdg_data_home
                .clone()
                .or_else(|| places.home.as_ref().map(|h| h.join(".local").join("share")));
            dirs.extend(data_home.map(|d| d.join("azahar-emu")));
            if layout == Layout::MacOs {
                dirs.extend(
                    places
                        .home
                        .as_ref()
                        .map(|h| h.join("Library").join("Application Support").join("Azahar")),
                );
            }
            dirs
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{user_dirs, Layout, Places};

    fn places() -> Places {
        Places {
            exe_dir: Some("C:/Apps/EmuDome".into()),
            cwd: Some("/w".into()),
            app_data: Some("C:/Users/u/AppData/Roaming".into()),
            xdg_data_home: None,
            home: Some("/home/u".into()),
        }
    }

    #[test]
    fn windows_looks_next_to_the_executable_then_in_roaming_app_data() {
        assert_eq!(
            user_dirs(Layout::Windows, &places()),
            vec![
                PathBuf::from("C:/Apps/EmuDome/user"),
                PathBuf::from("C:/Users/u/AppData/Roaming/Azahar"),
            ]
        );
        assert!(user_dirs(Layout::Windows, &Places::default()).is_empty());
    }

    #[test]
    fn linux_and_android_look_in_the_working_folder_then_the_xdg_data_folder() {
        assert_eq!(
            user_dirs(Layout::Xdg, &places()),
            vec![
                PathBuf::from("/w/user"),
                PathBuf::from("/home/u/.local/share/azahar-emu"),
            ]
        );
        let xdg = Places {
            xdg_data_home: Some("/x/data".into()),
            ..places()
        };
        assert_eq!(
            user_dirs(Layout::Xdg, &xdg)[1],
            PathBuf::from("/x/data/azahar-emu")
        );
        assert!(user_dirs(Layout::Xdg, &Places::default()).is_empty());
    }

    #[test]
    fn macos_adds_application_support() {
        assert_eq!(
            user_dirs(Layout::MacOs, &places()).last(),
            Some(&PathBuf::from("/home/u/Library/Application Support/Azahar"))
        );
    }
}
