// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names or prefixes ending in *...]
// Chrome must already run headless with --remote-debugging-port=<cdpPort>.
import fs from "node:fs";
import path from "node:path";

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
    mobile: false,
  });
  // A headless page has no focus of its own, so `:focus` (the focus ring) never matches; the
  // scenarios that show the ring ask for it.
  if (s.focus)
    await p.send("Emulation.setFocusEmulationEnabled", { enabled: true });
  await p.send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "no-preference" }],
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
    });
    if (r.result?.exceptionDetails) console.log(name, "step failed", step);
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
