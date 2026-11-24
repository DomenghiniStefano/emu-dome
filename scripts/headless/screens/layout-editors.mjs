// The layout editors opened from Settings without a game, and the phone quick menu's «Modifica la
// disposizione» (Trello #111), in Italian. `layoutRows` gives the schema the two editor rows on
// their pages; `frameConsole` the console whose screens the C06 editor draws (the mock's
// `screens_phone_start`, #134). Named `layout-<screen>-<size>`: the Galaxy S23 as its webview
// reports itself (`p780` 360 × 780, `l780` 780 × 360) and the desktop (`d1280` 1280 × 800).
//
// - `settings-touch` / `settings-screens`: the page with its row at «Tutti i giochi»;
// - `touch-chooser` / `screens-chooser`: the row's button there asks for the console;
// - `touch-<console>`: the console chosen, C06 with the console's screens as placeholder frames on
//   the theme's flat ground;
// - `screens-<console>`: the console chosen, C09 with its placeholder screens;
// - `touch-console-gba`: the row at the console's scope (Game Boy Advance) opens C06 at once;
// - `qm-more` / `qm-layout`: the phone quick menu's «Altro» with the new row, and C06 opened by it.

const click = (re, sel = "button") =>
  `(() => { const b = [...document.querySelectorAll('${sel}')].find(b => /${re}/.test(b.textContent.trim())); b && b.click(); return !!b })()`;
const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
/** The phone's tab bar or side column, or the desktop's rail: the n-th section. */
const section = (n) =>
  `(() => { const b = document.querySelectorAll('nav')[0].querySelectorAll('button')[${n}]; b && b.click(); return !!b })()`;
const settings = section(4);
const page = (name) => click(`^${name}`, "nav button");
const consoleLevel = `(() => { const r = document.querySelectorAll('[role=region] [role=radio]')[1]; r && r.click(); return !!r })()`;
const touchPage = [settings, page("Controlli touch")];
const screensPage = [settings, page("Schermi di DS e 3DS")];
const openTouch = click("^Apri l’editor$");
const openScreens = click("^Personalizza gli schermi$");
const choose = (name) => click(`^${name}$`, "[role=alertdialog] button, [role=dialog] button");

const SCREENS = {
  "settings-touch": { mock: {}, steps: touchPage },
  "settings-screens": { mock: {}, steps: screensPage },
  "touch-chooser": { mock: {}, steps: [...touchPage, openTouch] },
  "screens-chooser": { mock: {}, steps: [...screensPage, openScreens] },
  "touch-gba": {
    mock: { frameConsole: "gba" },
    steps: [...touchPage, openTouch, choose("Game Boy Advance")],
  },
  "touch-nds": {
    mock: { frameConsole: "nds" },
    steps: [...touchPage, openTouch, choose("Nintendo DS")],
  },
  "touch-3ds": {
    mock: { frameConsole: "3ds" },
    steps: [...touchPage, openTouch, choose("Nintendo 3DS")],
  },
  "touch-console-gba": {
    mock: { frameConsole: "gba" },
    steps: [...touchPage, consoleLevel, openTouch],
  },
  "screens-nds": {
    mock: {},
    steps: [...screensPage, openScreens, choose("Nintendo DS")],
  },
  "screens-3ds": {
    mock: {},
    steps: [...screensPage, openScreens, choose("Nintendo 3DS")],
  },
  // #156: «Affiancati» tapped; the panel keeps its side («Salva» does not move).
  "screens-3ds-side": {
    mock: {},
    steps: [...screensPage, openScreens, choose("Nintendo 3DS"), click("^Affiancati$")],
  },
};
const PHONE_GAME = {
  "qm-more": { mock: { game: "gba", slots: "full" }, steps: [escape, click("^Altro$")] },
  "qm-layout": {
    mock: { game: "gba", slots: "full" },
    steps: [escape, click("^Altro$"), click("^Modifica la disposizione")],
  },
};

// The Galaxy A71 too (`p915` 412 × 915, `l915`; #140, #156).
const PHONE = { p780: [360, 780], l780: [780, 360], p915: [412, 915], l915: [915, 412] };
const DESK = { d1280: [1280, 800] };

export const layoutEditorScenarios = {};
const add = (name, s, [width, height], phone) =>
  (layoutEditorScenarios[`layout-${name}`] = {
    ...s,
    mock: {
      ...s.mock,
      layoutRows: true,
      lang: "it",
      ...(phone ? { device: "phone", phone: true } : {}),
    },
    ...(phone ? { touch: true } : {}),
    width,
    height,
    stepMs: 500,
  });
for (const [screen, s] of Object.entries(SCREENS)) {
  for (const [size, wh] of Object.entries(PHONE)) add(`${screen}-${size}`, s, wh, true);
  // The desktop: C09 works with the mouse; the touch row stays the phone's («Solo su questo
  // telefono.»).
  if (screen.includes("screens") || screen === "settings-touch")
    for (const [size, wh] of Object.entries(DESK)) add(`${screen}-${size}`, s, wh, false);
}
for (const [screen, s] of Object.entries(PHONE_GAME))
  for (const [size, wh] of Object.entries(PHONE)) add(`${screen}-${size}`, s, wh, true);
