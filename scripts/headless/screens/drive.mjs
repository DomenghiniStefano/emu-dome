// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names, or a prefix ending in *...]
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
const THEMES = ["dark", "black", "light"];
const SIZES = { small: [1024, 640], large: [1920, 1080] };
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
  // A name ending in `*` takes every scenario that starts with it (`shell-*`).
  const wanted = (o) =>
    o.endsWith("*") ? name.startsWith(o.slice(0, -1)) : o === name;
  if (only.length && !only.some(wanted)) continue;
  const p = await page();
  await p.send("Page.enable");
  await p.send("Emulation.setDeviceMetricsOverride", {
    width: s.width ?? 1280,
    height: s.height ?? 800,
    deviceScaleFactor: 1,
    mobile: !!s.touch,
  });
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
    const r = await p.send("Runtime.evaluate", {
      expression: step,
      returnByValue: true,
      awaitPromise: true,
    });
    if (r.result?.exceptionDetails)
      console.log(name, "step failed", step, r.result.exceptionDetails.text);
    await sleep(1500);
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
