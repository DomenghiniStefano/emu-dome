//! How much room is left on a disk (E13 «Telefono quasi pieno», Trello #29). One API for every
//! system: `fs2` asks Windows (`GetDiskFreeSpaceExW`) or the Unix `statvfs` (Linux, Android)
//! for the volume holding the path. Read only.

use std::io;
use std::path::Path;

/// The room on the volume that holds a path, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskSpace {
    /// What this process may still write (the user's quota, not the root's reserve).
    pub free: u64,
    /// The volume's size.
    pub total: u64,
}

/// The room on the volume that holds `path`, an existing file or folder (Windows answers for a
/// missing path on an existing drive too; the Unix systems refuse it).
pub fn disk_space(path: &Path) -> io::Result<DiskSpace> {
    Ok(DiskSpace {
        free: fs2::available_space(path)?,
        total: fs2::total_space(path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::disk_space;

    #[test]
    fn the_folder_of_this_crate_has_a_size_and_some_room() {
        let space = disk_space(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(space.total > 0);
        assert!(space.free <= space.total);
    }
}
