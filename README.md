# EmuDome

One emulator for **Game Boy Advance, Nintendo DS and Nintendo 3DS**, on Windows, Linux and Android.
Rust shell with Tauri 2 and a Vite + Vue interface; the emulation cores are interchangeable libretro libraries
(mGBA, melonDS DS, Azahar). EmuDome is free software (GPL-3.0-or-later) and free of charge.

**No games, BIOS, firmware or keys.** EmuDome ships none and downloads none. You add your own games, dumped
from cartridges you own; 3DS games must already be decrypted by you. The cores use their built-in open source
BIOS replacements.

> State: **Phase 0, feasibility prototype.** It is meant to find out whether a game drawn natively
> under a transparent web interface holds up, before building the rest.
> Full analysis: see the document "EmuDome — Analisi e integrazioni".

## What Phase 0 has

- Loading of a libretro core (mGBA) and of a GBA game.
- Two video paths, comparable from the quick menu:
  - **Native**: wgpu draws on the window, the transparent webview sits on top (default on Windows and Android).
  - **In the webview**: the frames reach a canvas through the `emu://` protocol (default on Linux).
- Audio with resampling and latency control.
- Keyboard and controllers (SDL3).
- Speed 0.5×–4× and Max, turbo while holding Tab, pause.
- States in 3 slots, automatic game save (`.sav`).
- Quick menu with Esc or the controller's Home button.
- Interface in English and Italian (vue-i18n), chosen from the system language.

## Running on Windows

Requirements: stable Rust, Node 22.13+ or 24 (`.nvmrc`; `engines` in `app/package.json`, the check stops on another Node), pnpm 12 (the frontend refuses npm), the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (WebView2 is already on Windows 10/11),
and CMake for the SDL3 build (on PATH, or in the `CMAKE` variable; VS Build Tools 2022 ships one).

```powershell
./scripts/fetch-cores.ps1                     # downloads the cores pinned in cores.lock into cores/
git config core.hooksPath scripts/git-hooks   # once per clone: the pre-commit hook
cd app
pnpm install
pnpm tauri dev
```

Every check (format, lint, types, clippy, tests, the project's gates) is one command: `./scripts/check.ps1`
(`sh scripts/check` on Linux or Git Bash). The list lives in `scripts/check`. While working on a branch,
`./scripts/check.ps1 --fast` skips the Android clippy steps and tests only the Rust packages changed since `develop`.

Then "Open a game". The executable also accepts a game path as an argument and opens it right away. To try it without games there is `tools/test-rom/emudome-test.gba`:
magenta screen, it turns green while A is held (X key).

### Controls

| Key | Function |
| --- | --- |
| Arrows | D-pad |
| X / Z | A / B |
| A / S | L / R |
| Enter / Right Shift | Start / Select |
| Tab (held) | Fast forward |
| F1 / F4 | Quick save / load in the current slot |
| F2 / F3 | Previous / next slot |
| F9 | Show FPS |
| F11 | Full screen |
| Esc | Quick menu |

The full list of keys and controller shortcuts: `docs/combo-tasti.md`, section 5.

Controllers: positional mapping (right button = A, bottom = B), Home button = menu.

## What to check (Phase 0 verdict)

1. The game is visible **under** the interface: the quick menu and the toasts are on top, with the game visible around them.
2. Resizing the window leaves no black areas or flicker.
3. F9 shows about 59.7 FPS at 1×; 2× shows about 120.
4. The audio does not crackle and does not lag after a few minutes.
5. Save and load work; closing and reopening the game keeps the `.sav`.

If item 1 does not work on Windows, switch to "In the webview" from the quick menu and compare.
It can also be forced at startup with `EMUDOME_VIDEO=webview` (or `native`).

### Already verified (Linux, virtual X11, mGBA core 0.10.2)

- Windowless libretro host: loading, input, 1× ≈ 59.7 FPS, 2× ≈ 120 FPS, Max ≈ 900 FPS, states and `.sav`.
- Full app in "In the webview" mode: game on screen, buttons, quick menu over the game, 2× speed, saving to a slot.
- Native mode on Linux/GTK: **does not work**, the wgpu surface covers the interface. That is why the webview is the Linux default.
- Not verified: Windows (neither mode), real audio, controllers, Android.

Windows results are logged in `docs/fase0-esito.md`.

## Layout

```
crates/
  libretro-sys/      libretro API types
  emudome-core/      libretro host: core loading, emulation thread, states, speed, input, error codes
  emudome-platform/  everything that depends on the operating system (one module per OS, macOS stub)
app/
  src/               Vue interface (Vite); i18n/ holds the message files (en.json, it.json)
  src-tauri/         Tauri shell: window, wgpu video, cpal audio, SDL3 controllers, commands
scripts/             core downloads, scripts/check (every check), git hooks
tools/test-rom/      GBA test ROM (homebrew, made here)
tools/test-rom-nds/  NDS test ROM (homebrew, made here: both screens, A, touch)
```

Windowless test of the core host alone (handy on Linux/CI):

```bash
cargo run -p emudome-core --example headless -- <core path> tools/test-rom/emudome-test.gba
```

## Platforms

Planned builds: Windows, Linux, Android. macOS and iOS are not compiled (they need the Apple
developer license), but the system-dependent code is isolated in `emudome-platform`
so macOS can be added without rewrites.

## Contributing and security

See `CONTRIBUTING.md` (contributions are under GPL-3.0-or-later, `./scripts/check.ps1` before a pull request)
and `SECURITY.md` (how to report a security problem privately).

## License

EmuDome is licensed under the GNU General Public License, version 3 **or (at your option) any later version**
(GPL-3.0-or-later): see `LICENSE`. The libretro cores are separate projects with their own licenses; every
third-party component and its license is listed in `THIRD_PARTY.md`.
EmuDome does not include or download Nintendo games, BIOS, firmware or keys.

Nintendo, Game Boy Advance, Nintendo DS and Nintendo 3DS are trademarks of Nintendo. EmuDome is not affiliated
with or endorsed by Nintendo.
