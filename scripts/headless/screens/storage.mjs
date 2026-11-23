// «Gestisci lo spazio» (Trello #29, S1–S3), in Italian. `spaceRow` gives the schema a Saves page
// with its row; `storage: "low"` shows E13; `cloud: "in"` tells which games Drive has (the mock's
// `cloud_games`: «Coral Orbit» and «Dual Screen Tales»). Named `space-<screen>-<size>`: the
// Galaxy S23 as its webview reports itself (`p780` 360 × 780, `l780` 780 × 360) and the desktop
// (`d1280` 1280 × 800).
//
// - `e13`: the shell with E13's warning (the phone only);
// - `page`: E13's «Gestisci lo spazio» (on the desktop, Settings › Salvataggi's row): the screen;
// - `page-end`: the same scrolled to its end («Altri dati»);
// - `remove`: «Togli dal telefono» asked on the first game;
// - `temp`: «Svuota» asked;
// - `quick`: «Salvataggi rapidi › Scegli», one game picked;
// - `shots`: «Screenshot › Apri», one shot picked.

const click = (re, sel = "button") =>
  `(() => { const b = [...document.querySelectorAll('${sel}')].find(b => /${re}/.test((b.textContent + ' ' + (b.getAttribute('aria-label') ?? '')).trim())); b && b.click(); return !!b })()`;
/** The desktop's rail or the phone's tab bar: the n-th section. */
const section = (n) =>
  `(() => { const b = document.querySelectorAll('nav')[0].querySelectorAll('button')[${n}]; b && b.click(); return !!b })()`;
const fromE13 = click("^Gestisci lo spazio");
// Settings › Salvataggi by its category in the page (the rail has a «Salvataggi» too), its row.
const fromSettings = [section(4), click("^Salvataggi", "main button"), click("^Apri$")];
const scrollEnd = `(() => { const m = document.querySelector('main main') ?? document.querySelector('main'); m.scrollTop = m.scrollHeight; return true })()`;

const open = (phone) => (phone ? [fromE13] : fromSettings);
const SCREENS = {
  e13: () => [],
  page: (phone) => open(phone),
  "page-end": (phone) => [...open(phone), scrollEnd],
  remove: (phone) => [...open(phone), click("^Togli da")],
  temp: (phone) => [...open(phone), scrollEnd, click("^Svuota$")],
  quick: (phone) => [...open(phone), click("^Scegli$"), click("^Dual Screen Tales")],
  shots: (phone) => [
    ...open(phone),
    click("^Apri$"),
    `(() => { const b = document.querySelector('main [role=checkbox]'); b && b.click(); return !!b })()`,
  ],
};

const PHONE = { p780: [360, 780], l780: [780, 360] };
const DESK = { d1280: [1280, 800] };

export const storageScenarios = {};
const add = (name, steps, [width, height], phone) =>
  (storageScenarios[`space-${name}`] = {
    mock: {
      spaceRow: true,
      storage: "low",
      cloud: "in",
      screenshots: true,
      lang: "it",
      ...(phone ? { device: "phone", phone: true } : {}),
    },
    steps,
    ...(phone ? { touch: true } : {}),
    width,
    height,
    stepMs: 500,
  });
for (const [screen, steps] of Object.entries(SCREENS)) {
  for (const [size, wh] of Object.entries(PHONE)) add(`${screen}-${size}`, steps(true), wh, true);
  if (screen !== "e13")
    for (const [size, wh] of Object.entries(DESK))
      add(`${screen}-${size}`, steps(false), wh, false);
}
