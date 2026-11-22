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
// `--families` (#102): the generated games grouped two by two (one tile per pair).
const grouped = args.includes("--families");
const [cdpPort, baseUrl, count = "3000"] = args.filter(
  (a) => a !== "--check" && a !== "--families",
);
if (!cdpPort || !baseUrl) {
  console.error(
    "usage: node library-scale.mjs <cdpPort> <baseUrl> [games] [--check] [--families]",
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
    JSON.stringify({
      lang: "en",
      manyGames: Number(count),
      manyFamilies: grouped,
    }),
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
  const grid = "document.querySelector('[data-game]').parentElement";
  const coverOf = (game) =>
    `document.querySelector('[data-game="${game}"] [data-cover-part=cover]')`;
  /** The focused tile's place: its left edge and its top in the scrolled content. */
  const place = () =>
    js(
      `(() => { const r = document.activeElement.getBoundingClientRect(); return { left: Math.round(r.left), top: Math.round(r.top + ${main}.scrollTop) } })()`,
    );
  const focusFirst = async () => {
    await js(`${main}.scrollTop = 0`);
    await sleep(300);
    await js(
      "document.querySelector('[data-cover-part=cover]').focus({ preventScroll: true })",
    );
    await sleep(100);
  };

  check("the list is windowed", (await stats()).tiles < 200, true);

  // Down by rows: two steps (cover → «Gioca» or «…» → the cover below), same column, lower.
  await focusFirst();
  const start = await place();
  let ok = true;
  let last = start.top;
  for (let i = 0; i < 20; i++) {
    await js("__step('down')");
    await js("__step('down')");
    const at = await js("__at()");
    const p = await place();
    if (at.part !== "cover" || p.left !== start.left || p.top <= last)
      ok = false;
    last = p.top;
  }
  check("down: 20 rows, each a cover in the same column, lower", ok, true);
  check(
    "down: the focused tile is in view",
    await js(
      `(() => { const r = document.activeElement.getBoundingClientRect(), m = ${main}.getBoundingClientRect(); return r.top >= m.top - 1 && r.bottom <= m.bottom + 1 })()`,
    ),
    true,
  );
  await js("__step('right')");
  check("right: the next cover", (await js("__at()")).part, "cover");
  await js("__step('down')");
  await js("__step('right')");
  check(
    "right along the strip: a «Gioca» or a «…»",
    ["play", "more"].includes((await js("__at()")).part),
    true,
  );

  // Leaving the grid at its edges, by nav.ts's spatial rule.
  await focusFirst();
  await js("__step('up')");
  check(
    "up from the first row leaves the grid",
    await js(
      `document.activeElement !== document.body && !${grid}.contains(document.activeElement)`,
    ),
    true,
  );
  await focusFirst();
  await js("__step('left')");
  check(
    "left from the first column reaches the rail",
    await js("!!document.activeElement.closest('nav')"),
    true,
  );

  // Scrolled away with the wheel: the focused tile leaves the DOM, the grid holds the focus
  // (no ring), a step goes on from that tile.
  await focusFirst();
  const parked = await js("__at()");
  await js(`${main}.scrollTop = ${main}.scrollHeight`);
  await sleep(400);
  check(
    "scrolled away: the focused tile left the DOM",
    await js(`!document.querySelector('[data-game="${parked.game}"]')`),
    true,
  );
  check(
    "scrolled away: the grid holds the focus",
    await js(`document.activeElement === ${grid}`),
    true,
  );
  check(
    "scrolled away: no ring on the grid",
    await js(
      `(() => { const s = getComputedStyle(${grid}); return s.outlineStyle === 'none' && s.boxShadow === 'none' })()`,
    ),
    true,
  );
  await js("__step('down')");
  await sleep(300);
  check(
    "a step from the parked grid moves from that tile",
    await js("__at()"),
    { game: parked.game, part: "play" },
  );
  // Scrolling back by hand gives the focus back to the parked tile.
  await focusFirst();
  await js(`${main}.scrollTop = ${main}.scrollHeight`);
  await sleep(400);
  await js(`${main}.scrollTop = 0`);
  await sleep(400);
  check(
    "scrolled back: the tile has the focus again",
    await js("__at()"),
    parked,
  );

  // Back from the details of a game far down: its cover gets the focus, although its row is not
  // in the DOM when the library mounts again.
  await js(`${main}.scrollTop = ${main}.scrollHeight`);
  await sleep(400);
  const far = await js(
    `[...document.querySelectorAll('[data-game]')].at(-1).getAttribute('data-game')`,
  );
  await js(`${coverOf(far)}.click()`);
  await sleep(800);
  check(
    "details open",
    await js("!document.querySelector('[data-game]')"),
    true,
  );
  await js(
    `import('/src/nav.ts').then(m => m.dispatchNav('back')).then(() => true)`,
  );
  await sleep(1000);
  check("back from details: its cover has the focus", await js("__at()"), {
    game: far,
    part: "cover",
  });
  check(
    "back from details: its cover is in view",
    await js(
      `(() => { const r = document.activeElement.getBoundingClientRect(), m = ${main}.getBoundingClientRect(); return r.top >= m.top - 1 && r.bottom <= m.bottom + 1 })()`,
    ),
    true,
  );
  // Scrolling with the wheel up from there, into rows never measured: what is on screen moves
  // by the scroll only (the rows measured above it do not make it jump).
  const jumps = await js(`(async () => {
    const box = ${main}
    const frame = () => new Promise(r => requestAnimationFrame(() => setTimeout(r)))
    let worst = 0
    for (let i = 0; i < 25; i++) {
      const m = box.getBoundingClientRect()
      const tile = [...document.querySelectorAll('[data-game]')].find(t => t.getBoundingClientRect().top > m.top + 200)
      if (!tile) return -1
      const id = tile.getAttribute('data-game')
      const before = tile.getBoundingClientRect().top
      box.scrollTop -= 600
      const want = before + 600
      await frame(); await frame(); await frame()
      const now = document.querySelector('[data-game="' + id + '"]')
      if (now) worst = Math.max(worst, Math.abs(now.getBoundingClientRect().top - want))
    }
    return Math.round(worst)
  })()`);
  check(
    `wheel up into unmeasured rows: nothing jumps (worst ${jumps} px, 1 is rounding)`,
    jumps >= 0 && jumps <= 1,
    true,
  );
  console.log(`checks: ${failures ? `${failures} failed` : "all passed"}`);
}

ws.close();
await fetch(`http://127.0.0.1:${cdpPort}/json/close/${target.id}`);
process.exit(failures ? 1 : 0);
