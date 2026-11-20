//! The file side of [`super`]: read a picked file into a [`Probe`], put a copy where the core
//! looks for it, delete that copy, list the valid copies in place. Only EmuDome's own copies in
//! the core's system folder are ever written or deleted; the user's file is only read.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::{verify, BiosTable, Probe, Slot, SlotSpec, Verdict, HEAD_LEN, NAND_FOOTER_LEN};

/// Files up to this size are hashed whole; a larger one cannot be a BIOS (a DSi memory image is
/// checked by its footer).
const MAX_HASHED: u64 = 1024 * 1024;

/// The system folder of `core` under `root` (the one the host answers to
/// `GET_SYSTEM_DIRECTORY` for that core).
pub fn core_dir(root: &Path, core: &str) -> PathBuf {
    root.join(core)
}

/// Where the copy of `spec` lives in a core's system folder.
pub fn copy_path(core_dir: &Path, spec: &SlotSpec) -> PathBuf {
    spec.file
        .split('/')
        .fold(core_dir.to_path_buf(), |p, part| p.join(part))
}

/// Reads what [`super::verify`] needs of a file.
pub fn probe(path: &Path) -> io::Result<Probe> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    if len <= MAX_HASHED {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        return Ok(Probe::of(&bytes));
    }
    let mut head = vec![0; HEAD_LEN];
    file.read_exact(&mut head)?;
    let mut tail = vec![0; NAND_FOOTER_LEN];
    file.seek(SeekFrom::End(-(NAND_FOOTER_LEN as i64)))?;
    file.read_exact(&mut tail)?;
    Ok(Probe {
        len,
        head,
        tail,
        md5: None,
    })
}

/// Copies `source` to where the core looks for `spec`, through [`crate::write_atomic`] (the core
/// never sees half a file). The bytes copied are checked again: a file that changed since it was
/// probed, and is no longer the slot's, is refused (`InvalidData`) and nothing is written.
pub fn install(core_dir: &Path, spec: &SlotSpec, source: &Path) -> io::Result<()> {
    let bytes = std::fs::read(source)?;
    let probe = probe_bytes(&bytes);
    if verify(spec, &probe) != Verdict::Valid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("not the {:?} file", spec.slot),
        ));
    }
    crate::write_atomic(&copy_path(core_dir, spec), &bytes)
}

/// The probe of bytes in memory, hashed only when small enough to be a BIOS (as [`probe`]).
fn probe_bytes(bytes: &[u8]) -> Probe {
    if bytes.len() as u64 <= MAX_HASHED {
        return Probe::of(bytes);
    }
    Probe {
        len: bytes.len() as u64,
        head: bytes[..HEAD_LEN].to_vec(),
        tail: bytes[bytes.len() - NAND_FOOTER_LEN..].to_vec(),
        md5: None,
    }
}

/// Deletes the copy of `spec`, if there is one.
pub fn remove(core_dir: &Path, spec: &SlotSpec) -> io::Result<()> {
    match std::fs::remove_file(copy_path(core_dir, spec)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        Ok(()) | Err(_) => Ok(()),
    }
}

/// Whether a valid copy of `spec` is in place.
pub fn is_in_place(core_dir: &Path, spec: &SlotSpec) -> bool {
    let path = copy_path(core_dir, spec);
    path.is_file() && probe(&path).is_ok_and(|p| verify(spec, &p) == Verdict::Valid)
}

/// The slots of `table` with a valid copy in place. A copy that is no longer valid (changed on
/// disk) is deleted, so the core never loads it.
pub fn in_place(core_dir: &Path, table: &BiosTable) -> Vec<Slot> {
    let mut out = Vec::new();
    for spec in table.slots {
        if !copy_path(core_dir, spec).is_file() {
            continue;
        }
        if is_in_place(core_dir, spec) {
            out.push(spec.slot);
        } else {
            log::warn!("{:?}: the copy is no longer valid, deleted", spec.slot);
            if let Err(e) = remove(core_dir, spec) {
                log::warn!("{:?}: {e}", spec.slot);
            }
        }
    }
    out
}
