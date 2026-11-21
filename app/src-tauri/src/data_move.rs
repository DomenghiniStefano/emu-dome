//! The one-time copy of the data to the new app identifier's folders
//! (`emudome_core::data_move`, Trello #90), run at the very start of `setup`: before the window
//! (whose webview profile lives in the local data folder) and before anything opens a data file.
//!
//! - Never with `EMUDOME_DATA_DIR` (a test or bench run never touches the real folders), and
//!   not where the old folders cannot be reached (Android: `emudome_platform::old_data_dirs`).
//! - A copy that did not finish makes this run use the old folders (`dirs::use_old`); the
//!   problem reaches the interface as a health code (`dataMoveFailed`, `dataMoveConflict`).
//! - It runs on the main thread and the window appears after it: a large data folder makes the
//!   first start of the new build slower, once.

use std::path::Path;
use std::time::SystemTime;

use emudome_core::data_move::{self, Folder, Plan};
use emudome_core::ErrorCode;
use emudome_platform::DataDirs;
use tauri::Manager;

/// Runs the copy if it is due; the problem to show, if any.
pub fn run(app: &tauri::App) -> Option<ErrorCode> {
    let path = app.path();
    let (data, local) = match (path.app_data_dir(), path.app_local_data_dir()) {
        (Ok(data), Ok(local)) => (data, local),
        (Err(e), _) | (_, Err(e)) => {
            // `dirs` fails the same way right after, and the app does not start.
            log::warn!("data move: no data folder: {e}");
            return None;
        }
    };
    let (plan, old) = plan(crate::dirs::is_overridden(), &data, &local)?;
    let now = crate::unix_ms(SystemTime::now()).unwrap_or_default();
    let outcome = data_move::run(&plan, now);
    log::info!(
        "data move: {:?}, {} folders copied, {} errors",
        outcome.report.kind,
        outcome.report.copied.len(),
        outcome.report.errors.len()
    );
    if outcome.use_old {
        crate::dirs::use_old(old);
    }
    outcome.problem
}

/// The copy to make and the old folders, or `None`: with `EMUDOME_DATA_DIR` (`overridden`), or
/// where the old folders cannot be reached.
fn plan(overridden: bool, data: &Path, local: &Path) -> Option<(Plan, DataDirs)> {
    if overridden {
        return None;
    }
    let old = emudome_platform::old_data_dirs(data, local)?;
    let plan = Plan {
        folders: vec![
            Folder {
                old: old.data.clone(),
                new: data.to_path_buf(),
            },
            Folder {
                old: old.local.clone(),
                new: local.to_path_buf(),
            },
        ],
        skip: emudome_platform::WEBVIEW_PROFILE_DIRS
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
    };
    Some((plan, old))
}

#[cfg(test)]
mod tests {
    use super::plan;
    use std::path::Path;

    #[test]
    fn emudome_data_dir_skips_the_copy() {
        let data = Path::new("base/Roaming/it.domehub.emudome");
        let local = Path::new("base/Local/it.domehub.emudome");
        assert!(plan(true, data, local).is_none());
        // Without it the copy is planned wherever the old folders can be reached.
        let planned = plan(false, data, local);
        assert_eq!(
            planned.is_some(),
            emudome_platform::old_data_dirs(data, local).is_some()
        );
        if let Some((plan, old)) = planned {
            assert_eq!(plan.folders[0].new, data);
            assert_eq!(plan.folders[0].old, old.data);
            assert_eq!(plan.folders[1].new, local);
            assert_eq!(plan.folders[1].old, old.local);
        }
    }
}
