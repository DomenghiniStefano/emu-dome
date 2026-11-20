//! mGBA's BIOS files: in its system folder under the names it looks for. With `cores.mgbabios`
//! on it uses the file of the console it runs when the file is there, and its built-in
//! replacement otherwise; only valid files are ever in place ([`super::files`]), so it needs no
//! option of its own.

use super::{BiosTable, Check, Plan, Slot, SlotSpec};
use crate::settings::{ConsoleId, Value};

pub(super) static TABLE: BiosTable = BiosTable {
    core: "mgba",
    slots: &[
        SlotSpec {
            slot: Slot::GbaBios,
            setting: Some("bios.bgba"),
            file: "gba_bios.bin",
            sizes: &[16 * 1024],
            check: Check::Md5(&["a860e8c0b6d573d191e4ec7db1b1e4f6"]),
        },
        // The Game Boy files have no settings row yet (the catalogue has `bios.bgba` only).
        SlotSpec {
            slot: Slot::GbBoot,
            setting: None,
            file: "gb_bios.bin",
            sizes: &[256],
            check: Check::Md5(&["32fbbd84168d3482956eb3c5051637f5"]),
        },
        SlotSpec {
            slot: Slot::GbcBoot,
            setting: None,
            file: "gbc_bios.bin",
            sizes: &[2304],
            check: Check::Md5(&["dbfce9db9deaa2567f6a84fde55f9680"]),
        },
        SlotSpec {
            slot: Slot::SgbBoot,
            setting: None,
            file: "sgb_bios.bin",
            sizes: &[256],
            check: Check::Md5(&["d574d4f9c12f305074798f54c091a8b4"]),
        },
    ],
    policy: nothing_to_set,
};

fn nothing_to_set(_: &[Slot], _: &dyn Fn(&str) -> Option<Value>, _: ConsoleId) -> Plan {
    Plan::default()
}
