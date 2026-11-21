//! melonDS DS's system files, in the `melonDS DS` folder of its system folder (where it looks
//! first). Option keys and values as melonDS DS 1.4.0 declares them; the firmware and memory
//! image options take the file's path relative to the system folder (measured with
//! `headless --list-options`: a valid firmware in `melonDS DS/firmware.bin` is offered as
//! `melonDS DS/firmware.bin`).
//!
//! The rule ([`policy`]):
//! - the DS BIOS (ARM7, ARM9) and firmware all in place → `melonds_sysfile_mode` = `native`;
//!   anything missing → `builtin` (FreeBIOS and a generated firmware), which boots DS games;
//! - DSi mode needs those plus the DSi BIOS, firmware and memory image: without them the game
//!   runs in DS mode, and asking for DSi (`consoles.dsmode` = `dsi`) reports
//!   [`ErrorCode::BiosDsiFilesMissing`]; `auto` becomes DS silently;
//! - booting through the DS menu (`cores.mdsboot` = `menu`) needs the DS files: without them the
//!   game boots directly and [`ErrorCode::BiosDsMenuFilesMissing`] is reported.

use super::{text, BiosTable, Check, Plan, Slot, SlotSpec};
use crate::settings::{ConsoleId, Value};
use crate::ErrorCode;

/// Firmware sizes the core accepts.
const FIRMWARE_SIZES: &[u64] = &[128 * 1024, 256 * 1024, 512 * 1024];
const DS_FIRMWARE: &str = "melonDS DS/firmware.bin";
const DSI_FIRMWARE: &str = "melonDS DS/dsi_firmware.bin";
const DSI_NAND: &str = "melonDS DS/dsi_nand.bin";

pub(super) static TABLE: BiosTable = BiosTable {
    core: crate::core_profile::CoreId::MelonDsDs,
    slots: &[
        SlotSpec {
            slot: Slot::DsArm7,
            setting: Some("bios.bds7"),
            file: "melonDS DS/bios7.bin",
            sizes: &[16 * 1024],
            check: Check::Md5(&["df692a80a5b1bc90728bc3dfc76cd948"]),
        },
        SlotSpec {
            slot: Slot::DsArm9,
            setting: Some("bios.bds9"),
            file: "melonDS DS/bios9.bin",
            sizes: &[4 * 1024],
            check: Check::Md5(&["a392174eb3e572fed6447e956bde4b25"]),
        },
        SlotSpec {
            slot: Slot::DsFirmware,
            setting: Some("bios.bdsfw"),
            file: DS_FIRMWARE,
            sizes: FIRMWARE_SIZES,
            // DS, DS Lite, iQue DS, iQue DS Lite.
            check: Check::FirmwareHeader {
                console_types: &[0xFF, 0x20, 0x43, 0x63],
            },
        },
        // The DSi BIOS and firmware have no settings row yet (`bios.bdsi` is the memory image).
        SlotSpec {
            slot: Slot::DsiArm7,
            setting: None,
            file: "melonDS DS/dsi_bios7.bin",
            sizes: &[64 * 1024],
            check: Check::SizeOnly,
        },
        SlotSpec {
            slot: Slot::DsiArm9,
            setting: None,
            file: "melonDS DS/dsi_bios9.bin",
            sizes: &[64 * 1024],
            check: Check::SizeOnly,
        },
        SlotSpec {
            slot: Slot::DsiFirmware,
            setting: None,
            file: DSI_FIRMWARE,
            sizes: FIRMWARE_SIZES,
            check: Check::FirmwareHeader {
                console_types: &[0x57],
            },
        },
        SlotSpec {
            slot: Slot::DsiNand,
            setting: Some("bios.bdsi"),
            file: DSI_NAND,
            sizes: &[],
            check: Check::NandFooter,
        },
    ],
    policy,
};

const DS_FILES: &[Slot] = &[Slot::DsArm7, Slot::DsArm9, Slot::DsFirmware];
const DSI_FILES: &[Slot] = &[
    Slot::DsiArm7,
    Slot::DsiArm9,
    Slot::DsiFirmware,
    Slot::DsiNand,
];

fn policy(in_place: &[Slot], value: &dyn Fn(&str) -> Option<Value>, _: ConsoleId) -> Plan {
    let has = |files: &[Slot]| files.iter().all(|s| in_place.contains(s));
    let ds = has(DS_FILES);
    let dsi = ds && has(DSI_FILES);
    let mut plan = Plan::default();
    if ds {
        plan.options.push(("melonds_sysfile_mode", "native"));
        plan.options
            .push(("melonds_firmware_nds_path", DS_FIRMWARE));
    } else {
        plan.options.push(("melonds_sysfile_mode", "builtin"));
    }
    if dsi {
        plan.options
            .push(("melonds_firmware_dsi_path", DSI_FIRMWARE));
        plan.options.push(("melonds_dsi_nand_path", DSI_NAND));
    } else {
        match text(value, "consoles.dsmode").as_deref() {
            Some("dsi") => {
                plan.options.push(("melonds_console_mode", "ds"));
                plan.fallbacks.push(ErrorCode::BiosDsiFilesMissing);
            }
            Some("ds") => {}
            // `auto` (and no value: the core's default is `auto`) would pick DSi for a DSi game.
            Some(_) | None => plan.options.push(("melonds_console_mode", "ds")),
        }
    }
    if !ds && text(value, "cores.mdsboot").as_deref() == Some("menu") {
        plan.options.push(("melonds_boot_mode", "direct"));
        plan.fallbacks.push(ErrorCode::BiosDsMenuFilesMissing);
    }
    plan
}
