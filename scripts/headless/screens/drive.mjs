// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names...]
// A name ending in * picks every scenario starting with it (`details-*`).
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
const THEMES = { dark: undefined, black: "black", light: "light" };
const SIZES = { small: [1024, 640], large: [1920, 1080] };
for (const [state, s] of Object.entries(DETAILS_STATES)) {
  for (const [themeName, theme] of Object.entries(THEMES)) {
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
  files: tabState("Pocket Quest Advance", 5),
})) {
  for (const [themeName, theme] of Object.entries(THEMES)) {
    SCENARIOS[`details-phone-${state}-${themeName}`] = {
      ...s,
      mock: { ...s.mock, theme, device: "phone" },
      width: 390,
      height: 844,
    };
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

for (const [name, s] of Object.entries(SCENARIOS)) {
  const picked = (o) =>
    o.endsWith("*") ? name.startsWith(o.slice(0, -1)) : name === o;
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
  // A focused element matches :focus only in a focused page; headless pages are not, unless
  // emulated. Only where a scenario shows the focus ring (a text field's caret would blink).
  if (s.focus) {
    await p.send("Emulation.setFocusEmulationEnabled", { enabled: true });
  }
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
