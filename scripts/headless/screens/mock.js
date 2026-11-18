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
  // `unreachableFolder`: the last game lives in a folder that is gone (E01).
  if (scenario.unreachableFolder) {
    const last = games[games.length - 1];
    last.folder = "D:/Old games";
    last.path = `D:/Old games/${last.title}.${last.console}`;
    last.available = false;
  }
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
    settings_schema: schema,
    settings_recent: [],
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
  // Events: `window.__mockEmit(name, payload)` calls the handlers the app listens with.
  const listeners = [];
  window.__mockEmit = (event, payload) => {
    for (const l of listeners) {
      if (l.event === event)
        callbacks.get(l.handler)?.({ event, id: 0, payload });
    }
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
        listeners.push({ event: args.event, handler: args.handler });
        return next++;
      }
      if (cmd === "plugin:event|unlisten") return null;
      if (cmd === "settings_page") return rows(args.page);
      if (cmd === "library_scan") return null;
      if (cmd in answers) return JSON.parse(JSON.stringify(answers[cmd]));
      return null;
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
})();
