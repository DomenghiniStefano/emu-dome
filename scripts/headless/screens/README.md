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
  A name ending in `*` takes every scenario starting with it (`shell-*`). A scenario can set
  `width` / `height`, `touch` (a touch screen: `(pointer: coarse)`), `reducedMotion`, and `freeze`
  (CSS animations stopped at their first frame). Its steps can reach the app's own modules
  (`import('/src/gameFlow.ts')`) and play a backend event with the mock's `__emit(name, payload)`.
- Screenshots stay outside the repository. Stop the dev server and Chrome by PID when done.
