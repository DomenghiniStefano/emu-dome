//! The static catalogue: every setting of the app, page by page, in the design's order
//! (Cowork's settings catalogue, 155 entries, Trello comment of 2026-11-17), plus the Game Boy,
//! Game Boy Color and DSi BIOS rows (owner, 2026-11-21, Trello #89): 160.
//!
//! Option ids are stable snake_case English ids derived from the design's Italian labels; the
//! visible texts live in the interface's message files (`settings.<page>.<key>.option.<id>`).
//!
//! Slider ranges and steps, the chips' defaults and the per-option platforms are Cowork's
//! (design review of 2026-11-17, Trello #26).

use super::schema::{
    ActionStyle, BindingTable, ChoiceStyle, DefaultRule, DefaultValue, Device, Entry, Kind, Levels,
    PageGroup, Platform, Section, ShowIf, Unit,
};
use super::ConsoleId;

/// Phase the app is in: entries of later phases stay hidden.
pub const CURRENT_PHASE: u8 = 1;

/// The groups of the category list, in order, with their pages (text `settings.groups.<id>`).
pub static GROUPS: &[PageGroup] = &[
    PageGroup {
        id: "game",
        pages: &["video", "fx", "screens", "audio", "speed", "saves"],
    },
    PageGroup {
        id: "controls",
        pages: &["pad", "touch"],
    },
    PageGroup {
        id: "app",
        pages: &["general", "library", "cloud", "a11y", "presets"],
    },
    PageGroup {
        id: "system",
        pages: &["bios", "consoles", "advanced", "online", "about"],
    },
];

/// Page ids in display order (T02–T19 and Cloud; T01 «In primo piano» is the interface's own
/// first screen, not a page of entries).
pub static PAGES: &[&str] = &[
    "video", "fx", "screens", "audio", "speed", "saves", "pad", "touch", "general", "library",
    "cloud", "a11y", "presets", "bios", "consoles", "advanced", "online", "about",
];

/// The page of a key, by default: the part before the first dot.
const fn key_page(key: &'static str) -> &'static str {
    let bytes = key.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i] != b'.' {
        i += 1;
    }
    key.split_at(i).0
}

// ---- Builders: an entry is phase 1, on every device, not per device, unless a modifier says so ----

const fn e(key: &'static str, kind: Kind, default: DefaultValue, levels: Levels) -> Entry {
    Entry {
        key,
        page: key_page(key),
        kind,
        default,
        default_rules: &[],
        levels,
        per_device: false,
        platform: Platform::All,
        phase: 1,
        most_used: false,
        show_if: None,
        needs_native_video: false,
        wired: true,
    }
}

/// On / off (`sw`).
const fn sw(key: &'static str, default: bool, levels: Levels) -> Entry {
    e(key, Kind::Toggle, DefaultValue::Bool(default), levels)
}

const fn choice(
    key: &'static str,
    style: ChoiceStyle,
    options: &'static [&'static str],
    default: &'static str,
    levels: Levels,
) -> Entry {
    e(
        key,
        Kind::Choice {
            style,
            options,
            more_at_runtime: false,
            option_platforms: &[],
        },
        DefaultValue::Text(default),
        levels,
    )
}

/// Single choice shown as segments (`seg`).
const fn seg(
    key: &'static str,
    options: &'static [&'static str],
    default: &'static str,
    levels: Levels,
) -> Entry {
    choice(key, ChoiceStyle::Segments, options, default, levels)
}

/// Single choice shown as cards with previews (`cards`).
const fn cards(
    key: &'static str,
    options: &'static [&'static str],
    default: &'static str,
    levels: Levels,
) -> Entry {
    choice(key, ChoiceStyle::Cards, options, default, levels)
}

/// Single choice shown as a list (`sel`).
const fn sel(
    key: &'static str,
    options: &'static [&'static str],
    default: &'static str,
    levels: Levels,
) -> Entry {
    choice(key, ChoiceStyle::List, options, default, levels)
}

/// A list whose options continue at run time after the fixed ones (audio devices, the
/// user's saved layouts): the fixed ones are the schema's, the rest come from the device.
const fn sel_more(
    key: &'static str,
    options: &'static [&'static str],
    default: &'static str,
    levels: Levels,
) -> Entry {
    let mut entry = sel(key, options, default, levels);
    entry.kind = Kind::Choice {
        style: ChoiceStyle::List,
        options,
        more_at_runtime: true,
        option_platforms: &[],
    };
    entry
}

/// Multiple choice (`chips`) with the options on by default, in order.
const fn chips(
    key: &'static str,
    options: &'static [&'static str],
    default: &'static [&'static str],
    levels: Levels,
) -> Entry {
    e(
        key,
        Kind::Multi { options },
        DefaultValue::List(default),
        levels,
    )
}

/// Cards that each apply a bundle of settings (`presets.preset`): no stored value.
/// Not wired until the bundles exist.
const fn apply(key: &'static str, options: &'static [&'static str], levels: Levels) -> Entry {
    e(key, Kind::Apply { options }, DefaultValue::None, levels).not_wired()
}

const fn slider(
    key: &'static str,
    default: i64,
    (min, max, step): (i64, i64, i64),
    unit: Unit,
    levels: Levels,
) -> Entry {
    e(
        key,
        Kind::Slider {
            min,
            max,
            step,
            unit,
        },
        DefaultValue::Int(default),
        levels,
    )
}

/// Percentage slider on 0..=100, step 5.
const fn pct(key: &'static str, default: i64, levels: Levels) -> Entry {
    slider(key, default, (0, 100, 5), Unit::Percent, levels)
}

/// A file path, empty for none (`file`).
const fn file(key: &'static str, levels: Levels) -> Entry {
    e(key, Kind::File, DefaultValue::Text(""), levels)
}

/// An action is not wired until the interface carries it out.
const fn action(key: &'static str, style: ActionStyle, levels: Levels) -> Entry {
    e(key, Kind::Action { style }, DefaultValue::None, levels).not_wired()
}

/// A button that does something (`act`).
const fn act(key: &'static str, levels: Levels) -> Entry {
    action(key, ActionStyle::Button, levels)
}

/// Opens another screen (`link`).
const fn link(key: &'static str, levels: Levels) -> Entry {
    action(key, ActionStyle::Link, levels)
}

/// Shows information (`info`).
/// A binding list (`pad.remap`, `pad.combos`): overrides of the defaults, empty by default.
const fn bindings(key: &'static str, table: BindingTable, levels: Levels) -> Entry {
    e(
        key,
        Kind::Bindings { table },
        DefaultValue::List(&[]),
        levels,
    )
}

const fn info(key: &'static str, levels: Levels) -> Entry {
    action(key, ActionStyle::Info, levels)
}

impl Entry {
    /// The value belongs to this device («per dispositivo»).
    const fn device(self) -> Self {
        Self {
            per_device: true,
            ..self
        }
    }

    /// «solo PC».
    const fn pc(self) -> Self {
        Self {
            platform: Platform::Desktop,
            ..self
        }
    }

    /// «solo telefono».
    const fn phone(self) -> Self {
        Self {
            platform: Platform::Phone,
            ..self
        }
    }

    /// Works only on the native (Accelerated) video path (the screen effects).
    const fn native_video(self) -> Self {
        Self {
            needs_native_video: true,
            ..self
        }
    }

    /// Options of a choice that exist only on some devices.
    const fn options_on(self, option_platforms: &'static [(&'static str, Platform)]) -> Self {
        if let Kind::Choice {
            style,
            options,
            more_at_runtime,
            ..
        } = self.kind
        {
            Self {
                kind: Kind::Choice {
                    style,
                    options,
                    more_at_runtime,
                    option_platforms,
                },
                ..self
            }
        } else {
            self
        }
    }

    /// Shown on `page` instead of its key's prefix (the keys never change).
    const fn on_page(self, page: &'static str) -> Self {
        Self { page, ..self }
    }

    /// Shown only while `key` is one of `values` (at the level being edited).
    const fn show_if(self, key: &'static str, values: &'static [&'static str]) -> Self {
        Self {
            show_if: Some(ShowIf {
                key,
                values,
                not: false,
            }),
            ..self
        }
    }

    /// Shown only while `key` is none of `values`.
    const fn show_unless(self, key: &'static str, values: &'static [&'static str]) -> Self {
        Self {
            show_if: Some(ShowIf {
                key,
                values,
                not: true,
            }),
            ..self
        }
    }

    /// Nothing reads the value yet: shown disabled with the reason (X05).
    const fn not_wired(self) -> Self {
        Self {
            wired: false,
            ..self
        }
    }

    /// Ships in a later roadmap phase.
    const fn phase(self, phase: u8) -> Self {
        Self { phase, ..self }
    }

    /// Another default on some devices or for some consoles, first match wins.
    const fn defaults(self, default_rules: &'static [DefaultRule]) -> Self {
        Self {
            default_rules,
            ..self
        }
    }

    /// Listed in «Usate più spesso» (T01, board Settings2, 17/11): changed at the Global level
    /// right from the first settings page.
    const fn most_used(self) -> Self {
        Self {
            most_used: true,
            ..self
        }
    }
}

const G: Levels = Levels::GLOBAL;
const C: Levels = Levels::CONSOLE;
const GM: Levels = Levels::GAME;
const GC: Levels = Levels::GLOBAL_CONSOLE;
const GG: Levels = Levels::GLOBAL_GAME;
const CG: Levels = Levels::CONSOLE_GAME;
const ALL: Levels = Levels::ALL;

/// The «Avanzate» page (T14).
const ADV: &str = "advanced";

pub static ENTRIES: &[Entry] = &[
    // ---- Video (video) ----
    seg("video.scale", &["fit", "integer", "fill"], "fit", ALL).most_used(),
    sel(
        "video.aspect",
        &["original", "4_3", "16_9"],
        "original",
        ALL,
    )
    .not_wired(),
    seg("video.filter", &["none", "light", "soft"], "light", ALL),
    cards(
        "video.surround",
        &["black", "cover", "blur", "builtin", "image"],
        "black",
        ALL,
    )
    .options_on(&[("blur", Platform::Desktop)]),
    cards(
        "video.surroundSet",
        &["aurora", "sunset", "night", "forest", "graphite", "grid"],
        "aurora",
        ALL,
    )
    .show_if("video.surround", &["builtin"]),
    file("video.surroundImg", ALL).show_if("video.surround", &["image"]),
    slider("video.surroundDim", 40, (0, 80, 5), Unit::Percent, ALL)
        .show_unless("video.surround", &["black"]),
    sw("video.surroundAnim", false, G).show_if("video.surround", &["builtin"]),
    sw("video.fps", false, G).not_wired(),
    seg(
        "video.window",
        &["windowed", "fullscreen", "borderless"],
        "windowed",
        G,
    )
    .device()
    .pc()
    .not_wired(),
    seg("video.winsize", &["1x", "2x", "3x", "4x", "free"], "3x", GC)
        .device()
        .pc()
        .not_wired(),
    seg("video.portrait", &["center", "top"], "top", ALL)
        .phone()
        .phase(4)
        .not_wired(),
    seg(
        "video.orient",
        &["auto", "portrait", "landscape"],
        "auto",
        CG,
    )
    .phone()
    .phase(4)
    .not_wired(),
    seg(
        "video.path",
        &["accelerated", "compatible"],
        "accelerated",
        G,
    )
    .device(),
    sw("video.vsync", true, ALL).device(),
    sel(
        "video.api",
        &["auto", "vulkan", "d3d12", "opengl"],
        "auto",
        G,
    )
    .device()
    .not_wired(),
    // 3DS only (Azahar, `core_values/azahar.rs`); the other cores list them as unsupported.
    sel("video.res3d", &["1x", "2x", "3x", "4x"], "1x", CG).phase(2),
    sel(
        "video.texf",
        &["none", "bicubic", "scaleforce", "xbrz", "mmpx", "anime4k"],
        "none",
        CG,
    )
    .phase(3),
    // ---- Effetti schermo (fx) ----
    cards(
        "fx.fxpick",
        &[
            "none",
            "lcd",
            "lcd_grid",
            "crt",
            "crt_curved",
            "soft_pixels",
            // The user's own `.slangp`, chosen in `fx.fxfile` (none chosen: no effect).
            "file",
        ],
        "none",
        ALL,
    )
    .most_used()
    .native_video(),
    pct("fx.fxint", 60, ALL).native_video(),
    slider("fx.fxbright", 110, (80, 150, 5), Unit::Percent, ALL).native_video(),
    // The path of the user's `.slangp` (empty: none), used by the «file» option of `fx.fxpick`.
    file("fx.fxfile", G).native_video(),
    // ---- Schermi di DS e 3DS (screens) ----
    chips(
        "screens.cycle",
        &["vertical", "side", "big_small", "top_only"],
        &["vertical", "side", "big_small", "top_only"],
        CG,
    )
    .phase(2),
    link("screens.custom", GM).phase(2),
    slider("screens.ratio", 70, (50, 90, 5), Unit::Percent, CG).phase(2),
    // Its «Come sul DS» button sets 90 px (the real gap at 1×).
    slider("screens.gap", 0, (0, 120, 1), Unit::Px, CG).phase(2),
    seg(
        "screens.rot",
        &["none", "left", "right", "flip"],
        "none",
        GM,
    )
    .phase(2),
    seg("screens.swap", &["toggle", "hold"], "toggle", G).phase(2),
    sw("screens.second", false, G)
        .device()
        .pc()
        .phase(5)
        .not_wired(),
    sel(
        "screens.stereo",
        &["off", "side", "anaglyph", "interlaced"],
        "off",
        GM,
    )
    .phase(3)
    .not_wired(),
    // ---- Audio (audio) ----
    pct("audio.vol", 80, GG).most_used(),
    seg("audio.ff", &["mute", "pitch_up", "same_pitch"], "mute", GG),
    sel("audio.mic", &["blow", "real", "silence"], "blow", CG)
        .phase(2)
        .not_wired(),
    sel_more("audio.out", &["system"], "system", G)
        .device()
        .pc()
        .not_wired(),
    seg("audio.lat", &["low", "balanced", "safe"], "balanced", G)
        .device()
        .not_wired(),
    // ---- Velocità di gioco (speed) ----
    seg("speed.spstart", &["0.5", "1", "2"], "1", GG),
    seg("speed.ffmode", &["hold", "toggle"], "hold", G).not_wired(),
    seg("speed.ffval", &["2", "3", "4", "max"], "4", GG).most_used(),
    seg("speed.slow", &["0.25", "0.5", "0.75"], "0.5", G),
    // On by default for GB, GBC, GBA and NDS on a computer; off on a phone (memory, battery) and
    // for 3DS (states of tens of MB). Owner, 2026-11-21 (Trello #89).
    sw("speed.rew", true, ALL).defaults(&[
        DefaultRule {
            device: Some(Device::Phone),
            consoles: &[],
            value: DefaultValue::Bool(false),
        },
        DefaultRule {
            device: None,
            consoles: &[ConsoleId::N3ds],
            value: DefaultValue::Bool(false),
        },
    ]),
    slider("speed.rewlen", 30, (10, 120, 10), Unit::Seconds, ALL).device(),
    sel("speed.runahead", &["off", "1", "2", "3"], "off", GM)
        .phase(5)
        .not_wired(),
    // ---- Salvataggi (saves) ----
    seg("saves.slots", &["3", "5", "10"], "10", G),
    seg("saves.autoq", &["1min", "5min", "never"], "5min", ALL).most_used(),
    sw("saves.autoexit", true, ALL),
    seg("saves.resume", &["ask", "always", "ingame"], "always", ALL),
    sw("saves.undo", true, G),
    slider("saves.backups", 5, (0, 10, 1), Unit::Copies, G),
    seg("saves.rtc", &["real", "offset", "fixed"], "real", GM).not_wired(),
    act("saves.import", GM),
    act("saves.savedir", G).device(),
    // ---- Controller e tastiera (pad) ----
    link("pad.dev1", G),
    link("pad.dev2", G).pc(),
    seg("pad.ab", &["nintendo", "printed"], "nintendo", G),
    bindings("pad.remap", BindingTable::Remap, ALL),
    link("pad.turbo", ALL),
    sel("pad.mod", &["l3", "r3", "select"], "l3", G),
    bindings("pad.combos", BindingTable::Combos, ALL),
    sel("pad.lid", &["l3_select", "menu_only"], "l3_select", GC)
        .phase(2)
        .not_wired(),
    slider("pad.dead", 15, (0, 50, 1), Unit::Percent, G),
    slider("pad.stickdpad", 50, (20, 90, 5), Unit::Percent, GC),
    slider("pad.trig", 50, (10, 90, 5), Unit::Percent, G),
    slider("pad.rumble", 70, (0, 100, 10), Unit::Percent, GG).phase(2),
    sel(
        "pad.gyro",
        &["controller", "phone", "mouse", "right_stick", "off"],
        "controller",
        CG,
    )
    .options_on(&[("phone", Platform::Phone)])
    .phase(3)
    .not_wired(),
    sel(
        "pad.pointer",
        &["mouse", "right_stick", "touch"],
        "mouse",
        CG,
    )
    .options_on(&[("touch", Platform::Touch)])
    .phase(2),
    seg("pad.confirm", &["right", "bottom"], "right", G),
    sel("pad.repeat", &["slow", "normal", "fast"], "normal", G),
    // ---- Controlli touch (touch) ----
    link("touch.tedit", ALL).phone().phase(4),
    sel_more("touch.tport", &["default"], "default", ALL)
        .phone()
        .phase(4)
        .not_wired(),
    sel_more("touch.tland", &["default"], "default", ALL)
        .phone()
        .phase(4)
        .not_wired(),
    link("touch.tskin", G).phone().phase(4),
    slider("touch.topac", 60, (10, 100, 5), Unit::Percent, ALL)
        .phone()
        .phase(4),
    slider("touch.thapt", 40, (0, 100, 10), Unit::Percent, G)
        .device()
        .phone()
        .phase(4)
        .not_wired(),
    seg("touch.thide", &["never", "5s", "10s"], "never", G)
        .phone()
        .phase(4)
        .not_wired(),
    sw("touch.tpad", true, G).phone().phase(4).not_wired(),
    // ---- Generale (general) ----
    sel("general.lang", &["system", "it", "en"], "system", G),
    seg(
        "general.theme",
        &["system", "dark", "black", "light"],
        "system",
        G,
    )
    .most_used(),
    e(
        "general.accent",
        Kind::Colour {
            swatches: &["#faa40f", "#7152f7", "#2f7ae5", "#1f9d8a", "#c2417a"],
        },
        // Amber, the app icon's colour (Trello #96). The interface reads it, and the swatches,
        // from the generated catalogue (`app/src/generated/settings-catalog.json`, `colours`).
        DefaultValue::Text("#faa40f"),
        G,
    ),
    seg("general.ui", &["auto", "desktop", "tv"], "auto", G)
        .device()
        .pc()
        .not_wired(),
    seg("general.start", &["library", "last_game"], "library", G).not_wired(),
    sw("general.bgpause", true, G).device().not_wired(),
    sw("general.confirmdel", true, G).not_wired(),
    seg("general.updates", &["auto", "ask", "off"], "ask", G)
        .device()
        .not_wired(),
    // ---- Libreria (library) ----
    e("library.folders", Kind::Folders, DefaultValue::List(&[]), G).device(),
    sw("library.scanstart", true, G).device(),
    sw("library.watch", false, G).device().pc(),
    seg(
        "library.sort",
        &["last_played", "name", "play_time", "added"],
        "last_played",
        G,
    ),
    sw("library.group", false, G).not_wired(),
    sel("library.region", &["eu_us", "us_eu", "jp"], "eu_us", G).not_wired(),
    link("library.hidden", G),
    sel(
        "library.covers",
        &["color_title", "rom_icon", "title_only"],
        "color_title",
        G,
    )
    .not_wired(),
    // ---- Cloud (cloud) ----
    act("cloud.acct", G).device(),
    action("cloud.space", ActionStyle::Storage, G),
    sw("cloud.ssaves", true, G).not_wired(),
    seg("cloud.squick", &["all", "last", "none"], "all", GC).not_wired(),
    sw("cloud.ssettings", true, G).not_wired(),
    link("cloud.sgames", G),
    seg("cloud.sauto", &["game", "manual"], "game", G).not_wired(),
    sw("cloud.swifi", true, G).device().phone().not_wired(),
    // ---- Accessibilità (a11y) ----
    slider("a11y.textsize", 100, (80, 150, 10), Unit::Percent, G).device(),
    seg("a11y.motion", &["system", "on", "off"], "system", G),
    seg("a11y.transp", &["system", "on", "off"], "system", G),
    sw("a11y.toggle", false, G),
    sel(
        "a11y.cblind",
        &["none", "protan", "deutan", "tritan"],
        "none",
        GG,
    ),
    // ---- Preimpostazioni (presets) ----
    // An action, not a value: applying one changes the settings it bundles.
    // TODO(presets): the bundles, and the confirmation that lists their changes.
    apply(
        "presets.preset",
        &["faithful", "performance", "quality", "battery"],
        G,
    ),
    act("presets.export", G),
    act("presets.importset", G),
    // ---- BIOS e firmware (bios) ----
    // The Game Boy / Color boot ROMs and the DSi BIOS and firmware: owner, 2026-11-21 (Trello #89).
    file("bios.bgb", G).device(),
    file("bios.bgbc", G).device(),
    file("bios.bgba", G).device(),
    file("bios.bds7", G).device().phase(2),
    file("bios.bds9", G).device().phase(2),
    file("bios.bdsfw", G).device().phase(2),
    file("bios.bdsi7", G).device().phase(2),
    file("bios.bdsi9", G).device().phase(2),
    file("bios.bdsifw", G).device().phase(2),
    file("bios.bdsi", G).device().phase(2),
    file("bios.b3ds", G).device().phase(3).not_wired(),
    // ---- Opzioni per console (consoles) ----
    seg(
        "consoles.gbmodel",
        &["auto", "gb", "gbc", "sgb"],
        "auto",
        CG,
    ),
    sel(
        "consoles.gbpal",
        &["green", "pocket", "light", "gbc"],
        "green",
        CG,
    ),
    sw("consoles.sgb", true, CG),
    seg("consoles.gbacc", &["off", "gba", "auto"], "auto", CG),
    sel("consoles.gbablend", &["off", "auto", "on"], "off", CG),
    pct("consoles.solar", 50, GM),
    seg("consoles.dsmode", &["auto", "ds", "dsi"], "auto", GM).phase(2),
    // Empty = the OS user name, resolved when the DS core arrives.
    e("consoles.dsname", Kind::Text, DefaultValue::Text(""), C).phase(2),
    sel(
        "consoles.dslang",
        &["ja", "en", "fr", "de", "it", "es"],
        "it",
        C,
    )
    .phase(2),
    sel(
        "consoles.slot2",
        &["none", "rumble", "memory", "gba_game"],
        "none",
        GM,
    )
    .phase(2),
    seg("consoles.n3dsmodel", &["o3ds", "n3ds"], "n3ds", CG).phase(3),
    sel(
        "consoles.n3dsreg",
        &["auto", "jpn", "usa", "eur", "kor", "chn", "twn"],
        "auto",
        CG,
    )
    .phase(3),
    // ---- Avanzate (advanced, T14): Prestazioni, motori, Strumenti avanzati ----
    // The keys keep their `perf.` / `cores.` / `adv.` prefix (files, i18n); the page is `advanced`.
    seg(
        "perf.frameskip",
        &["auto", "never", "1", "2", "3", "4"],
        "auto",
        ALL,
    )
    .device()
    .on_page(ADV),
    sw("perf.jit", true, CG).phase(2).on_page(ADV).not_wired(),
    sw("perf.shcache", true, C)
        .device()
        .phase(3)
        .on_page(ADV)
        .not_wired(),
    // 3DS only (Azahar); the other cores list it as unsupported.
    slider("perf.cpuclock", 100, (50, 400, 25), Unit::Percent, GM)
        .phase(3)
        .on_page(ADV),
    sw("cores.mgbabios", true, CG).on_page(ADV),
    sw("cores.mgbaskip", false, CG).on_page(ADV),
    sw("cores.mgbalp", false, CG).on_page(ADV),
    sw("cores.mgbaopp", false, CG).on_page(ADV),
    link("cores.mgbaall", CG).on_page(ADV),
    sel("cores.mdsrender", &["software", "opengl"], "software", CG)
        .phase(2)
        .on_page(ADV),
    seg("cores.mdsboot", &["direct", "menu"], "direct", CG)
        .phase(2)
        .on_page(ADV),
    link("cores.mdsall", CG).phase(2).on_page(ADV),
    seg("cores.azaudio", &["hle", "lle"], "hle", CG)
        .phase(3)
        .on_page(ADV),
    link("cores.azall", CG).phase(3).on_page(ADV),
    sw("adv.advon", false, G).phase(5).on_page(ADV).not_wired(),
    sw("adv.inputs", false, G).phase(5).on_page(ADV).not_wired(),
    sw("adv.framecnt", false, G)
        .phase(5)
        .on_page(ADV)
        .not_wired(),
    link("adv.movies", G).phase(5).on_page(ADV),
    link("adv.cheatsearch", G).phase(5).on_page(ADV),
    link("adv.devtools", G).pc().phase(5).on_page(ADV),
    // ---- Online e integrazioni (online) ----
    // Empty = none («Nessuno»).
    e("online.wfc", Kind::Text, DefaultValue::Text(""), G).phase(2),
    act("online.ra", G).device().phase(5),
    sw("online.discord", false, G)
        .device()
        .pc()
        .phase(5)
        .not_wired(),
    // ---- Informazioni (about) ----
    info("about.ver", G),
    act("about.upd", G),
    info("about.emgba", G),
    info("about.emds", G).phase(2),
    info("about.emaz", G).phase(3),
    link("about.lic", G),
    act("about.log", G),
];

/// Extra words the settings search matches, in every language, beside the entry's name and
/// description: `(key, words)`, where a key `<page>.*` covers every entry of the page
/// (Cowork, Trello #41). Matched without case and accents, like the names.
pub static SYNONYMS: &[(&str, &[&str])] = &[
    ("video.vsync", &["tearing", "strappo"]),
    (
        "video.scale",
        &["zoom", "pixel perfect", "integer", "intero"],
    ),
    ("video.filter", &["bilineare", "bilinear", "smooth", "blur"]),
    (
        "fx.fxpick",
        &["shader", "filtro", "filter", "crt", "scanline", "lcd"],
    ),
    (
        "video.path",
        &[
            "schermo nero",
            "black screen",
            "nativo",
            "native",
            "webview",
        ],
    ),
    ("video.api", &["vulkan", "directx", "d3d", "opengl"]),
    ("general.theme", &["dark mode", "modalità scura", "oled"]),
    (
        "a11y.textsize",
        &["font", "carattere", "testo grande", "large text"],
    ),
    ("audio.vol", &["suono", "sound"]),
    ("speed.ffval", &["turbo", "avanti veloce", "fast forward"]),
    ("speed.rew", &["rewind", "indietro"]),
    (
        "perf.frameskip",
        &["lag", "scatti", "stutter", "lento", "slow"],
    ),
    ("speed.runahead", &["input lag", "ritardo", "latency"]),
    ("perf.jit", &["dynarec", "recompiler"]),
    ("pad.mod", &["hotkey", "scorciatoie", "shortcuts"]),
    ("pad.dead", &["deadzone", "drift"]),
    ("saves.slots", &["save state", "stati", "slot"]),
    ("saves.autoq", &["autosave"]),
    ("cloud.ssaves", &["sync", "sincronizzazione", "drive"]),
    (
        "video.surround",
        &["bordi", "borders", "wallpaper", "sfondo"],
    ),
    ("bios.*", &["bios", "firmware"]),
    (
        "general.lang",
        &["lingua", "language", "italiano", "english"],
    ),
];

/// The sections of each page, in display order, with their entries in order: the headings the
/// interface shows (`settings.sections.<page>.<id>`). Every entry is in exactly one section of
/// its page (tested).
pub static SECTIONS: &[Section] = &[
    Section {
        page: "video",
        id: "image",
        keys: &[
            "video.scale",
            "video.aspect",
            "video.filter",
            "video.surround",
            "video.surroundSet",
            "video.surroundImg",
            "video.surroundDim",
            "video.surroundAnim",
            "video.fps",
        ],
    },
    Section {
        page: "video",
        id: "window",
        keys: &[
            "video.window",
            "video.winsize",
            "video.portrait",
            "video.orient",
        ],
    },
    Section {
        page: "video",
        id: "gpu",
        keys: &["video.path", "video.vsync", "video.api"],
    },
    Section {
        page: "video",
        id: "3d",
        keys: &["video.res3d", "video.texf"],
    },
    Section {
        page: "fx",
        id: "effect",
        keys: &["fx.fxpick"],
    },
    Section {
        page: "fx",
        id: "adjust",
        keys: &["fx.fxint", "fx.fxbright", "fx.fxfile"],
    },
    Section {
        page: "screens",
        id: "layout",
        keys: &[
            "screens.cycle",
            "screens.custom",
            "screens.ratio",
            "screens.gap",
            "screens.rot",
        ],
    },
    Section {
        page: "screens",
        id: "other",
        keys: &["screens.swap", "screens.second", "screens.stereo"],
    },
    Section {
        page: "audio",
        id: "audio",
        keys: &["audio.vol", "audio.ff", "audio.mic"],
    },
    Section {
        page: "audio",
        id: "device",
        keys: &["audio.out", "audio.lat"],
    },
    Section {
        page: "speed",
        id: "speed",
        keys: &["speed.spstart"],
    },
    Section {
        page: "speed",
        id: "ff",
        keys: &["speed.ffmode", "speed.ffval", "speed.slow"],
    },
    Section {
        page: "speed",
        id: "rewind",
        keys: &["speed.rew", "speed.rewlen", "speed.runahead"],
    },
    Section {
        page: "saves",
        id: "quick",
        keys: &[
            "saves.slots",
            "saves.autoq",
            "saves.autoexit",
            "saves.resume",
            "saves.undo",
        ],
    },
    Section {
        page: "saves",
        id: "ingame",
        keys: &["saves.backups", "saves.rtc", "saves.import"],
    },
    Section {
        page: "saves",
        id: "folder",
        keys: &["saves.savedir"],
    },
    Section {
        page: "pad",
        id: "devices",
        keys: &["pad.dev1", "pad.dev2"],
    },
    Section {
        page: "pad",
        id: "buttons",
        keys: &["pad.ab", "pad.remap", "pad.turbo"],
    },
    Section {
        page: "pad",
        id: "shortcuts",
        keys: &["pad.mod", "pad.combos", "pad.lid"],
    },
    Section {
        page: "pad",
        id: "sticks",
        keys: &[
            "pad.dead",
            "pad.stickdpad",
            "pad.trig",
            "pad.rumble",
            "pad.gyro",
            "pad.pointer",
        ],
    },
    Section {
        page: "pad",
        id: "menus",
        keys: &["pad.confirm", "pad.repeat"],
    },
    Section {
        page: "touch",
        id: "layout",
        keys: &["touch.tedit", "touch.tport", "touch.tland", "touch.tskin"],
    },
    Section {
        page: "touch",
        id: "behaviour",
        keys: &["touch.topac", "touch.thapt", "touch.thide", "touch.tpad"],
    },
    Section {
        page: "general",
        id: "look",
        keys: &[
            "general.lang",
            "general.theme",
            "general.accent",
            "general.ui",
        ],
    },
    Section {
        page: "general",
        id: "behaviour",
        keys: &[
            "general.start",
            "general.bgpause",
            "general.confirmdel",
            "general.updates",
        ],
    },
    Section {
        page: "library",
        id: "folders",
        keys: &["library.folders", "library.scanstart", "library.watch"],
    },
    Section {
        page: "library",
        id: "view",
        keys: &[
            "library.sort",
            "library.group",
            "library.region",
            "library.hidden",
            "library.covers",
        ],
    },
    Section {
        page: "cloud",
        id: "account",
        keys: &["cloud.acct", "cloud.space"],
    },
    Section {
        page: "cloud",
        id: "what",
        keys: &[
            "cloud.ssaves",
            "cloud.squick",
            "cloud.ssettings",
            "cloud.sgames",
        ],
    },
    Section {
        page: "cloud",
        id: "when",
        keys: &["cloud.sauto", "cloud.swifi"],
    },
    Section {
        page: "a11y",
        id: "ui",
        keys: &["a11y.textsize", "a11y.motion", "a11y.transp"],
    },
    Section {
        page: "a11y",
        id: "game",
        keys: &["a11y.toggle", "a11y.cblind"],
    },
    Section {
        page: "presets",
        id: "ready",
        keys: &["presets.preset"],
    },
    Section {
        page: "presets",
        id: "file",
        keys: &["presets.export", "presets.importset"],
    },
    Section {
        page: "bios",
        id: "gb",
        keys: &["bios.bgb", "bios.bgbc"],
    },
    Section {
        page: "bios",
        id: "gba",
        keys: &["bios.bgba"],
    },
    Section {
        page: "bios",
        id: "nds",
        keys: &[
            "bios.bds7",
            "bios.bds9",
            "bios.bdsfw",
            "bios.bdsi7",
            "bios.bdsi9",
            "bios.bdsifw",
            "bios.bdsi",
        ],
    },
    Section {
        page: "bios",
        id: "3ds",
        keys: &["bios.b3ds"],
    },
    Section {
        page: "consoles",
        id: "gb",
        keys: &["consoles.gbmodel", "consoles.gbpal", "consoles.sgb"],
    },
    Section {
        page: "consoles",
        id: "gba",
        keys: &["consoles.gbacc", "consoles.gbablend", "consoles.solar"],
    },
    Section {
        page: "consoles",
        id: "nds",
        keys: &[
            "consoles.dsmode",
            "consoles.dsname",
            "consoles.dslang",
            "consoles.slot2",
        ],
    },
    Section {
        page: "consoles",
        id: "3ds",
        keys: &["consoles.n3dsmodel", "consoles.n3dsreg"],
    },
    Section {
        page: "advanced",
        id: "perf",
        keys: &[
            "perf.frameskip",
            "perf.jit",
            "perf.shcache",
            "perf.cpuclock",
        ],
    },
    Section {
        page: "advanced",
        id: "mgba",
        keys: &[
            "cores.mgbabios",
            "cores.mgbaskip",
            "cores.mgbalp",
            "cores.mgbaopp",
            "cores.mgbaall",
        ],
    },
    Section {
        page: "advanced",
        id: "melonds",
        keys: &["cores.mdsrender", "cores.mdsboot", "cores.mdsall"],
    },
    Section {
        page: "advanced",
        id: "azahar",
        keys: &["cores.azaudio", "cores.azall"],
    },
    Section {
        page: "advanced",
        id: "tools",
        keys: &[
            "adv.advon",
            "adv.inputs",
            "adv.framecnt",
            "adv.movies",
            "adv.cheatsearch",
            "adv.devtools",
        ],
    },
    Section {
        page: "online",
        id: "online",
        keys: &["online.wfc", "online.ra", "online.discord"],
    },
    Section {
        page: "about",
        id: "app",
        keys: &["about.ver", "about.upd"],
    },
    Section {
        page: "about",
        id: "engines",
        keys: &["about.emgba", "about.emds", "about.emaz"],
    },
    Section {
        page: "about",
        id: "help",
        keys: &["about.lic", "about.log"],
    },
];

#[cfg(test)]
mod tests {
    use super::{ENTRIES, PAGES};
    use crate::settings::schema::{DefaultValue, Kind, Platform};

    #[test]
    fn has_the_designs_entries_with_unique_keys_on_known_pages() {
        // The design's 155, plus five BIOS rows (Trello #89).
        assert_eq!(ENTRIES.len(), 160);
        for (i, entry) in ENTRIES.iter().enumerate() {
            assert!(
                ENTRIES[..i].iter().all(|other| other.key != entry.key),
                "duplicate key {}",
                entry.key
            );
            assert!(PAGES.contains(&entry.page()), "unknown page {}", entry.key);
            assert!((1..=5).contains(&entry.phase), "phase of {}", entry.key);
        }
    }

    #[test]
    fn pages_come_in_catalogue_order() {
        let order: Vec<&str> = ENTRIES.iter().fold(Vec::new(), |mut acc, entry| {
            if acc.last() != Some(&entry.page()) {
                acc.push(entry.page());
            }
            acc
        });
        assert_eq!(order, PAGES);
    }

    #[test]
    fn every_default_is_valid_for_its_kind() {
        for entry in ENTRIES {
            let value = entry.default_value();
            if entry.has_value() {
                assert!(
                    value.as_ref().is_some_and(|v| entry.accepts_value(v)),
                    "{}: invalid default {value:?}",
                    entry.key
                );
            } else {
                assert_eq!(entry.default, DefaultValue::None, "{}", entry.key);
            }
        }
    }

    #[test]
    fn theme_options_match_the_app_themes() {
        let theme = ENTRIES.iter().find(|e| e.key == "general.theme");
        let options = theme.map(|e| (e.kind, e.default));
        assert!(matches!(
            options,
            Some((
                Kind::Choice {
                    options: &["system", "dark", "black", "light"],
                    ..
                },
                DefaultValue::Text("system")
            ))
        ));
    }

    #[test]
    fn the_most_used_are_six_global_phase_one_values_for_every_device() {
        let most: Vec<&str> = ENTRIES
            .iter()
            .filter(|e| e.most_used)
            .map(|e| e.key)
            .collect();
        // Board Settings2 (T01, 17/11), in catalogue order.
        assert_eq!(
            most,
            [
                "video.scale",
                "fx.fxpick",
                "audio.vol",
                "speed.ffval",
                "saves.autoq",
                "general.theme"
            ]
        );
        for entry in ENTRIES.iter().filter(|e| e.most_used) {
            assert!(entry.has_value(), "{}", entry.key);
            assert!(entry.levels.global, "{} is shown at Global", entry.key);
            assert_eq!(entry.phase, 1, "{}", entry.key);
            assert_eq!(entry.platform, Platform::All, "{}", entry.key);
            assert!(!entry.per_device, "{}", entry.key);
        }
    }
}
