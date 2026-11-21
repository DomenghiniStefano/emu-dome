//! Whether a 3DS game file is encrypted (E10). Azahar plays decrypted games only, and EmuDome
//! never decrypts (no keys, CLAUDE.md): an encrypted game is never added to the library and is
//! refused before the core sees it, with a message that says what to do. Identification only:
//! a flag in the container's header is read, nothing is ever decrypted.
//!
//! The rule reads the containers' own headers (3dbrew's NCSD and NCCH layouts):
//! - NCSD (`.3ds`, `.cci`): magic `NCSD` at 0x100; partition table at 0x120, (offset, size)
//!   pairs in media units of 0x200 bytes; partition 0 is the game's NCCH;
//! - NCCH (`.cxi`, `.app`, or inside an NCSD): magic `NCCH` at 0x100; flags at 0x188, whose
//!   byte 7 (0x18F) has bit 0x04 «NoCrypto» set when the content is stored decrypted.
//!
//! `.3dsx` and `.elf` are homebrew, never encrypted. Pure parsing on byte slices, with
//! [`check_file`] the thin reading shell around it.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Bytes of an NCSD or NCCH header the rule reads.
pub const HEADER_LEN: usize = 0x200;
/// The unit of the NCSD partition table.
pub const MEDIA_UNIT: u64 = 0x200;

/// What the header says about the game's content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encryption {
    /// Stored decrypted: Azahar can run it.
    Decrypted,
    /// Encrypted: refused with E10.
    Encrypted,
    /// No header to read (homebrew, another container, a file too short): the core decides.
    Unknown,
}

/// The container a file extension names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// `.3ds` / `.cci`: an NCSD card image.
    Ncsd,
    /// `.cxi` / `.app`: a bare NCCH.
    Ncch,
    /// `.3dsx`, `.elf` and the rest: nothing to check.
    Other,
}

impl Container {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "3ds" | "cci" => Self::Ncsd,
            "cxi" | "app" => Self::Ncch,
            _ => Self::Other,
        }
    }
}

/// The byte offset of partition 0 (the game's NCCH) in an NCSD header; `None` without the
/// `NCSD` magic.
pub fn ncsd_game_partition(header: &[u8]) -> Option<u64> {
    if header.get(0x100..0x104)? != b"NCSD" {
        return None;
    }
    let offset = u32::from_le_bytes(header.get(0x120..0x124)?.try_into().ok()?);
    Some(u64::from(offset) * MEDIA_UNIT)
}

/// Whether an NCCH header says its content is encrypted; `None` without the `NCCH` magic.
pub fn ncch_encrypted(header: &[u8]) -> Option<bool> {
    if header.get(0x100..0x104)? != b"NCCH" {
        return None;
    }
    const NO_CRYPTO: u8 = 0x04;
    Some(header.get(0x18F)? & NO_CRYPTO == 0)
}

/// The rule on a file's bytes: `read(offset)` returns [`HEADER_LEN`] bytes at `offset`, or `None`
/// past the end.
pub fn check(container: Container, mut read: impl FnMut(u64) -> Option<Vec<u8>>) -> Encryption {
    let ncch_at = match container {
        Container::Other => return Encryption::Unknown,
        Container::Ncch => Some(0),
        Container::Ncsd => read(0).as_deref().and_then(ncsd_game_partition),
    };
    let encrypted = ncch_at
        .and_then(&mut read)
        .as_deref()
        .and_then(ncch_encrypted);
    match encrypted {
        Some(true) => Encryption::Encrypted,
        Some(false) => Encryption::Decrypted,
        None => Encryption::Unknown,
    }
}

/// [`check`] on a file, by its extension. A file that cannot be read is `Unknown`: opening it
/// then reports the real error.
pub fn check_file(path: &Path) -> Encryption {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Ok(mut file) = File::open(path) else {
        return Encryption::Unknown;
    };
    check(Container::from_extension(&ext), |at| {
        let mut buf = vec![0u8; HEADER_LEN];
        file.seek(SeekFrom::Start(at)).ok()?;
        file.read_exact(&mut buf).ok()?;
        Some(buf)
    })
}

#[cfg(test)]
mod tests {
    use super::{check, ncch_encrypted, ncsd_game_partition, Container, Encryption, HEADER_LEN};

    /// An NCCH header with the given flags byte 7.
    fn ncch(flags7: u8) -> Vec<u8> {
        let mut h = vec![0u8; HEADER_LEN];
        h[0x100..0x104].copy_from_slice(b"NCCH");
        h[0x18F] = flags7;
        h
    }

    /// A file image: an NCSD header whose partition 0 starts at `units` media units, and `part`
    /// there.
    fn ncsd(units: u32, part: &[u8]) -> Vec<u8> {
        let at = units as usize * 0x200;
        let mut file = vec![0u8; at + part.len()];
        file[0x100..0x104].copy_from_slice(b"NCSD");
        file[0x120..0x124].copy_from_slice(&units.to_le_bytes());
        file[at..].copy_from_slice(part);
        file
    }

    fn reader(file: &[u8]) -> impl FnMut(u64) -> Option<Vec<u8>> + '_ {
        move |at| {
            let at = at as usize;
            file.get(at..at + HEADER_LEN).map(<[u8]>::to_vec)
        }
    }

    #[test]
    fn the_no_crypto_flag_tells_a_decrypted_ncch() {
        assert_eq!(ncch_encrypted(&ncch(0x04)), Some(false));
        assert_eq!(ncch_encrypted(&ncch(0x05)), Some(false));
        assert_eq!(ncch_encrypted(&ncch(0x00)), Some(true));
        assert_eq!(ncch_encrypted(&ncch(0x20)), Some(true), "seed crypto");
        assert_eq!(ncch_encrypted(&[0u8; HEADER_LEN]), None);
        assert_eq!(ncch_encrypted(&[0u8; 16]), None);
    }

    #[test]
    fn a_card_image_is_judged_by_its_game_partition() {
        assert_eq!(
            ncsd_game_partition(&ncsd(0x4000, &[])[..HEADER_LEN]),
            Some(0x80_0000)
        );
        let decrypted = ncsd(4, &ncch(0x04));
        let encrypted = ncsd(4, &ncch(0x00));
        assert_eq!(
            check(Container::Ncsd, reader(&decrypted)),
            Encryption::Decrypted
        );
        assert_eq!(
            check(Container::Ncsd, reader(&encrypted)),
            Encryption::Encrypted
        );
        // Partition 0 points past the end, or holds no NCCH.
        let short = ncsd(400, &[]);
        assert_eq!(check(Container::Ncsd, reader(&short)), Encryption::Unknown);
        let empty = ncsd(4, &[0u8; HEADER_LEN]);
        assert_eq!(check(Container::Ncsd, reader(&empty)), Encryption::Unknown);
    }

    #[test]
    fn a_bare_ncch_is_judged_by_itself() {
        assert_eq!(
            check(Container::Ncch, reader(&ncch(0x04))),
            Encryption::Decrypted
        );
        assert_eq!(
            check(Container::Ncch, reader(&ncch(0x01))),
            Encryption::Encrypted
        );
    }

    #[test]
    fn homebrew_and_other_files_are_not_checked() {
        assert_eq!(Container::from_extension("3DSX"), Container::Other);
        assert_eq!(Container::from_extension("CCI"), Container::Ncsd);
        assert_eq!(Container::from_extension("app"), Container::Ncch);
        assert_eq!(
            check(Container::Other, reader(&ncch(0))),
            Encryption::Unknown
        );
        // An NCSD extension on a file without the magic.
        assert_eq!(
            check(Container::Ncsd, reader(&ncch(0))),
            Encryption::Unknown
        );
    }
}
