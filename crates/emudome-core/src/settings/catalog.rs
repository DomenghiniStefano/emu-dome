//! The static catalogue: every setting of the app, page by page, in the design's order
//! (Cowork's settings catalogue, 157 entries, Trello comment of 2026-11-17).
//!
//! Option ids are stable snake_case English ids derived from the design's Italian labels; the
//! visible texts live in the interface's message files (`settings.<page>.<key>.option.<id>`).
//!
//! Slider ranges and steps are not in the design, which gives only the default: the ranges below
//! are a sensible guess that contains the default on the step grid, and Cowork confirms them in 2b.

use super::schema::{ActionStyle, ChoiceStyle, DefaultValue, Entry, Kind, Levels, Platform, Unit};

/// Phase the app is in: entries of later phases stay hidden.
pub const CURRENT_PHASE: u8 = 1;

/// Page ids in display order.
pub static PAGES: &[&str] = &[
    "video", "fx", "screens", "audio", "speed", "saves", "pad", "touch", "general", "library",
    "cloud", "a11y", "presets", "bios", "consoles", "cores", "perf", "online", "adv", "about",
];

// ---- Builders: an entry is phase 1, on every device, not per device, unless a modifier says so ----

const fn e(key: &'static str, kind: Kind, default: DefaultValue, levels: Levels) -> Entry {
    Entry {
        key,
        kind,
        default,
        levels,
        per_device: false,
        platform: Platform::All,
        phase: 1,
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
            placeholder: false,
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

/// Single choice shown as a list (`sel`). The design does not list these options yet: the set
/// holds only the default's id, and it is marked as a placeholder.
// TODO(2b): options from Cowork for every `sel` entry.
const fn sel(key: &'static str, default: &'static [&'static str], levels: Levels) -> Entry {
    e(
        key,
        Kind::Choice {
            style: ChoiceStyle::List,
            options: default,
            placeholder: true,
        },
        DefaultValue::Text(default[0]),
        levels,
    )
}

/// Multiple choice (`chips`). The design gives no default: every option is on.
// TODO(2b): the default selection from Cowork.
const fn chips(key: &'static str, options: &'static [&'static str], levels: Levels) -> Entry {
    e(
        key,
        Kind::Multi { options },
        DefaultValue::List(options),
        levels,
    )
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

const fn action(key: &'static str, style: ActionStyle, levels: Levels) -> Entry {
    e(key, Kind::Action { style }, DefaultValue::None, levels)
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

    /// Ships in a later roadmap phase.
    const fn phase(self, phase: u8) -> Self {
        Self { phase, ..self }
    }
}

const G: Levels = Levels::GLOBAL;
const C: Levels = Levels::CONSOLE;
const GM: Levels = Levels::GAME;
const GC: Levels = Levels::GLOBAL_CONSOLE;
const GG: Levels = Levels::GLOBAL_GAME;
const CG: Levels = Levels::CONSOLE_GAME;
const ALL: Levels = Levels::ALL;

pub static ENTRIES: &[Entry] = &[
    // ---- Video (video) ----
    seg("video.scale", &["fit", "integer", "fill"], "fit", ALL),
    sel("video.aspect", &["console"], ALL),
    seg("video.filter", &["none", "light", "soft"], "light", ALL),
    // `blurred_game` is PC only in the design: per-option platforms come with 2b.
    cards(
        "video.surround",
        &[
            "black",
            "cover_colour",
            "blurred_game",
            "backdrop",
            "custom_image",
        ],
        "black",
        ALL,
    ),
    cards(
        "video.surroundSet",
        &["aurora", "sunset", "night", "forest", "graphite", "checks"],
        "aurora",
        ALL,
    ),
    file("video.surroundImg", ALL),
    pct("video.surroundDim", 40, ALL),
    sw("video.surroundAnim", false, G),
    sw("video.fps", false, G),
    seg(
        "video.window",
        &["windowed", "fullscreen", "borderless"],
        "windowed",
        G,
    )
    .device()
    .pc(),
    seg("video.winsize", &["x1", "x2", "x3", "x4", "free"], "x3", GC)
        .device()
        .pc(),
    seg("video.portrait", &["center", "top"], "top", ALL)
        .phone()
        .phase(4),
    seg(
        "video.orient",
        &["auto", "portrait", "landscape"],
        "auto",
        CG,
    )
    .phone()
    .phase(4),
    seg(
        "video.path",
        &["accelerated", "compatible"],
        "accelerated",
        G,
    )
    .device(),
    sw("video.vsync", true, ALL).device(),
    sel("video.api", &["auto"], G).device(),
    sel("video.res3d", &["x1"], CG).phase(2),
    sel("video.texf", &["none"], CG).phase(3),
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
        ],
        "none",
        ALL,
    ),
    pct("fx.fxint", 60, ALL),
    slider("fx.fxbright", 110, (50, 150, 5), Unit::Percent, ALL),
    act("fx.fxfile", G),
    // ---- Schermi di DS e 3DS (screens) ----
    chips(
        "screens.cycle",
        &["vertical", "side_by_side", "large_small", "top_only"],
        CG,
    )
    .phase(2),
    link("screens.custom", GM).phase(2),
    slider("screens.ratio", 70, (50, 90, 5), Unit::Percent, CG).phase(2),
    slider("screens.gap", 0, (0, 100, 1), Unit::Px, CG).phase(2),
    seg(
        "screens.rot",
        &["none", "left", "right", "upside_down"],
        "none",
        GM,
    )
    .phase(2),
    seg("screens.swap", &["toggle", "hold"], "toggle", G).phase(2),
    sw("screens.second", false, G).device().pc().phase(5),
    sel("screens.stereo", &["off"], GM).phase(3),
    // ---- Audio (audio) ----
    pct("audio.vol", 80, GG),
    seg(
        "audio.ff",
        &["mute", "higher_pitch", "same_pitch"],
        "mute",
        GG,
    ),
    sel("audio.mic", &["simulated_blow"], CG).phase(2),
    sel("audio.out", &["system"], G).device().pc(),
    seg("audio.lat", &["low", "balanced", "safe"], "balanced", G).device(),
    // ---- Velocità di gioco (speed) ----
    seg("speed.spstart", &["x0_5", "x1", "x2"], "x1", GG),
    chips(
        "speed.splist",
        &["x0_5", "x1", "x1_5", "x2", "x3", "x4", "max"],
        G,
    ),
    seg("speed.ffmode", &["hold", "toggle"], "hold", G),
    seg("speed.ffval", &["x2", "x3", "x4", "max"], "x4", GG),
    seg("speed.slow", &["x0_25", "x0_5", "x0_75"], "x0_5", G),
    sw("speed.rew", true, ALL),
    slider("speed.rewlen", 30, (10, 120, 5), Unit::Seconds, ALL).device(),
    sel("speed.runahead", &["off"], GM).phase(5),
    // ---- Salvataggi (saves) ----
    seg("saves.slots", &["three", "five", "ten"], "ten", G),
    seg(
        "saves.autoq",
        &["every_minute", "every_5_minutes", "never"],
        "every_5_minutes",
        ALL,
    ),
    sw("saves.autoexit", true, ALL),
    seg(
        "saves.resume",
        &["ask", "always", "game_save"],
        "always",
        ALL,
    ),
    sw("saves.undo", true, G),
    slider("saves.backups", 5, (1, 20, 1), Unit::Copies, G),
    seg("saves.rtc", &["real", "shifted", "fixed_date"], "real", GM),
    act("saves.import", GM),
    act("saves.savedir", G).device(),
    // ---- Controller e tastiera (pad) ----
    link("pad.dev1", G),
    link("pad.dev2", G).pc(),
    seg("pad.ab", &["nintendo", "printed"], "nintendo", G),
    link("pad.remap", ALL),
    link("pad.turbo", ALL),
    sel("pad.mod", &["l3"], G),
    link("pad.combos", ALL),
    sel("pad.lid", &["l3_select"], GC).phase(2),
    pct("pad.dead", 15, G),
    pct("pad.stickdpad", 50, GC),
    pct("pad.trig", 50, G),
    pct("pad.rumble", 70, GG).phase(2),
    sel("pad.gyro", &["controller_gyro"], CG).phase(3),
    sel("pad.pointer", &["mouse"], CG).phase(2),
    seg("pad.confirm", &["a_right", "a_bottom"], "a_right", G),
    sel("pad.repeat", &["normal"], G),
    // ---- Controlli touch (touch) ----
    link("touch.tedit", ALL).phone().phase(4),
    sel("touch.tport", &["default"], ALL).phone().phase(4),
    sel("touch.tland", &["default"], ALL).phone().phase(4),
    link("touch.tskin", G).phone().phase(4),
    pct("touch.topac", 60, ALL).phone().phase(4),
    pct("touch.thapt", 40, G).device().phone().phase(4),
    seg(
        "touch.thide",
        &["never", "after_5s", "after_10s"],
        "never",
        G,
    )
    .phone()
    .phase(4),
    sw("touch.tpad", true, G).phone().phase(4),
    // ---- Generale (general) ----
    sel("general.lang", &["system"], G),
    seg(
        "general.theme",
        &["system", "dark", "black", "light"],
        "system",
        G,
    ),
    e(
        "general.accent",
        Kind::Colour {
            swatches: &["#7152f7", "#2f7ae5", "#1f9d8a", "#d9622b", "#c2417a"],
        },
        DefaultValue::Text("#7152f7"),
        G,
    ),
    seg("general.ui", &["auto", "desktop", "tv"], "auto", G)
        .device()
        .pc(),
    seg("general.start", &["library", "last_game"], "library", G),
    sw("general.bgpause", true, G).device(),
    sw("general.confirmdel", true, G),
    link("general.qmorder", G),
    seg("general.updates", &["auto", "ask", "off"], "ask", G).device(),
    // ---- Libreria (library) ----
    e("library.folders", Kind::Folders, DefaultValue::List(&[]), G).device(),
    sw("library.scanstart", true, G).device(),
    sw("library.watch", false, G).device().pc(),
    seg(
        "library.sort",
        &["last_played", "name", "play_time", "date_added"],
        "last_played",
        G,
    ),
    sw("library.group", false, G),
    sel("library.region", &["europe_america"], G),
    link("library.hidden", G),
    sel("library.covers", &["colour_title"], G),
    // ---- Cloud (cloud) ----
    act("cloud.acct", G).device(),
    action("cloud.space", ActionStyle::Storage, G),
    sw("cloud.ssaves", true, G),
    seg("cloud.squick", &["all", "latest", "none"], "all", GC),
    sw("cloud.ssettings", true, G),
    link("cloud.sgames", G),
    seg(
        "cloud.sauto",
        &["start_and_exit", "manual"],
        "start_and_exit",
        G,
    ),
    sw("cloud.swifi", true, G).device().phone(),
    // ---- Accessibilità (a11y) ----
    slider("a11y.textsize", 100, (80, 200, 10), Unit::Percent, G).device(),
    seg("a11y.motion", &["system", "yes", "no"], "system", G),
    seg("a11y.transp", &["system", "yes", "no"], "system", G),
    sw("a11y.toggle", false, G),
    sel("a11y.cblind", &["none"], GG),
    // ---- Preimpostazioni (presets) ----
    // TODO(2b): the design gives no default preset; the first one stands in.
    cards(
        "presets.preset",
        &["faithful", "performance", "quality", "battery_saver"],
        "faithful",
        G,
    ),
    act("presets.export", G),
    act("presets.importset", G),
    // ---- BIOS e firmware (bios) ----
    file("bios.bgba", G).device(),
    file("bios.bds7", G).device().phase(2),
    file("bios.bds9", G).device().phase(2),
    file("bios.bdsfw", G).device().phase(2),
    file("bios.bdsi", G).device().phase(2),
    file("bios.b3ds", G).device().phase(3),
    // ---- Opzioni per console (consoles) ----
    seg(
        "consoles.gbmodel",
        &["auto", "game_boy", "game_boy_color", "super_game_boy"],
        "auto",
        CG,
    ),
    sel("consoles.gbpal", &["original_green"], CG),
    sw("consoles.sgb", true, CG),
    seg("consoles.gbacc", &["off", "gba_screen", "auto"], "auto", CG),
    sel("consoles.gbablend", &["off"], CG),
    pct("consoles.solar", 50, GM),
    seg("consoles.dsmode", &["auto", "ds", "dsi"], "auto", GM).phase(2),
    // Empty = the OS user name, resolved when the DS core arrives.
    e("consoles.dsname", Kind::Text, DefaultValue::Text(""), C).phase(2),
    sel("consoles.dslang", &["italian"], C).phase(2),
    sel("consoles.slot2", &["none"], GM).phase(2),
    seg("consoles.n3dsmodel", &["original", "new"], "new", CG).phase(3),
    sel("consoles.n3dsreg", &["auto"], CG).phase(3),
    // ---- Opzioni avanzate dell'emulatore (cores) ----
    sw("cores.mgbabios", true, CG),
    sw("cores.mgbaskip", false, CG),
    sw("cores.mgbalp", false, CG),
    sw("cores.mgbaopp", false, CG),
    link("cores.mgbaall", CG),
    sel("cores.mdsrender", &["software"], CG).phase(2),
    seg("cores.mdsboot", &["direct", "ds_menu"], "direct", CG).phase(2),
    link("cores.mdsall", CG).phase(2),
    seg("cores.azaudio", &["hle", "lle"], "hle", CG).phase(3),
    link("cores.azall", CG).phase(3),
    // ---- Prestazioni (perf) ----
    seg(
        "perf.frameskip",
        &["auto", "never", "one", "two", "three", "four"],
        "auto",
        ALL,
    )
    .device(),
    sw("perf.jit", true, CG).phase(2),
    sw("perf.shcache", true, C).device().phase(3),
    slider("perf.cpuclock", 100, (50, 200, 10), Unit::Percent, GM).phase(3),
    // ---- Online e integrazioni (online) ----
    // Empty = none («Nessuno»).
    e("online.wfc", Kind::Text, DefaultValue::Text(""), G).phase(2),
    act("online.ra", G).device().phase(5),
    sw("online.discord", false, G).device().pc().phase(5),
    // ---- Strumenti avanzati (adv) ----
    sw("adv.advon", false, G).phase(5),
    sw("adv.inputs", false, G).phase(5),
    sw("adv.framecnt", false, G).phase(5),
    link("adv.movies", G).phase(5),
    link("adv.cheatsearch", G).phase(5),
    link("adv.devtools", G).pc().phase(5),
    // ---- Informazioni (about) ----
    info("about.ver", G),
    act("about.upd", G),
    info("about.emgba", G),
    info("about.emds", G).phase(2),
    info("about.emaz", G).phase(3),
    link("about.lic", G),
    act("about.log", G),
];

#[cfg(test)]
mod tests {
    use super::{ENTRIES, PAGES};
    use crate::settings::schema::{DefaultValue, Kind};

    #[test]
    fn has_the_designs_157_entries_with_unique_keys_on_known_pages() {
        assert_eq!(ENTRIES.len(), 157);
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
}
