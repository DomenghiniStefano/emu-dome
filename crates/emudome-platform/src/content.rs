//! Games given as a `content://` document (Android's document picker, «Open with»), made
//! loadable by a core without copying them (CLAUDE.md, engineering rule 4: a 3DS game is GBs).
//!
//! The system hands a file descriptor for the document. A core wants a path, with the game's
//! extension (cores and EmuDome pick the console by it), so the path given is a symbolic link in
//! the app's private cache, named after the document (`<cache>/content/<fd>/<name>`), pointing to
//! `/proc/self/fd/<fd>`: opening it opens the same file again, as long as the descriptor stays
//! open. The descriptor and the link live as long as the game ([`crate::ContentFile`]).
//!
//! What it cannot do: a document that is not a file on the device (a pipe from a cloud
//! provider) cannot be read again from the start, so it is refused ([`crate::ContentError::NotLocal`]).
//! The pure parts are here, compiled and tested on every system; the system calls are in
//! `android.rs`.
#![cfg_attr(
    not(target_os = "android"),
    allow(dead_code, reason = "used by android.rs; tested on every system")
)]

use std::path::{Path, PathBuf};

/// Whether `given` is a `content://` URI rather than a filesystem path (the scheme is
/// case-insensitive).
pub fn is_content_uri(given: &str) -> bool {
    given
        .get(..CONTENT_SCHEME.len())
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case(CONTENT_SCHEME))
}

const CONTENT_SCHEME: &str = "content://";

/// The folder of the app's cache holding the links (removed whole at the first open of a
/// process: what a killed process left behind).
pub(crate) const LINKS_DIR: &str = "content";

/// The link's file name: the document's display name, made a single safe path component. The
/// extension is kept (the console is chosen by it); a missing or empty name gives `game`.
pub(crate) fn link_name(display: Option<&str>) -> String {
    let name: String = display
        .unwrap_or_default()
        .chars()
        .map(|c| match c {
            '/' | '\\' | '\0' => '_',
            c => c,
        })
        .collect();
    // No hidden file, never `.` or `..`.
    let name = name.trim_start_matches('.').trim();
    if name.is_empty() {
        "game".to_owned()
    } else {
        name.to_owned()
    }
}

/// The folder of one descriptor's link: one folder per descriptor, so two documents with the
/// same name never collide.
pub(crate) fn link_dir(cache: &Path, fd: i32) -> PathBuf {
    cache.join(LINKS_DIR).join(fd.to_string())
}

/// The path that opens the file of the descriptor `fd` again, in this process.
pub(crate) fn fd_path(fd: i32) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{fd}"))
}

#[cfg(test)]
mod tests {
    use super::{fd_path, is_content_uri, link_dir, link_name};
    use std::path::Path;

    #[test]
    fn a_content_uri_is_told_from_a_path() {
        assert!(is_content_uri(
            "content://com.android.externalstorage.documents/document/primary%3AROMs%2Fgame.gba"
        ));
        assert!(is_content_uri("CONTENT://media/external/file/12"));
        assert!(!is_content_uri("/storage/emulated/0/ROMs/game.gba"));
        assert!(!is_content_uri("C:\\ROMs\\content.gba"));
        assert!(!is_content_uri("file:///sdcard/game.gba"));
        assert!(!is_content_uri("content:/"));
        assert!(!is_content_uri(""));
    }

    #[test]
    fn the_link_keeps_the_name_and_extension_as_one_safe_component() {
        assert_eq!(
            link_name(Some("Pokemon Clone (Rev 1).gba")),
            "Pokemon Clone (Rev 1).gba"
        );
        assert_eq!(link_name(Some("a/b\\c\0d.nds")), "a_b_c_d.nds");
        assert_eq!(link_name(Some("../up.3ds")), "_up.3ds");
        assert_eq!(link_name(Some(".hidden.gb")), "hidden.gb");
        assert_eq!(link_name(Some("..")), "game");
        assert_eq!(link_name(Some("   ")), "game");
        assert_eq!(link_name(Some("")), "game");
        assert_eq!(link_name(None), "game");
    }

    #[test]
    fn each_descriptor_has_its_own_folder_and_proc_path() {
        let cache = Path::new("/data/user/0/app/cache");
        assert_eq!(
            link_dir(cache, 42),
            Path::new("/data/user/0/app/cache/content/42")
        );
        assert_ne!(link_dir(cache, 42), link_dir(cache, 43));
        assert_eq!(fd_path(7), Path::new("/proc/self/fd/7"));
    }
}
