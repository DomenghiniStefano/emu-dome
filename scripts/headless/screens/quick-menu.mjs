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

const PHONE_STATES = {
  home: { mock: { slots: "full" }, steps: [escape] },
  slots: { mock: { slots: "full" }, steps: [escape, click("^(More slots|Altri slot)$")] },
  speed: { mock: { trial: true }, steps: [escape, click("^(Set speeds|Imposta le velocità)$")] },
  screens: { mock: { trial: true }, steps: [escape, click("^(Screens and picture|Schermi e immagine)$")] },
  effects: {
    mock: { nativeVideo: true, trial: "crt" },
    steps: [escape, click("^(Screens and picture|Schermi e immagine)$")],
  },
  restart: { mock: {}, steps: [escape, click("^(Restart game|Riavvia il gioco)$")] },
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
