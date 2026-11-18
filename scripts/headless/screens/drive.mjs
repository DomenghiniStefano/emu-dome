// Headless screenshots over CDP. Usage:
//   node drive.mjs <cdpPort> <baseUrl> <outDir> [scenario names...]
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
  ...quickMenuScenarios,
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
  // A name ending in `*` selects every scenario with that prefix (`qm-*`).
  const picked = only.some((o) =>
    o.endsWith("*") ? name.startsWith(o.slice(0, -1)) : o === name,
  );
  if (only.length && !picked) continue;
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
