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

use std::path::Path;

use crate::input::Button;
use crate::save_path::{PathLimits, SaveDepth, SavePathTooLong};
use crate::screens::Split;
use crate::settings::core_values::{self, CoreTable};
use crate::settings::ConsoleId;
use crate::GameKey;

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

/// Why loading states is refused for the open game: its core would crash the whole app. Saving
/// states stays allowed. The interface shows the loads (quick menu, resume question) disabled with
/// the reason; the session refuses them ([`crate::ErrorCode::StateLoadUnsupported`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "StateLoadBlock")]
pub enum StateLoadBlock {
    /// Azahar 2126.2 on Vulkan, content with title ID 0 (every `.3dsx` homebrew): loading a
    /// state crashes the process in `vk_pipeline_cache.cpp::SwitchCache` (Trello #93,
    /// `docs/upstream/azahar-vulkan-state-load-title-id-0.md`). Rewind loads states too.
    VulkanNoTitleId,
}

impl StateLoadBlock {
    /// Core options the same fault makes the shell force, whatever the settings ask for.
    /// [`StateLoadBlock::VulkanNoTitleId`]: Azahar also crashes at boot with its disk shader
    /// cache off (`perf.shcache`), the same missing pipeline cache, so the cache stays on.
    pub fn forced_options(self) -> &'static [(&'static str, &'static str)] {
        match self {
            StateLoadBlock::VulkanNoTitleId => &[("citra_use_disk_shader_cache", "enabled")],
        }
    }
}

/// The core options a session starts with: `options` (the settings', the profile's forced ones,
/// the system files'), then what `block` forces ([`StateLoadBlock::forced_options`]), which
/// wins over them.
pub fn with_block_options(
    options: &[(String, String)],
    block: Option<StateLoadBlock>,
) -> Vec<(String, String)> {
    let forced = block
        .map(StateLoadBlock::forced_options)
        .unwrap_or_default();
    options
        .iter()
        .filter(|(k, _)| !forced.iter().any(|(f, _)| f == k))
        .cloned()
        .chain(
            forced
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        )
        .collect()
}

/// Whether states may be loaded (quick loads, the auto quick save, «Annulla il caricamento»,
/// rewind) for `game` on `core` drawing with `renderer`: `None` when they may, else why not.
/// Only Azahar on Vulkan refuses, for content Azahar gives title ID 0 ([`crate::n3ds::title_id`]);
/// on its software renderer the same loads work. Until Azahar fixes it upstream.
pub fn state_load_block(
    core: CoreId,
    renderer: Renderer,
    game: &GameKey,
) -> Option<StateLoadBlock> {
    let crashes =
        core == CoreId::Azahar && renderer == Renderer::Vulkan && crate::n3ds::title_id(game) == 0;
    crashes.then_some(StateLoadBlock::VulkanNoTitleId)
}

/// How a core closes and opens the console's lid (Trello #28). Never faked by pausing: a core
/// with no way is [`Lid::Unsupported`], and the interface shows the lid disabled with the reason
/// (`pad.lid` in its [`CoreProfile::unsupported_settings`], `lidSupported` in the screens' status).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lid {
    /// No lid, or no way to close it through libretro.
    Unsupported,
    /// The core flips its lid when `press` goes down while `hold` is held (one flip per press);
    /// the shell presses both for a moment ([`LID_PRESS`]). `press` also reaches the game then.
    Chord { hold: Button, press: Button },
}

/// How long the shell holds a lid chord: a few frames, so the core's next polls see it go down
/// even when a frame is late. A paused game polls nothing and misses it.
pub const LID_PRESS: std::time::Duration = std::time::Duration::from_millis(100);

impl Lid {
    pub fn supported(self) -> bool {
        self != Lid::Unsupported
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
    /// The deepest paths the core creates by itself under the game's folder when it loads a
    /// game, for a core that aborts the process past the system's path limit (Azahar); the
    /// shell checks them first ([`CoreProfile::check_save_dir`]). `None`: nothing deep, or a
    /// core that reports the failure itself.
    pub save_depth: Option<SaveDepth>,
    /// Settings that act through cores but have no equivalent in this one: on its consoles the
    /// interface shows them disabled with the reason (`unsupportedConsoles` in the schema).
    pub unsupported_settings: &'static [&'static str],
    /// How the lid is closed, if it can be.
    pub lid: Lid,
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

    /// Whether the files the core creates by itself fit under `save_dir` (the game's folder,
    /// [`crate::GameFiles::dir`]) within `limits` (this process's, from the platform): checked
    /// before the core loads, since past them Azahar aborts the whole process (Trello #94).
    pub fn check_save_dir(
        &self,
        save_dir: &Path,
        limits: PathLimits,
    ) -> Result<(), SavePathTooLong> {
        match &self.save_depth {
            Some(depth) => crate::save_path::check(save_dir, depth, limits),
            None => Ok(()),
        }
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
    save_depth: None,
    // 3D rendering settings (Azahar's): a 2D core has no internal resolution, texture filter
    // or CPU clock option; no JIT either (an interpreter, 0.11-212 `--list-options`) and no
    // shader cache.
    unsupported_settings: &[
        "video.res3d",
        "video.texf",
        "perf.cpuclock",
        "perf.jit",
        "perf.shcache",
    ],
    // One screen, no lid (the lid rows show on DS and 3DS games only).
    lid: Lid::Unsupported,
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
    // Its DLDI / DSi SD card images sit one folder down (`melonDS DS/`).
    save_depth: None,
    // melonDS DS has no frameskip option (1.4.0, `config/constants.hpp`), no texture filter or
    // CPU clock; its internal resolution belongs to the OpenGL renderer the shell refuses; its
    // software renderer keeps no shader cache.
    unsupported_settings: &[
        "perf.frameskip",
        "video.res3d",
        "video.texf",
        "perf.cpuclock",
        "perf.shcache",
    ],
    // melonDS DS 1.4.0 `input/joypad.cpp`: Y pressed while L2 is held flips the lid
    // (`NDS::SetLidClosed(!IsLidClosed())`); no core option changes it.
    lid: Lid::Chord {
        hold: Button::L2,
        press: Button::Y,
    },
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
    save_depth: Some(crate::n3ds::SAVE_DEPTH),
    // Azahar 2126.2 has no frameskip option (`headless --list-options`), and no lid: nothing in
    // its libretro frontend (`src/citra_libretro/`) closes it, and its PTM service's
    // `shell_open` is never set to false.
    unsupported_settings: &["perf.frameskip", "pad.lid"],
    lid: Lid::Unsupported,
    options_row: "cores.azall",
    // TODO(Phase 3): measured when the headless host runs 3DS games (#46); its row is hidden
    // until then (phase 3).
    declared_options: None,
};

#[cfg(test)]
mod tests {
    use super::{
        row_count, state_load_block, unsupported_consoles, with_block_options, CoreId, Renderer,
        StateLoadBlock,
    };
    use crate::settings::core_values::core_options;
    use crate::settings::{ConsoleId, Value};
    use crate::GameKey;

    /// Trello #93 / #99: with the shader cache off Azahar on Vulkan crashes at boot on content
    /// with title ID 0, so for those games the cache stays on whatever `perf.shcache` says.
    #[test]
    fn the_shader_cache_stays_on_where_turning_it_off_crashes() {
        let off = core_options(CoreId::Azahar, ConsoleId::N3ds, |k| {
            (k == "perf.shcache").then_some(Value::Bool(false))
        });
        let cache = |options: &[(String, String)]| -> Vec<String> {
            options
                .iter()
                .filter(|(k, _)| k == "citra_use_disk_shader_cache")
                .map(|(_, v)| v.clone())
                .collect()
        };
        assert_eq!(cache(&off), ["disabled"]);
        // Only for the games whose loads are blocked; the other options are kept.
        let blocked = with_block_options(&off, Some(StateLoadBlock::VulkanNoTitleId));
        assert_eq!(cache(&blocked), ["enabled"]);
        assert_eq!(blocked.len(), off.len());
        assert_eq!(with_block_options(&off, None), off);
    }

    /// Trello #93: Azahar on Vulkan crashes loading a state of content with title ID 0.
    #[test]
    fn state_loads_are_refused_only_for_azahar_on_vulkan_without_a_title_id() {
        let key = |text: &str| GameKey::parse(text).unwrap();
        let homebrew = key("3ds:da39a3ee5e6b4b0d3255bfef95601890afd80709");
        let no_program_id = key("3ds:00000000000000000000");
        let retail = key("3ds:00040000000308000000");
        let blocked = Some(StateLoadBlock::VulkanNoTitleId);
        assert_eq!(
            state_load_block(CoreId::Azahar, Renderer::Vulkan, &homebrew),
            blocked
        );
        assert_eq!(
            state_load_block(CoreId::Azahar, Renderer::Vulkan, &no_program_id),
            blocked
        );
        assert_eq!(
            state_load_block(CoreId::Azahar, Renderer::Vulkan, &retail),
            None
        );
        // The software renderer loads them fine.
        assert_eq!(
            state_load_block(CoreId::Azahar, Renderer::Software, &homebrew),
            None
        );
        // The other cores never refuse.
        let gba = key("gba:9a3e");
        for core in [CoreId::Mgba, CoreId::MelonDsDs] {
            for renderer in [Renderer::Software, Renderer::Vulkan] {
                assert_eq!(state_load_block(core, renderer, &gba), None);
            }
        }
        // As the interface reads it.
        assert_eq!(
            serde_json::to_value(StateLoadBlock::VulkanNoTitleId).unwrap(),
            serde_json::json!("vulkanNoTitleId")
        );
    }

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

    /// Trello #94: only Azahar's save folder is checked before the load.
    #[test]
    fn only_azahar_checks_how_deep_its_files_go() {
        use crate::save_path::{PathLimits, PathUnit};
        let legacy = PathLimits {
            dir: 247,
            file: 259,
            unit: PathUnit::Utf16,
        };
        let deep = std::path::PathBuf::from(format!("C:/{}", "d".repeat(200)));
        assert!(CoreId::Azahar
            .profile()
            .check_save_dir(&deep, legacy)
            .is_err());
        assert!(CoreId::Azahar
            .profile()
            .check_save_dir(std::path::Path::new("C:/emudome/saves/3ds/00"), legacy)
            .is_ok());
        for core in [CoreId::Mgba, CoreId::MelonDsDs] {
            assert!(core.profile().check_save_dir(&deep, legacy).is_ok());
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
