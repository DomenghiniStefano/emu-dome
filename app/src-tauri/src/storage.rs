//! E13 «Telefono quasi pieno» (Trello #29, the owner's option C): how much room is left where
//! the app keeps its data, and whether that is little enough to warn. The interface asks at
//! startup, when the shell shows again after a game, and when the library changes (a game
//! downloaded); it shows the warning on the phone only, but the answer is the same everywhere.

use emudome_platform::DiskSpace;
use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

/// Below this much free room the warning shows, whatever the disk's size: a 3DS game is GBs,
/// and a save that cannot be written is lost.
pub const LOW_BYTES: u64 = 1024 * 1024 * 1024;

/// Below this share of the disk free (1 / 20 = 5 %) the warning shows too.
pub const LOW_SHARE_DIVISOR: u64 = 20;

/// The room on the data folder's disk. TypeScript: `StorageStatus` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/")]
pub struct StorageStatus {
    /// Bytes the app may still write there.
    pub free: u64,
    /// The disk's size in bytes.
    pub total: u64,
    /// Little enough to warn ([`is_low`]).
    pub low: bool,
}

/// Whether `free` bytes of `total` are little enough to warn: under 1 GiB, or under 5 % of the
/// disk.
pub fn is_low(free: u64, total: u64) -> bool {
    free < LOW_BYTES || free < total / LOW_SHARE_DIVISOR
}

impl From<DiskSpace> for StorageStatus {
    fn from(space: DiskSpace) -> Self {
        StorageStatus {
            free: space.free,
            total: space.total,
            low: is_low(space.free, space.total),
        }
    }
}

/// The room left where the saves are written (the synced data folder). `None` when the system
/// does not say (logged): no warning then, since nothing the user can do follows from it.
#[tauri::command]
pub fn storage_status(app: AppHandle) -> Option<StorageStatus> {
    let dir = match crate::dirs::data(&app) {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("storage: no data folder: {e}");
            return None;
        }
    };
    match emudome_platform::disk_space(&dir) {
        Ok(space) => Some(space.into()),
        Err(e) => {
            log::warn!("storage: room on {} not read: {e}", dir.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_low, StorageStatus, LOW_BYTES};
    use emudome_platform::DiskSpace;

    const GIB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn under_one_gib_free_is_low_on_any_disk() {
        assert!(is_low(380 * 1024 * 1024, 128 * GIB));
        assert!(is_low(LOW_BYTES - 1, 8 * GIB));
        assert!(!is_low(LOW_BYTES, 8 * GIB));
    }

    #[test]
    fn under_five_percent_free_is_low_on_a_large_disk() {
        // 128 GiB: 5 % is 6.4 GiB.
        assert!(is_low(6 * GIB, 128 * GIB));
        assert!(!is_low(7 * GIB, 128 * GIB));
    }

    #[test]
    fn an_empty_disk_report_is_low() {
        assert!(is_low(0, 0));
    }

    #[test]
    fn the_status_carries_the_decision() {
        let s = StorageStatus::from(DiskSpace {
            free: 40 * GIB,
            total: 128 * GIB,
        });
        assert_eq!(
            s,
            StorageStatus {
                free: 40 * GIB,
                total: 128 * GIB,
                low: false
            }
        );
    }
}
