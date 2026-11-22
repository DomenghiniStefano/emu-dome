// The touch controls' scenarios (Trello #29), on a phone (a touch screen, the schema's device a
// phone), in Italian as the mockup: the overlay in game for GB, GBA, DS and 3DS at the card's four
// sizes (portrait 390 × 844 and 360 × 740, landscape 844 × 390 and 740 × 360), the built-in skins,
// the C06 editor (opened from the quick menu's «Controlli»: as it opens, a tool open, «Altro»
// open, folded into the pill; at 320 px too, the narrowest phone), and E14 (the last controller
// gone: the question, or the toast with «Passa al tocco»).

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
};
const phone = (mock) => ({ game: "gba", phone: true, lang: "it", ...mock });

export const touchScenarios = {};
const add = (name, s, sizes = Object.keys(SIZES)) => {
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
