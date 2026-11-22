//! melonDS DS (Nintendo DS). Keys and values as melonDS DS 1.4.0 declares them
//! (`headless --list-options`, `src/libretro/config/constants.hpp`).
//!
//! Live or restart, from melonDS DS's source (`core/core.cpp`): on `GET_VARIABLE_UPDATE` it
//! re-applies the renderer, the screen layout, input, microphone and network
//! (`CoreState::ApplyConfig`); the console model, firmware, boot mode and slot-2 device are set
//! when the console is built, so they wait for the game to be opened again.
//!
//! Not here: the options that hold the screens where the shell cuts them and the user-name mode,
//! which are not settings (`crate::nds::FORCED_OPTIONS`); the system files (`crate::bios`); the DS user name
//! (`consoles.dsname`) is not a core option but the frontend's answer to `GET_USERNAME`
//! (`crate::nds::username`).

use super::{Apply, CoreTable, Map, Rule};
use crate::settings::ConsoleId;

const NDS: &[ConsoleId] = &[ConsoleId::Nds];

pub(super) static TABLE: CoreTable = CoreTable {
    core: crate::core_profile::CoreId::MelonDsDs,
    rules: &[
        // ---- Console options, Nintendo DS ----
        // DSi mode needs the DS and DSi system files, which have no built-in replacement:
        // without them `crate::bios` runs the game as a DS (`bios/melondsds.rs`).
        Rule {
            setting: "consoles.dsmode",
            consoles: NDS,
            map: Map::Choice(&[
                ("auto", &[("melonds_console_mode", "auto")]),
                ("ds", &[("melonds_console_mode", "ds")]),
                ("dsi", &[("melonds_console_mode", "dsi")]),
            ]),
            apply: Apply::Restart,
        },
        Rule {
            setting: "consoles.dslang",
            consoles: NDS,
            map: Map::Choice(&[
                ("ja", &[("melonds_firmware_language", "ja")]),
                ("en", &[("melonds_firmware_language", "en")]),
                ("fr", &[("melonds_firmware_language", "fr")]),
                ("de", &[("melonds_firmware_language", "de")]),
                ("it", &[("melonds_firmware_language", "it")]),
                ("es", &[("melonds_firmware_language", "es")]),
            ]),
            apply: Apply::Restart,
        },
        Rule {
            setting: "consoles.slot2",
            consoles: NDS,
            map: Map::Choice(&[
                // Nothing in the slot: «auto» puts nothing there when no GBA game is loaded.
                ("none", &[("melonds_slot2_device", "auto")]),
                ("rumble", &[("melonds_slot2_device", "rumble-pak")]),
                ("memory", &[("melonds_slot2_device", "expansion-pak")]),
                // TODO(#45): load the GBA game into slot 2 (melonDS DS's subsystem content).
                ("gba_game", &[("melonds_slot2_device", "auto")]),
            ]),
            apply: Apply::Restart,
        },
        // ---- melonDS DS ----
        // OpenGL needs a GL context the host does not offer (Trello #16): melonDS DS then falls
        // back to software by itself (measured, docs/feasibility.md).
        Rule {
            setting: "cores.mdsrender",
            consoles: NDS,
            map: Map::Choice(&[
                ("software", &[("melonds_render_mode", "software")]),
                ("opengl", &[("melonds_render_mode", "opengl")]),
            ]),
            apply: Apply::Live,
        },
        // «menu» boots through the DS menu, which needs the DS system files: without them
        // `crate::bios` boots the game directly (`bios/melondsds.rs`).
        Rule {
            setting: "cores.mdsboot",
            consoles: NDS,
            map: Map::Choice(&[
                ("direct", &[("melonds_boot_mode", "direct")]),
                ("menu", &[("melonds_boot_mode", "native")]),
            ]),
            apply: Apply::Restart,
        },
        // ---- Advanced › Performance ----
        // Declared by the Windows and the Android arm64 builds alike (1.4.0); its label says
        // «Takes effect at next restart».
        Rule {
            setting: "perf.jit",
            consoles: NDS,
            map: Map::Toggle {
                option: "melonds_jit_enable",
                on: "enabled",
                off: "disabled",
            },
            apply: Apply::Restart,
        },
    ],
};
