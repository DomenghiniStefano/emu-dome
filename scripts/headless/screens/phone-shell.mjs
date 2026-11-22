// The phone's shell (Trello #29): the tab bar upright, the rail as a column on the camera's edge
// on its side (both ways round: `orientation` sets `screen.orientation.type`), the quick menu on
// its side, and E13's warning. Device «phone», a touch screen, in Italian. Named
// `phone-<screen>-<size>`: `p412` 412 × 892 (Galaxy S23), `p390` 390 × 844, `p360` 360 × 740 and
// `p780` 360 × 780 (the S23 with its display zoomed, #105) upright, `l892` 892 × 412, `l844`
// 844 × 390, `l780` 780 × 360 and `l740` 740 × 360 on the side, camera left (`-left`) or right (`-right`).

/** The phone's tab bar or side column: the n-th section. */
const section = (n) =>
  `(() => { const b = document.querySelectorAll('nav[aria-label] button')[${n}]; b && b.click(); return !!b })()`;
const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
/** Android's back button: Rust sends it as B (`emu://nav` back) outside a game. */
const back = `window.__emit('emu://nav', { action: 'back' })`;
const click = (re) =>
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /${re}/.test(b.textContent.trim())); b && b.click(); return !!b })()`;

const SHELL = {
  // The library (#29): the list by default, the compact grid when `library.view` says so (three
  // columns upright, more on the side), with more games than fit and quick saves (`gameSaves`:
  // the game played last carries «Riprendi»); the regional versions' tags.
  library: { mock: {}, steps: [] },
  "library-list": { mock: { manyGames: 12, gameSaves: true }, steps: [] },
  "library-grid": {
    mock: { manyGames: 12, gameSaves: true, libraryView: "grid" },
    steps: [],
  },
  "library-versions": { mock: { versions: true }, steps: [] },
  "library-versions-grid": {
    mock: { versions: true, libraryView: "grid" },
    steps: [],
  },
  // The switch: from the list to the grid with the toolbar's «Griglia».
  "library-switch": {
    mock: { manyGames: 12 },
    steps: [click("^Griglia$")],
  },
  // The order's panel (L03) from the phone's toolbar.
  "library-filters": {
    mock: {},
    steps: [`document.querySelector('main [aria-haspopup=dialog]').click()`],
  },
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
  // The status bar's room (#29): headless Chrome has no safe area, so the step gives the shell a
  // 32 px one (the S23's status bar) and the top bar and the side column must reach under it;
  // the empty library on a touch screen offers no dragging.
  "safe-empty": {
    mock: { emptyLibrary: true },
    steps: [
      `document.documentElement.style.setProperty('--spacing-safe-top', '32px'); true`,
    ],
  },
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

const PORTRAIT = {
  p412: [412, 892],
  p390: [390, 844],
  p360: [360, 740],
  p780: [360, 780],
};
const LANDSCAPE = {
  l892: [892, 412],
  l844: [844, 390],
  l780: [780, 360],
  l740: [740, 360],
};
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

// «Sfondo ai lati del gioco» under the native video (#105): a built-in background with one hole
// per screen where Rust places it (the mock's `screens_placements`), the room between two screens
// apart painted. Headless has no native surface: the holes show the page's black. The core's
// `started` event gives the frame's size.
const started = (w, h) =>
  `window.__emit('emu://session', { type: 'started', width: ${w}, height: ${h} }); true`;
const SURROUND = {
  nds: { w: 256, h: 384 },
  "3ds": { w: 400, h: 480 },
  gba: { w: 240, h: 160 },
};
for (const [game, { w, h }] of Object.entries(SURROUND)) {
  const s = {
    mock: { game, frame: true, nativeVideo: true, surround: "builtin" },
    steps: [started(w, h)],
  };
  for (const size of ["p412", "p780"])
    add(`phone-surround-${game}-${size}`, s, PORTRAIT[size], {
      type: "portraitPrimary",
      angle: 0,
    });
  for (const size of ["l892", "l780"])
    add(`phone-surround-${game}-${size}`, s, LANDSCAPE[size], SIDES.left);
}
