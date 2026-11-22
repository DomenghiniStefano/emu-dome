# Screen screenshots (design system migration)

Headless screenshots of the real interface on a mocked Tauri bridge, and a pixel compare, used to
prove that a refactor changes nothing on screen (Trello #62).

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
  A name ending in `*` takes every scenario starting with it (`shell-*`, `saves-*`,
  `controls-*`, `qm-*`: the quick menu's, from `quick-menu.mjs`). A scenario can set `width` /
  `height`, `touch` (a touch screen: `(pointer: coarse)`), `reducedMotion`, `focus` (the focus
  ring shows), and `freeze` (CSS animations stopped at their first frame). Its steps can reach the
  app's own modules (`import('/src/components/game/gameFlow.ts')`) and play a backend event with the mock's
  `__emit(name, payload)` (`emu://menu` opens the quick menu as a controller).
- `manyGames: N` adds N generated games (the `lib-many*` scenarios: 3000). `library-scale.mjs <cdp>
  <url> [games] [--check]` loads such a library and prints the tiles in the DOM and the time per
  d-pad step (`dispatchNav`); `--check` also checks the windowed grid (row / column moves, leaving at
  its edges, the parked focus, back from a game's details far down, no jump while scrolling).
- `screenshots: true` gives every game two screenshots (a GBA one and a DS one, drawn as SVG) for L12
  «Da uno screenshot» and «Adatta la copertina» (the `lib-cover-shots-*` / `lib-cover-crop*` scenarios, #85).
- In-game scenarios set `game` in the mock: the game opens at startup (`true` for the shell's
  screens, `"gba"` / `"nds"` for the quick menu's).
- TV mode (#30, the `tv-*` scenarios at 1280 × 720, 1920 × 1080 and 3840 × 2160): `fullscreen` with
  `pads` turns it on by itself, `ui: "tv"` fixes it (no controller: the Nintendo letters); the steps
  play the controller with `emu://nav` and `emu://menu`.
- The search in the quick menu (#32, the `qm-search-*` scenarios, in Italian): Ctrl + K or `emu://nav`
  `search` over the menu, a query typed, a setting chosen; dark small / large, light small and TV.
- Screenshots stay outside the repository. Stop the dev server and Chrome by PID when done.
