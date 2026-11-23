// The touch controls' scenarios (Trello #29), on a phone (a touch screen, the schema's device a
// phone), in Italian as the mockup: the overlay in game for GB, GBA, DS and 3DS at the card's four
// sizes (portrait 390 × 844 and 360 × 740, landscape 844 × 390 and 740 × 360), the two styles,
// the C06 editor (opened from the quick menu's «Controlli»: as it opens, a tool open, «Altro»
// open, folded into the pill; at 320 px too, the narrowest phone), and E14 (the last controller
// gone: the question, or the toast with «Passa al tocco»). The phone's layouts with a game
// picture (`frame`, the `touch-frame-*` / `touch-qm-*` / `touch-start-failed-*` scenarios) add
// 412 × 892 and 892 × 412: where the game sits beside the controls and the quick menu (opened
// upright, then the phone turned: the `resize` step), and G02 at phone widths. `touch-look-*`:
// the compact controls on the Galaxy S23 as it reports itself (360 × 780), both styles.

const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
const click = (re) =>
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /${re}/.test(b.textContent.trim())); b && b.click(); return !!b })()`;
// On the phone «Controlli» is in the quick menu's «Altro», not on its first level (#108).
const openEditor = [
  escape,
  click("^(More|Altro)$"),
  click("^(Controls|Controlli)"),
  `(() => { const b = document.querySelector('[data-qm-touch-edit]'); b && b.click(); return !!b })()`,
];
const pad = {
  id: 1,
  name: "Xbox Wireless Controller",
  family: "xbox",
  vendor: 1118,
  product: 2835,
  battery: 80,
};
const disconnect = `window.__emit('emu://gamepad-disconnected', ${JSON.stringify(pad)})`;

const SIZES = {
  p390: [390, 844],
  p360: [360, 740],
  l844: [844, 390],
  l740: [740, 360],
  // A 412 × 892 phone (#29).
  p412: [412, 892],
  l892: [892, 412],
  // The Galaxy S23 as its webview reports itself (#29, the owner's phone, 2026-11-22).
  p780: [360, 780],
  l780: [780, 360],
};
/** The card's four sizes, the defaults of `add`. */
const CARD = ["p390", "p360", "l844", "l740"];
const phone = (mock) => ({ game: "gba", phone: true, lang: "it", ...mock });

export const touchScenarios = {};
const add = (name, s, sizes = CARD) => {
  for (const size of sizes) {
    const [width, height] = SIZES[size] ?? [320, 640];
    touchScenarios[`touch-${name}-${size}`] = {
      mock: phone(s.mock),
      steps: s.steps ?? [],
      ...(s.stepMs ? { stepMs: s.stepMs } : {}),
      touch: true,
      width,
      height,
    };
  }
};

for (const game of ["gb", "gba", "nds", "3ds"]) add(`play-${game}`, { mock: { game } });
add("skin-outline", { mock: { touchRows: { "touch.tskin": "outline" } } }, ["p390", "l844"]);
add("skin-outline-nds", { mock: { game: "nds", touchRows: { "touch.tskin": "outline" } } }, [
  "p390",
  "l844",
]);
add("editor-gba", { steps: openEditor });
add("editor-3ds", { mock: { game: "3ds" }, steps: openEditor });
add("editor-gba", { steps: openEditor }, ["p320"]);
add("editor-size", { steps: [...openEditor, click("^(Dimensione)$")] }, ["l844"]);
add(
  "editor-more",
  { steps: [...openEditor, `document.querySelector('[aria-label="Altro"]').click()`] },
  ["p320", "p360", "p390"],
);
// Android's back with a tool open (#132): the panel closes, the editor stays.
const systemBack = `window.__emit('emu://nav', { action: 'back', system: true }); true`;
const sizeTool = [
  ...openEditor,
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /^Dimensione$/.test(b.textContent.trim())) ?? document.querySelector('[aria-label="Altro"]'); b && b.click(); return !!b })()`,
  `(() => { const m = [...document.querySelectorAll('[role=menuitem]')].find(b => /Dimensione/.test(b.textContent)); m && m.click(); return true })()`,
];
add("editor-size-open", { steps: sizeTool }, ["p780", "l780"]);
add("editor-size-back", { steps: [...sizeTool, systemBack] }, ["p780", "l780"]);
add("editor-folded", { steps: [...openEditor, click("Riduci$")] }, ["p390", "l740"]);
add("lost-ask", { mock: { pads: true }, steps: [disconnect] }, ["p390", "l844"]);
// G06 on the phone (#118): no hint with only the touch controls, Home with a controller, Esc
// once a physical key was pressed (a `{ key }` step).
const showHint = `import('/src/components/game/gameFlow.ts').then(m => { m.flow.hint = true; return true })`;
add("hint-touch", { steps: [showHint], stepMs: 200 }, ["p780"]);
add("hint-pad", { mock: { pads: true }, steps: [showHint], stepMs: 200 }, ["p780"]);
add("hint-keyboard", { steps: [{ key: "ShiftLeft" }, showHint], stepMs: 200 }, ["p780"]);
add(
  "lost-toast",
  // In game a toast stays 2 s: the picture is taken before it goes.
  {
    mock: { pads: true, touchRows: { "touch.tlost": "touch" } },
    steps: [disconnect],
    stepMs: 100,
  },
  ["p390", "l844"],
);

// The game beside the controls, at every size (#29: the S23's empty band upright, the clusters
// over the picture on its side).
const ALL = Object.keys(SIZES);
for (const game of ["gb", "gba", "nds"])
  add(`frame-${game}`, { mock: { game, frame: true } }, ALL);
// DS and 3DS on a phone (owner, 2026-11-22): the two screens large and apart, upright with L /
// menu / R in the strip between them, on its side side by side at the top. The S23 with its
// camera's 32 px (upright at the top, on its side on the left), as the webview reports it.
const camera = (side, px = 32) =>
  `(() => { document.documentElement.style.setProperty('--spacing-safe-${side}', '${px}px'); window.dispatchEvent(new Event('resize')); return true })()`;
for (const game of ["nds", "3ds"]) {
  add(`apart-${game}`, { mock: { game, frame: true } }, ["p360", "l740", "p390", "l844"]);
  add(`apart-${game}`, { mock: { game, frame: true }, steps: [camera("top")] }, ["p412"]);
  add(`apart-${game}`, { mock: { game, frame: true }, steps: [camera("left")] }, ["l892"]);
}
// The compact controls (owner, 2026-11-22) on the S23 as it reports itself, its camera's 38 px:
// the two DS screens edge to edge upright, both styles («Fumé» the default, «Contorno»), and GBA.
for (const [style, rows] of [
  ["smoke", {}],
  ["outline", { "touch.tskin": "outline" }],
]) {
  for (const game of ["nds", "3ds", "gba"]) {
    const mock = { game, frame: true, touchRows: rows };
    add(`look-${style}-${game}`, { mock, steps: [camera("top", 38)] }, ["p780"]);
    add(`look-${style}-${game}`, { mock, steps: [camera("left", 38)] }, ["l780"]);
  }
}
// `video.scale` on the S23 (#105): a GBA as large as fits by default (`touch-frame-gba-*`,
// `touch-look-*-gba-*`), and each choice fixed: «Adatta», «Pixel interi» (floored), «Riempi».
for (const scale of ["fit", "integer", "fill"])
  add(`scale-${scale}-gba`, { mock: { game: "gba", frame: true, videoScale: scale } }, [
    "p780",
    "l780",
  ]);
// The quick menu over the game, upright and on its side; opened upright then turned, and back.
add("qm-frame", { mock: { frame: true, slots: "some" }, steps: [escape] }, ALL);
add(
  "qm-turned",
  { mock: { frame: true, slots: "some" }, steps: [escape, { resize: [892, 412] }] },
  ["p412"],
);
add(
  "qm-turned-back",
  { mock: { frame: true, slots: "some" }, steps: [escape, { resize: [412, 892] }] },
  ["l892"],
);
// G02 for a game file that cannot be read (#128): Rust's code as it comes (the file, its place
// and the cause as values) turned into the dialog by the app's own `startProblem`; the place a
// long Android document URI with no break point, clamped to two lines.
export const startFailedStep = (file, path, cause) =>
  `Promise.all([import('/src/components/game/gameFlow.ts'), import('/src/components/game/launch.ts')]).then(([f, l]) => { f.flow.startError = { title: ${JSON.stringify(file.replace(/\.[^.]+$/, ""))}, ...l.startProblem({ code: 'romUnreadable', params: { file: ${JSON.stringify(file)}, path: ${JSON.stringify(path)}, cause: ${JSON.stringify(cause)} } }) }; return true })`;
const startFailed = startFailedStep(
  "emudome-test.gba",
  "content://com.android.externalstorage.documents/tree/primary%3ADownload%2FROMs%2FGame%20Boy%20Advance/document/primary%3ADownload%2FROMs%2FGame%20Boy%20Advance%2Femudome-test.gba",
  "permissionDenied",
);
add("start-failed", { mock: { game: false }, steps: [startFailed] }, [
  "p412",
  "p390",
  "p780",
  "p360",
  "p320",
  "l780",
]);
