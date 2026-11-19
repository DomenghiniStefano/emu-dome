// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names or prefixes ending in *...]
// Chrome must already run headless with --remote-debugging-port=<cdpPort>.
import fs from "node:fs";
import path from "node:path";
import { quickMenuScenarios } from "./quick-menu.mjs";

const [cdpPort, baseUrl, outDir, ...only] = process.argv.slice(2);
const here = path.dirname(
  new URL(import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1"),
);
const mockSource = fs.readFileSync(path.join(here, "mock.js"), "utf8");
fs.mkdirSync(outDir, { recursive: true });

// The shell's rail is the first <nav> of the page (no class: it survives the migration).
const rail = (n) =>
  `document.querySelector('nav').querySelectorAll('button')[${n}].click()`;

// Library and shell (stream A of #62): every state in the three themes, at the smallest and the
// largest window. A step is a script, `{ key }` (a trusted key press, so nav.ts shows its focus
// ring) or `{ hover }` (the pointer over the first element matching a selector).
const SIZES = { small: [1024, 640], large: [1920, 1080] };
const THEMES = ["dark", "black", "light"];
const firstCover = "[data-game] button";
const LIBRARY = {
  lib: { mock: {}, steps: [] },
  "lib-empty": { mock: { emptyLibrary: true }, steps: [] },
  "lib-filters": {
    mock: {},
    steps: [`document.querySelector('main [aria-haspopup=dialog]').click()`],
  },
  "lib-cover-menu": {
    mock: {},
    steps: [
      `document.querySelector('[data-game] [aria-haspopup=menu]').click()`,
    ],
  },
  "lib-cover-hover": { mock: {}, steps: [{ hover: firstCover }] },
  "lib-cover-focus": {
    mock: {},
    steps: [{ key: "ArrowDown" }, { key: "ArrowDown" }, { key: "ArrowDown" }],
  },
  "lib-rail-focus": {
    mock: {},
    steps: [{ key: "ArrowLeft" }, { key: "ArrowDown" }],
  },
  "lib-unreachable": { mock: { unreachableFolder: true }, steps: [] },
  // The sweep is paused at a fixed point, so two runs draw the same frame.
  "lib-scanning": {
    mock: { scanning: true },
    steps: [
      `__mockEmit('library://scan', { phase: 'progress', checked: 40, found: 12 })`,
      `document.getAnimations().forEach((a) => { a.pause(); a.currentTime = 700 })`,
    ],
  },
  "lib-added": {
    mock: { withheld: 2 },
    steps: [
      `__mockEmit('library://scan', { phase: 'started' }); __mockLibrary.games = __mockAllGames; __mockEmit('library://scan', { phase: 'done', found: 5, added: 2, unreachable: [] })`,
    ],
  },
  "lib-no-match": {
    mock: {},
    steps: [
      `document.querySelector('main [aria-haspopup=dialog]').click()`,
      `document.querySelectorAll('[role=checkbox]')[0].click(); document.querySelectorAll('[role=checkbox]')[1].click()`,
      `document.querySelector('main [aria-haspopup=dialog]').click()`,
    ],
  },
  "lib-cover-picker": {
    mock: {},
    steps: [
      `document.querySelector('${firstCover}').click()`,
      `document.querySelector('main [aria-haspopup=menu]').click()`,
      `document.querySelector('[role=menuitem]').click()`,
    ],
  },
  placeholder: { mock: {}, steps: [rail(3)] },
};
const LIBRARY_SHOTS = Object.fromEntries(
  Object.entries(LIBRARY).flatMap(([name, s]) =>
    THEMES.flatMap((theme) =>
      Object.entries(SIZES).map(([size, [width, height]]) => [
        `${name}-${theme}-${size}`,
        { ...s, mock: { ...s.mock, theme }, width, height },
      ]),
    ),
  ),
);
const SCENARIOS = {
  ...LIBRARY_SHOTS,
  "library-dark": { mock: {}, steps: [] },
  "library-light": { mock: { theme: "light" }, steps: [] },
  "library-black": { mock: { theme: "black" }, steps: [] },
  "library-empty": { mock: { emptyLibrary: true }, steps: [] },
  saves: { mock: {}, steps: [rail(1)] },
  controls: { mock: {}, steps: [rail(2)] },
  cloud: { mock: {}, steps: [rail(3)] },
  settings: { mock: {}, steps: [rail(4)] },
  "settings-page": {
    mock: {},
    steps: [
      rail(4),
      `(() => { const b = [...document.querySelectorAll('button')].find(b => /Appearance|General|Generale/i.test(b.textContent)); b && b.click(); return !!b })()`,
    ],
  },
  "settings-light": { mock: { theme: "light" }, steps: [rail(4)] },
  "first-run": { mock: { firstRun: true }, steps: [] },
  "kit-dark": { mock: {}, steps: [], hash: "#kit", height: 1500 },
  "kit-black": {
    mock: {},
    steps: [
      `document.querySelectorAll('[data-slot=segmented-item]')[6].click()`,
    ],
    hash: "#kit",
    height: 1500,
  },
  "kit-light": {
    mock: {},
    steps: [
      `document.querySelectorAll('[data-slot=segmented-item]')[7].click()`,
    ],
    hash: "#kit",
    height: 1500,
  },
  "kit-select-open": {
    mock: {},
    steps: [
      `(() => { const t = document.querySelector('[data-slot=select-trigger]'); t.focus(); t.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })) })()`,
    ],
    hash: "#kit",
    height: 1500,
  },
  "kit-light-accent": {
    mock: {},
    steps: [
      `document.querySelectorAll('[data-slot=segmented-item]')[7].click()`,
      `document.querySelectorAll('[data-slot=segmented-item]')[3].click()`,
    ],
    hash: "#kit",
    height: 1500,
  },
  "kit-dialog-open": {
    mock: {},
    steps: [`document.querySelector('[data-slot=dialog-trigger]').click()`],
    hash: "#kit",
    height: 1500,
  },
  ...quickMenuScenarios,
};

// Saves (V01–V06, and the slot tile in Game details): every state in Scuro / Nero / Chiaro, at
// the smallest (1024×640) and the largest (1920×1080) window. Steps find controls by their text
// or label in English or Italian (headless Chrome follows the system language), so they work on
// the code before and after a refactor. `#conflict` needs ConflictTwoUp mounted by hand for the
// run (no screen shows it yet).
const T = {
  more: ["More actions for the slot", "Altre azioni per lo slot"],
  import: ["Import…", "Importa…"],
  delete: ["Delete", "Elimina"],
  pick: ["Choose a file…", "Scegli un file…"],
  other: ["Choose another game…", "Scegli un altro gioco…"],
};
const press = (texts, nth = 0) =>
  `[...document.querySelectorAll('button')].filter(b => ${JSON.stringify(texts)}.some(t => b.textContent.trim() === t || b.getAttribute('aria-label') === t))[${nth}].click()`;
const focusOn = (selector) =>
  `(() => { document.documentElement.setAttribute('data-nav', ''); document.querySelector(${JSON.stringify(selector)}).focus() })()`;
const slotMore = T.more.map((t) => `[aria-label="${t}"]`).join(", ");
const SAVES_STATES = {
  empty: { mock: { saves: false }, steps: [rail(1)] },
  filled: { steps: [rail(1)] },
  "search-none": {
    steps: [
      rail(1),
      `(() => { const i = document.querySelector('input[type=search]'); i.value = 'zzz'; i.dispatchEvent(new Event('input')) })()`,
    ],
  },
  "focus-slot": { steps: [rail(1), focusOn(slotMore)], focus: true },
  "focus-game": {
    steps: [rail(1), focusOn("aside [aria-current]")],
    focus: true,
  },
  quick: { steps: [rail(1), press(T.more)] },
  "quick-delete": {
    // Slot 1: the Automatico slot (the first) cannot be deleted.
    steps: [rail(1), press(T.more, 1), press(T.delete)],
  },
  import: { steps: [rail(1), press(T.import)] },
  "import-picked": {
    steps: [rail(1), press(T.import), press(T.pick)],
  },
  "import-choose": {
    steps: [
      rail(1),
      press(T.import),
      press(T.pick),
      press(T.other),
    ],
  },
  conflict: { steps: [rail(1)], hash: "#conflict" },
  details: { steps: [`document.querySelector('[data-game] button').click()`] },
  "details-quick": {
    steps: [
      `document.querySelector('[data-game] button').click()`,
      press(T.more),
    ],
  },
};
for (const [state, s] of Object.entries(SAVES_STATES)) {
  for (const theme of ["dark", "black", "light"]) {
    for (const [size, width, height] of [
      ["s", 1024, 640],
      ["l", 1920, 1080],
    ]) {
      SCENARIOS[`saves-${state}-${theme}-${size}`] = {
        mock: { saves: true, ...s.mock, theme },
        steps: s.steps,
        hash: s.hash,
        focus: s.focus,
        width,
        height,
      };
    }
  }
}

// The shell (App.vue, first run, flow dialogs, toasts, overlays, touch): each screen in the three
// themes at the smallest and the largest window, named `shell-<screen>-<theme>-<size>`.
// Steps reach the running app through the dev server's own modules (`import('/src/…')`, the same
// instances the app uses), the mock's `__emit` (a backend event) and the toast stack's exposed
// `push`. `freeze` stops CSS animations at their first frame, so a moving bar shoots the same.
const flowSet = (state) =>
  `import('/src/gameFlow.ts').then(m => Object.assign(m.flow, ${state}))`;
const firstRunNext = `document.querySelector('footer [data-nav-initial]').click()`;
const toast = (text, kind, opts) =>
  `document.querySelector('[aria-live=polite]').__vueParentComponent.exposed.push(${JSON.stringify(text)}, '${kind}', { ms: 600000, ...${opts ?? "{}"} })`;
const FIXED = 1795000000000;
const SHELL = {
  "first-run-welcome": { mock: { firstRun: true }, steps: [] },
  "first-run-games": { mock: { firstRun: true }, steps: [firstRunNext] },
  "first-run-controller": {
    mock: { firstRun: true, pad: true },
    steps: [firstRunNext, firstRunNext],
  },
  "first-run-ready": {
    mock: { firstRun: true },
    steps: [firstRunNext, firstRunNext],
  },
  "flow-crash": {
    mock: {},
    steps: [
      flowSet(
        `{ crashed: { console: 'gba', at: ${FIXED - 120000}, detail: 'x' } }`,
      ),
    ],
  },
  "flow-crash-nosave": {
    mock: {},
    steps: [flowSet(`{ crashed: { console: 'nds', at: null, detail: 'x' } }`)],
  },
  "flow-slow": {
    mock: { game: true },
    steps: [flowSet(`{ slow: { fps: 41.2, target: 59.7 } }`)],
  },
  "flow-start-error": {
    mock: {},
    steps: [
      flowSet(
        `{ startError: { title: 'Pocket Quest Advance', reason: 'The file could not be read.' } }`,
      ),
    ],
  },
  "flow-broken": {
    mock: {},
    steps: [
      flowSet(
        `{ saveBroken: { title: 'Pocket Quest Advance', backupAt: ${FIXED - 86400000} } }`,
      ),
    ],
  },
  "flow-broken-nobackup": {
    mock: {},
    steps: [
      flowSet(
        `{ saveBroken: { title: 'Pocket Quest Advance', backupAt: null } }`,
      ),
    ],
  },
  "flow-resume": {
    mock: { game: true },
    steps: [flowSet(`{ resumeAsk: true }`)],
  },
  "flow-launching": {
    mock: {},
    freeze: true,
    steps: [
      flowSet(
        `{ launching: { title: 'Pocket Quest Advance', colour: '#2f5d8a' } }`,
      ),
    ],
  },
  "flow-closing": {
    mock: {},
    freeze: true,
    steps: [
      flowSet(`{ closing: { title: 'Pocket Quest Advance', autosave: true } }`),
    ],
  },
  // «Riduci le animazioni»: the bars stand still and full.
  "flow-closing-still": {
    mock: {},
    reducedMotion: true,
    steps: [
      flowSet(`{ closing: { title: 'Pocket Quest Advance', autosave: true } }`),
    ],
  },
  "flow-hint": { mock: { game: true }, steps: [flowSet(`{ hint: true }`)] },
  "toasts-shell": {
    mock: {},
    steps: [
      toast("Xbox Wireless Controller connected", "info"),
      toast(
        "The folder could not be read. Check that it still exists.",
        "error",
      ),
    ],
  },
  "toasts-game": {
    mock: { game: true },
    steps: [
      toast("Loaded slot 2", "info", `{ action: { label: 'Undo', run() {} } }`),
      toast("Loaded slot 3", "info", `{ note: 'Undo from the quick menu' }`),
      toast("Slot 4", "info"),
    ],
  },
  "game-hud": {
    mock: { game: true },
    steps: [
      `__emit('emu://hotkey', { kind: 'speed', speed: { kind: 'factor', value: 2 } })`,
      `__emit('emu://hotkey', { kind: 'paused', paused: true })`,
      `__emit('emu://hotkey', { kind: 'fps' })`,
    ],
  },
  drop: {
    mock: {},
    steps: [`__emit('tauri://drag-over', { position: { x: 10, y: 10 } })`],
  },
};
for (const [screen, s] of Object.entries(SHELL))
  for (const theme of THEMES)
    for (const [size, [width, height]] of Object.entries(SIZES))
      SCENARIOS[`shell-${screen}-${theme}-${size}`] = {
        ...s,
        mock: { ...s.mock, theme },
        width,
        height,
      };
// Phone: the touch overlay in game, and the first run's phone layout (portrait only).
const PHONE = {
  touch: { mock: { game: true }, steps: [], touch: true },
  "first-run-phone-welcome": {
    mock: { firstRun: true, device: "phone" },
    steps: [],
    touch: true,
  },
  "first-run-phone-ready": {
    mock: { firstRun: true, device: "phone" },
    steps: [firstRunNext, firstRunNext],
    touch: true,
  },
};
const PHONE_SIZES = { portrait: [390, 844], landscape: [844, 390] };
for (const [screen, s] of Object.entries(PHONE))
  for (const theme of THEMES)
    for (const [size, [width, height]] of Object.entries(PHONE_SIZES)) {
      if (screen.startsWith("first-run") && size === "landscape") continue;
      SCENARIOS[`shell-${screen}-${theme}-${size}`] = {
        ...s,
        mock: { ...s.mock, theme },
        width,
        height,
      };
    }

// Controls (C01–C04, E05), stream E of #62: every view and state, in the three themes, at the
// smallest (1024×640) and a large (1920×1080) window. Steps find elements by role, ARIA label or
// English text, never by a scoped class, so the same steps drive the code before and after.
const pointerPress = (expr) => `__press(${expr})`;
const button = (text) =>
  `[...document.querySelectorAll('button, [role=radio]')].find(b => b.textContent.includes(${JSON.stringify(text)}))`;
const tab = (n) => pointerPress(`document.querySelectorAll('[role=tab]')[${n}]`);
const padState = (held, axes = [0, 0, 0, 0, 0, 0]) =>
  `__mockEmit('emu://pad-state', { pads: [{ id: 1, held: ${JSON.stringify(held)}, axes: ${JSON.stringify(axes)} }] })`;
const remapChip = (n) =>
  pointerPress(
    `document.querySelectorAll('[aria-label*="Press to change it"]')[${n}]`,
  );
const shortcutChange = (n) =>
  pointerPress(
    `document.querySelectorAll('[aria-label^="Change the shortcut"]')[${n}]`,
  );
const keyboard = pointerPress(button("Keyboard"));
const CONTROLS = {
  e05: { pads: false, steps: [] },
  overview: { steps: [] },
  "overview-held": { steps: [padState(["south", "r1"])] },
  "overview-keyboard": { steps: [keyboard] },
  remap: { steps: [pointerPress(button("Remap buttons"))] },
  "remap-capture": { steps: [pointerPress(button("Remap buttons")), remapChip(1)] },
  "remap-conflict": {
    steps: [pointerPress(button("Remap buttons")), remapChip(1), padState(["east"])],
  },
  "remap-held": {
    steps: [pointerPress(button("Remap buttons")), padState(["west", "l2"])],
  },
  "remap-keyboard": { steps: [keyboard, pointerPress(button("Remap buttons"))] },
  shortcuts: { steps: [pointerPress(button("Edit shortcuts"))] },
  "shortcuts-capture": {
    steps: [pointerPress(button("Edit shortcuts")), shortcutChange(1)],
  },
  "shortcuts-conflict": {
    steps: [
      pointerPress(button("Edit shortcuts")),
      shortcutChange(1),
      padState(["l1"]),
    ],
  },
  "shortcuts-keyboard": { steps: [keyboard, pointerPress(button("Edit shortcuts"))] },
  sticks: {
    steps: [
      pointerPress(button("Sticks, triggers")),
      padState([], [16000, -8000, 0, 0, 0, 0]),
    ],
  },
  "sticks-live": {
    steps: [
      pointerPress(button("Sticks, triggers")),
      padState([], [-32767, 32767, 0, 0, 0, 0]),
      `document.querySelectorAll('main').forEach(m => (m.scrollTop = 99999))`,
    ],
  },
  "sticks-tab": { steps: [pointerPress(button("Remap buttons")), tab(2)] },
};
for (const [view, c] of Object.entries(CONTROLS)) {
  for (const theme of ["dark", "black", "light"]) {
    for (const [size, [width, height]] of Object.entries(SIZES)) {
      SCENARIOS[`controls-${view}-${theme}-${size}`] = {
        mock: { theme, pads: c.pads ?? true, lang: "en" },
        steps: [rail(2), ...c.steps],
        width,
        height,
      };
    }
  }
}

// Game details (L04–L10), opened from a Library cover (its label in English or Italian, the
// browser's language): every tab, empty and filled, the «…» menu, the focus ring, in each theme at
// the smallest and the largest window; plus the phone in portrait (device «phone», 390 × 844).
// Named `details-<state>-<theme>-<size>` and `details-phone-<tab>-<theme>`.
const openDetails = (title) =>
  `[...document.querySelectorAll('[aria-label]')].find(b => ['Open the details of ${title}', 'Apri i dettagli di ${title}'].includes(b.getAttribute('aria-label'))).click()`;
// A tab by position: mousedown for Reka's trigger, click for a plain button.
const detailsTab = (n) =>
  `(() => { const t = document.querySelectorAll('[role=tab]')[${n}]; t.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, button: 0 })); t.click() })()`;
// The tab row at the top of the window, so a tab's content is in the shot.
const toTabs = `document.querySelector('[role=tablist]').scrollIntoView({ block: 'start' })`;
const showFocus = `document.documentElement.setAttribute('data-nav', '')`;
const filled = { gameSaves: true, gameOverrides: true };
const tabState = (title, tab, mock = {}) => ({
  mock,
  steps: [
    openDetails(title),
    ...(tab === null ? [] : [detailsTab(tab)]),
    toTabs,
  ],
});
const DETAILS_STATES = {
  saves: { mock: filled, steps: [openDetails("Pocket Quest Advance")] },
  "saves-tab": tabState("Pocket Quest Advance", null, filled),
  "saves-empty": tabState("Tiny Blocks", null),
  settings: tabState("Pocket Quest Advance", 1, filled),
  "settings-empty": tabState("Tiny Blocks", 1),
  controls: tabState("Dual Screen Tales", 2, filled),
  "controls-gba": tabState("Pocket Quest Advance", 2),
  cheats: tabState("Star Pilot", 3),
  patches: tabState("Star Pilot", 4),
  files: tabState("Pocket Quest Advance", 5),
  menu: {
    mock: {},
    steps: [
      openDetails("Pocket Quest Advance"),
      `document.querySelector('[aria-haspopup=menu]').click()`,
    ],
  },
  // The focus ring on «Riprendi» (focused on opening), then on a tab (of a short tab: a long one
  // squeezes the tab row to nothing at the small window, a bug of its own).
  focus: {
    mock: filled,
    steps: [openDetails("Pocket Quest Advance"), showFocus],
    focus: true,
  },
  "focus-tab": {
    mock: filled,
    steps: [
      openDetails("Pocket Quest Advance"),
      detailsTab(1),
      showFocus,
      `document.querySelectorAll('[role=tab]')[2].focus()`,
      toTabs,
    ],
    focus: true,
  },
};
const DETAILS_THEMES = { dark: undefined, black: "black", light: "light" };
for (const [state, s] of Object.entries(DETAILS_STATES)) {
  for (const [themeName, theme] of Object.entries(DETAILS_THEMES)) {
    for (const [sizeName, [width, height]] of Object.entries(SIZES)) {
      SCENARIOS[`details-${state}-${themeName}-${sizeName}`] = {
        ...s,
        mock: { ...s.mock, theme },
        width,
        height,
      };
    }
  }
}
for (const [state, s] of Object.entries({
  saves: { mock: filled, steps: [openDetails("Pocket Quest Advance")] },
  "saves-tab": tabState("Pocket Quest Advance", null, filled),
  settings: tabState("Pocket Quest Advance", 1, filled),
  controls: tabState("Dual Screen Tales", 2, filled),
  cheats: tabState("Star Pilot", 3),
  files: tabState("Pocket Quest Advance", 5),
  menu: {
    mock: {},
    steps: [
      openDetails("Pocket Quest Advance"),
      `document.querySelector('[aria-haspopup=menu]').click()`,
    ],
  },
})) {
  for (const [themeName, theme] of Object.entries(DETAILS_THEMES)) {
    SCENARIOS[`details-phone-${state}-${themeName}`] = {
      ...s,
      mock: { ...s.mock, theme, device: "phone" },
      width: 390,
      height: 844,
    };
  }
}

// Settings (T01-T19, SettingRow, SettingSelect), on the rich mock schema (every kind of row), in
// the three themes at the smallest and the largest window. Names are the Italian ones: headless
// Chrome on the owner's machine reports an Italian locale. The helpers work on the old and the
// new markup alike: they find things by text, aria and role, never by a screen's class names.
const SETTINGS_HELPERS = `(() => {
  const byText = (sel, text) =>
    [...document.querySelectorAll(sel)].find((b) => b.textContent.trim().startsWith(text));
  window.__clickText = (sel, text) => { const b = byText(sel, text); b && b.click(); return !!b };
  // A select field by its accessible name: Reka's trigger opens on pointerdown, the old one on click.
  window.__openSelect = (label) => {
    const f = document.querySelector('[aria-label="' + label + '"][aria-expanded]');
    if (!f) return false;
    if (f.dataset.slot === 'select-trigger')
      f.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' }));
    else f.click();
    return true;
  };
  window.__search = (q) => {
    const i = document.querySelector('input[type=search]');
    i.value = q;
    i.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  };
  // The row's own button with no text and no role (a multi row's «edit the list»).
  window.__rowBareButton = (name) => {
    let el = [...document.querySelectorAll('main span, main div')].find((e) => e.textContent.trim() === name);
    while (el) {
      const b = [...el.querySelectorAll('button')].find(
        (x) => !x.textContent.trim() && !x.getAttribute('role') && !x.getAttribute('aria-label'),
      );
      if (b) { b.click(); return true }
      el = el.parentElement;
    }
    return false;
  };
  return true;
})()`;
const page_ = (name) => `__clickText('nav button', ${JSON.stringify(name)})`;
const consoleLevel = `(() => { const r = document.querySelectorAll('[role=region] [role=radio]')[1]; r && r.click(); return !!r })()`;
// «Solo questo gioco»: back to the library, a game's details, its settings tab, «Apri tutte…».
const gameLevel = [
  rail(0),
  `(() => { const b = document.querySelector('[aria-label^="Apri i dettagli di"]'); b && b.click(); return !!b })()`,
  // Reka tabs switch on mousedown, so a plain click() leaves the tab where it was.
  `(() => { const t = [...document.querySelectorAll('[role=tab]')].find(e => e.textContent.trim().startsWith("Impostazioni")); t?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, button: 0 })); t?.click(); return !!t })()`,
  `__clickText('button', "Apri tutte le impostazioni")`,
];
const SETTINGS_SCENES = {
  overview: [],
  "game-overview": gameLevel,
  "video-game": [...gameLevel, page_("Video")],
  "general-game": [...gameLevel, page_("Generale")],
  general: [page_("Generale")],
  video: [page_("Video")],
  "video-console": [page_("Video"), consoleLevel],
  "video-select-open": [page_("Video"), `__openSelect("Proporzioni")`],
  "scope-select-open": [
    page_("Video"),
    consoleLevel,
    `__openSelect("Scegli la console")`,
  ],
  "only-changed": [
    page_("Video"),
    `document.querySelector('[role=region] [role=switch]').click()`,
  ],
  fx: [page_("Effetti schermo")],
  "fx-console": [page_("Effetti schermo"), consoleLevel],
  a11y: [page_("Accessibilità")],
  "a11y-chips-edit": [
    page_("Accessibilità"),
    `__rowBareButton("Filtro per il daltonismo sul gioco")`,
  ],
  library: [page_("Libreria")],
  presets: [page_("Preimpostazioni")],
  pad: [page_("Controller e tastiera")],
  bios: [page_("BIOS e firmware")],
  online: [page_("Online e integrazioni")],
  "search-hits": [`__search("sync")`],
  "search-none": [`__search("zzzz")`],
};
// The same at 110% interface scale (CSS zoom): the select lists are placed through Floating UI.
for (const scene of ["video-select-open", "scope-select-open", "a11y"])
  SCENARIOS[`set-${scene}-zoom110`] = {
    mock: { richSettings: true, textsize: 110 },
    steps: [SETTINGS_HELPERS, rail(4), ...SETTINGS_SCENES[scene]],
    width: 1280,
    height: 800,
    stepMs: 600,
  };
const SET_SIZES = { s: [1024, 640], l: [1920, 1080] };
for (const theme of ["dark", "black", "light"])
  for (const [size, [width, height]] of Object.entries(SET_SIZES))
    for (const [scene, steps] of Object.entries(SETTINGS_SCENES))
      SCENARIOS[`set-${scene}-${theme}-${size}`] = {
        mock: { theme, richSettings: true },
        steps: [SETTINGS_HELPERS, rail(4), ...steps],
        width,
        height,
        stepMs: 600,
      };
// The other screens that render a setting row or a select field: the Controls tabs (C03, C04),
// the quick menu's speeds (G16) and the first run's language.
// A controller connected, then the overview's way into that tab.
const padTab = (button) => [rail(2), `__clickText('button', ${JSON.stringify(button)})`];
const esc = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
const ROW_HOSTS = {
  "ctl-shortcuts": { pad: true, steps: padTab("Modifica le scorciatoie") },
  "ctl-sticks": { pad: true, steps: padTab("Stick, grilletti") },
  "qm-home": { inGame: true, steps: [esc] },
  "qm-speed": {
    inGame: true,
    steps: [esc, `__clickText('button', "Imposta le velocità")`],
  },
  "first-run": { firstRun: true, steps: [] },
  "first-run-lang-open": {
    firstRun: true,
    steps: [`__openSelect(${JSON.stringify("Lingua")})`],
  },
};
for (const theme of ["dark", "light"])
  for (const [name, { steps, ...mock }] of Object.entries(ROW_HOSTS))
    SCENARIOS[`set-host-${name}-${theme}`] = {
      mock: { theme, richSettings: true, ...mock },
      steps: [SETTINGS_HELPERS, ...steps],
      stepMs: 600,
    };

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function page() {
  const res = await fetch(`http://127.0.0.1:${cdpPort}/json/new?about:blank`, {
    method: "PUT",
  });
  const target = await res.json();
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, ko) => {
    ws.onopen = ok;
    ws.onerror = ko;
  });
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    }
  };
  const send = (method, params = {}) =>
    new Promise((ok) => {
      const n = ++id;
      pending.set(n, ok);
      ws.send(JSON.stringify({ id: n, method, params }));
    });
  const close = async () => {
    ws.close();
    await fetch(`http://127.0.0.1:${cdpPort}/json/close/${target.id}`);
  };
  return { send, close };
}

/** A trusted key press, or the pointer moved over an element. */
async function act(p, step) {
  if (step.key) {
    for (const type of ["keyDown", "keyUp"]) {
      await p.send("Input.dispatchKeyEvent", {
        type,
        key: step.key,
        code: step.key,
        windowsVirtualKeyCode:
          { ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40 }[
            step.key
          ] ?? 0,
      });
    }
    return;
  }
  const r = await p.send("Runtime.evaluate", {
    expression: `(() => { const r = document.querySelector(${JSON.stringify(step.hover)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2] })()`,
    returnByValue: true,
  });
  const [x, y] = r.result.result.value;
  await p.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
}

for (const [name, s] of Object.entries(SCENARIOS)) {
  // A name ending in `*` takes every scenario that starts with it (`saves-*`).
  const wanted = (o) =>
    o.endsWith("*") ? name.startsWith(o.slice(0, -1)) : name === o;
  if (only.length && !only.some(wanted)) continue;
  const p = await page();
  await p.send("Page.enable");
  await p.send("Emulation.setDeviceMetricsOverride", {
    width: s.width ?? 1280,
    height: s.height ?? 800,
    deviceScaleFactor: 1,
    mobile: !!s.touch,
  });
  // A headless page has no focus of its own, so `:focus` (the focus ring) never matches; the
  // scenarios that show the ring ask for it.
  if (s.focus)
    await p.send("Emulation.setFocusEmulationEnabled", { enabled: true });
  // A touch screen: `(pointer: coarse)` matches, as on a phone.
  if (s.touch)
    await p.send("Emulation.setTouchEmulationEnabled", {
      enabled: true,
      maxTouchPoints: 5,
    });
  if (s.freeze) {
    await p.send("Animation.enable");
    await p.send("Animation.setPlaybackRate", { playbackRate: 0 });
  }
  await p.send("Emulation.setEmulatedMedia", {
    features: [
      {
        name: "prefers-reduced-motion",
        value: s.reducedMotion ? "reduce" : "no-preference",
      },
    ],
  });
  await p.send("Page.addScriptToEvaluateOnNewDocument", {
    source: mockSource.replace("__SCENARIO__", JSON.stringify(s.mock)),
  });
  await p.send("Page.navigate", { url: baseUrl + (s.hash ?? "") });
  await sleep(3500);
  for (const step of s.steps) {
    if (typeof step === "object") {
      await act(p, step);
      await sleep(600);
      continue;
    }
    const r = await p.send("Runtime.evaluate", {
      expression: step,
      returnByValue: true,
      awaitPromise: true,
    });
    // A step that throws, or returns false (a helper that found nothing), is reported.
    if (r.result?.exceptionDetails || r.result?.result?.value === false)
      console.log(
        name,
        "step failed",
        String(step).slice(0, 80),
        r.result?.exceptionDetails?.text ?? "",
      );
    await sleep(s.stepMs ?? 1500);
  }
  await sleep(1500);
  const shot = await p.send("Page.captureScreenshot", { format: "png" });
  fs.writeFileSync(
    path.join(outDir, `${name}.png`),
    Buffer.from(shot.result.data, "base64"),
  );
  const calls = await p.send("Runtime.evaluate", {
    expression: "JSON.stringify([...new Set(window.__mockCalls || [])])",
    returnByValue: true,
  });
  console.log(name, "ok", calls.result?.result?.value);
  await p.close();
}
