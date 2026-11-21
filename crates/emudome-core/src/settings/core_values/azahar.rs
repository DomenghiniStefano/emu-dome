//! Azahar (Nintendo 3DS). Keys and values as Azahar 2126.2 declares them (`headless
//! --list-options`, `src/citra_libretro/core_settings.cpp`); Azahar compares values
//! case-sensitively.
//!
//! Live or restart: Azahar re-reads its options at `GET_VARIABLE_UPDATE` (`retro_run`:
//! `ParseCoreOptions`, `ApplySettings`), but the console model, region, audio engine and CPU
//! clock are set when the system boots, and a new internal resolution or texture filter rebuilds
//! the renderer's caches under a running game, which was not tested: every rule here waits for
//! the game to be opened again.
//!
//! Not here: the options that hold the screens where the shell cuts them and keep the saves in
//! the game's folder (`crate::n3ds::FORCED_OPTIONS`), and the renderer (`citra_graphics_api`,
//! `CoreProfile::hw_render`). Not mapped, each shown disabled until it is: `perf.shcache` (Azahar
//! 2126.2 crashes with its disk shader cache off on content without a title ID), `screens.stereo`
//! (stereo modes change the frame the shell cuts, #28), `pad.gyro` (no sensor interface yet),
//! `bios.b3ds` (3DS system files are a NAND tree, not a file the BIOS rows check).

use super::{Apply, CoreTable, Map, Rule};
use crate::settings::ConsoleId;

const N3DS: &[ConsoleId] = &[ConsoleId::N3ds];

pub(super) static TABLE: CoreTable = CoreTable {
    core: crate::core_profile::CoreId::Azahar,
    rules: &[
        // ---- Console options, Nintendo 3DS ----
        Rule {
            setting: "consoles.n3dsmodel",
            consoles: N3DS,
            map: Map::Choice(&[
                ("o3ds", &[("citra_is_new_3ds", "Old 3DS")]),
                ("n3ds", &[("citra_is_new_3ds", "New 3DS")]),
            ]),
            apply: Apply::Restart,
        },
        Rule {
            setting: "consoles.n3dsreg",
            consoles: N3DS,
            map: Map::Choice(&[
                ("auto", &[("citra_region_value", "Auto")]),
                ("jpn", &[("citra_region_value", "Japan")]),
                ("usa", &[("citra_region_value", "USA")]),
                ("eur", &[("citra_region_value", "Europe")]),
                ("kor", &[("citra_region_value", "Korea")]),
                ("chn", &[("citra_region_value", "China")]),
                ("twn", &[("citra_region_value", "Taiwan")]),
            ]),
            apply: Apply::Restart,
        },
        // ---- Video ----
        Rule {
            setting: "video.res3d",
            consoles: N3DS,
            map: Map::Choice(&[
                ("1x", &[("citra_resolution_factor", "1")]),
                ("2x", &[("citra_resolution_factor", "2")]),
                ("3x", &[("citra_resolution_factor", "3")]),
                ("4x", &[("citra_resolution_factor", "4")]),
            ]),
            apply: Apply::Restart,
        },
        Rule {
            setting: "video.texf",
            consoles: N3DS,
            map: Map::Choice(&[
                ("none", &[("citra_texture_filter", "none")]),
                ("bicubic", &[("citra_texture_filter", "Bicubic")]),
                ("scaleforce", &[("citra_texture_filter", "ScaleForce")]),
                ("xbrz", &[("citra_texture_filter", "xBRZ")]),
                ("mmpx", &[("citra_texture_filter", "MMPX")]),
                ("anime4k", &[("citra_texture_filter", "Anime4K Ultrafast")]),
            ]),
            apply: Apply::Restart,
        },
        // ---- Performance ----
        // 50..=400 in steps of 25: every value is one of Azahar's (25..=400, steps of 25).
        Rule {
            setting: "perf.cpuclock",
            consoles: N3DS,
            map: Map::Number {
                option: "citra_cpu_clock_percentage",
            },
            apply: Apply::Restart,
        },
        // ---- Azahar ----
        Rule {
            setting: "cores.azaudio",
            consoles: N3DS,
            map: Map::Choice(&[
                ("hle", &[("citra_audio_emulation", "hle")]),
                ("lle", &[("citra_audio_emulation", "lle")]),
            ]),
            apply: Apply::Restart,
        },
    ],
};
