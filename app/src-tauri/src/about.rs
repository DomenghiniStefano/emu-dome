//! The «Informazioni» page's actions (Trello #44): the licences (`about.lic`) and exporting the
//! log (`about.log`).
//!
//! The licences are never typed here: EmuDome's comes from the workspace's `Cargo.toml`, each
//! core's from `cores.lock` (compiled into `emudome-platform`, `shipped_cores`), and the other
//! components' list is `THIRD_PARTY.md`, generated from the dependency trees. Their texts are the
//! files the packages carry (`package/<platform>.json`, written by `scripts/cores.mjs`):
//! `LICENSE.txt` and `THIRD_PARTY.md` at the resources' root, the cores' under `cores/notices/`.
//! A development build has no resources folder of its own: it reads the same files in the
//! repository.

use std::path::PathBuf;

use emudome_core::ErrorCode;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

/// What a licence row is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub enum LicenceKind {
    /// EmuDome itself.
    App,
    /// An emulation core (its `name` is the core's).
    Core,
    /// The libraries EmuDome is built with (Rust crates, npm packages): `THIRD_PARTY.md`.
    Components,
}

/// One row of the licences page. TypeScript: `Licence` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub struct Licence {
    /// `emudome`, a core's id (`mgba`), or `components`.
    pub id: String,
    pub kind: LicenceKind,
    /// The core's or the app's name; empty for the components (the interface names them).
    pub name: String,
    /// The consoles a core runs (`GB, GBC, GBA`), empty otherwise.
    pub consoles: String,
    /// An SPDX expression; empty for the components (each has its own, in the file).
    pub spdx: String,
    pub version: String,
    /// Where the source is (the repository at the pinned commit for a core).
    pub source: String,
    /// The licence files, read with `about_licence_text`.
    pub files: Vec<String>,
}

/// The file EmuDome's licence is packaged as (`package/<platform>.json`).
const APP_LICENCE: &str = "LICENSE.txt";
/// The other components' list, packaged at the resources' root.
const COMPONENTS: &str = "THIRD_PARTY.md";

/// Every row, EmuDome first, then the cores this build ships, then the other components.
pub fn licences() -> Vec<Licence> {
    let mut rows = vec![Licence {
        id: "emudome".into(),
        kind: LicenceKind::App,
        name: "EmuDome".into(),
        consoles: String::new(),
        spdx: env!("CARGO_PKG_LICENSE").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        source: env!("CARGO_PKG_REPOSITORY").into(),
        files: vec![APP_LICENCE.into()],
    }];
    let platform = emudome_platform::CORES_PLATFORM.unwrap_or_default();
    rows.extend(
        emudome_platform::shipped_cores(platform)
            .into_iter()
            .map(|c| Licence {
                source: format!("{}/tree/{}", c.repo, c.commit),
                id: c.id,
                kind: LicenceKind::Core,
                name: c.name,
                consoles: c.consoles,
                spdx: c.spdx,
                version: c.version,
                files: c.notices,
            }),
    );
    rows.push(Licence {
        id: "components".into(),
        kind: LicenceKind::Components,
        name: String::new(),
        consoles: String::new(),
        spdx: String::new(),
        version: String::new(),
        source: env!("CARGO_PKG_REPOSITORY").into(),
        files: vec![COMPONENTS.into()],
    });
    rows
}

/// Where `file` of row `id` is packaged, relative to the resources' root; `None` for a file the
/// row does not list (no other path is ever read).
fn packaged_path(rows: &[Licence], id: &str, file: &str) -> Option<String> {
    let row = rows.iter().find(|r| r.id == id)?;
    if !row.files.iter().any(|f| f == file) {
        return None;
    }
    Some(match row.kind {
        LicenceKind::App | LicenceKind::Components => file.to_owned(),
        LicenceKind::Core => format!("cores/notices/{file}"),
    })
}

/// The same file in the repository, for a development build (`cores/notices/<platform>/`).
fn repo_path(packaged: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    match packaged.strip_prefix("cores/notices/") {
        Some(file) => root
            .join("cores/notices")
            .join(emudome_platform::CORES_PLATFORM.unwrap_or_default())
            .join(file),
        None if packaged == APP_LICENCE => root.join("LICENSE"),
        None => root.join(packaged),
    }
}

/// The licences page's rows.
#[tauri::command]
pub fn about_licences() -> Vec<Licence> {
    licences()
}

/// The text of one licence file of a row.
#[tauri::command]
pub async fn about_licence_text(
    app: AppHandle,
    id: String,
    file: String,
) -> Result<String, ErrorCode> {
    crate::blocking(app, move |app| {
        let packaged =
            packaged_path(&licences(), &id, &file).ok_or_else(|| ErrorCode::LicenceUnreadable {
                detail: format!("{id}: no file {file}"),
            })?;
        let mut tried = Vec::new();
        if let Ok(dir) = app.path().resource_dir() {
            tried.push(dir.join(&packaged));
        }
        if cfg!(debug_assertions) {
            tried.push(repo_path(&packaged));
        }
        let mut last = String::from("no resources folder");
        for path in tried {
            match std::fs::read(&path) {
                Ok(bytes) => return Ok(String::from_utf8_lossy(&bytes).into_owned()),
                Err(e) => last = format!("{}: {e}", path.display()),
            }
        }
        Err(ErrorCode::LicenceUnreadable { detail: last })
    })
    .await?
}

/// `about.log`: this run's log file, saved where the user chooses. `None` when the dialog was
/// cancelled, else the name of the file written.
#[tauri::command]
pub async fn about_export_log(app: AppHandle) -> Result<Option<String>, ErrorCode> {
    crate::blocking(app, |app| {
        if emudome_platform::log_file::current().is_none() {
            return Err(ErrorCode::LogUnavailable);
        }
        let Some(dest) = emudome_platform::save_file(
            app,
            emudome_platform::log_file::FILE_NAME,
            "Log",
            &["log"],
        ) else {
            return Ok(None);
        };
        let failed = |e: std::io::Error| ErrorCode::LogExportFailed {
            detail: e.to_string(),
        };
        let bytes = emudome_platform::log_file::read_current().map_err(failed)?;
        emudome_core::write_atomic(&dest, &bytes).map_err(failed)?;
        log::info!("log exported to {}", dest.display());
        Ok(dest.file_name().map(|n| n.to_string_lossy().into_owned()))
    })
    .await?
}

/// Starts writing the log file in `<app local data>/logs/` (setup, once the folder is known).
pub fn attach_log(app: &AppHandle) {
    let Some(dir) = crate::dirs::local_or_log(app, "the log file") else {
        return;
    };
    match emudome_platform::log_file::attach(&dir.join("logs")) {
        Ok(path) => log::info!("log file: {}", path.display()),
        Err(e) => log::warn!("no log file in this run: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{licences, packaged_path, repo_path, LicenceKind};

    #[test]
    fn the_rows_are_the_app_the_shipped_cores_and_the_components() {
        let rows = licences();
        assert_eq!(rows[0].id, "emudome");
        assert_eq!(rows[0].spdx, "GPL-3.0-or-later");
        assert_eq!(rows.last().map(|r| r.kind), Some(LicenceKind::Components));
        for core in rows.iter().filter(|r| r.kind == LicenceKind::Core) {
            assert!(!core.spdx.is_empty(), "{}", core.id);
            assert!(core.source.contains("/tree/"), "{}", core.id);
            assert!(!core.files.is_empty(), "{}", core.id);
        }
    }

    #[test]
    fn only_the_files_a_row_lists_are_read() {
        let rows = licences();
        assert_eq!(
            packaged_path(&rows, "emudome", "LICENSE.txt").as_deref(),
            Some("LICENSE.txt")
        );
        assert_eq!(packaged_path(&rows, "emudome", "../secret"), None);
        assert_eq!(packaged_path(&rows, "nobody", "LICENSE.txt"), None);
        assert_eq!(
            packaged_path(&rows, "components", "THIRD_PARTY.md").as_deref(),
            Some("THIRD_PARTY.md")
        );
        if let Some(core) = rows.iter().find(|r| r.kind == LicenceKind::Core) {
            let file = &core.files[0];
            assert_eq!(
                packaged_path(&rows, &core.id, file),
                Some(format!("cores/notices/{file}"))
            );
        }
    }

    #[test]
    fn a_development_build_reads_the_repository_files() {
        assert!(repo_path("LICENSE.txt").ends_with("LICENSE"));
        assert!(repo_path("THIRD_PARTY.md").ends_with("THIRD_PARTY.md"));
        assert!(repo_path("LICENSE.txt").exists());
        assert!(repo_path("THIRD_PARTY.md").exists());
    }
}
