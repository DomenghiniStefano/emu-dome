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
  // Saves (scenario.saves): three games with saves, one with quick saves, backups and a broken
  // in-game save (V01–V05). The thumbnails are inline SVG, so no request leaves the page.
  const thumb = (a, b) =>
    "data:image/svg+xml," +
    encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="16" shape-rendering="crispEdges"><rect width="24" height="16" fill="${a}"/><rect x="4" y="9" width="16" height="4" fill="${b}"/><rect x="10" y="3" width="4" height="4" fill="${b}"/></svg>`,
    );
  const savesOf = {
    "Pocket Quest Advance": {
      summary: { inGame: true, quickCount: 3, bytes: 1310720, lastChange: FIXED - 3600000, broken: true },
      game: {
        inGame: { size: 131072, modified: FIXED - 3600000 },
        quick: [
          { slot: 0, size: 393216, savedAt: FIXED - 3600000, thumbUrl: thumb("#2f5d8a", "#f5a524"), outdated: false },
          { slot: 1, size: 393216, savedAt: FIXED - 2 * day, thumbUrl: thumb("#1f6f78", "#eceef3"), outdated: true },
          { slot: 3, size: 393216, savedAt: null, thumbUrl: null, outdated: false },
        ],
        backups: [
          { name: "b2", at: FIXED - day, reason: "launch", size: 131072 },
          { name: "b1", at: FIXED - 5 * day, reason: "import", size: 131072 },
        ],
        broken: ["Pocket Quest Advance.sav.broken"],
      },
    },
    "Dual Screen Tales": {
      summary: { inGame: true, quickCount: 0, bytes: 524288, lastChange: FIXED - 3 * day, broken: false },
      game: { inGame: { size: 524288, modified: FIXED - 3 * day }, quick: [], backups: [], broken: [] },
    },
    "Star Pilot": {
      summary: { inGame: false, quickCount: 1, bytes: 393216, lastChange: FIXED - 10 * day, broken: false },
      game: {
        inGame: null,
        quick: [{ slot: 2, size: 393216, savedAt: FIXED - 10 * day, thumbUrl: thumb("#2b3a8c", "#3ed68e"), outdated: false }],
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
      return savesOf[titleOf(args.path)]?.game ?? { inGame: null, quick: [], backups: [], broken: [] };
    if (cmd === "saves_import_pick")
      return { path: "C:/Saves/Pocket Quest Advance.dsv", fileName: "Pocket Quest Advance.dsv", format: "desmume", size: 131072 };
    return undefined;
  };
  const hour = 3600000;
  const slotThumb = (colour) =>
    `data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='6' height='4'><rect width='6' height='4' fill='%23${colour}'/><rect width='3' height='2' fill='%23ffffff'/></svg>`;
  const slotColours = ["2f5d8a", "7a3b8c", "2b7a4b", "a0522d", "1f6f78"];
  const listSlots = (count) =>
    Array.from({ length: count + 1 }, (_, slot) => {
      const full =
        scenario.slots === "full" ? slot < 6 : scenario.slots === "some" && slot === 0;
      return {
        slot,
        savedAt: full ? FIXED - (slot + 1) * 5 * hour : null,
        thumbUrl: full ? slotThumb(slotColours[slot % slotColours.length]) : null,
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
    settings_schema: schema,
    settings_recent: [],
    settings_game_overrides: [],
    settings_game_scope: null,
    saves_summary: [],
    // `pads` (Controls) or `pad` (first run S04): one controller connected.
    gamepad_list: scenario.pads || scenario.pad
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
    "plugin:app|version": "0.0.1",
  };
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
  // A scenario step fires a backend event (`emu://pad-state`, …) to the page's listeners.
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
        setTimeout(() => window.__emit("emu://event", { type: "stateSaved", slot: args.slot }), 50);
      if (cmd === "plugin:event|unlisten") return null;
      if (cmd === "settings_page") return rows(args.page);
      if (cmd === "library_scan") return null;
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
