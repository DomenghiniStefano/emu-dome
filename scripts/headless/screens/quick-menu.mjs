// The quick menu's scenarios (Trello #62, stream F): a game opened at startup by the mock, the
// menu opened with Esc (keyboard hints) or `emu://menu` (controller hints), then each panel.
// Every state in Scuro / Nero / Chiaro at the smallest (1024 × 640) and a large (1920 × 1080)
// window; the phone's bottom sheet at 390 × 844 in portrait.

const escape = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', bubbles: true }))`;
const controllerMenu = `window.__emit('emu://menu')`;
/** Clicks the first button whose text matches `re` (a regular expression source: the labels in
 * English and Italian, since headless Chrome takes the system's language). */
const click = (re) =>
  `(() => { const b = [...document.querySelectorAll('button')].find(b => /${re}/.test(b.textContent.trim())); b.click(); return !!b })()`;

const STATES = {
  home: { mock: { slots: "full" }, steps: [escape] },
  "home-empty": { mock: {}, steps: [escape] },
  "home-auto": { mock: { slots: "some" }, steps: [escape, click("^(Automatic|Automatico)")] },
  "home-mute": { mock: { slots: "full" }, steps: [escape, click("^(Mute|Muto)$")] },
  saved: { mock: { slots: "full" }, steps: [escape, click("^(Save to slot|Salva nello slot)")] },
  "nds-controller": {
    mock: { slots: "full", game: "nds" },
    steps: [
      `window.__emit('emu://session',{ type: 'stateLoaded', slot: 2 })`,
      controllerMenu,
    ],
  },
  slots: { mock: { slots: "full" }, steps: [escape, click("^(More slots|Altri slot)")] },
  speed: { mock: {}, steps: [escape, click("^(Set speeds|Imposta le velocità)$")] },
  screens: { mock: {}, steps: [escape, click("^(Screens and picture|Schermi e immagine)$")] },
  controls: { mock: {}, steps: [escape, click("^(Controls|Controlli)$")] },
  shortcuts: {
    mock: {},
    steps: [escape, click("^(Controls|Controlli)$"), click("^(Controller shortcuts|Scorciatoie del controller)")],
  },
  restart: { mock: {}, steps: [escape, click("^(Restart game|Riavvia il gioco)$")] },
  trial: { mock: { trial: true }, steps: [escape] },
  "trial-speed": { mock: { trial: true }, steps: [escape, click("^(Set speeds|Imposta le velocità)$")] },
  "trial-screens": {
    mock: { trial: true },
    steps: [escape, click("^(Screens and picture|Schermi e immagine)$")],
  },
  // DS screens (#28): swap and the lid among the tools (a 3DS core has no lid), G09's layout
  // card and «Personalizza gli schermi», the C09 editor open and folded, G15 with the lid closed.
  "nds-tools": { mock: { slots: "full", game: "nds", swapped: true }, steps: [escape] },
  "nds-lidless": { mock: { slots: "full", game: "nds", lidless: true }, steps: [escape] },
  "nds-screens": {
    mock: { game: "nds" },
    steps: [escape, click("^(Screens and picture|Schermi e immagine)$")],
  },
  "nds-editor": {
    mock: { game: "nds" },
    steps: [
      escape,
      click("^(Screens and picture|Schermi e immagine)$"),
      click("(Customise the screens|Personalizza gli schermi)$"),
    ],
  },
  "nds-editor-folded": {
    mock: { game: "nds" },
    steps: [
      escape,
      click("^(Screens and picture|Schermi e immagine)$"),
      click("(Customise the screens|Personalizza gli schermi)$"),
      click("^(Collapse|Riduci)"),
    ],
  },
  "nds-lid": { mock: { game: "nds", lid: true }, steps: [] },
  // Screen effects (#43): on the Accelerated method with «TV CRT» picked «in prova»; on
  // Compatible (every other scenario) the effect card is locked with its reason.
  effects: {
    mock: { nativeVideo: true, trial: "crt" },
    steps: [escape, click("^(Screens and picture|Schermi e immagine)$")],
  },
};

// The phone (owner, 2026-11-22, proposal «A»): the first level, «Altro» and its pages.
const more = click("^(More|Altro)$");
const PHONE_STATES = {
  home: { mock: { slots: "full" }, steps: [escape] },
  slots: { mock: { slots: "full" }, steps: [escape, more, click("^(Saves|Salvataggi)")] },
  speed: { mock: { trial: true }, steps: [escape, more, click("^(Speed|Velocità)")] },
  screens: { mock: { trial: true }, steps: [escape, more, click("^(Screens and picture|Schermi e immagine)$")] },
  effects: {
    mock: { nativeVideo: true, trial: "crt" },
    steps: [escape, more, click("^(Screens and picture|Schermi e immagine)$")],
  },
  restart: { mock: {}, steps: [escape, more, click("^(Restart game|Riavvia il gioco)$")] },
};
// The same on the Galaxy S23 as its webview reports itself (360 × 780 upright, 780 × 360 on its
// side), a touch screen over a game picture, in Italian; GBA and DS: `qm-s23-*`.
const S23 = { p780: [360, 780], l780: [780, 360] };
const S23_STATES = {
  home: { mock: { slots: "some" }, steps: [escape] },
  "home-empty": { mock: {}, steps: [escape] },
  "home-rewind": { mock: { slots: "some", rewind: true }, steps: [escape] },
  "home-on": {
    mock: { slots: "some", rewind: true },
    steps: [escape, click("^(Speed up|Accelera)$"), click("^(Mute|Silenzia)$")],
  },
  saved: { mock: { slots: "some" }, steps: [escape, click("^(Quick save|Salva rapido)")] },
  loaded: {
    mock: { slots: "some" },
    steps: [`window.__emit('emu://session',{ type: 'stateLoaded', slot: 1 })`, escape],
  },
  more: { mock: { slots: "some" }, steps: [escape, more] },
  saves: { mock: { slots: "some" }, steps: [escape, more, click("^(Saves|Salvataggi)")] },
  "saves-auto": {
    mock: { slots: "some" },
    steps: [escape, more, click("^(Saves|Salvataggi)"), click("(Automatic|Automatico)")],
  },
  speed: { mock: { slots: "some" }, steps: [escape, more, click("^(Speed|Velocità)")] },
  restart: { mock: {}, steps: [escape, more, click("^(Restart game|Riavvia il gioco)$")] },
};

const THEMES = ["dark", "black", "light"];
const SIZES = { small: [1024, 640], large: [1920, 1080] };

export const quickMenuScenarios = {};
for (const theme of THEMES) {
  for (const [state, s] of Object.entries(STATES)) {
    for (const [size, [width, height]] of Object.entries(SIZES)) {
      quickMenuScenarios[`qm-${state}-${theme}-${size}`] = {
        mock: { game: "gba", ...s.mock, theme },
        steps: s.steps,
        width,
        height,
      };
    }
  }
  for (const [state, s] of Object.entries(PHONE_STATES)) {
    quickMenuScenarios[`qm-phone-${state}-${theme}`] = {
      mock: { game: "gba", phone: true, ...s.mock, theme },
      steps: s.steps,
      width: 390,
      height: 844,
    };
  }
}
for (const game of ["gba", "nds"])
  for (const [state, s] of Object.entries(S23_STATES))
    for (const [size, [width, height]] of Object.entries(S23))
      quickMenuScenarios[`qm-s23-${game}-${state}-${size}`] = {
        mock: { game, phone: true, frame: true, lang: "it", ...s.mock, theme: "dark" },
        steps: s.steps,
        touch: true,
        width,
        height,
      };
// G09 on the phone (#140, #167): the cards' values readable at 360 and 412 px, upright and on
// its side, and `video.scale` held by a touch placement (`phoneHeld`).
const G09_SIZES = { ...S23, p915: [412, 915], l915: [915, 412] };
const openG09 = [escape, more, click("^(Screens and picture|Schermi e immagine)$")];
for (const game of ["gba", "nds"])
  for (const [name, extra] of [
    ["", {}],
    ["-held", { phoneHeld: true }],
  ])
    for (const [size, [width, height]] of Object.entries(G09_SIZES))
      quickMenuScenarios[`qm-g09-${game}${name}-${size}`] = {
        mock: { game, phone: true, frame: true, lang: "it", theme: "dark", ...extra },
        steps: openG09,
        touch: true,
        width,
        height,
      };
quickMenuScenarios["qm-s23-gba-home-en-p780"] = {
  mock: { game: "gba", phone: true, frame: true, lang: "en", slots: "some", rewind: true },
  steps: [escape],
  touch: true,
  width: 360,
  height: 780,
};
// «Chiudere senza salvare?» (`unsaved`: the game ran since the last quick save): the phone strip's
// «Chiudi» on the S23 upright and on its side, the desktop footer's «Chiudi il gioco» at
// 1280 × 800, and TV; `-lost` with `saves.autoexit` off. In Italian, dark.
const closeStrip = click("^(Close|Chiudi)$");
const closeFooter = click("(Close game|Chiudi il gioco)$");
for (const [name, extra] of [
  ["", {}],
  ["-lost", { noAutoExit: true }],
]) {
  for (const [size, [width, height]] of Object.entries(S23))
    quickMenuScenarios[`qm-unsaved${name}-${size}`] = {
      mock: { game: "gba", phone: true, frame: true, lang: "it", unsaved: true, ...extra, theme: "dark" },
      steps: [escape, closeStrip],
      touch: true,
      width,
      height,
    };
  quickMenuScenarios[`qm-unsaved${name}-desk`] = {
    mock: { game: "gba", lang: "it", unsaved: true, ...extra, theme: "dark" },
    steps: [escape, closeFooter],
    width: 1280,
    height: 800,
  };
}
quickMenuScenarios["qm-unsaved-tv"] = {
  mock: { game: "gba", lang: "it", unsaved: true, ui: "tv", pads: true, theme: "dark" },
  steps: [controllerMenu, closeFooter],
  width: 1920,
  height: 1080,
};
// The controller's way through it: the focus starts on «Salva e chiudi», B (`emu://nav` back)
// answers «Annulla» and the quick menu is still there. A step answering false prints `step failed`.
quickMenuScenarios["qm-unsaved-nav-desk"] = {
  mock: { game: "gba", lang: "it", unsaved: true, theme: "dark" },
  steps: [
    controllerMenu,
    closeFooter,
    `/Salva e chiudi/.test(document.activeElement?.textContent ?? '')`,
    `window.__emit('emu://nav', { action: 'back' })`,
    `!/Chiudere senza salvare/.test(document.body.textContent) && /Chiudi il gioco/.test(document.body.textContent)`,
  ],
  focus: true,
  width: 1280,
  height: 800,
};
// Nothing unsaved: «Chiudi» closes straight away (G14 «Chiusura»), no question.
quickMenuScenarios["qm-unsaved-none-desk"] = {
  mock: { game: "gba", lang: "it", theme: "dark" },
  steps: [escape, closeFooter],
  width: 1280,
  height: 800,
};
// The desktop quick menu at 1280 × 800, which the phone's changes leave as it was.
for (const state of ["home", "slots", "controls"])
  quickMenuScenarios[`qm-desk-${state}`] = {
    mock: { game: "gba", ...STATES[state].mock, theme: "dark" },
    steps: STATES[state].steps,
    width: 1280,
    height: 800,
  };

// The search everywhere from the quick menu (#32), in Italian as the mockup: Ctrl + K over the
// menu (keyboard) or Y (`emu://nav` `search`, controller); before typing (the games played last,
// the one being played disabled), the game's shortcuts that run here and those that cannot, the
// settings G09 shows «in prova» beside those it does not, a two-screen game's screen shortcuts,
// and a setting chosen (G09 opens). Dark at both sizes, light at the small one, and TV.
const ctrlK = `window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', key: 'k', ctrlKey: true, bubbles: true }))`;
const padSearch = `window.__emit('emu://nav', { action: 'search' })`;
const typeQuery = (q) =>
  `(() => { const i = document.querySelector('[role=dialog] input'); if (!i) return false; i.value = ${JSON.stringify(q)}; i.dispatchEvent(new Event('input', { bubbles: true })) })()`;
// Taken by the field (cancelled), so `dispatchEvent` answers false: the step answers true.
const enterInQuery = `(document.querySelector('[role=dialog] input').dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', bubbles: true, cancelable: true })), true)`;
const SEARCH_STATES = {
  hint: { mock: {}, steps: [escape] },
  "hint-controller": { mock: {}, steps: [controllerMenu] },
  open: { mock: {}, steps: [escape, ctrlK] },
  slot: { mock: {}, steps: [escape, ctrlK, typeQuery("slot")] },
  video: { mock: {}, steps: [escape, ctrlK, typeQuery("video")] },
  "nds-screens": { mock: { game: "nds" }, steps: [controllerMenu, padSearch, typeQuery("schermi")] },
  chosen: { mock: {}, steps: [escape, ctrlK, typeQuery("dimensione"), enterInQuery] },
};
for (const [state, s] of Object.entries(SEARCH_STATES)) {
  for (const [variant, theme, [width, height]] of [
    ["dark-small", "dark", SIZES.small],
    ["dark-large", "dark", SIZES.large],
    ["light-small", "light", SIZES.small],
  ])
    quickMenuScenarios[`qm-search-${state}-${variant}`] = {
      mock: { game: "gba", slots: "full", search: true, lang: "it", ...s.mock, theme },
      steps: s.steps,
      width,
      height,
    };
  quickMenuScenarios[`qm-search-${state}-tv`] = {
    mock: {
      game: "gba",
      slots: "full",
      search: true,
      lang: "it",
      ui: "tv",
      pads: true,
      ...s.mock,
      theme: "dark",
    },
    steps: s.steps,
    width: 1920,
    height: 1080,
  };
}
