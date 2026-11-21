//! The user's own BIOS and firmware files (`bios.*` settings, Trello #68).
//!
//! EmuDome never ships, downloads or names a source for these files: the user picks a file they
//! own, EmuDome checks it and copies it into the core's own system folder (the folder the host
//! answers to `GET_SYSTEM_DIRECTORY`), under the name the core looks for. Without a file every
//! core uses its built-in replacement.
//!
//! # How it is split
//!
//! - this module: what a slot is ([`SlotSpec`]), the pure check of a file against it
//!   ([`verify`], [`identify`], [`assess`]) and what the settings row shows ([`status`]);
//! - one table per core, `bios/<core>.rs` (like `settings/core_values/<core>.rs`): its slots and
//!   its [`Policy`], the pure rule from the files in place and the settings to the core options
//!   it starts with ([`plan`]);
//! - [`files`]: the thin file layer (read a file into a [`Probe`], copy, delete, list what is in
//!   place).
//!
//! # Adding a core (extension point)
//!
//! Write `bios/<core>.rs` with `pub(super) static TABLE: BiosTable`, add it to [`TABLES`].

pub mod files;
mod melondsds;
mod mgba;

use md5::{Digest, Md5};
use serde::Serialize;
use ts_rs::TS;

use crate::settings::{ConsoleId, Value};
use crate::ErrorCode;

/// One file a core can use. TypeScript: `BiosSlot` (generated).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "BiosSlot")]
pub enum Slot {
    /// Game Boy Advance BIOS (mGBA).
    GbaBios,
    /// Game Boy boot ROM (mGBA).
    GbBoot,
    /// Game Boy Color boot ROM (mGBA).
    GbcBoot,
    /// Super Game Boy boot ROM (mGBA).
    SgbBoot,
    /// Nintendo DS ARM7 BIOS (melonDS DS).
    DsArm7,
    /// Nintendo DS ARM9 BIOS (melonDS DS).
    DsArm9,
    /// Nintendo DS firmware (melonDS DS).
    DsFirmware,
    /// Nintendo DSi ARM7 BIOS (melonDS DS, DSi mode).
    DsiArm7,
    /// Nintendo DSi ARM9 BIOS (melonDS DS, DSi mode).
    DsiArm9,
    /// Nintendo DSi firmware (melonDS DS, DSi mode).
    DsiFirmware,
    /// Nintendo DSi internal memory image (melonDS DS, DSi mode).
    DsiNand,
}

/// How a file is recognised as the one of a slot.
#[derive(Clone, Copy, Debug)]
pub enum Check {
    /// The whole file's MD5 (lower-case hex) is one of these: the values the libretro
    /// documentation of the core publishes. A hash identifies a file; it is not the file.
    Md5(&'static [&'static str]),
    /// A firmware dump: every console's differs (its own user settings and network address), so
    /// it has no hash. Its header is checked as the core checks it: console type at 0x1D is one
    /// of these, the two bytes at 0x1E are unused (0xFF).
    FirmwareHeader { console_types: &'static [u8] },
    /// A DSi internal memory image: unique to each console. It must end with the 64-byte
    /// «nocash» footer the core reads (magic [`NAND_FOOTER_MAGIC`]).
    NandFooter,
    /// No published hash: the size is the only check; the core checks the content itself.
    SizeOnly,
}

/// The magic at the start of a DSi memory image's 64-byte footer.
pub const NAND_FOOTER_MAGIC: &[u8; 16] = b"DSi eMMC CID/CPU";
/// The footer's length.
pub const NAND_FOOTER_LEN: usize = 64;
/// How many bytes of a file's start a [`Probe`] keeps (enough for a firmware header).
pub const HEAD_LEN: usize = 0x20;

/// One slot of a core.
#[derive(Clone, Copy, Debug)]
pub struct SlotSpec {
    pub slot: Slot,
    /// The settings row the user picks it with; `None`: no row yet (the slot is still known, so
    /// a file of it picked on another row is recognised).
    pub setting: Option<&'static str>,
    /// Where the core looks for it, relative to its system folder (`/` separated).
    pub file: &'static str,
    /// Accepted sizes in bytes; empty: any.
    pub sizes: &'static [u64],
    pub check: Check,
}

/// What a core does with the files in place: core options to start with (they win over the
/// settings' and the forced ones) and the codes of what it could not do.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub options: Vec<(&'static str, &'static str)>,
    /// What was asked and fell back (e.g. DSi mode without the DSi files), for the interface.
    pub fallbacks: Vec<ErrorCode>,
}

/// A core's rule: the slots in place (valid), the settings of the game, its console → [`Plan`].
pub type Policy = fn(&[Slot], &dyn Fn(&str) -> Option<Value>, ConsoleId) -> Plan;

/// The slots of one core and its policy.
#[derive(Debug)]
pub struct BiosTable {
    /// The core's name in the shell (`mgba`, `melondsds`).
    pub core: &'static str,
    pub slots: &'static [SlotSpec],
    pub policy: Policy,
}

/// Every core's table.
pub static TABLES: &[&BiosTable] = &[&mgba::TABLE, &melondsds::TABLE];

/// The table of `core`, if it has one.
pub fn table(core: &str) -> Option<&'static BiosTable> {
    TABLES.iter().copied().find(|t| t.core == core)
}

/// The slot a settings row picks, and its core.
pub fn slot_for_setting(setting: &str) -> Option<(&'static BiosTable, &'static SlotSpec)> {
    TABLES.iter().copied().find_map(|t| {
        t.slots
            .iter()
            .find(|s| s.setting == Some(setting))
            .map(|s| (t, s))
    })
}

/// What [`verify`] needs of a file: its size, its first and last bytes, its MD5 when the file is
/// small enough to hash ([`files::probe`] reads it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    pub len: u64,
    /// Up to [`HEAD_LEN`] bytes.
    pub head: Vec<u8>,
    /// Up to [`NAND_FOOTER_LEN`] bytes.
    pub tail: Vec<u8>,
    /// Lower-case hex; `None` for a file too large to be a BIOS.
    pub md5: Option<String>,
}

impl Probe {
    /// The probe of a whole file held in memory (tests, small files).
    pub fn of(bytes: &[u8]) -> Self {
        Self {
            len: bytes.len() as u64,
            head: bytes[..bytes.len().min(HEAD_LEN)].to_vec(),
            tail: bytes[bytes.len().saturating_sub(NAND_FOOTER_LEN)..].to_vec(),
            md5: Some(md5_hex(bytes)),
        }
    }
}

/// The MD5 of `bytes`, lower-case hex.
pub fn md5_hex(bytes: &[u8]) -> String {
    Md5::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A file checked against one slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// It is the slot's file (by hash, header or, without a published hash, size).
    Valid,
    /// Right size, but not a file EmuDome knows for the slot.
    Unknown,
    /// Not a size the slot's file can have.
    WrongSize,
}

/// Checks a file against a slot.
pub fn verify(spec: &SlotSpec, probe: &Probe) -> Verdict {
    if !spec.sizes.is_empty() && !spec.sizes.contains(&probe.len) {
        return Verdict::WrongSize;
    }
    let ok = match spec.check {
        Check::Md5(known) => probe
            .md5
            .as_deref()
            .is_some_and(|h| known.iter().any(|k| k.eq_ignore_ascii_case(h))),
        Check::FirmwareHeader { console_types } => {
            probe.head.len() >= HEAD_LEN
                && console_types.contains(&probe.head[0x1D])
                && probe.head[0x1E..0x20] == [0xFF, 0xFF]
        }
        Check::NandFooter => {
            probe.len > NAND_FOOTER_LEN as u64
                && probe.tail.len() == NAND_FOOTER_LEN
                && probe.tail.starts_with(NAND_FOOTER_MAGIC)
        }
        Check::SizeOnly => true,
    };
    if ok {
        Verdict::Valid
    } else {
        Verdict::Unknown
    }
}

/// The slot a file is, among slots with a hash (a size or a header alone does not tell one
/// file from another).
pub fn identify<'a>(slots: impl IntoIterator<Item = &'a SlotSpec>, probe: &Probe) -> Option<Slot> {
    slots
        .into_iter()
        .filter(|s| matches!(s.check, Check::Md5(_)))
        .find(|s| verify(s, probe) == Verdict::Valid)
        .map(|s| s.slot)
}

/// What to do with a file the user picked for a slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Assessment {
    /// Copy it into the core's system folder: the core uses it.
    Install,
    /// Keep the choice but do not give it to the core («not recognised: built-in in use»).
    KeepUnrecognised,
    /// Refuse it: the setting does not change.
    Refuse(ErrorCode),
}

/// Decides what a picked file is for `spec`, knowing every core's slots (`all`) to say when it
/// is the file of another slot.
pub fn assess<'a>(
    spec: &SlotSpec,
    all: impl IntoIterator<Item = &'a SlotSpec>,
    probe: &Probe,
) -> Assessment {
    let verdict = verify(spec, probe);
    if verdict == Verdict::Valid {
        return Assessment::Install;
    }
    if let Some(other) = identify(all, probe).filter(|s| *s != spec.slot) {
        return Assessment::Refuse(ErrorCode::BiosOtherSlot { slot: other });
    }
    match verdict {
        Verdict::WrongSize => Assessment::Refuse(ErrorCode::BiosWrongSize { size: probe.len }),
        Verdict::Unknown | Verdict::Valid => Assessment::KeepUnrecognised,
    }
}

/// What a `bios.*` row shows. Mirror: `BiosStatus` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    /// No file chosen: the core's replacement.
    Builtin,
    /// The user's file is in place and valid.
    Verified,
    /// A file is chosen but not in place (not recognised): the core's replacement.
    Unrecognised,
}

/// The row's status from its value (the chosen file, empty = none) and whether a valid copy is
/// in place.
pub fn status(chosen: bool, in_place: bool) -> Status {
    match (chosen, in_place) {
        (false, _) => Status::Builtin,
        (true, true) => Status::Verified,
        (true, false) => Status::Unrecognised,
    }
}

/// The options and fallbacks of `core` for a game of `console`, with the valid files in place
/// (`in_place`) and the game's settings (`value`). A core without a table: nothing.
pub fn plan(
    core: &str,
    console: ConsoleId,
    in_place: &[Slot],
    value: &dyn Fn(&str) -> Option<Value>,
) -> Plan {
    table(core)
        .map(|t| (t.policy)(in_place, value, console))
        .unwrap_or_default()
}

/// A text setting's value, `None` for any other kind.
fn text(value: &dyn Fn(&str) -> Option<Value>, key: &str) -> Option<String> {
    match value(key) {
        Some(Value::Text(t)) => Some(t),
        Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => None,
    }
}

#[cfg(test)]
mod tests;
