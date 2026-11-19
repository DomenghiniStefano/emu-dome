// Pixel compare of <beforeDir>/<name>.png and <afterDir>/<name>.png, decoded by headless Chrome.
// Usage: node compare.mjs <cdpPort> <beforeDir> <afterDir>
// Keep the screenshots outside the repository (a scratch folder).
import fs from "node:fs";
import path from "node:path";
const [cdpPort, before, after] = process.argv.slice(2);
const res = await fetch(`http://127.0.0.1:${cdpPort}/json/new?about:blank`, {
  method: "PUT",
});
const t = await res.json();
const ws = new WebSocket(t.webSocketDebuggerUrl);
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
const fn = `async (a, b) => {
  const load = src => new Promise(ok => { const i = new Image(); i.onload = () => ok(i); i.src = src })
  const [x, y] = await Promise.all([load(a), load(b)])
  if (x.width !== y.width || x.height !== y.height) return { size: [x.width, x.height, y.width, y.height] }
  const px = img => { const c = document.createElement('canvas'); c.width = img.width; c.height = img.height
    const g = c.getContext('2d'); g.drawImage(img, 0, 0); return g.getImageData(0, 0, c.width, c.height).data }
  const p = px(x), q = px(y)
  let diff = 0, box = [1e9, 1e9, -1, -1]
  for (let i = 0; i < p.length; i += 4) {
    if (p[i] !== q[i] || p[i+1] !== q[i+1] || p[i+2] !== q[i+2] || p[i+3] !== q[i+3]) {
      diff++; const k = i / 4, xx = k % x.width, yy = Math.floor(k / x.width)
      box = [Math.min(box[0], xx), Math.min(box[1], yy), Math.max(box[2], xx), Math.max(box[3], yy)]
    }
  }
  return { pixels: x.width * x.height, diff, box: diff ? box : null }
}`;
for (const f of fs.readdirSync(before).filter((f) => f.endsWith(".png"))) {
  const a =
    "data:image/png;base64," +
    fs.readFileSync(path.join(before, f)).toString("base64");
  const b =
    "data:image/png;base64," +
    fs.readFileSync(path.join(after, f)).toString("base64");
  const r = await send("Runtime.evaluate", {
    expression: `(${fn})(${JSON.stringify(a)}, ${JSON.stringify(b)})`,
    awaitPromise: true,
    returnByValue: true,
  });
  console.log(f.padEnd(22), JSON.stringify(r.result.result.value));
}
ws.close();
await fetch(`http://127.0.0.1:${cdpPort}/json/close/${t.id}`);
