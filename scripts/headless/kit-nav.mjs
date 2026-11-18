// The design system's primitives with the app's navigation, in a real browser: the Kit page
// (`#kit`) in headless Chrome, driven by trusted key presses (the keyboard) and by nav.ts's
// `dispatchNav` (what the controller's `emu://nav` steps call). It checks what happy-dom cannot
// run (Reka's roving focus and slider keys) and the rule of nav.ts that hands those keys to Reka
// (docs/specs/2026-11-18-design-system-foundations.md, Decision 6). Not part of scripts/check: it
// needs a browser. Exit 0 = every check passed, 1 = a check failed (each is printed).
//
// Usage, headless and on ports nobody else uses (never take a busy port):
//   cd app; pnpm exec vite --port 5292 --strictPort
//   chrome --headless=new --disable-gpu --remote-debugging-port=9333 --user-data-dir=<scratch dir>
//   node scripts/headless/kit-nav.mjs 9333 http://localhost:5292/
const [cdpPort, baseUrl] = process.argv.slice(2)
if (!cdpPort || !baseUrl) {
  console.error('usage: node scripts/headless/kit-nav.mjs <cdpPort> <baseUrl>')
  process.exit(2)
}

// The Tauri bridge, answered with nothing: the Kit page only needs the app's modules to load.
const BRIDGE = `
window.__TAURI_INTERNALS__ = {
  metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
  transformCallback: () => 1,
  unregisterCallback() {},
  convertFileSrc: (p, protocol = 'asset') => 'http://' + protocol + '.localhost/' + encodeURIComponent(p),
  invoke: async cmd => (cmd === 'plugin:event|listen' ? 1 : cmd === 'theme' ? 'dark' : null),
}
window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} }
`

const res = await fetch(`http://127.0.0.1:${cdpPort}/json/new?about:blank`, { method: 'PUT' })
const target = await res.json()
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise(ok => (ws.onopen = ok))
let id = 0
const pending = new Map()
ws.onmessage = m => {
  const msg = JSON.parse(m.data)
  if (msg.id && pending.has(msg.id)) pending.get(msg.id)(msg)
}
const send = (method, params = {}) =>
  new Promise(ok => {
    const n = ++id
    pending.set(n, ok)
    ws.send(JSON.stringify({ id: n, method, params }))
  })
const sleep = ms => new Promise(r => setTimeout(r, ms))
const js = async expression => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true })
  if (r.result.exceptionDetails) throw new Error(r.result.exceptionDetails.text)
  return r.result.result.value
}
const VK = { ArrowRight: 39, ArrowLeft: 37, ArrowDown: 40, ArrowUp: 38, Enter: 13, Escape: 27 }
const press = async key => {
  const k = { key, code: key, windowsVirtualKeyCode: VK[key] }
  await send('Input.dispatchKeyEvent', { type: 'rawKeyDown', ...k })
  await send('Input.dispatchKeyEvent', { type: 'keyUp', ...k })
  await sleep(250)
}
/** A controller step: nav.ts's own dispatchNav, the module instance the page runs. */
const pad = async action => {
  await js(`import('/src/nav.ts').then(m => m.dispatchNav(${JSON.stringify(action)}))`)
  await sleep(250)
}
const focus = async selector => {
  await js(`document.querySelector(${JSON.stringify(selector)}).focus()`)
  await sleep(150)
}
const active = () => js('document.activeElement?.textContent?.trim() ?? ""')
const attr = (selector, name) =>
  js(`document.querySelector(${JSON.stringify(selector)})?.getAttribute(${JSON.stringify(name)})`)

let failures = 0
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want)
  if (!ok) failures++
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}: ${JSON.stringify(got)}${ok ? '' : ` (want ${JSON.stringify(want)})`}`)
}

await send('Page.enable')
await send('Page.addScriptToEvaluateOnNewDocument', { source: BRIDGE })
await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 1500, deviceScaleFactor: 1, mobile: false })
await send('Page.navigate', { url: `${baseUrl}#kit` })
await sleep(3500)

// Tabs: one key, one tab; the controller the same; Enter / A opens; past the end nav.ts moves on.
const tab = '[data-slot="tabs-trigger"]'
await focus(tab)
await press('ArrowRight')
check('tabs: ArrowRight moves one tab', await active(), 'Remap')
await pad('left')
check('tabs: a controller step moves one tab', await active(), 'Overview')
await pad('right')
await press('Enter')
check('tabs: Enter opens the focused tab', await js(`[...document.querySelectorAll('${tab}')].find(t => t.dataset.state === 'active')?.textContent.trim()`), 'Remap')
await pad('right')
check('tabs: past the last enabled tab, nav.ts moves on', await active(), 'Saves')

// Segmented: the first "Speed" group of the Kit's Segmented section.
const seg = '[data-slot="segmented"][aria-label="Speed"] [data-slot="segmented-item"]'
await focus(seg)
await press('ArrowRight')
check('segmented: ArrowRight moves one segment', await active(), 'Fast forward')
await pad('confirm')
check('segmented: A chooses', await js(`[...document.querySelectorAll('${seg}')].find(i => i.dataset.state === 'on')?.textContent.trim()`), 'Fast forward')

// Switch: one Enter, one toggle; one A, one toggle.
const sw = '[data-slot="switch"][aria-label="Off"]'
await focus(sw)
await press('Enter')
check('switch: Enter toggles once', await attr(sw, 'aria-checked'), 'true')
await pad('confirm')
check('switch: A toggles once', await attr(sw, 'aria-checked'), 'false')

// Slider: left / right step (keyboard and controller), down leaves it.
const thumb = '[role="slider"][aria-label="Volume"]'
await focus(thumb)
await press('ArrowRight')
check('slider: ArrowRight steps once', await attr(thumb, 'aria-valuenow'), '71')
await pad('right')
check('slider: a controller step steps once', await attr(thumb, 'aria-valuenow'), '72')
check('slider: the focus stays', await js('document.activeElement.getAttribute("role")'), 'slider')
await pad('down')
check('slider: down leaves it', (await js('document.activeElement.getAttribute("role")')) !== 'slider', true)

// Select: A opens on the chosen option, a step moves, A picks and the focus goes back.
const field = '[data-slot="select-trigger"][aria-label="Language"]'
await focus(field)
await pad('confirm')
check('select: A opens the list', await js('!!document.querySelector("[role=listbox]")'), true)
check('select: on the chosen option', await active(), 'Like the system')
await pad('down')
check('select: a step moves one option', await active(), 'English')
await pad('confirm')
check('select: A picks', await js(`document.querySelector('${field}').textContent.trim()`), 'English')
check('select: the list closes, the focus goes back', await js(`document.activeElement === document.querySelector('${field}')`), true)
await press('Enter')
await press('ArrowDown')
await press('Escape')
check('select: Escape closes without picking', await js(`document.querySelector('${field}').textContent.trim()`), 'English')
await pad('confirm')
await pad('back')
check('select: B closes', await js('!!document.querySelector("[role=listbox]")'), false)

// Dialog: opens on its data-nav-initial control; B and Escape close it; the focus goes back.
const trigger = '[data-slot="dialog-trigger"]'
await focus(trigger)
await press('Enter')
check('dialog: opens on its data-nav-initial control', await active(), 'Cancel')
await pad('right')
check('dialog: steps stay inside', await active(), 'Restart')
await pad('back')
check('dialog: B closes', await js('!!document.querySelector("[data-slot=dialog-content]")'), false)
check('dialog: the focus goes back to the trigger', await js(`document.activeElement === document.querySelector('${trigger}')`), true)
await press('Enter')
await press('Escape')
check('dialog: Escape closes, the focus goes back', await js(`!document.querySelector('[data-slot=dialog-content]') && document.activeElement === document.querySelector('${trigger}')`), true)

ws.close()
await fetch(`http://127.0.0.1:${cdpPort}/json/close/${target.id}`)
console.log(failures ? `${failures} failed` : 'all passed')
process.exit(failures ? 1 : 0)
