// The touch controls' scenarios (Trello #29), on a phone (a touch screen, the schema's device a
// phone), in Italian as the mockup: the overlay in game for GB, GBA, DS and 3DS at the card's four
// sizes (portrait 390 × 844 and 360 × 740, landscape 844 × 390 and 740 × 360), the built-in skins,
// the C06 editor (opened from the quick menu's «Controlli»: as it opens, a tool open, «Altro»
// open, folded into the pill; at 320 px too, the narrowest phone), and E14 (the last controller
// gone: the question, or the toast with «Passa al tocco»). The phone's layouts with a game
// picture (`frame`, the `touch-frame-*` / `touch-qm-*` / `touch-start-failed-*` scenarios) add the
// Galaxy S23's 412 × 892 and 892 × 412: where the game sits beside the controls and the quick
// menu (opened upright, then the phone turned: the `resize` step), and G02 at phone widths.

const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
const click = (re) =>
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /${re}/.test(b.textContent.trim())); b && b.click(); return !!b })()`;
const openEditor = [
  escape,
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
  // Galaxy S23 (#29, the PM's phone).
  p412: [412, 892],
  l892: [892, 412],
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
add("skin-outlines", { mock: { touchRows: { "touch.tskin": "outlines" } } }, ["p390", "l844"]);
add("skin-contrast", { mock: { game: "nds", touchRows: { "touch.tskin": "contrast" } } }, [
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
add("editor-folded", { steps: [...openEditor, click("Riduci$")] }, ["p390", "l740"]);
add("lost-ask", { mock: { pads: true }, steps: [disconnect] }, ["p390", "l844"]);
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
const camera = (side) =>
  `(() => { document.documentElement.style.setProperty('--spacing-safe-${side}', '32px'); window.dispatchEvent(new Event('resize')); return true })()`;
for (const game of ["nds", "3ds"]) {
  add(`apart-${game}`, { mock: { game, frame: true } }, ["p360", "l740", "p390", "l844"]);
  add(`apart-${game}`, { mock: { game, frame: true }, steps: [camera("top")] }, ["p412"]);
  add(`apart-${game}`, { mock: { game, frame: true }, steps: [camera("left")] }, ["l892"]);
}
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
// G02 with a long reason that has no break point for a line (the S23's Android path).
const startFailed = `import('/src/components/game/gameFlow.ts').then(m => { m.flow.startError = { title: 'emudome-test', reason: 'Non riesco a leggere il file del gioco. Controlla che sia ancora lì e che il suo disco sia collegato. Dettagli: /sdcard/Android/data/it.domehub.emudome/files/test/emudome-test.gba: Permission denied (os error 13)' }; return true })`;
add("start-failed", { mock: { game: false }, steps: [startFailed] }, ["p412", "p390", "p360", "p320"]);
