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
- `drive.mjs` lists the scenarios; a stream that migrates a screen adds the scenarios it needs. A
  name ending in `*` picks every scenario with that prefix (`qm-*`: the quick menu's, from
  `quick-menu.mjs`, each state in the three themes at 1024 × 640 and 1920 × 1080, plus the phone).
- In-game scenarios set `game` in the mock: the game opens at startup, and a step can send a
  backend event with `window.__emit(event, payload)` (`emu://menu` opens the menu as a controller).
- Screenshots stay outside the repository. Stop the dev server and Chrome by PID when done.
