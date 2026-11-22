# EmuDome

One emulator for **Game Boy, Game Boy Color, Game Boy Advance, Nintendo DS and Nintendo 3DS** (3DS in progress),
on Windows, Linux and Android. Rust shell with Tauri 2 and a Vite + Vue interface; the emulation cores are
interchangeable libretro libraries: mGBA (GB, GBC, GBA), melonDS DS (NDS), Azahar (3DS). EmuDome is free software
(GPL-3.0-or-later) and free of charge.

**No games, BIOS, firmware or keys.** EmuDome ships none and downloads none. You add your own games, dumped
from cartridges you own; 3DS games must already be decrypted by you. The cores run on their built-in open source
BIOS and firmware replacements; you can point EmuDome at your own dumps if you have them.

> State: **Phase 1, desktop shell; Phase 2 (NDS) started.** Phase 0, the feasibility prototype, is done: a game
> drawn natively under a transparent web interface holds up (`docs/fase0-esito.md`). 3DS is Phase 3, first milestone
> done: 3DS games open in the app on the Azahar core, rendered with Vulkan on the app's own GPU device (software
> rendering where Vulkan is not available); encrypted games are refused. Azahar is not packaged yet.
> Full analysis: see the document "EmuDome — Analisi e integrazioni".

## What it has

- A library of your games, identified by their content (not the file name), with covers and play time; the cover
  grid draws only the rows near the screen, so it stays quick with thousands of games. A cover can be generated,
  the ROM's icon, or made from one of your screenshots or your own image, framed in the 2:3 cover (never
  downloaded).
- Two video paths: **native** (wgpu draws on the window, the transparent webview sits on top; the default on
  Windows and Android) and **in the webview** (frames reach a canvas through the `emu://` protocol; the default on
  Linux). Screen effects (librashader presets) on the native path.
- The two DS and 3DS screens, with the touch screen driven by the mouse or by touch.
- Audio with resampling and latency control; fast forward, slow motion, rewind (off by default on phones and for
  3DS), pause, screenshots.
- Keyboard and controllers (SDL3 on desktop), remappable, with L3 shortcuts; every menu works with the controller.
- Save states in slots, the game's own saves, a quick menu (Esc or the controller's Home button).
- Settings at several levels (global, console, game) generated from one schema; three themes and five accent
  colours (amber by default).
- Interface in English and Italian (vue-i18n), chosen from the system language.

## Running on Windows

Requirements: stable Rust; Node in the range of `engines.node` in `app/package.json` (`.nvmrc` names one, the
check stops on another); pnpm as pinned by `packageManager` in `app/package.json` (the frontend refuses npm); the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (WebView2 is already on Windows 10/11); and CMake
for the SDL3 build (on PATH, or in the `CMAKE` variable; VS Build Tools 2022 ships one). The mGBA core is built
by EmuDome from source (`cores.lock`, Trello #88) and published in this repository's release `cores-mgba-<commit>`
(#95), which the fetch script downloads. Building it yourself (`-Build`, or when that download fails) also needs
git, VS Build Tools 2022 with the C++ tools and the Android NDK that `cores.lock` names (its clang-cl compiles mGBA).

```powershell
./scripts/fetch-cores.ps1                     # the cores pinned in cores.lock into cores/ (mGBA from our release)
git config core.hooksPath scripts/git-hooks   # once per clone: the pre-commit hook
cd app
pnpm install
pnpm tauri dev
```

On Linux the same steps work with `sh scripts/fetch-cores.sh` and Tauri's Linux prerequisites (WebKitGTK).

3DS games need the Azahar core, which the default fetch leaves out (it is not shipped yet):
`./scripts/fetch-cores.ps1 -Cores mgba,melondsds,azahar`. On Windows, turn on long paths
(`LongPathsEnabled`) if your data folder is deep: Azahar creates files up to 93 characters below each game's save
folder, and EmuDome refuses to start a game whose folder would push them past the limit.

Every check (format, lint, types, clippy, tests, licences, the project's gates) is one command:
`./scripts/check.ps1` (`sh scripts/check` on Linux or Git Bash). The list lives in `scripts/check`. While working
on a branch, `./scripts/check.ps1 --fast` skips the Android clippy steps and tests only the Rust packages changed
since `develop`.

Then "Open a game". The executable also accepts a game path as an argument and opens it right away. To try it
without games there is `tools/test-rom/emudome-test.gba`: magenta screen, it turns green while A is held (X key).

Your data (saves, settings, library, covers, system files) lives in the folders named after the app identifier,
`it.domehub.emudome`: `%APPDATA%` and `%LOCALAPPDATA%` on Windows, `~/.local/share` on Linux.
`EMUDOME_DATA_DIR=<folder>` puts all of it in another folder instead (test runs). Builds before 2026-11-21 used
`com.domenghini.emudome`: the first start of a newer build copies those folders to the new ones once, verifies the
copy, and leaves the old folders as they were (remove them by hand when you are sure). Android does not carry
over: the new identifier is a different app there, so install it fresh.

## Building for Android

Requirements, on top of the above: a JDK 17, the Android SDK (platform 37, build-tools, an NDK; Android Studio
installs them), `rustup target add aarch64-linux-android`, and on Windows developer mode on (Settings > System >
For developers). Then, with the phone connected over USB and USB debugging on:

```powershell
./scripts/android-build.ps1 -Install           # debug APK (arm64-v8a), installed with adb; without -Install it only builds
```

`sh scripts/android-build.sh --install` does the same on Linux. The script's header lists every step; a release
APK (`-Release`) is signed with a keystore kept outside the repository, named by environment variables (the
script's header, and `CLAUDE.md` «Commands»).

How a release is made: `docs/release.md`.

### Controls

Default keyboard keys (remappable in Controls):

| Key | Function |
| --- | --- |
| Arrows | D-pad |
| X / Z | A / B |
| S / A | X / Y |
| Q / W | L / R |
| 1 / 2 | L2 / R2 |
| Enter / Right Shift | Start / Select |
| Tab (held) | Fast forward |
| Backspace (held) | Rewind |
| F1 / F4 | Quick save / load in the current slot |
| F2 / F3 | Previous / next slot |
| F9 | Show FPS |
| F11 | Full screen |
| F12 | Screenshot |
| Esc | Quick menu |

The full list of keys and controller shortcuts: `docs/combo-tasti.md`, section 5.

Controllers: positional mapping (right button = A, bottom = B), Home button = menu.

## What has been verified

The full log, entry by entry, is `docs/fase0-esito.md`. In short:

- **Windows** (2026-11-16): native video under the quick menu (wgpu on Vulkan), 59.7 FPS at 1×, audio heard,
  controllers through SDL3 (Steam Controller, 8BitDo). Last reported open: the window turns transparent in
  webview mode.
- **Linux** (virtual X11, mGBA 0.10.2): windowless host 1× ≈ 59.7 FPS, 2× ≈ 120, Max ≈ 900; the app in webview
  mode works. Native mode does not (the GTK surface covers the webview): that is why the webview is the Linux
  default.
- **Android** (Galaxy A71, 2026-11-16): native video under the WebView, ≈ 59.8 FPS (Vulkan), touch controls, the
  surface survives background and resume. Not tested yet: audio quality and delay (the test ROM is silent),
  controllers (SDL3 is not built for Android), landscape layout.
- **NDS** (melonDS DS, windowless, Windows, 2026-11-19): two screens, touch, 60 FPS at 1× on the test ROM, states.
- **3DS** (Azahar, windowless, Windows, 2026-11-21, homebrew): through the shell's host on the app's Vulkan
  device, 58.7-60.3 FPS at 1× (debug build), Max 685-802, two screens and touch; software renderer 43 FPS.
  Not tested yet in the app window (the owner's UAT), nor audio and controllers. Known Azahar crash: loading a
  state of content without a title ID (all `.3dsx` homebrew).

## Layout

```
crates/
  libretro-sys/      libretro API types
  emudome-core/      libretro host and the shell's pure logic: core loading, emulation thread, states, speed,
                     input routing, settings engine, library, saves, error codes
  emudome-platform/  everything that depends on the operating system or the form factor (one module per OS,
                     macOS stub; the Android Gradle library)
  hw-render-spike/   hardware-rendering spike, its own workspace, never built by the checks
app/
  src/               Vue interface (Vite); i18n/ holds the message files (en.json, it.json)
  src-tauri/         Tauri shell: window, wgpu video, cpal audio, SDL3 controllers, commands, packaging
scripts/             core downloads, Android build, scripts/check (every check), git hooks
tools/test-rom/      GBA test ROM (homebrew, made here)
tools/test-rom-nds/  NDS test ROM (homebrew, made here: both screens, A, touch)
docs/                decisions, specs, test logs, the release flow
```

Windowless test of the core host alone (handy on Linux/CI):

```bash
cargo run -p emudome-core --example headless -- <core path> tools/test-rom/emudome-test.gba
```

## Platforms

Windows, Linux and Android. macOS and iOS are not compiled (they need the Apple developer license), but the
system-dependent code is isolated in `emudome-platform` so macOS can be added without rewrites.

## Contributing and security

See `CONTRIBUTING.md` (contributions are under GPL-3.0-or-later, `./scripts/check.ps1` before a pull request)
and `SECURITY.md` (how to report a security problem privately).

## License

EmuDome is licensed under the GNU General Public License, version 3 **or (at your option) any later version**
(GPL-3.0-or-later): see `LICENSE`. The libretro cores are separate projects with their own licenses; every
third-party component and its license is listed in `THIRD_PARTY.md`.
EmuDome does not include or download Nintendo games, BIOS, firmware or keys.

Nintendo, Game Boy, Game Boy Color, Game Boy Advance, Nintendo DS and Nintendo 3DS are trademarks of Nintendo.
EmuDome is not affiliated with or endorsed by Nintendo.
