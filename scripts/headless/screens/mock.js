// Tauri bridge mock for headless screenshots. The scenario placeholder is replaced by the driver.
(() => {
  const scenario = __SCENARIO__;
  const FIXED = 1795000000000;
  // `lang: "en"`: the interface in English whatever the machine's locale, for steps that find
  // elements by their text.
  if (scenario.lang) {
    Object.defineProperty(navigator, "language", { get: () => scenario.lang });
    Object.defineProperty(navigator, "languages", {
      get: () => [scenario.lang],
    });
  }
  Date.now = () => FIXED;
  const day = 86400000;
  // A stand-in screenshot (#85): a `w` × `h` picture, a gradient with a grid and a sun, as an SVG
  // data URL.
  const shotSvg = (w, h, from, to) =>
    "data:image/svg+xml," +
    encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">` +
        `<defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs>` +
        `<rect width="${w}" height="${h}" fill="url(#g)"/>` +
        `<circle cx="${w * 0.75}" cy="${h * 0.25}" r="${Math.min(w, h) * 0.12}" fill="#ffe9a8"/>` +
        `<rect x="0" y="${h * 0.7}" width="${w}" height="${h * 0.3}" fill="#3a7a44"/>` +
        `</svg>`,
    );
  const games = [
    ["gba:01", "gba", "Pocket Quest Advance", "#2f5d8a", 3 * 3600, FIXED - day],
    [
      "nds:02",
      "nds",
      "Dual Screen Tales",
      "#7a3b8c",
      12 * 3600,
      FIXED - 3 * day,
    ],
    ["gb:03", "gb", "Tiny Blocks", "#2b7a4b", 600, null],
    ["gbc:04", "gbc", "Colour Runner", "#a0522d", 0, null],
    ["gba:05", "gba", "Star Pilot", "#1f6f78", 7200, FIXED - 10 * day],
  ].map(([id, console_, title, colour, play, last]) => ({
    id,
    console: console_,
    title,
    path: `C:/Games/${title}.${console_}`,
    folder: "C:/Games",
    size: 4194304,
    available: true,
    playTimeSecs: play,
    lastPlayed: last,
    addedAt: FIXED - 20 * day,
    cover: "generated",
    coverUrl: null,
    hasIcon: false,
    colour,
    favourite: id === "gba:01",
    hidden: false,
    lastStateThumbUrl: null,
    renamed: false,
    romTitle: title,
    code: "ABCD",
    version: 0,
    region: "europe",
    core: "mgba",
    settingsGame: id,
  }));
  // `manyGames: N`: N more games after the five above (Trello #91, the windowed grid), never
  // played, titles of one to three lines, every seventh one not reachable (its «Gioca» disabled).
  // Deterministic: two runs build the same list.
  const WORDS = [
    "Pocket",
    "Quest",
    "Star",
    "Tiny",
    "Colour",
    "Runner",
    "Tales",
    "Advance",
  ];
  for (let n = 0; n < (scenario.manyGames ?? 0); n++) {
    const consoles = ["gba", "nds", "gb", "gbc"];
    const console_ = consoles[n % consoles.length];
    const words = 1 + (n % 5);
    const title = [
      ...Array.from(
        { length: words },
        (_, w) => WORDS[(n + w * 3) % WORDS.length],
      ),
      String(n + 1),
    ].join(" ");
    const id = `${console_}:${(1000 + n).toString(16)}`;
    games.push({
      ...games[2],
      id,
      console: console_,
      title,
      romTitle: title,
      settingsGame: id,
      path: `C:/Games/${title}.${console_}`,
      colour: ["#2f5d8a", "#7a3b8c", "#2b7a4b", "#a0522d", "#1f6f78"][n % 5],
      favourite: false,
      available: n % 7 !== 6,
      playTimeSecs: 0,
      lastPlayed: null,
    });
  }
  // `unreachableFolder`: the last game lives in a folder that is gone (E01).
  if (scenario.unreachableFolder) {
    const last = games[games.length - 1];
    last.folder = "D:/Old games";
    last.path = `D:/Old games/${last.title}.${last.console}`;
    last.available = false;
  }
  // The regional versions grouped (#102), as Rust sends them. `versions`: «Dual Screen Tales»
  // (nds:02, American, played 3 days ago: the tile) in three versions, the European one played a
  // month ago, the Japanese one never. `manyFamilies`: the generated games two by two, the first of
  // each pair the tile.
  const families = [];
  if (scenario.versions) {
    Object.assign(games[1], { region: "usa", code: "ADAE" });
    games.push(
      {
        ...games[1],
        id: "nds:21",
        settingsGame: "nds:21",
        title: "Dual Screen Tales",
        region: "europe",
        code: "ADAP",
        playTimeSecs: 2 * 3600,
        lastPlayed: FIXED - 30 * day,
      },
      {
        ...games[1],
        id: "nds:22",
        settingsGame: "nds:22",
        title: "Futari no Monogatari",
        romTitle: "Futari no Monogatari",
        region: "japan",
        code: "ADAJ",
        playTimeSecs: 0,
        lastPlayed: null,
      },
    );
    families.push({
      id: "nds:ADA",
      shown: "nds:02",
      versions: ["nds:02", "nds:21", "nds:22"],
      pinned: false,
    });
  }
  if (scenario.manyFamilies)
    for (let i = 5; i + 1 < games.length; i += 2)
      families.push({
        id: `many:${i}`,
        shown: games[i].id,
        versions: [games[i].id, games[i + 1].id],
        pinned: false,
      });
  // `withheld`: the last N games arrive with a scan; a step puts them back with
  // `__mockLibrary.games = __mockAllGames` between the scan's `started` and `done` events.
  const listed = games.slice(0, games.length - (scenario.withheld ?? 0));
  const library = {
    games: scenario.emptyLibrary ? [] : listed,
    folders: scenario.emptyLibrary
      ? []
      : [
          { path: "C:/Games", reachable: true },
          ...(scenario.unreachableFolder
            ? [{ path: "D:/Old games", reachable: false }]
            : []),
        ],
    scanning: !!scenario.scanning,
    families,
  };
  window.__mockLibrary = library;
  window.__mockAllGames = games;
  const entry = (key, page, kind, def, extra = {}) => ({
    key,
    page,
    kind,
    default: def,
    levels: ["default", "global", "console", "game", "session"],
    perDevice: false,
    platform: "all",
    phase: 0,
    hiddenOptions: [],
    synonyms: [],
    mostUsed: true,
    needsNativeVideo: false,
    // `SettingEntry.unsupportedConsoles` (#78): always present in the generated type.
    unsupportedConsoles: [],
    ...extra,
  });
  const choice = (style, options) => ({
    type: "choice",
    style,
    options,
    moreAtRuntime: false,
  });
  const entries = [
    entry(
      "general.theme",
      "general",
      choice("segments", ["system", "dark", "black", "light"]),
      "system",
    ),
    entry(
      "general.accent",
      "general",
      {
        type: "colour",
        // As catalog.rs (Trello #96): amber, the icon's, first and default.
        swatches: ["#faa40f", "#7152f7", "#2f7ae5", "#1f9d8a", "#c2417a"],
      },
      "#faa40f",
    ),
    entry(
      "general.ui",
      "general",
      choice("cards", ["auto", "desktop", "tv"]),
      "auto",
    ),
    entry(
      "general.lang",
      "general",
      choice("list", ["system", "it", "en"]),
      "system",
    ),
    entry("general.bgpause", "general", { type: "toggle" }, true),
    entry("general.confirmdel", "general", { type: "toggle" }, false),
    entry(
      "general.start",
      "general",
      choice("segments", ["library", "last_game"]),
      "library",
    ),
    entry(
      "a11y.textsize",
      "a11y",
      { type: "slider", min: 80, max: 150, step: 10, unit: "percent" },
      100,
    ),
    entry(
      "a11y.motion",
      "a11y",
      choice("segments", ["system", "on", "off"]),
      "system",
    ),
    entry(
      "a11y.cblind",
      "a11y",
      { type: "multi", options: ["protan", "deutan", "tritan"] },
      ["deutan"],
    ),
  ];
  // Controls (C01–C04): the `pad` page, only in the scenarios with a controller connected.
  if (scenario.pads) {
    const pct = (min, max, step) => ({
      type: "slider",
      min,
      max,
      step,
      unit: "percent",
    });
    entries.push(
      entry(
        "pad.ab",
        "pad",
        choice("segments", ["nintendo", "printed"]),
        "nintendo",
      ),
      entry("pad.mod", "pad", choice("list", ["l3", "r3", "select"]), "l3"),
      entry("pad.dead", "pad", pct(0, 50, 1), 15),
      entry("pad.stickdpad", "pad", pct(20, 90, 5), 50),
      entry("pad.trig", "pad", pct(10, 90, 5), 50),
      entry("pad.rumble", "pad", pct(0, 100, 10), 70, { phase: 2 }),
      entry(
        "pad.confirm",
        "pad",
        choice("segments", ["right", "bottom"]),
        "right",
      ),
      entry(
        "pad.repeat",
        "pad",
        choice("list", ["slow", "normal", "fast"]),
        "normal",
      ),
    );
  }
  // In game (quick menu scenarios, quick-menu.mjs): the rows the quick menu reads.
  if (scenario.game) {
    entries.push(
      entry(
        "video.scale",
        "video",
        choice("segments", ["fit", "integer", "fill"]),
        "fit",
      ),
      entry(
        "video.filter",
        "video",
        choice("segments", ["none", "light", "soft"]),
        "light",
      ),
      // The effects work on the Accelerated method only (X05); «Effetto da file» stays hidden
      // until a file is chosen.
      entry(
        "fx.fxpick",
        "fx",
        choice("cards", [
          "none",
          "lcd",
          "lcd_grid",
          "crt",
          "crt_curved",
          "soft_pixels",
          "file",
        ]),
        "none",
        { needsNativeVideo: true },
      ),
      entry(
        "audio.ff",
        "audio",
        choice("segments", ["mute", "pitch_up", "same_pitch"]),
        "mute",
      ),
      entry(
        "speed.ffval",
        "speed",
        choice("segments", ["2", "3", "4", "max"]),
        "4",
      ),
      entry(
        "speed.slow",
        "speed",
        choice("segments", ["0.25", "0.5", "0.75"]),
        "0.5",
      ),
      entry("saves.slots", "saves", choice("segments", ["3", "5", "10"]), "10"),
      entry("saves.undo", "saves", { type: "toggle" }, true),
      entry(
        "touch.topac",
        "touch",
        { type: "slider", min: 10, max: 100, step: 5, unit: "percent" },
        60,
        { phase: 4 },
      ),
    );
  }
  // Changes «in prova» (Session level) in the quick menu.
  // `trial: "crt"`: the quick pick «TV CRT» applied (fit, no smoothing, the CRT effect).
  const trial =
    scenario.trial === "crt"
      ? { "video.scale": "fit", "video.filter": "none", "fx.fxpick": "crt" }
      : scenario.trial
        ? { "video.filter": "soft", "speed.ffval": "2" }
        : {};
  const schema = {
    // `device` (the shell's phone first run) or `phone` (the quick menu's sheet).
    device: scenario.device ?? (scenario.phone ? "phone" : "desktop"),
    currentPhase: 1,
    pages: ["general", "a11y"],
    groups: [{ id: "app", pages: ["general", "a11y"] }],
    sections: [
      {
        page: "general",
        id: "look",
        keys: ["general.theme", "general.accent", "general.ui", "general.lang"],
      },
      {
        page: "general",
        id: "behaviour",
        keys: ["general.bgpause", "general.confirmdel", "general.start"],
      },
      {
        page: "a11y",
        id: "look",
        keys: ["a11y.textsize", "a11y.motion", "a11y.cblind"],
      },
    ],
    entries,
  };
  // `search` (X01, #32): a Video page with the rows the search scenarios find, their synonyms as
  // the catalogue has them («tearing» finds nothing by name and suggests V-Sync).
  if (scenario.search) {
    entries.push(
      entry("video.vsync", "video", { type: "toggle" }, true, {
        synonyms: ["tearing", "strappo"],
      }),
      entry(
        "video.path",
        "video",
        choice("cards", ["accelerated", "compatible"]),
        "accelerated",
        { synonyms: ["schermo nero", "black screen", "nativo", "native", "webview"] },
      ),
    );
    schema.pages.push("video");
    schema.groups.push({ id: "game", pages: ["video"] });
    schema.sections.push({
      page: "video",
      id: "gpu",
      keys: ["video.vsync", "video.path"],
    });
  }
  // `rtc` (#102): a Saves page with the game clock's rows, `saves.rtc` set to `rtc` ("offset" or
  // "fixed"), so its offset sliders or its date and time show.
  if (scenario.rtc) {
    const GM = ["default", "global", "game", "session"];
    const clock = { levels: GM, mostUsed: false, wired: true };
    const offset = { ...clock, showIf: { key: "saves.rtc", values: ["offset"], not: false } };
    const slider = (min, max, unit) => ({ type: "slider", min, max, step: 1, unit });
    entries.push(
      entry("saves.rtc", "saves", choice("segments", ["real", "offset", "fixed"]), "real", clock),
      entry("saves.rtcdays", "saves", slider(-364, 364, "days"), 0, offset),
      entry("saves.rtchours", "saves", slider(-23, 23, "hours"), 0, offset),
      entry("saves.rtcminutes", "saves", slider(-59, 59, "minutes"), 0, offset),
      entry(
        "saves.rtcdate",
        "saves",
        { type: "dateTime", minYear: 2000, maxYear: 2099 },
        "2004-01-01T00:00",
        { ...clock, showIf: { key: "saves.rtc", values: ["fixed"], not: false } },
      ),
    );
    schema.pages.push("saves");
    schema.groups.push({ id: "game", pages: ["saves"] });
    schema.sections.push({
      page: "saves",
      id: "ingame",
      keys: ["saves.rtc", "saves.rtcdays", "saves.rtchours", "saves.rtcminutes", "saves.rtcdate"],
    });
  }
  // `notifications` (X03, #32): what the bell shows, newest first, two unread.
  const notifications = scenario.notifications
    ? [
        {
          id: 3,
          at: Date.now() - 5 * 60_000,
          read: false,
          notice: { kind: "controllerConnected", name: "8BitDo Ultimate" },
        },
        {
          id: 2,
          at: Date.now() - 3 * 3_600_000,
          read: false,
          notice: { kind: "controllerConnected", name: "" },
        },
        {
          id: 1,
          at: Date.now() - 26 * 3_600_000,
          read: true,
          notice: { kind: "controllerConnected", name: "Xbox Wireless Controller" },
        },
      ]
    : [];
  const values = { "general.bgpause": true, "general.confirmdel": false };
  // TV mode (#30): `ui` fixes `general.ui` ("tv" / "desktop"; Automatica otherwise).
  if (scenario.ui) values["general.ui"] = scenario.ui;
  if (scenario.rtc) {
    values["saves.rtc"] = scenario.rtc;
    values["saves.rtcdays"] = -3;
    values["saves.rtchours"] = 5;
    values["saves.rtcdate"] = "2024-02-29T07:05";
  }
  const rows = (page) =>
    entries
      .filter((e) => e.page === page)
      .map((e) => ({
        key: e.key,
        value:
          e.key in trial
            ? trial[e.key]
            : e.key in values
              ? values[e.key]
              : e.default,
        source:
          e.key in trial
            ? "session"
            : e.key === "general.bgpause"
              ? "global"
              : "default",
        changed: e.key === "general.bgpause" || e.key in trial,
      }));
  // `richSettings`: every kind of setting row on a few pages (the Settings stream's scenarios),
  // with values set per level so changed, inherited and locked rows show. Off by default, so the
  // other scenarios keep the small schema above.
  const rich = scenario.richSettings ? richSettings() : null;
  function richSettings() {
    const G = ["default", "global"];
    const ALL = ["default", "global", "console", "game", "session"];
    const more = [
      entry(
        "video.scale",
        "video",
        choice("segments", ["fit", "integer", "fill"]),
        "fit",
      ),
      entry(
        "video.aspect",
        "video",
        choice("list", ["original", "4_3", "16_9"]),
        "original",
        { mostUsed: false },
      ),
      entry(
        "video.surround",
        "video",
        choice("cards", ["black", "cover", "blur", "builtin", "image"]),
        "black",
        { mostUsed: false },
      ),
      entry("video.surroundImg", "video", { type: "file" }, "", {
        mostUsed: false,
        showIf: { key: "video.surround", values: ["image"], not: false },
      }),
      entry(
        "video.surroundDim",
        "video",
        { type: "slider", min: 0, max: 80, step: 5, unit: "percent" },
        40,
        {
          mostUsed: false,
          showIf: { key: "video.surround", values: ["black"], not: true },
        },
      ),
      entry("video.fps", "video", { type: "toggle" }, false, {
        levels: G,
        mostUsed: false,
      }),
      entry(
        "video.window",
        "video",
        choice("segments", ["windowed", "fullscreen", "borderless"]),
        "windowed",
        { levels: G, perDevice: true, platform: "desktop", mostUsed: false },
      ),
      entry("video.vsync", "video", { type: "toggle" }, true, {
        perDevice: true,
        mostUsed: false,
      }),
      entry(
        "fx.fxpick",
        "fx",
        choice("cards", [
          "none",
          "lcd",
          "lcd_grid",
          "crt",
          "crt_curved",
          "soft_pixels",
        ]),
        "lcd",
      ),
      entry(
        "fx.fxint",
        "fx",
        { type: "slider", min: 0, max: 100, step: 5, unit: "percent" },
        60,
        { mostUsed: false },
      ),
      entry("fx.fxfile", "fx", { type: "action", style: "button" }, null, {
        levels: G,
        mostUsed: false,
      }),
      entry(
        "presets.preset",
        "presets",
        {
          type: "apply",
          options: ["faithful", "performance", "quality", "battery"],
        },
        null,
        { levels: G, mostUsed: false },
      ),
      entry(
        "presets.export",
        "presets",
        { type: "action", style: "button" },
        null,
        { levels: G, mostUsed: false },
      ),
      entry("library.folders", "library", { type: "folders" }, [], {
        levels: G,
        perDevice: true,
        mostUsed: false,
      }),
      entry("pad.remap", "pad", { type: "bindings", table: "remap" }, null, {
        mostUsed: false,
      }),
      entry("online.wfc", "online", { type: "text" }, "", {
        levels: G,
        mostUsed: false,
      }),
      entry("bios.bgba", "bios", { type: "file" }, "", {
        levels: G,
        perDevice: true,
        mostUsed: false,
      }),
    ];
    // `inGame`: the quick menu's speed rows (G16), shown at the Session level.
    const speed = scenario.inGame
      ? [
          entry(
            "speed.ffval",
            "speed",
            choice("segments", ["2", "3", "4", "max"]),
            "4",
          ),
          entry(
            "speed.slow",
            "speed",
            choice("segments", ["0.25", "0.5", "0.75"]),
            "0.5",
            {
              levels: G,
            },
          ),
          entry(
            "audio.ff",
            "speed",
            choice("segments", ["mute", "pitch_up", "same_pitch"]),
            "mute",
          ),
        ]
      : [];
    // The rows of the Controls section's shortcuts and sticks tabs (C03, C04).
    const pad = [
      entry("pad.mod", "pad", choice("list", ["l3", "r3", "select"]), "l3", {
        levels: G,
        mostUsed: false,
      }),
      entry(
        "pad.dead",
        "pad",
        { type: "slider", min: 0, max: 50, step: 1, unit: "percent" },
        15,
        { levels: G, mostUsed: false },
      ),
      entry(
        "pad.stickdpad",
        "pad",
        { type: "slider", min: 20, max: 90, step: 5, unit: "percent" },
        50,
        { mostUsed: false },
      ),
      entry(
        "pad.trig",
        "pad",
        { type: "slider", min: 10, max: 90, step: 5, unit: "percent" },
        50,
        { levels: G, mostUsed: false },
      ),
      entry(
        "pad.rumble",
        "pad",
        { type: "slider", min: 0, max: 100, step: 10, unit: "percent" },
        70,
        { mostUsed: false, phase: 2 },
      ),
      entry(
        "pad.confirm",
        "pad",
        choice("segments", ["right", "bottom"]),
        "right",
        { levels: G, mostUsed: false },
      ),
      entry(
        "pad.repeat",
        "pad",
        choice("list", ["slow", "normal", "fast"]),
        "normal",
        { levels: G, mostUsed: false },
      ),
    ];
    const all = [...entries, ...more, ...speed, ...pad];
    const pages = [
      "video",
      "fx",
      "pad",
      "general",
      "library",
      "a11y",
      "presets",
      "bios",
      "online",
    ];
    const s = {
      device: "desktop",
      currentPhase: 1,
      pages,
      groups: [
        { id: "game", pages: ["video", "fx"] },
        { id: "controls", pages: ["pad"] },
        { id: "app", pages: ["general", "library", "a11y", "presets"] },
        { id: "system", pages: ["bios", "online"] },
      ],
      sections: [
        ...schema.sections,
        {
          page: "video",
          id: "image",
          keys: [
            "video.scale",
            "video.aspect",
            "video.surround",
            "video.surroundImg",
            "video.surroundDim",
          ],
        },
        {
          page: "video",
          id: "window",
          keys: ["video.fps", "video.window", "video.vsync"],
        },
        { page: "fx", id: "effect", keys: ["fx.fxpick", "fx.fxfile"] },
        { page: "fx", id: "adjust", keys: ["fx.fxint"] },
        { page: "pad", id: "buttons", keys: ["pad.remap"] },
        { page: "library", id: "folders", keys: ["library.folders"] },
        { page: "presets", id: "ready", keys: ["presets.preset"] },
        { page: "presets", id: "file", keys: ["presets.export"] },
        { page: "bios", id: "gba", keys: ["bios.bgba"] },
        { page: "online", id: "online", keys: ["online.wfc"] },
      ],
      entries: all,
    };
    const vals = {
      ...values,
      "video.surround": "image",
      "video.scale": "integer",
      "fx.fxint": 75,
      "library.folders": ["C:/Games", "D:/Emulation/Handhelds"],
      // The interface scale (CSS zoom on <html>): 100% unless the scenario asks for another.
      "a11y.textsize": scenario.textsize ?? 100,
      "a11y.cblind": ["deutan", "protan"],
      "speed.ffval": "3",
      "pad.dead": 20,
    };
    // Where each value comes from, per level edited: changed here, inherited from another level.
    const sources = {
      global: {
        "pad.dead": "global",
        "general.bgpause": "global",
        "video.fps": "global",
        "fx.fxint": "global",
      },
      session: { "speed.ffval": "session", "speed.slow": "global" },
      console: {
        "video.scale": "console",
        "video.fps": "global",
        "fx.fxint": "global",
        "general.bgpause": "global",
      },
    };
    const recent = [
      {
        key: "video.scale",
        level: "console",
        console: "gba",
        game: null,
        at: FIXED - 3600000,
      },
      {
        key: "fx.fxint",
        level: "global",
        console: null,
        game: null,
        at: FIXED - day,
      },
      {
        key: "general.theme",
        level: "global",
        console: null,
        game: null,
        at: FIXED - 4 * day,
      },
    ];
    const rowsAt = (page, scope) => {
      const src = sources[scope?.level] ?? {};
      return all
        .filter((e) => e.page === page)
        .map((e) => ({
          key: e.key,
          value: e.key in vals ? vals[e.key] : e.default,
          source: src[e.key] ?? "default",
          changed: !!src[e.key],
        }));
    };
    return { schema: s, recent, rowsAt };
  }
  const remapButtons = [
    "a",
    "b",
    "x",
    "y",
    "l",
    "r",
    "start",
    "select",
    "up",
    "down",
    "left",
    "right",
  ];
  const bindings = {
    remap: remapButtons.map((b) => ({
      button: b,
      source: b === "a" ? "east" : "south",
      default: "south",
      changed: b === "a",
    })),
    combos: [
      {
        hotkey: "menu",
        pos: "start",
        default: "start",
        status: "active",
        toRemember: true,
      },
      {
        hotkey: "save",
        pos: "r1",
        default: "r1",
        status: "active",
        toRemember: true,
      },
      {
        hotkey: "load",
        pos: "l1",
        default: "l1",
        status: "active",
        toRemember: true,
      },
      {
        hotkey: "swap",
        pos: "north",
        default: "north",
        status: "dualScreen",
        toRemember: false,
      },
    ],
    modifier: "l3",
  };
  // Saves (scenario.saves): three games with saves, one with quick saves, backups and a broken
  // in-game save (V01–V05). The thumbnails are inline SVG, so no request leaves the page.
  const thumb = (a, b) =>
    "data:image/svg+xml," +
    encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="16" shape-rendering="crispEdges"><rect width="24" height="16" fill="${a}"/><rect x="4" y="9" width="16" height="4" fill="${b}"/><rect x="10" y="3" width="4" height="4" fill="${b}"/></svg>`,
    );
  const savesOf = {
    "Pocket Quest Advance": {
      summary: {
        inGame: true,
        quickCount: 3,
        bytes: 1310720,
        lastChange: FIXED - 3600000,
        broken: true,
      },
      game: {
        inGame: { size: 131072, modified: FIXED - 3600000 },
        quick: [
          {
            slot: 0,
            size: 393216,
            savedAt: FIXED - 3600000,
            thumbUrl: thumb("#2f5d8a", "#f5a524"),
            outdated: false,
          },
          {
            slot: 1,
            size: 393216,
            savedAt: FIXED - 2 * day,
            thumbUrl: thumb("#1f6f78", "#eceef3"),
            outdated: true,
          },
          {
            slot: 3,
            size: 393216,
            savedAt: null,
            thumbUrl: null,
            outdated: false,
          },
        ],
        backups: [
          { name: "b2", at: FIXED - day, reason: "launch", size: 131072 },
          { name: "b1", at: FIXED - 5 * day, reason: "import", size: 131072 },
        ],
        broken: ["Pocket Quest Advance.sav.broken"],
      },
    },
    "Dual Screen Tales": {
      summary: {
        inGame: true,
        quickCount: 0,
        bytes: 524288,
        lastChange: FIXED - 3 * day,
        broken: false,
      },
      game: {
        inGame: { size: 524288, modified: FIXED - 3 * day },
        quick: [],
        backups: [],
        broken: [],
      },
    },
    "Star Pilot": {
      summary: {
        inGame: false,
        quickCount: 1,
        bytes: 393216,
        lastChange: FIXED - 10 * day,
        broken: false,
      },
      game: {
        inGame: null,
        quick: [
          {
            slot: 2,
            size: 393216,
            savedAt: FIXED - 10 * day,
            thumbUrl: thumb("#2b3a8c", "#3ed68e"),
            outdated: false,
          },
        ],
        backups: [],
        broken: [],
      },
    },
  };
  const titleOf = (path) => path.replace(/^.*\//, "").replace(/\.[^.]+$/, "");
  const savesAnswer = (cmd, args) => {
    if (cmd === "saves_summary")
      return args.paths
        .filter((p) => savesOf[titleOf(p)])
        .map((p) => ({ path: p, ...savesOf[titleOf(p)].summary }));
    if (cmd === "saves_game")
      return (
        savesOf[titleOf(args.path)]?.game ?? {
          inGame: null,
          quick: [],
          backups: [],
          broken: [],
        }
      );
    if (cmd === "saves_import_pick")
      return {
        path: "C:/Saves/Pocket Quest Advance.dsv",
        fileName: "Pocket Quest Advance.dsv",
        format: "desmume",
        size: 131072,
      };
    return undefined;
  };
  const hour = 3600000;
  const slotThumb = (colour) =>
    `data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='4'><rect width='6' height='4' fill='%23${colour}'/><rect width='3' height='2' fill='%23ffffff'/></svg>`;
  const slotColours = ["2f5d8a", "7a3b8c", "2b7a4b", "a0522d", "1f6f78"];
  const listSlots = (count) =>
    Array.from({ length: count + 1 }, (_, slot) => {
      const full =
        scenario.slots === "full"
          ? slot < 6
          : scenario.slots === "some" && slot === 0;
      return {
        slot,
        savedAt: full ? FIXED - (slot + 1) * 5 * hour : null,
        thumbUrl: full
          ? slotThumb(slotColours[slot % slotColours.length])
          : null,
      };
    });
  const gameInfo = {
    title:
      scenario.game === "nds" ? "Dual Screen Tales" : "Pocket Quest Advance",
    console: scenario.game === "nds" ? "NDS" : "GBA",
    consoleId: scenario.game === "nds" ? "nds" : "gba",
    core: scenario.game === "nds" ? "melondsds" : "mgba",
    resumable: false,
    autosaveOnExit: true,
  };
  const answers = {
    // `game` (the quick menu's, named by console) or `inGame` (the settings' row hosts): a game
    // opens at startup.
    startup_game: scenario.game
      ? `C:/Games/${gameInfo.title}.${gameInfo.consoleId}`
      : scenario.inGame
        ? "C:/Games/Pocket Quest Advance.gba"
        : null,
    open_game: scenario.inGame
      ? {
          title: "Pocket Quest Advance",
          console: "Game Boy Advance",
          consoleId: "gba",
          core: "mgba",
          resumable: false,
          autosaveOnExit: false,
        }
      : gameInfo,
    // `nativeVideo`: the Accelerated method in use (the screen effects can change).
    video_mode: scenario.nativeVideo ? ["native", true] : ["webview", false],
    // `fullscreen`: the window is full screen (with `pads`, TV mode turns on by itself, #30).
    is_fullscreen: !!scenario.fullscreen,
    current_slot: 1,
    first_run_show: !!scenario.firstRun,
    // `game: true`: a game opens at startup (the shell's in-game screens). The quick menu's
    // scenarios name the console (`game: "gba"`) and take `gameInfo` above.
    ...(scenario.game === true
      ? {
          startup_game: "C:/Games/Pocket Quest Advance.gba",
          open_game: {
            title: "Pocket Quest Advance",
            console: "Game Boy Advance",
            consoleId: "gba",
            core: "mgba",
            resumable: false,
            autosaveOnExit: true,
          },
          stats: {
            running: false,
            emuFps: 59.7,
            targetFps: 59.7,
            speed: { kind: "factor", value: 1 },
            paused: false,
            frames: 600,
            autoSavedAt: null,
          },
        }
      : {}),
    library_get: library,
    notifications_list: notifications,
    // `screenshots: true` (#85): a GBA and a DS screenshot (both screens stacked), drawn as SVG
    // so no image file is needed; «Immagine tua» picks a tall picture.
    library_screenshots: scenario.screenshots
      ? [
          {
            name: "2026-11-22_10-11-12.png",
            url: shotSvg(240, 160, "#1e5d8c", "#86c0a0"),
            takenAt: FIXED - 3600000,
            width: 240,
            height: 160,
          },
          {
            name: "2026-11-21_09-00-00.png",
            url: shotSvg(256, 384, "#2b1d4a", "#e39a5b"),
            takenAt: FIXED - day,
            width: 256,
            height: 384,
          },
        ]
      : [],
    library_pick_cover_image: {
      token: 1,
      url: shotSvg(900, 1600, "#3b2d5c", "#faa40f"),
      width: 900,
      height: 1600,
    },
    library_set_cover_image: null,
    settings_schema: rich ? rich.schema : schema,
    settings_recent: rich ? rich.recent : [],
    settings_game_overrides: [],
    settings_game_scope: null,
    saves_summary: [],
    // `pads` (Controls) or `pad` (first run S04): one controller connected.
    gamepad_list:
      scenario.pads || scenario.pad
        ? [
            {
              id: 1,
              name: "Xbox Wireless Controller",
              family: "xbox",
              vendor: 1118,
              product: 2835,
              battery: scenario.pads ? 80 : null,
            },
          ]
        : [],
    controls_bindings: bindings,
    theme: scenario.theme ?? "dark",
    bench_active: false,
    // NDS screens (#28): `swapped`, `lid` (closed: G15) and `lidless` (a core without one) set
    // the status; the editor starts from a vertical stack and has no layout saved.
    screens_status: {
      dualScreen: scenario.game === "nds",
      layout: "vertical",
      cycle: ["vertical", "side", "big_small", "top_only"],
      swapped: !!scenario.swapped,
      swapHold: false,
      hasCustom: false,
      lidSupported: !scenario.lidless,
      lidClosed: !!scenario.lid,
    },
    screens_custom: { screens: [] },
    screens_preset_custom: {
      screens: [
        { id: "top", x: 0.3125, y: 0, w: 0.375, h: 0.5 },
        { id: "bottom", x: 0.3125, y: 0.5, w: 0.375, h: 0.5 },
      ],
    },
    "plugin:app|version": "0.0.1",
  };
  // Game details (L05, L06) with something in them: quick saves, an in-game save with backups
  // and a set-aside broken one, settings of the game's own.
  if (scenario.gameSaves) {
    answers.saves_game = {
      inGame: { size: 65536, modified: FIXED - 2 * 3600000 },
      quick: [
        {
          slot: 0,
          size: 1024,
          savedAt: FIXED - 3600000,
          thumbUrl: null,
          outdated: false,
        },
        {
          slot: 1,
          size: 1024,
          savedAt: FIXED - day,
          thumbUrl: null,
          outdated: false,
        },
        {
          slot: 2,
          size: 1024,
          savedAt: FIXED - 5 * day,
          thumbUrl: null,
          outdated: true,
        },
      ],
      backups: [
        { name: "b2", at: FIXED - day, reason: "launch", size: 65536 },
        { name: "b1", at: FIXED - 4 * day, reason: "import", size: 65536 },
      ],
      broken: ["Pocket Quest Advance.sav.broken"],
    };
  }
  if (scenario.gameOverrides) {
    answers.settings_game_overrides = [
      { key: "general.bgpause", page: "general", value: false, base: true },
      { key: "a11y.textsize", page: "a11y", value: 120, base: null },
    ];
  }
  const callbacks = new Map();
  /** event name -> the callback ids listening to it, for `__mockEmit`. */
  const listeners = new Map();
  let next = 1;
  window.__mockCalls = [];
  // The G06 hint shows on the first few games only (a count in localStorage): off, so two runs
  // in the same Chrome profile draw the same thing. A scenario shows it on purpose.
  try {
    localStorage.setItem("emudome.menuHintShown", "99");
  } catch {}
  // A scenario step fires a backend event (`emu://gamepad-state`, …) to the page's listeners.
  // `__mockEmit` (Controls) and `__emit` (the shell, the quick menu: `__emit('emu://menu')`)
  // are the same.
  window.__mockEmit = window.__emit = (event, payload = null) => {
    for (const id of listeners.get(event) ?? [])
      callbacks.get(id)?.({ event, id: 0, payload });
  };
  // A pointer press as a user makes it (Reka's tabs act on mousedown, not on click).
  window.__press = (el) => {
    const opts = {
      bubbles: true,
      cancelable: true,
      button: 0,
      pointerType: "mouse",
    };
    el.dispatchEvent(new PointerEvent("pointerdown", opts));
    el.dispatchEvent(new MouseEvent("mousedown", opts));
    el.focus();
    el.dispatchEvent(new PointerEvent("pointerup", opts));
    el.dispatchEvent(new MouseEvent("mouseup", opts));
    el.dispatchEvent(new MouseEvent("click", opts));
  };
  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" },
    },
    transformCallback(cb, once) {
      const id = next++;
      callbacks.set(id, cb);
      return id;
    },
    unregisterCallback(id) {
      callbacks.delete(id);
    },
    convertFileSrc(path, protocol = "asset") {
      return `http://${protocol}.localhost/${encodeURIComponent(path)}`;
    },
    async invoke(cmd, args) {
      window.__mockCalls.push(cmd);
      if (cmd === "plugin:event|listen") {
        const ids = listeners.get(args.event) ?? [];
        listeners.set(args.event, [...ids, args.handler]);
        return next++;
      }
      if (cmd === "list_slots") return listSlots(args.count);
      if (cmd === "save_state")
        setTimeout(
          () =>
            window.__emit("emu://session", {
              type: "stateSaved",
              slot: args.slot,
            }),
          50,
        );
      if (cmd === "plugin:event|unlisten") return null;
      if (cmd === "settings_page")
        return rich ? rich.rowsAt(args.page, args.scope) : rows(args.page);
      if (cmd === "library_scan") return null;
      // The shell asks both at start (useShellEvents.ts): nothing moving, nothing broken.
      if (cmd === "saves_migration_status") return { running: false, problem: null };
      if (cmd === "diagnostics") return [];
      if (scenario.saves && cmd.startsWith("saves_")) {
        const a = savesAnswer(cmd, args);
        if (a !== undefined) return JSON.parse(JSON.stringify(a));
      }
      // S04 is a step only while a controller is connected, as in Rust.
      if (cmd === "first_run_steps")
        return args.controller
          ? ["welcome", "games", "controller", "ready"]
          : ["welcome", "games", "ready"];
      if (cmd in answers) return JSON.parse(JSON.stringify(answers[cmd]));
      return null;
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
})();
