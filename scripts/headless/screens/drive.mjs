// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names...]
// Chrome must already run headless with --remote-debugging-port=<cdpPort>.
import fs from "node:fs";
import path from "node:path";

const [cdpPort, baseUrl, outDir, ...only] = process.argv.slice(2);
const here = path.dirname(
  new URL(import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1"),
);
const mockSource = fs.readFileSync(path.join(here, "mock.js"), "utf8");
fs.mkdirSync(outDir, { recursive: true });

const rail = (n) => `document.querySelectorAll('.rail-item')[${n}].click()`;
const SCENARIOS = {
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
};

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
  `__clickText('[role=tab]', "Impostazioni")`,
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
const SIZES = { s: [1024, 640], l: [1920, 1080] };
for (const theme of ["dark", "black", "light"])
  for (const [size, [width, height]] of Object.entries(SIZES))
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

for (const [name, s] of Object.entries(SCENARIOS)) {
  // A name ending in `*` selects every scenario starting with it (`set-*`).
  const picked = (o) =>
    o.endsWith("*") ? name.startsWith(o.slice(0, -1)) : o === name;
  if (only.length && !only.some(picked)) continue;
  const p = await page();
  await p.send("Page.enable");
  await p.send("Emulation.setDeviceMetricsOverride", {
    width: s.width ?? 1280,
    height: s.height ?? 800,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await p.send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "no-preference" }],
  });
  await p.send("Page.addScriptToEvaluateOnNewDocument", {
    source: mockSource.replace("__SCENARIO__", JSON.stringify(s.mock)),
  });
  await p.send("Page.navigate", { url: baseUrl + (s.hash ?? "") });
  await sleep(3500);
  for (const step of s.steps) {
    const r = await p.send("Runtime.evaluate", {
      expression: step,
      returnByValue: true,
    });
    if (r.result?.exceptionDetails || r.result?.result?.value === false)
      console.log(name, "step failed", step.slice(0, 80));
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
