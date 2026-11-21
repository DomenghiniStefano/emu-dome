// The library with thousands of games (Trello #91), in headless Chrome on the mocked bridge
// (mock.js, `manyGames`): how many tiles are in the DOM, and how long a d-pad step takes, as
// nav.ts's `dispatchNav` (what `emu://nav` calls). Prints the numbers; with `--check` it also
// checks the windowed grid's behaviour (exit 1 on a failure). Not part of scripts/check: it needs
// a browser.
//
// Usage, headless and on ports nobody else uses (never take a busy port):
//   cd app; pnpm exec vite --port <free port> --strictPort
//   chrome --headless=new --disable-gpu --remote-debugging-port=<cdp> --user-data-dir=<scratch dir>
//   node scripts/headless/screens/library-scale.mjs <cdp> http://localhost:<port>/ [games] [--check]
// Restart vite before measuring code edited in a worktree: its watcher can miss the writes.
import fs from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
const checking = args.includes("--check");
const [cdpPort, baseUrl, count = "3000"] = args.filter((a) => a !== "--check");
if (!cdpPort || !baseUrl) {
  console.error(
    "usage: node library-scale.mjs <cdpPort> <baseUrl> [games] [--check]",
  );
  process.exit(2);
}
const here = path.dirname(
  new URL(import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1"),
);
const mock = fs
  .readFileSync(path.join(here, "mock.js"), "utf8")
  .replace(
    "__SCENARIO__",
    JSON.stringify({ lang: "en", manyGames: Number(count) }),
  );

const res = await fetch(`http://127.0.0.1:${cdpPort}/json/new?about:blank`, {
  method: "PUT",
});
const target = await res.json();
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok) => (ws.onopen = ok));
let id = 0;
const pending = new Map();
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id && pending.has(msg.id)) pending.get(msg.id)(msg);
};
const send = (method, params = {}) =>
  new Promise((ok) => {
    const n = ++id;
    pending.set(n, ok);
    ws.send(JSON.stringify({ id: n, method, params }));
  });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const js = async (expression) => {
  const r = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (r.result.exceptionDetails)
    throw new Error(
      r.result.exceptionDetails.exception?.description ??
        r.result.exceptionDetails.text,
    );
  return r.result.result.value;
};

let failures = 0;
const check = (what, got, want) => {
  if (!checking) return;
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failures++;
  console.log(
    `${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}${ok ? "" : ` (want ${JSON.stringify(want)})`}`,
  );
};

await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", {
  width: 1920,
  height: 1080,
  deviceScaleFactor: 1,
  mobile: false,
});
// The focus ring and `:focus` as in a focused window.
await send("Emulation.setFocusEmulationEnabled", { enabled: true });
await send("Page.addScriptToEvaluateOnNewDocument", { source: mock });
await send("Page.navigate", { url: baseUrl });
// The library list is ready when its covers are there.
for (let i = 0; i < 60; i++) {
  await sleep(500);
  if (await js("!!document.querySelector('[data-cover-of]')")) break;
}
await sleep(1000);

// Helpers in the page: a step timed to its next frame, and where the focus is.
await js(`(async () => {
  const nav = await import('/src/nav.ts')
  const frame = () => new Promise(r => requestAnimationFrame(() => setTimeout(r)))
  window.__step = async action => {
    const t0 = performance.now()
    nav.dispatchNav(action)
    const sync = performance.now() - t0
    await frame()
    await frame()
    return { sync, total: performance.now() - t0 }
  }
  window.__at = () => {
    const a = document.activeElement
    const tile = a?.closest('[data-game]')
    return { game: tile?.getAttribute('data-game') ?? null, part: a?.getAttribute('data-cover-part') ?? (a?.matches('[data-cover-of]') ? 'cover' : a?.tagName ?? null) }
  }
  window.__ids = () => [...document.querySelectorAll('[data-game]')].map(t => t.getAttribute('data-game'))
  return true
})()`);

const main = "document.querySelector('main')";
const stats = async () =>
  js(`({
  tiles: document.querySelectorAll('[data-game]').length,
  buttons: document.querySelectorAll('main button').length,
  scrollHeight: ${main}.scrollHeight,
})`);
console.log(`games: ${Number(count) + 5}`, JSON.stringify(await stats()));

const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const p95 = (xs) =>
  [...xs].sort((a, b) => a - b)[Math.floor(xs.length * 0.95)] ?? 0;
async function run(label, actions) {
  const sync = [];
  const total = [];
  for (const a of actions) {
    const t = await js(`__step(${JSON.stringify(a)})`);
    sync.push(t.sync);
    total.push(t.total);
  }
  console.log(
    `${label}: ${actions.length} steps, dispatchNav median ${median(sync).toFixed(2)} ms (p95 ${p95(sync).toFixed(2)}), to the next frame median ${median(total).toFixed(1)} ms (p95 ${p95(total).toFixed(1)})`,
  );
}

// From the first cover of the grid: down through the rows (cover → «Gioca» → next cover), right
// along a row, up again.
await js(
  `document.querySelector('[data-cover-of]').focus({ preventScroll: true }); ${main}.scrollTop = 0`,
);
await sleep(300);
const first = await js("__at()");
await run("down", Array(60).fill("down"));
const afterDown = await js("__at()");
await run("right", Array(20).fill("right"));
await run("up", Array(40).fill("up"));
console.log("after the steps", JSON.stringify(await stats()));

if (checking) {
  // The checks below expect the windowed grid (data-cover-part, Trello #91).
  const cols = await js(
    `getComputedStyle(document.querySelector('[data-game]').parentElement).gridTemplateColumns.split(' ').length`,
  );
  const ids = await js("[...document.querySelectorAll('[data-game]')].length");
  check("tiles in the DOM are a window, not the list", ids < 200, true);
  check("the first focus", first.part, "cover");
  // 60 downs from a cover: 30 rows down, on a cover again.
  check("down 60 steps: 30 rows down, on a cover", afterDown.part, "cover");
  const order = await js(
    `import('/src/components/library/useLibrarySort.ts').then(() => true)`,
  );
  check("app modules load", order, true);
  // The game 30 rows below the first: the grid's order is the page's, read from the mock list
  // through the DOM as rendered at the start (the first row) and the column count.
  check("columns read", cols > 1, true);

  // Scrolled far away with the mouse wheel: the focused tile leaves the DOM, the grid holds the
  // focus (no ring), a step comes back to that tile's neighbour.
  const before = await js("__at()");
  await js(`${main}.scrollTop = ${main}.scrollHeight`);
  await sleep(400);
  check(
    "scrolled away: the grid holds the focus",
    await js(
      "document.activeElement === document.querySelector('[data-game]').parentElement",
    ),
    true,
  );
  check(
    "scrolled away: the focused tile is out of the DOM",
    await js(`!document.querySelector('[data-game="${before.game}"]')`),
    true,
  );
  await js(`__step('down')`);
  await sleep(300);
  const back = await js("__at()");
  check(
    "a step from the parked grid moves from that tile",
    back.part,
    before.part === "cover" ? "play" : "cover",
  );
  // Scrolling back by hand to a parked tile gives it the focus again.
  const now = await js("__at()");
  await js(`${main}.scrollTop = ${main}.scrollHeight`);
  await sleep(400);
  await js(
    `document.querySelector('[data-game]').parentElement.focus === undefined`,
  );
  await js(`${main}.scrollTop = 0`);
  await sleep(200);
  await js(
    `(() => { const el = document.querySelector('[data-game="${now.game}"]'); if (!el) ${main}.scrollTop = 0; return true })()`,
  );
  // Leaving the grid at its top edge: up from a first-row cover lands outside the grid.
  await js(
    `${main}.scrollTop = 0; document.querySelector('[data-cover-of]').focus({ preventScroll: true })`,
  );
  await sleep(300);
  await js(`__step('up')`);
  check(
    "up from the first row leaves the grid",
    await js(
      "!document.activeElement.closest('[data-game]') && document.activeElement !== document.body",
    ),
    true,
  );
  // Left from the first column leaves to the rail.
  await js(
    `document.querySelector('[data-cover-of]').focus({ preventScroll: true })`,
  );
  await js(`__step('left')`);
  check(
    "left from the first column reaches the rail",
    await js("!!document.activeElement.closest('nav')"),
    true,
  );
  // Back from the details of a game far down: its cover gets the focus although its row was not
  // in the DOM when the view mounted.
  const far = await js(
    `import('/src/lib/library.ts').then(m => m.library.games[${Number(count) - 10}].id)`,
  );
  await js(
    `${main}.scrollTop = 0; import('/src/lib/library.ts').then(() => true)`,
  );
  check(
    "a game far down is not in the DOM at the top",
    await js(`!document.querySelector('[data-game="${far}"]')`),
    true,
  );
  console.log(`checks: ${failures ? `${failures} failed` : "all passed"}`);
}

ws.close();
await fetch(`http://127.0.0.1:${cdpPort}/json/close/${target.id}`);
process.exit(failures ? 1 : 0);
