//! Synthetic data only: no real BIOS bytes. The lookup is tested on fake slots with the MD5 of
//! made-up bytes; the real tables only for their shape.

use std::collections::HashSet;

use super::{
    assess, files, identify, md5_hex, plan, slot_for_setting, status, verify, Assessment,
    BiosTable, Check, Probe, Slot, SlotSpec, Status, Verdict, NAND_FOOTER_LEN, NAND_FOOTER_MAGIC,
    TABLES,
};
use crate::core_profile::CoreId;
use crate::settings::test_dir::TestDir;
use crate::settings::{entry, ConsoleId, Kind, Value};
use crate::ErrorCode;

/// Made-up content: `len` bytes from a seed.
fn fake(seed: u8, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| seed.wrapping_add((i % 251) as u8))
        .collect()
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn hashed(slot: Slot, bytes: &[u8]) -> SlotSpec {
    SlotSpec {
        slot,
        setting: None,
        file: "x.bin",
        sizes: &[16],
        check: Check::Md5(Box::leak(Box::new([leak(md5_hex(bytes))]))),
    }
}

#[test]
fn a_known_hash_of_the_right_size_is_valid() {
    let file = fake(1, 16);
    let spec = hashed(Slot::GbaBios, &file);
    assert_eq!(verify(&spec, &Probe::of(&file)), Verdict::Valid);
    assert_eq!(verify(&spec, &Probe::of(&fake(2, 16))), Verdict::Unknown);
    assert_eq!(verify(&spec, &Probe::of(&fake(1, 17))), Verdict::WrongSize);
}

#[test]
fn identify_finds_the_slot_by_hash_only() {
    let a = fake(1, 16);
    let b = fake(2, 16);
    let slots = [
        hashed(Slot::GbaBios, &a),
        hashed(Slot::DsArm7, &b),
        SlotSpec {
            slot: Slot::DsiArm7,
            setting: None,
            file: "y.bin",
            sizes: &[16],
            check: Check::SizeOnly,
        },
    ];
    assert_eq!(identify(&slots, &Probe::of(&b)), Some(Slot::DsArm7));
    // Right size for the size-only slot, but a size tells nothing: no slot.
    assert_eq!(identify(&slots, &Probe::of(&fake(3, 16))), None);
}

#[test]
fn assess_installs_keeps_or_refuses() {
    let a = fake(1, 16);
    let b = fake(2, 16);
    let slots = [hashed(Slot::GbaBios, &a), hashed(Slot::DsArm7, &b)];
    assert_eq!(
        assess(&slots[0], &slots, &Probe::of(&a)),
        Assessment::Install
    );
    assert_eq!(
        assess(&slots[0], &slots, &Probe::of(&b)),
        Assessment::Refuse(ErrorCode::BiosOtherSlot { slot: Slot::DsArm7 })
    );
    assert_eq!(
        assess(&slots[0], &slots, &Probe::of(&fake(9, 16))),
        Assessment::KeepUnrecognised
    );
    assert_eq!(
        assess(&slots[0], &slots, &Probe::of(&fake(9, 3))),
        Assessment::Refuse(ErrorCode::BiosWrongSize { size: 3 })
    );
}

fn firmware(console_type: u8, unused: u8) -> Vec<u8> {
    let mut f = vec![0; 64];
    f[0x1D] = console_type;
    f[0x1E] = unused;
    f[0x1F] = unused;
    f
}

#[test]
fn a_firmware_is_checked_by_its_header() {
    let spec = SlotSpec {
        slot: Slot::DsFirmware,
        setting: None,
        file: "f.bin",
        sizes: &[64],
        check: Check::FirmwareHeader {
            console_types: &[0xFF, 0x20],
        },
    };
    assert_eq!(
        verify(&spec, &Probe::of(&firmware(0x20, 0xFF))),
        Verdict::Valid
    );
    assert_eq!(
        verify(&spec, &Probe::of(&firmware(0x12, 0xFF))),
        Verdict::Unknown
    );
    assert_eq!(
        verify(&spec, &Probe::of(&firmware(0xFF, 0x00))),
        Verdict::Unknown
    );
}

#[test]
fn a_memory_image_is_checked_by_its_footer() {
    let spec = SlotSpec {
        slot: Slot::DsiNand,
        setting: None,
        file: "n.bin",
        sizes: &[],
        check: Check::NandFooter,
    };
    let mut image = fake(4, 1000);
    assert_eq!(verify(&spec, &Probe::of(&image)), Verdict::Unknown);
    let at = image.len() - NAND_FOOTER_LEN;
    image[at..at + 16].copy_from_slice(NAND_FOOTER_MAGIC);
    assert_eq!(verify(&spec, &Probe::of(&image)), Verdict::Valid);
}

#[test]
fn status_follows_the_choice_and_the_copy() {
    assert_eq!(status(false, false), Status::Builtin);
    assert_eq!(status(false, true), Status::Builtin);
    assert_eq!(status(true, true), Status::Verified);
    assert_eq!(status(true, false), Status::Unrecognised);
}

#[test]
fn the_tables_are_well_formed() {
    let mut slots = HashSet::new();
    let mut files = HashSet::new();
    let mut hashes = HashSet::new();
    for table in TABLES {
        for spec in table.slots {
            assert!(slots.insert(spec.slot), "{:?} twice", spec.slot);
            assert!(files.insert((table.core, spec.file)), "{} twice", spec.file);
            assert!(!spec.file.starts_with('/') && !spec.file.contains(".."));
            match spec.check {
                Check::Md5(known) => {
                    assert!(
                        !known.is_empty() && spec.sizes.len() == 1,
                        "{:?}",
                        spec.slot
                    );
                    for h in known {
                        assert!(h.len() == 32 && h.bytes().all(|b| b.is_ascii_hexdigit()));
                        assert!(hashes.insert(*h), "{h} twice");
                    }
                }
                Check::FirmwareHeader { console_types } => assert!(!console_types.is_empty()),
                Check::NandFooter => {}
                Check::SizeOnly => assert!(!spec.sizes.is_empty()),
            }
            // A row of a slot is a `bios.*` file row of the catalogue, and picks one slot only.
            if let Some(key) = spec.setting {
                assert!(
                    entry(key).is_some_and(|e| key.starts_with("bios.") && e.kind == Kind::File),
                    "{key}"
                );
                assert_eq!(slot_for_setting(key).map(|(_, s)| s.slot), Some(spec.slot));
            }
        }
    }
}

fn settings(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<Value> {
    move |key| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| Value::Text((*v).to_owned()))
    }
}

fn option(p: &super::Plan, key: &str) -> Option<&'static str> {
    p.options
        .iter()
        .rev()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

const DS: &[Slot] = &[Slot::DsArm7, Slot::DsArm9, Slot::DsFirmware];
const DS_AND_DSI: &[Slot] = &[
    Slot::DsArm7,
    Slot::DsArm9,
    Slot::DsFirmware,
    Slot::DsiArm7,
    Slot::DsiArm9,
    Slot::DsiFirmware,
    Slot::DsiNand,
];

fn nds(in_place: &[Slot], pairs: &'static [(&'static str, &'static str)]) -> super::Plan {
    plan(
        CoreId::MelonDsDs,
        ConsoleId::Nds,
        in_place,
        &settings(pairs),
    )
}

#[test]
fn melonds_uses_the_built_in_files_until_every_ds_file_is_in_place() {
    let none = nds(&[], &[("consoles.dsmode", "ds")]);
    assert_eq!(option(&none, "melonds_sysfile_mode"), Some("builtin"));
    assert!(none.fallbacks.is_empty());
    let partial = nds(&DS[..2], &[("consoles.dsmode", "ds")]);
    assert_eq!(option(&partial, "melonds_sysfile_mode"), Some("builtin"));
    let all = nds(DS, &[("consoles.dsmode", "ds")]);
    assert_eq!(option(&all, "melonds_sysfile_mode"), Some("native"));
    assert_eq!(
        option(&all, "melonds_firmware_nds_path"),
        Some("melonDS DS/firmware.bin")
    );
}

#[test]
fn dsi_mode_without_its_files_runs_as_a_ds_and_says_so() {
    let p = nds(DS, &[("consoles.dsmode", "dsi")]);
    assert_eq!(option(&p, "melonds_console_mode"), Some("ds"));
    assert_eq!(p.fallbacks, vec![ErrorCode::BiosDsiFilesMissing]);
    // `auto` (or no value) becomes DS silently.
    let auto = nds(&[], &[("consoles.dsmode", "auto")]);
    assert_eq!(option(&auto, "melonds_console_mode"), Some("ds"));
    assert!(auto.fallbacks.is_empty());
    assert_eq!(option(&nds(&[], &[]), "melonds_console_mode"), Some("ds"));
    // Every file: the setting's own mode stays, the DSi paths are given.
    let full = nds(DS_AND_DSI, &[("consoles.dsmode", "dsi")]);
    assert_eq!(option(&full, "melonds_console_mode"), None);
    assert_eq!(
        option(&full, "melonds_dsi_nand_path"),
        Some("melonDS DS/dsi_nand.bin")
    );
    assert!(full.fallbacks.is_empty());
    // DSi files without the DS ones are not enough.
    let no_ds = nds(&DS_AND_DSI[3..], &[("consoles.dsmode", "dsi")]);
    assert_eq!(no_ds.fallbacks, vec![ErrorCode::BiosDsiFilesMissing]);
}

#[test]
fn the_ds_menu_without_the_ds_files_boots_directly_and_says_so() {
    let p = nds(&[], &[("consoles.dsmode", "ds"), ("cores.mdsboot", "menu")]);
    assert_eq!(option(&p, "melonds_boot_mode"), Some("direct"));
    assert_eq!(p.fallbacks, vec![ErrorCode::BiosDsMenuFilesMissing]);
    let ok = nds(DS, &[("consoles.dsmode", "ds"), ("cores.mdsboot", "menu")]);
    assert_eq!(option(&ok, "melonds_boot_mode"), None);
    assert!(ok.fallbacks.is_empty());
}

#[test]
fn mgba_and_unknown_cores_set_nothing() {
    let p = plan(CoreId::Mgba, ConsoleId::Gba, &[], &settings(&[]));
    assert_eq!(p, super::Plan::default());
    assert_eq!(
        plan(CoreId::Azahar, ConsoleId::N3ds, &[], &settings(&[])),
        super::Plan::default()
    );
}

#[test]
fn install_copies_remove_deletes_only_the_copy_and_invalid_copies_go() {
    let tmp = TestDir::new("bios-files");
    let dir = tmp.path("");
    let user = dir.join("mine.rom");
    let bytes = fake(5, 16);
    std::fs::write(&user, &bytes).unwrap();
    let good = hashed(Slot::GbaBios, &bytes);
    let mut other = hashed(Slot::GbBoot, &fake(6, 16));
    other.file = "sub/other.bin";
    let slots: &'static [SlotSpec] = Box::leak(Box::new([good, other]));
    let table = BiosTable {
        core: CoreId::Mgba,
        slots,
        policy: |_, _, _| super::Plan::default(),
    };
    let core = files::core_dir(&dir.join("system"), "test");

    assert_eq!(files::probe(&user).unwrap(), Probe::of(&bytes));
    files::install(&core, &slots[0], &user).unwrap();
    assert_eq!(std::fs::read(core.join("x.bin")).unwrap(), bytes);
    assert!(files::is_in_place(&core, &slots[0]));
    // Not the other slot's file: refused, nothing written.
    let refused = files::install(&core, &slots[1], &user).unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::InvalidData);
    assert!(!core.join("sub").exists());
    // A copy that changed on disk (no longer valid) is deleted when the game opens.
    crate::write_atomic(&core.join("sub").join("other.bin"), &bytes).unwrap();
    assert_eq!(files::in_place(&core, &table), vec![Slot::GbaBios]);
    assert!(!core.join("sub").join("other.bin").exists());

    files::remove(&core, &slots[0]).unwrap();
    files::remove(&core, &slots[0]).unwrap(); // nothing left: not an error
    assert!(!core.join("x.bin").exists());
    assert_eq!(
        std::fs::read(&user).unwrap(),
        bytes,
        "the user's file stays"
    );
    assert!(files::in_place(&core, &table).is_empty());
    // Only the copy ever went: the user's file and EmuDome's folders, no temporary file left.
    let mut left: Vec<_> = std::fs::read_dir(&core)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    left.sort();
    assert_eq!(left, vec![std::ffi::OsString::from("sub")]);
}

#[test]
fn a_large_file_is_probed_by_its_ends() {
    let dir = TestDir::new("bios-large");
    let path = dir.path("image.bin");
    let mut bytes = fake(7, 1024 * 1024 + 100);
    let at = bytes.len() - NAND_FOOTER_LEN;
    bytes[at..at + 16].copy_from_slice(NAND_FOOTER_MAGIC);
    std::fs::write(&path, &bytes).unwrap();
    let p = files::probe(&path).unwrap();
    assert_eq!(p.md5, None);
    assert_eq!(p.len, bytes.len() as u64);
    assert!(p.tail.starts_with(NAND_FOOTER_MAGIC));
    // Installed whole, checked by its footer.
    let spec = SlotSpec {
        slot: Slot::DsiNand,
        setting: None,
        file: "melonDS DS/n.bin",
        sizes: &[],
        check: Check::NandFooter,
    };
    let core = dir.path("system");
    files::install(&core, &spec, &path).unwrap();
    assert_eq!(
        std::fs::read(core.join("melonDS DS").join("n.bin")).unwrap(),
        bytes
    );
    assert!(files::is_in_place(&core, &spec));
}

#[test]
fn the_wired_rows_are_the_ones_the_interface_lists() {
    // Mirror: `BIOS_ROWS` in app/src/lib/settingsModel.ts (the rows whose picker checks the file).
    let mut rows: Vec<&str> = TABLES
        .iter()
        .flat_map(|t| t.slots.iter().filter_map(|s| s.setting))
        .collect();
    rows.sort_unstable();
    assert_eq!(
        rows,
        [
            "bios.bds7",
            "bios.bds9",
            "bios.bdsfw",
            "bios.bdsi",
            "bios.bdsi7",
            "bios.bdsi9",
            "bios.bdsifw",
            "bios.bgb",
            "bios.bgba",
            "bios.bgbc"
        ]
    );
}

#[test]
fn every_bios_file_row_picks_a_slot_of_the_core_that_reads_it() {
    // Trello #89: the Game Boy / Color boot ROMs go to mGBA, the DSi files to melonDS DS (whose
    // DSi mode needs them all, `dsi_mode_without_its_files_runs_as_a_ds_and_says_so`).
    for (key, core, slot) in [
        ("bios.bgb", CoreId::Mgba, Slot::GbBoot),
        ("bios.bgbc", CoreId::Mgba, Slot::GbcBoot),
        ("bios.bgba", CoreId::Mgba, Slot::GbaBios),
        ("bios.bds7", CoreId::MelonDsDs, Slot::DsArm7),
        ("bios.bds9", CoreId::MelonDsDs, Slot::DsArm9),
        ("bios.bdsfw", CoreId::MelonDsDs, Slot::DsFirmware),
        ("bios.bdsi7", CoreId::MelonDsDs, Slot::DsiArm7),
        ("bios.bdsi9", CoreId::MelonDsDs, Slot::DsiArm9),
        ("bios.bdsifw", CoreId::MelonDsDs, Slot::DsiFirmware),
        ("bios.bdsi", CoreId::MelonDsDs, Slot::DsiNand),
    ] {
        let (table, spec) = slot_for_setting(key).unwrap_or_else(|| panic!("{key}: no slot"));
        assert_eq!((table.core, spec.slot), (core, slot), "{key}");
    }
    // Every wired file row of the BIOS page has its slot; the one without is shown as not
    // wired yet (3DS).
    for e in crate::settings::page_entries("bios") {
        assert_eq!(
            e.wired,
            slot_for_setting(e.key).is_some(),
            "{}: wired only with a slot",
            e.key
        );
    }
}
