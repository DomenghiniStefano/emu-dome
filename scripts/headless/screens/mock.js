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
  // In game (quick menu scenarios, quick-menu.mjs): the rows the quick menu reads.
  if (scenario.game) {
    entries.push(
      entry("video.scale", "video", choice("segments", ["fit", "integer", "fill"]), "fit"),
      entry("video.filter", "video", choice("segments", ["none", "light", "soft"]), "light"),
      entry(
        "fx.fxpick",
        "fx",
        choice("cards", ["none", "lcd", "lcd_grid", "crt", "crt_curved", "soft_pixels"]),
        "none",
      ),
      entry("audio.ff", "audio", choice("segments", ["mute", "pitch_up", "same_pitch"]), "mute"),
      entry("speed.ffval", "speed", choice("segments", ["2", "3", "4", "max"]), "4"),
      entry("speed.slow", "speed", choice("segments", ["0.25", "0.5", "0.75"]), "0.5"),
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
  const trial = scenario.trial ? { "video.filter": "soft", "speed.ffval": "2" } : {};
  const schema = {
    device: scenario.phone ? "phone" : "desktop",
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
  const hour = 3600000;
  const thumb = (colour) =>
    `data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='4'><rect width='6' height='4' fill='%23${colour}'/><rect width='3' height='2' fill='%23ffffff'/></svg>`;
  const slotColours = ["2f5d8a", "7a3b8c", "2b7a4b", "a0522d", "1f6f78"];
  const listSlots = (count) =>
    Array.from({ length: count + 1 }, (_, slot) => {
      const full =
        scenario.slots === "full" ? slot < 6 : scenario.slots === "some" && slot === 0;
      return {
        slot,
        savedAt: full ? FIXED - (slot + 1) * 5 * hour : null,
        thumbUrl: full ? thumb(slotColours[slot % slotColours.length]) : null,
      };
    });
  const gameInfo = {
    title: scenario.game === "nds" ? "Dual Screen Tales" : "Pocket Quest Advance",
    console: scenario.game === "nds" ? "NDS" : "GBA",
    consoleId: scenario.game === "nds" ? "nds" : "gba",
    core: scenario.game === "nds" ? "melondsds" : "mgba",
    resumable: false,
    autosaveOnExit: true,
  };
  const answers = {
    startup_game: scenario.game ? `C:/Games/${gameInfo.title}.${gameInfo.consoleId}` : null,
    open_game: gameInfo,
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
  const listeners = new Map();
  let next = 1;
  // A scenario step can send a backend event: `__emit('emu://menu')`.
  window.__emit = (event, payload = null) => {
    for (const id of listeners.get(event) ?? [])
      callbacks.get(id)?.({ event, id: 0, payload });
  };
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
      if (cmd === "plugin:event|listen") {
        listeners.set(args.event, [...(listeners.get(args.event) ?? []), args.handler]);
        return next++;
      }
      if (cmd === "list_slots") return listSlots(args.count);
      if (cmd === "save_state")
        setTimeout(() => window.__emit("emu://event", { type: "stateSaved", slot: args.slot }), 50);
      if (cmd === "plugin:event|unlisten") return null;
      if (cmd === "settings_page") return rows(args.page);
      if (cmd === "library_scan") return null;
      if (cmd in answers) return JSON.parse(JSON.stringify(answers[cmd]));
      return null;
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
})();
