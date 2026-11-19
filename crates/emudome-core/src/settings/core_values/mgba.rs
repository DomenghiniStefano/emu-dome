//! mGBA (Game Boy, Game Boy Color, Game Boy Advance). Keys and values as mGBA 0.11-212 declares
//! them (`headless --list-options`); «Restart» where its own label says «(Restart)».

use super::{Apply, CoreTable, Map, Rule};
use crate::settings::ConsoleId;

const GB: &[ConsoleId] = &[ConsoleId::Gb, ConsoleId::Gbc];
const GBA: &[ConsoleId] = &[ConsoleId::Gba];
const ALL: &[ConsoleId] = &[ConsoleId::Gb, ConsoleId::Gbc, ConsoleId::Gba];

pub(super) static TABLE: CoreTable = CoreTable {
    core: "mgba",
    rules: &[
        // ---- Console options, Game Boy / Game Boy Color ----
        Rule {
            setting: "consoles.gbmodel",
            consoles: GB,
            map: Map::Choice(&[
                ("auto", &[("mgba_gb_model", "Autodetect")]),
                ("gb", &[("mgba_gb_model", "Game Boy")]),
                ("gbc", &[("mgba_gb_model", "Game Boy Color")]),
                ("sgb", &[("mgba_gb_model", "Super Game Boy")]),
            ]),
            apply: Apply::Restart,
        },
        // The palette of black-and-white games. «gbc» = the colours a Game Boy Color gives the
        // games it knows (mGBA's hardware presets); a game it does not know gets the core's
        // default palette (Grayscale).
        Rule {
            setting: "consoles.gbpal",
            consoles: GB,
            map: Map::Choice(&[
                (
                    "green",
                    &[
                        ("mgba_gb_colors", "DMG Green"),
                        ("mgba_gb_colors_preset", "0"),
                    ],
                ),
                (
                    "pocket",
                    &[
                        ("mgba_gb_colors", "GB Pocket"),
                        ("mgba_gb_colors_preset", "0"),
                    ],
                ),
                (
                    "light",
                    &[
                        ("mgba_gb_colors", "GB Light"),
                        ("mgba_gb_colors_preset", "0"),
                    ],
                ),
                (
                    "gbc",
                    &[
                        ("mgba_gb_colors", "Grayscale"),
                        ("mgba_gb_colors_preset", "1"),
                    ],
                ),
            ]),
            // `mgba_gb_colors_preset` is «(Restart)»: the whole choice waits for it.
            apply: Apply::Restart,
        },
        Rule {
            setting: "consoles.sgb",
            consoles: GB,
            map: Map::Toggle {
                option: "mgba_sgb_borders",
                on: "ON",
                off: "OFF",
            },
            apply: Apply::Restart,
        },
        // ---- Console options, Game Boy Advance ----
        Rule {
            setting: "consoles.gbacc",
            consoles: GBA,
            map: Map::Choice(&[
                ("off", &[("mgba_color_correction", "OFF")]),
                ("gba", &[("mgba_color_correction", "GBA")]),
                ("auto", &[("mgba_color_correction", "Auto")]),
            ]),
            apply: Apply::Live,
        },
        // «auto» = mGBA's smart mix, which blends only the pixels that flicker.
        Rule {
            setting: "consoles.gbablend",
            consoles: GBA,
            map: Map::Choice(&[
                ("off", &[("mgba_interframe_blending", "OFF")]),
                ("auto", &[("mgba_interframe_blending", "mix_smart")]),
                ("on", &[("mgba_interframe_blending", "mix")]),
            ]),
            apply: Apply::Live,
        },
        // 0..100 % → mGBA's levels 0..10 (its «sensor» value, a real light sensor, is not used).
        Rule {
            setting: "consoles.solar",
            consoles: GBA,
            map: Map::Percent {
                option: "mgba_solar_sensor_level",
                max: 10,
            },
            apply: Apply::Live,
        },
        // ---- Advanced › mGBA ----
        Rule {
            setting: "cores.mgbabios",
            consoles: ALL,
            map: Map::Toggle {
                option: "mgba_use_bios",
                on: "ON",
                off: "OFF",
            },
            apply: Apply::Restart,
        },
        Rule {
            setting: "cores.mgbaskip",
            consoles: ALL,
            map: Map::Toggle {
                option: "mgba_skip_bios",
                on: "ON",
                off: "OFF",
            },
            apply: Apply::Restart,
        },
        Rule {
            setting: "cores.mgbalp",
            consoles: ALL,
            map: Map::Toggle {
                option: "mgba_audio_low_pass_filter",
                on: "enabled",
                off: "disabled",
            },
            apply: Apply::Live,
        },
        Rule {
            setting: "cores.mgbaopp",
            consoles: ALL,
            map: Map::Toggle {
                option: "mgba_allow_opposing_directions",
                on: "yes",
                off: "no",
            },
            apply: Apply::Live,
        },
    ],
};
