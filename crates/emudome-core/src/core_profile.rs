//! What EmuDome knows about each core, in one place: [`CoreId`] names the core (never a string
//! matched by hand), [`CoreProfile`] says which consoles it runs, the options the shell always
//! forces, how its frame holds the screens, whether it can ask for hardware rendering, how its
//! saves are kept, and which settings it cannot honour. The settings → core options tables stay
//! in `settings/core_values/<core>.rs` ([`CoreProfile::option_table`]), the system files in
//! `bios/<core>.rs`.
//!
//! Adding a core: a [`CoreId`] variant (its name is the libretro file's, `<name>_libretro.*`, and
//! the `cores.lock` key), its [`CoreProfile`] in [`CoreId::profile`], then its option and BIOS
//! tables if it has any. The compiler lists every `match` that has to learn it.

use crate::screens::Split;
use crate::settings::core_values::{self, CoreTable};
use crate::settings::ConsoleId;

/// The cores EmuDome runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoreId {
    /// mGBA: Game Boy, Game Boy Color, Game Boy Advance.
    Mgba,
    /// melonDS DS: Nintendo DS.
    #[serde(rename = "melondsds")]
    MelonDsDs,
    /// Azahar: Nintendo 3DS (Phase 3).
    Azahar,
}

impl CoreId {
    pub const ALL: [CoreId; 3] = [CoreId::Mgba, CoreId::MelonDsDs, CoreId::Azahar];

    /// The core's name: its libretro file (`<name>_libretro.*`), its `cores.lock` key, the
    /// interface's `details.files.core.<name>`.
    pub fn name(self) -> &'static str {
        match self {
            CoreId::Mgba => "mgba",
            CoreId::MelonDsDs => "melondsds",
            CoreId::Azahar => "azahar",
        }
    }

    pub fn from_name(name: &str) -> Option<CoreId> {
        CoreId::ALL.into_iter().find(|c| c.name() == name)
    }

    /// The core that runs a console's games.
    pub fn for_console(console: ConsoleId) -> CoreId {
        match console {
            ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba => CoreId::Mgba,
            ConsoleId::Nds => CoreId::MelonDsDs,
            ConsoleId::N3ds => CoreId::Azahar,
        }
    }

    pub fn profile(self) -> &'static CoreProfile {
        match self {
            CoreId::Mgba => &MGBA,
            CoreId::MelonDsDs => &MELONDSDS,
            CoreId::Azahar => &AZAHAR,
        }
    }
}

impl std::fmt::Display for CoreId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A graphics API a core can ask for with `SET_HW_RENDER`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HwContext {
    OpenGlCore,
    Vulkan,
}

/// How a core draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HwRender {
    /// The APIs it can ask for, in its order of preference; empty: software only.
    pub contexts: &'static [HwContext],
    /// It has a software renderer the shell can force (by a forced option), used where the
    /// shell has no context for it (Trello #16 / #46).
    pub software_fallback: bool,
    /// The core option that picks its renderer, when the shell must choose it (Azahar's default
    /// `auto` asks for OpenGL, which the shell has no context for).
    pub renderer_option: Option<RendererOption>,
}

/// A core option that picks the renderer: its key, and its value for each renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RendererOption {
    pub key: &'static str,
    /// The value for the Vulkan renderer, if the core has one.
    pub vulkan: Option<&'static str>,
    /// The value for the software renderer.
    pub software: &'static str,
}

/// The renderer a session runs a core with, as the shell chose it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    /// The core's software renderer (frames in memory).
    Software,
    /// Vulkan on the shell's device (`hw_render::vulkan`).
    Vulkan,
}

impl HwRender {
    /// The renderer to run with when the shell has a Vulkan device (`vulkan_device`): Vulkan
    /// when the core can ask for it and the shell can tell it to, else software.
    pub fn renderer(&self, vulkan_device: bool) -> Renderer {
        let vulkan = vulkan_device
            && self.contexts.contains(&HwContext::Vulkan)
            && self.renderer_option.is_some_and(|o| o.vulkan.is_some());
        if vulkan {
            Renderer::Vulkan
        } else {
            Renderer::Software
        }
    }

    /// Whether the core would ask for Vulkan and must be told to draw in software instead
    /// (a shell with no Vulkan device): the shell reports it as a fallback.
    pub fn falls_back_from_vulkan(&self, renderer: Renderer) -> bool {
        renderer == Renderer::Software
            && self.contexts.contains(&HwContext::Vulkan)
            && self.renderer_option.is_some_and(|o| o.vulkan.is_some())
    }
}

/// Where a core keeps a game's progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveModel {
    /// The battery save is the core's `RETRO_MEMORY_SAVE_RAM`: the shell reads and writes it as
    /// `game.sav` ([`crate::GameFiles`]).
    SaveRam,
    /// The battery save as above, plus files the core writes by itself in its save directory (the
    /// game's folder): melonDS DS's DLDI / DSi SD card images.
    SaveRamAndCoreFiles,
    /// Only files the core writes by itself (Azahar's emulated SD card and NAND): no
    /// `RETRO_MEMORY_SAVE_RAM`.
    CoreFiles,
}

/// One core as the shell runs it.
#[derive(Debug)]
pub struct CoreProfile {
    pub id: CoreId,
    /// The consoles whose games it runs.
    pub consoles: &'static [ConsoleId],
    /// The library's file extensions (`library::EXTENSIONS`, lowercase) whose files it opens.
    pub extensions: &'static [&'static str],
    /// Core options the shell always sets (`GET_VARIABLE` key and value), after the settings'
    /// and before the system files' ([`crate::bios`]).
    pub forced_options: &'static [(&'static str, &'static str)],
    /// How the frame holds the screens with the forced options.
    pub split: Split,
    pub hw_render: HwRender,
    pub save_model: SaveModel,
    /// Settings that act through cores but have no equivalent in this one: on its consoles the
    /// interface shows them disabled with the reason (`unsupportedConsoles` in the schema).
    pub unsupported_settings: &'static [&'static str],
    /// The settings row that opens the full list of this core's options (`cores.*all`).
    pub options_row: &'static str,
    /// How many options the pinned build declares, the number that row shows; `None` until
    /// measured.
    pub declared_options: Option<DeclaredOptions>,
}

/// The options a core declares (`SET_CORE_OPTIONS*` / `SET_VARIABLES`), counted on one build.
/// The core tells them only once loaded, so the count is measured on the build `cores.lock`
/// pins: `cargo run -p emudome-core --example headless -- <core library> <test rom>
/// --list-options`, one line each. `the_option_counts_were_measured_on_the_pinned_builds` fails
/// when the pin moves to another version, until the count is measured again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeclaredOptions {
    pub count: u32,
    /// `cores.lock`'s `source.version` of the build measured.
    pub version: &'static str,
}

/// The number a settings row shows (`{n}` in its text): the options a core declares, on its
/// «All … options» row. `None` for every other row, and while not measured.
pub fn row_count(setting: &str) -> Option<u32> {
    CoreId::ALL
        .iter()
        .map(|c| c.profile())
        .find(|p| p.options_row == setting)
        .and_then(|p| p.declared_options)
        .map(|d| d.count)
}

impl CoreProfile {
    /// The settings → core options table of this core, if it has one.
    pub fn option_table(&self) -> Option<&'static CoreTable> {
        core_values::table(self.id)
    }

    /// Its forced options as `GET_VARIABLE` key and value pairs, with the renderer option for
    /// `renderer` when the core has one.
    pub fn forced_pairs(&self, renderer: Renderer) -> Vec<(String, String)> {
        let pick = self.hw_render.renderer_option.map(|o| {
            let value = match renderer {
                Renderer::Vulkan => o.vulkan.unwrap_or(o.software),
                Renderer::Software => o.software,
            };
            (o.key, value)
        });
        self.forced_options
            .iter()
            .copied()
            .chain(pick)
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }
}

/// The consoles on which `setting` cannot work, because their core has no equivalent
/// ([`CoreProfile::unsupported_settings`]). Empty for every other setting.
pub fn unsupported_consoles(setting: &str) -> Vec<ConsoleId> {
    CoreId::ALL
        .iter()
        .map(|c| c.profile())
        .filter(|p| p.unsupported_settings.contains(&setting))
        .flat_map(|p| p.consoles.iter().copied())
        .collect()
}

static MGBA: CoreProfile = CoreProfile {
    id: CoreId::Mgba,
    consoles: &[ConsoleId::Gb, ConsoleId::Gbc, ConsoleId::Gba],
    extensions: &["gb", "gbc", "gba"],
    forced_options: &[],
    split: Split::Whole,
    hw_render: HwRender {
        contexts: &[],
        software_fallback: true,
        renderer_option: None,
    },
    save_model: SaveModel::SaveRam,
    // 3D rendering settings (Azahar's): a 2D core has no internal resolution, texture filter
    // or CPU clock option.
    unsupported_settings: &["video.res3d", "video.texf", "perf.cpuclock"],
    options_row: "cores.mgbaall",
    // Measured 2026-11-21 (Trello #89).
    declared_options: Some(DeclaredOptions {
        count: 17,
        version: "0.11-212-7a12d6d",
    }),
};

static MELONDSDS: CoreProfile = CoreProfile {
    id: CoreId::MelonDsDs,
    consoles: &[ConsoleId::Nds],
    extensions: &["nds"],
    forced_options: crate::nds::FORCED_OPTIONS,
    split: crate::nds::SPLIT,
    // OpenGL renderer (`cores.mdsrender`); without a GL context it falls back to software
    // by itself (docs/feasibility.md).
    hw_render: HwRender {
        contexts: &[HwContext::OpenGlCore],
        software_fallback: true,
        // `cores.mdsrender` picks it (settings); OpenGL is refused, the core then draws in
        // software by itself.
        renderer_option: None,
    },
    save_model: SaveModel::SaveRamAndCoreFiles,
    // melonDS DS has no frameskip option (1.4.0, `config/constants.hpp`), no texture filter or
    // CPU clock; its internal resolution belongs to the OpenGL renderer the shell refuses.
    unsupported_settings: &[
        "perf.frameskip",
        "video.res3d",
        "video.texf",
        "perf.cpuclock",
    ],
    options_row: "cores.mdsall",
    // Measured 2026-11-21 on the DS test ROM (Trello #89).
    declared_options: Some(DeclaredOptions {
        count: 84,
        version: "1.4.0",
    }),
};

static AZAHAR: CoreProfile = CoreProfile {
    id: CoreId::Azahar,
    consoles: &[ConsoleId::N3ds],
    // Azahar 2126.2's `valid_extensions` that the library knows: no `.cia` (installable archives
    // are installed by the standalone emulator, not run by the libretro core).
    extensions: &["3ds", "cci", "cxi", "3dsx"],
    forced_options: crate::n3ds::FORCED_OPTIONS,
    split: crate::n3ds::SPLIT,
    hw_render: HwRender {
        contexts: &[HwContext::Vulkan, HwContext::OpenGlCore],
        software_fallback: true,
        // Its default `auto` asks for OpenGL, which the shell has no context for: Vulkan on the
        // shell's device, else the software renderer (Azahar compares the values
        // case-sensitively).
        renderer_option: Some(RendererOption {
            key: "citra_graphics_api",
            vulkan: Some("Vulkan"),
            software: "Software",
        }),
    },
    save_model: SaveModel::CoreFiles,
    // Azahar 2126.2 has no frameskip option (`headless --list-options`).
    unsupported_settings: &["perf.frameskip"],
    options_row: "cores.azall",
    // TODO(Phase 3): measured when the headless host runs 3DS games (#46); its row is hidden
    // until then (phase 3).
    declared_options: None,
};

#[cfg(test)]
mod tests {
    use super::{row_count, unsupported_consoles, CoreId, Renderer};
    use crate::settings::ConsoleId;

    #[test]
    fn the_option_counts_were_measured_on_the_pinned_builds() {
        let lock: serde_json::Value =
            serde_json::from_str(include_str!("../../../cores.lock")).unwrap();
        for core in CoreId::ALL {
            let p = core.profile();
            // The row is a link of the catalogue.
            assert!(
                crate::settings::schema::entry(p.options_row).is_some_and(|e| matches!(
                    e.kind,
                    crate::settings::Kind::Action {
                        style: crate::settings::ActionStyle::Link
                    }
                )),
                "{}",
                p.options_row
            );
            if let Some(d) = p.declared_options {
                assert_eq!(
                    lock["cores"][core.name()]["source"]["version"],
                    serde_json::json!(d.version),
                    "{core}: the pin moved, measure its options again (`headless --list-options`)"
                );
                assert!(d.count > 0);
            }
        }
    }

    #[test]
    fn the_all_options_rows_get_their_count_and_no_other_row_does() {
        assert_eq!(row_count("cores.mgbaall"), Some(17));
        assert_eq!(row_count("cores.mdsall"), Some(84));
        assert_eq!(row_count("cores.azall"), None);
        assert_eq!(row_count("video.scale"), None);
        // In the schema the interface reads, only where there is one.
        let schema = crate::settings::schema::schema(crate::settings::schema::Device::Desktop);
        let json = serde_json::to_value(&schema.entries).unwrap();
        let row = |key: &str| {
            json.as_array()
                .unwrap()
                .iter()
                .find(|e| e["key"] == key)
                .cloned()
                .unwrap()
        };
        assert_eq!(row("cores.mgbaall")["count"], serde_json::json!(17));
        assert!(row("cores.azall").get("count").is_none());
        assert!(row("video.scale").get("count").is_none());
    }

    #[test]
    fn every_console_has_one_core_whose_profile_lists_it() {
        for console in ConsoleId::ALL {
            let core = CoreId::for_console(console);
            assert!(core.profile().consoles.contains(&console), "{console:?}");
            let owners = CoreId::ALL
                .iter()
                .filter(|c| c.profile().consoles.contains(&console))
                .count();
            assert_eq!(owners, 1, "{console:?}");
        }
        for core in CoreId::ALL {
            assert_eq!(core.profile().id, core);
            assert_eq!(CoreId::from_name(core.name()), Some(core));
            assert_eq!(
                serde_json::to_value(core).unwrap(),
                serde_json::json!(core.name())
            );
        }
    }

    #[test]
    fn each_core_opens_library_files_of_its_own_consoles() {
        for core in CoreId::ALL {
            let p = core.profile();
            for ext in p.extensions {
                assert!(crate::library::EXTENSIONS.contains(ext), "{core}: {ext}");
                let console = crate::library::console_of_extension(ext);
                assert!(
                    console.is_some_and(|c| p.consoles.contains(&c)),
                    "{core}: {ext}"
                );
            }
        }
        assert!(!CoreId::Azahar.profile().extensions.contains(&"cia"));
    }

    #[test]
    fn azahar_draws_with_vulkan_on_the_shells_device_else_in_software() {
        let p = CoreId::Azahar.profile();
        let api = |r| {
            p.forced_pairs(r)
                .into_iter()
                .find(|(k, _)| k == "citra_graphics_api")
                .map(|(_, v)| v)
        };
        assert_eq!(p.hw_render.renderer(true), Renderer::Vulkan);
        assert_eq!(p.hw_render.renderer(false), Renderer::Software);
        assert_eq!(api(Renderer::Vulkan).as_deref(), Some("Vulkan"));
        assert_eq!(api(Renderer::Software).as_deref(), Some("Software"));
        assert!(p.hw_render.falls_back_from_vulkan(Renderer::Software));
        assert!(!p.hw_render.falls_back_from_vulkan(Renderer::Vulkan));
        // The other cores never get a Vulkan renderer, nor a fallback to report.
        for core in [CoreId::Mgba, CoreId::MelonDsDs] {
            let h = core.profile().hw_render;
            assert_eq!(h.renderer(true), Renderer::Software);
            assert!(!h.falls_back_from_vulkan(Renderer::Software));
        }
    }

    #[test]
    fn frameskip_is_unsupported_on_ds_and_3ds_only() {
        assert_eq!(
            unsupported_consoles("perf.frameskip"),
            vec![ConsoleId::Nds, ConsoleId::N3ds]
        );
        assert!(unsupported_consoles("consoles.gbacc").is_empty());
        // The interface reads it from the schema (always present, empty for most rows).
        let schema = crate::settings::schema::schema(crate::settings::schema::Device::Desktop);
        let json = serde_json::to_value(&schema.entries).unwrap();
        let row = |key: &str| {
            json.as_array()
                .unwrap()
                .iter()
                .find(|e| e["key"] == key)
                .cloned()
                .unwrap()
        };
        assert_eq!(
            row("perf.frameskip")["unsupportedConsoles"],
            serde_json::json!(["nds", "3ds"])
        );
        assert_eq!(
            row("consoles.gbacc")["unsupportedConsoles"],
            serde_json::json!([])
        );
    }

    #[test]
    fn unsupported_settings_are_settings_no_table_of_that_core_maps() {
        for core in CoreId::ALL {
            let p = core.profile();
            for key in p.unsupported_settings {
                assert!(crate::settings::schema::entry(key).is_some(), "{key}");
                let mapped = p
                    .option_table()
                    .is_some_and(|t| t.rules.iter().any(|r| r.setting == *key));
                assert!(!mapped, "{core} maps {key} and lists it as unsupported");
            }
        }
    }
}
