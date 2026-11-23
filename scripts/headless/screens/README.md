# Screen screenshots

Headless screenshots of the real interface on a mocked Tauri bridge, and a pixel compare, used to
prove that a refactor changes nothing on screen and to look at a screen at given window sizes
without opening the app.

1. Start the interface: `cd app; pnpm dev --port <free port>` (never a port already in use).
2. Start Chrome headless, never visible: `chrome --headless=new --disable-gpu --remote-debugging-port=<cdp>
   --user-data-dir=<scratch folder>`.
3. `node scripts/headless/screens/drive.mjs <cdp> http://localhost:<port>/ <scratch>/before [scenario…]`
   on the code before the change, then the same into `<scratch>/after` on the code after it.
4. `node scripts/headless/screens/compare.mjs <cdp> <scratch>/before <scratch>/after`: 0 different
   pixels per screen is the goal.

- `mock.js` answers every `invoke` with fixed data (games, saves, settings) and fixes the clock, so
  two runs are identical. A scenario sets its options (`theme`, `emptyLibrary`, `firstRun`, …).
- `drive.mjs` lists the scenarios; a stream that migrates a screen adds the scenarios it needs.
  A `*` in a name stands for any text (`shell-*`, `saves-*`, `controls-*`, `qm-*`: the quick
  menu's, from `quick-menu.mjs`; `saves-*-dark-s`: one theme and size of each). Each scenario prints
  `step failed` for a step that threw or found nothing, and the page's `error:` / `warning:` console
  lines and uncaught exceptions: a stale mock shows there (the user only sees the generic «Something
  went wrong» toast, `errors.unexpected`). A scenario can set `width` /
  `height`, `touch` (a touch screen: `(pointer: coarse)`), `reducedMotion`, `focus` (the focus
  ring shows), and `freeze` (CSS animations stopped at their first frame). Its steps can reach the
  app's own modules (`import('/src/components/game/gameFlow.ts')`) and play a backend event with the mock's
  `__emit(name, payload)` (`emu://menu` opens the quick menu as a controller).
- `manyGames: N` adds N generated games (the `lib-many*` scenarios: 3000). `library-scale.mjs <cdp>
  <url> [games] [--check]` loads such a library and prints the tiles in the DOM and the time per
  d-pad step (`dispatchNav`); `--check` also checks the windowed grid (row / column moves, leaving at
  its edges, the parked focus, back from a game's details far down, no jump while scrolling).
  `--phone` does the same on a phone upright (412 × 892) with its list, `--phone --grid` with its
  compact grid (#29), `--phone --side` on its side (780 × 360) with the list in two columns (#105:
  also two columns, at least 6 games in view, right from «Gioca» to the next column).
- `phone-shell.mjs` also has the Galaxy S23 as its webview reports itself, 360 × 780 (`p780`) and 780 × 360
  (`l780`), and `phone-surround-<nds|3ds|gba>-<p412|p780|l892|l780>` (#105): the native video with
  a built-in side background (`surround: "builtin"` in the mock gives the video page that row and
  value), one hole per screen where the mock's `screens_placements` puts it (headless has no native
  surface: the holes show black). `phone-controls-pad*` (#118): Controls with a controller, without
  the keyboard's card, and with it once a physical key was pressed (`{ key }` step);
  `phone-settings-rich*` (#124): every row kind of `richSettings` in the phone's one column (the rich
  schema takes the scenario's device). #132: `phone-controls-touch*` (no controller, `layoutRows`: E05 a card
  over «Controlli touch»), `phone-settings-library*` (a locked list faded with its row),
  `phone-library-hidden*` / `phone-library-hidden-off*` (`hiddenGame: true` hides «Star Pilot»; Settings'
  «Mostra i giochi nascosti», L03's chip on the phone's toolbar, tapped off), and in `touch.mjs`
  `touch-editor-size-open*` / `touch-editor-size-back*` (Android's back closes C06's tool panel first).
- `libraryView: "list" | "grid"` fixes `library.view` (#29; otherwise the list on `device: "phone"`,
  the grid elsewhere, and the toolbar's switch writes it). The phone's library: the `phone-library*`
  scenarios of `phone-shell.mjs` (the list, the compact grid, the switch, the regional versions, L03);
  the desktop at 1280 × 800: `lib-desk-grid`, `lib-desk-list`. L03 when the window is short (#106):
  `phone-library-filters-end-*` walks the panel down to its last row with the controller (its body
  scrolls, «Ripristina» stays in view); `lib-desk-filters` and `lib-desk-filters-zoom120` (120 %
  interface scale, 1280 × 600) on the desktop.
- `screenshots: true` gives every game two screenshots (a GBA one and a DS one, drawn as SVG) for L12
  «Da uno screenshot» and «Adatta la copertina» (the `lib-cover-shots-*` / `lib-cover-crop*` scenarios, #85).
- `cloud` sets the Google Drive status (#31): `"in"`, `"conflict"`, `"offline"`, `"notConfigured"`, else signed
  out; signed in, Drive lists four games (one only on Drive), two devices and some activity, and the schema
  gains the Cloud page. The `cloud-*` scenarios open the Cloud section, its tabs, G04 / G05 / G03 / G14 and the
  library's «Solo su Google Drive».
- In-game scenarios set `game` in the mock: the game opens at startup (`true` for the shell's
  screens, `"gba"` / `"nds"` for the quick menu's). The phone's quick menu (#108): `qm-phone-*` at
  390 × 844 (`phone: true`), `qm-s23-<gba|nds>-*` on the S23 sizes with a game picture, and
  `qm-desk-*` at 1280 × 800 for the desktop menu.
- TV mode (#30, the `tv-*` scenarios at 1280 × 720, 1920 × 1080 and 3840 × 2160): `ui: "tv"` fixes it
  (with `pads` the Xbox glyphs, without the Nintendo letters); `pads` alone (Automatica) shows the TV
  question at startup (`tv-ask-*`); `richSettings` + `wiredSettings` give Settings every row kind,
  usable; the steps play the controller with `emu://nav` and `emu://menu`.
- The search in the quick menu (#32, the `qm-search-*` scenarios, in Italian): Ctrl + K or `emu://nav`
  `search` over the menu, a query typed, a setting chosen; dark small / large, light small and TV.
- The touch controls (#29, the `touch-*` scenarios from `touch.mjs`, in Italian, at 390 × 844, 360 × 740,
  844 × 390 and 740 × 360, some at 320 × 640): the overlay in game (`game: "gb" | "gba" | "nds" | "3ds"`), the
  styles «Fumé» (the default) and «Contorno» (`touch-skin-outline*`) and E14 (`touchRows` gives the touch page's
  values, `touchSaved` a saved landscape layout), and the C06 editor opened from the quick menu's «Altro ›
  Controlli» (a tool open, «Altro» open, folded). `touch-hint-<touch|pad|keyboard>-p780` (#118): G06 with only
  the touch controls (no hint), a controller (Home) and a physical key pressed (Esc). `touch-look-<smoke|outline>-<nds|3ds|gba>-<p780|l780>`: the compact controls
  on the Galaxy S23 as its webview reports itself (360 × 780, 780 × 360) with its camera's 38 px, both styles over
  a game picture.
- The phone's layouts with a game picture (#29, `touch-frame-*`, `touch-qm-frame-*`, `touch-qm-turned*`,
  `touch-start-failed-*`, also at 412 × 892 and 892 × 412): `frame: true` gives the webview canvas
  a GBA frame placed as `screens::compose` places it at `video.scale` (as large as fits on a phone, whole pixels on
  a computer; `videoScale: "fit" | "integer" | "fill"` fixes it, the `touch-scale-<scale>-gba-<p780|l780>` scenarios,
  #105) (mirrored in `mock.js`, at `frameDpr` device pixels per CSS pixel,
  2.625 by default, as the native video on that phone); with `game: "nds"` / `"3ds"` the frame has two screens
  (the top magenta, the touch screen teal) placed as `screens::compose` places them on a phone (`auto`, filling its
  room, the inset's `between`, `apart` / `top`; the `touch-apart-*` scenarios, the S23 ones with the camera's 32 px
  set on the `safe-*` tokens); a `{ resize: [w, h] }` step turns the phone. `PROBE=<js>`
  in the environment evaluates an expression on every page after its screenshot and prints it (the mock keeps the
  last `set_video_inset` in `window.__videoInset` and the last placements' area in `window.__lastArea`).
- The layout editors without a game (#111, the `layout-*` scenarios from `layout-editors.mjs`, in Italian, at
  360 × 780, 780 × 360 and 1280 × 800): `layoutRows` gives the schema the `screens.custom` and `touch.tedit` rows on
  their pages, `frameConsole` the console whose screens the mock's `screens_preview` places for C06; the rows,
  the console chooser, C06 / C09 over their placeholders, and the phone quick menu's «Altro › Modifica la
  disposizione».
- Screenshots stay outside the repository. Stop the dev server and Chrome by PID when done.
