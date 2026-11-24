//! Games given as a `content://` document (Android's document picker, «Open with»), made
//! loadable by a core without copying them (CLAUDE.md, engineering rule 4: a 3DS game is GBs).
//!
//! The system hands a file descriptor for the document. A core wants a path, with the game's
//! extension (cores and EmuDome pick the console by it), so the path given is a symbolic link in
//! the app's private cache, named after the document (`<cache>/content/<fd>/<name>`), pointing to
//! `/proc/self/fd/<fd>`: opening it opens the same file again, as long as the descriptor stays
//! open. The descriptor and the link live as long as the game ([`crate::ContentFile`]).
//!
//! Android 17 refuses opening `/proc/self/fd/<fd>` again for some providers (Downloads: the
//! descriptor is a FUSE file the app has no path permission for). So EmuDome itself reads the
//! descriptor (a duplicate of it, [`crate::ContentFile::reader`]): identification, hashing, the
//! icon and the bytes a core is given. The link is kept for its name and extension, and as the
//! path a core that opens the game by itself (Azahar, `need_fullpath`) is given, which works only
//! when the reopen does ([`crate::ContentFile::reopenable`]); otherwise that open is refused.
//!
//! What it cannot do: a document that is not a file on the device (a pipe from a cloud
//! provider) cannot be read again from the start, so it is refused ([`crate::ContentError::NotLocal`]).
//! A picked folder (`ACTION_OPEN_DOCUMENT_TREE`, Trello #47) is listed one folder per call into
//! the system ([`crate::list_document_tree`]); its documents open like any other.
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

/// `%XX` escapes decoded (UTF-8, lossy); a malformed escape is kept as it is.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        if let (b'%', Some(b)) = (bytes[i], hex) {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The name of a picked folder as its tree URI spells it, for when the provider cannot be asked
/// (the read permission was taken back, Trello #141): the last folder of the tree's document id
/// (`content://…/tree/primary%3ADownload%2FEmuDomeTest` is `EmuDomeTest`; an SD card's
/// `1234-5678:Games` is `Games`). `None` for a volume's root (`primary:`), and for an id that is
/// not `<volume>:<path>` (a provider whose ids are opaque, such as a cloud drive's): no name is
/// better than a made-up one.
pub fn document_tree_label(tree: &str) -> Option<String> {
    let (_, after) = tree.split_once("/tree/")?;
    let id = percent_decode(after.split('/').next().unwrap_or_default());
    let (volume, path) = id.split_once(':')?;
    let plain_volume = !volume.is_empty()
        && volume
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !plain_volume {
        return None;
    }
    path.split('/')
        .rev()
        .map(str::trim)
        .find(|segment| !segment.is_empty())
        .map(str::to_owned)
}

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

/// The path that opens the file of the descriptor `fd` again, in this process. Android 17 refuses
/// it for some providers' descriptors (Downloads: `Permission denied`): the descriptor itself is
/// what EmuDome reads ([`crate::ContentFile::reader`]); this path only for a core that opens the
/// game by itself, when it works ([`crate::ContentFile::reopenable`]).
pub(crate) fn fd_path(fd: i32) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{fd}"))
}

/// `SystemBridge.openDocument`'s failure codes (its `OPEN_*` constants): the kind of failure, so
/// the interface says a file is gone rather than unreadable (Trello #146).
const OPEN_NOT_FOUND: i32 = -2;
const OPEN_PERMISSION_DENIED: i32 = -3;

/// The kind of failure `SystemBridge.openDocument` answered with a negative `code`: -2 the
/// document is not there (`FileNotFoundException`), -3 the system refused it
/// (`SecurityException`: the permission was taken back), anything else (-1) another failure.
pub(crate) fn open_failure(code: i32) -> std::io::ErrorKind {
    match code {
        OPEN_NOT_FOUND => std::io::ErrorKind::NotFound,
        OPEN_PERMISSION_DENIED => std::io::ErrorKind::PermissionDenied,
        _ => std::io::ErrorKind::Other,
    }
}

/// Fields per child in `SystemBridge.listTree`'s answer: URI, name, size, mtime, kind.
const TREE_FIELDS: usize = 5;

/// `SystemBridge.listTree`'s flattened answer as entries. A size or time that is not a number
/// gives 0 (unknown); a trailing incomplete child is dropped.
pub(crate) fn parse_tree_children(fields: &[String]) -> Vec<crate::TreeEntry> {
    fields
        .as_chunks::<TREE_FIELDS>()
        .0
        .iter()
        .map(|[uri, name, size, mtime, kind]| crate::TreeEntry {
            uri: uri.clone(),
            name: name.clone(),
            size: size.parse().unwrap_or_default(),
            mtime: mtime.parse().unwrap_or_default(),
            dir: kind == "d",
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        document_tree_label, fd_path, is_content_uri, link_dir, link_name, open_failure,
        parse_tree_children, percent_decode,
    };
    use std::io::ErrorKind;
    use std::path::Path;

    #[test]
    fn a_tree_is_named_by_the_last_folder_of_its_id() {
        let ext = "content://com.android.externalstorage.documents/tree/";
        let label = |id: &str| document_tree_label(&format!("{ext}{id}"));
        assert_eq!(
            label("primary%3ADownload%2FEmuDomeTest").as_deref(),
            Some("EmuDomeTest")
        );
        assert_eq!(label("primary%3AROMs").as_deref(), Some("ROMs"));
        // Nested, with a trailing slash and spaces / non-ASCII letters escaped.
        assert_eq!(
            label("primary%3AGames%2FNintendo%20DS%2FGiochi%20%C3%A0%2F").as_deref(),
            Some("Giochi à")
        );
        // An SD card's volume id.
        assert_eq!(label("1234-5678%3AGames").as_deref(), Some("Games"));
        // A colon left unescaped reads the same (a slash cannot be: it ends the id).
        assert_eq!(label("primary:Roms").as_deref(), Some("Roms"));
        // A tree URI that names a document inside it: the tree's id counts.
        assert_eq!(
            label("primary%3AROMs/document/primary%3AROMs%2Fsub").as_deref(),
            Some("ROMs")
        );
        // The Downloads provider's raw paths.
        assert_eq!(
            document_tree_label(
                "content://com.android.providers.downloads.documents/tree/\
                 raw%3A%2Fstorage%2Femulated%2F0%2FDownload%2FGBA"
            )
            .as_deref(),
            Some("GBA")
        );
        // A volume's root, an empty id, an opaque id, no tree at all: no name.
        assert_eq!(label("primary%3A"), None);
        assert_eq!(label("primary%3A%2F"), None);
        assert_eq!(label(""), None);
        assert_eq!(label("downloads"), None);
        assert_eq!(label("acc%3D1%3Bdoc%3Dencoded%3Dabc%3Adef"), None);
        assert_eq!(label("%3AGames"), None);
        assert_eq!(
            document_tree_label("content://x/document/primary%3AA"),
            None
        );
        assert_eq!(document_tree_label(""), None);
    }

    #[test]
    fn percent_escapes_are_decoded() {
        assert_eq!(percent_decode("a%20b%2Fc"), "a b/c");
        assert_eq!(percent_decode("%C3%A8"), "è");
        // A malformed escape is kept as it is.
        assert_eq!(percent_decode("50%"), "50%");
        assert_eq!(percent_decode("a%2"), "a%2");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode(""), "");
    }

    #[test]
    fn an_open_failure_code_gives_its_kind() {
        assert_eq!(open_failure(-2), ErrorKind::NotFound);
        assert_eq!(open_failure(-3), ErrorKind::PermissionDenied);
        assert_eq!(open_failure(-1), ErrorKind::Other);
        // A code this build does not know (a newer bridge) is another failure.
        assert_eq!(open_failure(-9), ErrorKind::Other);
        assert_eq!(open_failure(i32::MIN), ErrorKind::Other);
    }

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
    fn a_tree_listing_becomes_entries() {
        let fields: Vec<String> = [
            "content://t/tree/a/document/a%2Fx.gba",
            "x.gba",
            "1048576",
            "1700000000000",
            "f",
            "content://t/tree/a/document/a%2Fsub",
            "sub",
            "0",
            "junk",
            "d",
            "content://t/tree/a/document/a%2Fcut",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
        let entries = parse_tree_children(&fields);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].uri, "content://t/tree/a/document/a%2Fx.gba");
        assert_eq!(entries[0].name, "x.gba");
        assert_eq!(
            (entries[0].size, entries[0].mtime),
            (1_048_576, 1_700_000_000_000)
        );
        assert!(!entries[0].dir);
        assert_eq!((entries[1].name.as_str(), entries[1].mtime), ("sub", 0));
        assert!(entries[1].dir);
        assert!(parse_tree_children(&[]).is_empty());
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
