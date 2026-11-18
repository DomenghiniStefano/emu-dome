// Tauri bridge mock for headless screenshots. The scenario placeholder is replaced by the driver.
(() => {
  const scenario = __SCENARIO__;
  const FIXED = 1795000000000;
  Date.now = () => FIXED;
  const day = 86400000;
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
  const library = {
    games: scenario.emptyLibrary ? [] : games,
    folders: scenario.emptyLibrary
      ? []
      : [{ path: "C:/Games", reachable: true }],
    scanning: false,
  };
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
        swatches: ["#7152f7", "#3b82f6", "#14b8a6", "#f97316", "#ec4899"],
      },
      "#7152f7",
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
  const schema = {
    device: "desktop",
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
  const values = { "general.bgpause": true, "general.confirmdel": false };
  const rows = (page) =>
    entries
      .filter((e) => e.page === page)
      .map((e) => ({
        key: e.key,
        value: e.key in values ? values[e.key] : e.default,
        source: e.key === "general.bgpause" ? "global" : "default",
        changed: e.key === "general.bgpause",
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
    const all = [...entries, ...more];
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
    };
    // Where each value comes from, per level edited: changed here, inherited from another level.
    const sources = {
      global: {
        "general.bgpause": "global",
        "video.fps": "global",
        "fx.fxint": "global",
      },
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
  const answers = {
    startup_game: null,
    video_mode: ["webview", false],
    is_fullscreen: false,
    current_slot: 1,
    first_run_show: !!scenario.firstRun,
    first_run_steps: ["welcome", "games", "controller", "ready"],
    library_get: library,
    settings_schema: rich ? rich.schema : schema,
    settings_recent: rich ? rich.recent : [],
    settings_game_overrides: [],
    settings_game_scope: null,
    saves_summary: [],
    gamepad_list: [],
    controls_bindings: bindings,
    theme: scenario.theme ?? "dark",
    bench_active: false,
    "plugin:app|version": "0.0.1",
  };
  const callbacks = new Map();
  let next = 1;
  window.__mockCalls = [];
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
      if (cmd === "plugin:event|listen") return next++;
      if (cmd === "plugin:event|unlisten") return null;
      if (cmd === "settings_page")
        return rich ? rich.rowsAt(args.page, args.scope) : rows(args.page);
      if (cmd === "library_scan") return null;
      if (cmd in answers) return JSON.parse(JSON.stringify(answers[cmd]));
      return null;
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
})();
