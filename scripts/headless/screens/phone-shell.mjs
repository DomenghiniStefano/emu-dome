// The phone's shell (Trello #29): the tab bar upright, the rail as a column on the camera's edge
// on its side (both ways round: `orientation` sets `screen.orientation.type`), the quick menu on
// its side, and E13's warning. Device «phone», a touch screen, in Italian. Named
// `phone-<screen>-<size>`: `p390` 390 × 844 and `p360` 360 × 740 upright, `l844` 844 × 390 and
// `l740` 740 × 360 on the side, camera left (`-left`) or right (`-right`).

/** The phone's tab bar or side column: the n-th section. */
const section = (n) =>
  `(() => { const b = document.querySelectorAll('nav[aria-label] button')[${n}]; b && b.click(); return !!b })()`;
const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
/** Android's back button: Rust sends it as B (`emu://nav` back) outside a game. */
const back = `window.__emit('emu://nav', { action: 'back' })`;
const click = (re) =>
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /${re}/.test(b.textContent.trim())); b && b.click(); return !!b })()`;

const SHELL = {
  library: { mock: {}, steps: [] },
  saves: { mock: {}, steps: [section(1)] },
  controls: { mock: {}, steps: [section(2)] },
  cloud: { mock: {}, steps: [section(3)] },
  settings: { mock: {}, steps: [section(4)] },
  // Upright: a page over the list, and a search's results under the field (#29).
  "settings-page": {
    mock: {},
    steps: [section(4), click("^(General|Generale)")],
  },
  "settings-search": {
    mock: {},
    steps: [
      section(4),
      `(() => { const i = document.querySelector('main input[type=search], input[type=search]'); i.value = 'tema'; i.dispatchEvent(new Event('input', { bubbles: true })); return true })()`,
    ],
  },
  storage: { mock: { storage: "low" }, steps: [] },
  // Android's back at the shell's root: Cloud, back to Saves, back to the Library, then the app
  // to the background (the mock's calls list `leave_to_background`). Settings › Generale, back
  // returns to the list first.
  "back-trail": {
    mock: {},
    steps: [section(1), section(3), back, back, back],
  },
  "back-settings": {
    mock: {},
    steps: [section(4), click("^(General|Generale)"), back],
  },
};
const GAME = {
  qm: { mock: { game: "gba", slots: "full" }, steps: [escape] },
  "qm-nds": { mock: { game: "nds", slots: "full" }, steps: [escape] },
  "qm-slots": {
    mock: { game: "gba", slots: "full" },
    steps: [escape, click("^(More slots|Altri slot)")],
  },
  "qm-screens": {
    mock: { game: "nds", slots: "full" },
    steps: [escape, click("^(Screens and picture|Schermi e immagine)$")],
  },
};

const PORTRAIT = { p390: [390, 844], p360: [360, 740] };
const LANDSCAPE = { l844: [844, 390], l740: [740, 360] };
const SIDES = {
  left: { type: "landscapePrimary", angle: 90 },
  right: { type: "landscapeSecondary", angle: 270 },
};

export const phoneShellScenarios = {};
const add = (name, s, [width, height], orientation) =>
  (phoneShellScenarios[name] = {
    ...s,
    mock: { ...s.mock, lang: "it", device: "phone" },
    touch: true,
    width,
    height,
    orientation,
  });
for (const [screen, s] of Object.entries(SHELL)) {
  for (const [size, wh] of Object.entries(PORTRAIT))
    add(`phone-${screen}-${size}`, s, wh, {
      type: "portraitPrimary",
      angle: 0,
    });
  for (const [size, wh] of Object.entries(LANDSCAPE))
    for (const [side, o] of Object.entries(SIDES))
      add(`phone-${screen}-${size}-${side}`, s, wh, o);
}
for (const [screen, s] of Object.entries(GAME))
  for (const [size, wh] of Object.entries(LANDSCAPE))
    add(`phone-${screen}-${size}`, s, wh, SIDES.left);
