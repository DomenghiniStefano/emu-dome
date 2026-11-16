# EmuDome: instructions for Claude Code

One emulator for **Game Boy Advance, Nintendo DS and Nintendo 3DS**, on **Windows, Linux and Android**.
Rust shell with Tauri 2 and a Vue/Vite interface. The emulation cores are interchangeable libretro libraries:

- GBA: mGBA
- NDS: melonDS DS
- 3DS: Azahar

## Language

- **Code comments and documentation** (this file, `docs/`, `README.md`, script comments and messages, log messages): **English**.
  Names in code (types, functions, variables) in English.
- **User interface**: **never hardcoded**. Every user-facing string lives in the vue-i18n message files
  (`app/src/i18n/`): `en.json` is the default and the fallback, `it.json` the Italian translation (same keys).
  The language follows the OS / browser locale; anything that is not Italian gets English.
- The Rust backend sends **structured data, never sentences**: errors are `ErrorCode` (`crates/emudome-core/src/error.rs`),
  serialized as `{ code, params }` and mirrored in `app/src/emu.ts`; events carry codes and values
  (e.g. `emu://gamepad-connected` with `{ name }`). The interface turns them into text in `app/src/i18n/labels.ts`
  (`errorText`, `speedLabel`, `gamepadName`). A new error = a new `ErrorCode` variant + its TypeScript mirror
  + `errors.<code>` in **both** message files.
- Kept file names: `docs/fase0-esito.md` and `docs/combo-tasti.md` keep their Italian names (an external Cowork session
  refers to them); their content is English.

## Current state: Phase 0, feasibility prototype

The goal is to find out whether the video architecture holds up, before building the rest.
**No push** until feasibility is confirmed. Local commits only when the user asks for them.

Already done and verified on Linux (virtual X11, mGBA 0.10.2):

- Windowless libretro host (`cargo run -p emudome-core --example headless -- <core> tools/test-rom/emudome-test.gba`):
  1× ≈ 59.7 FPS, 2× ≈ 120, Max ≈ 900; save states and `.sav` work.
- Full app in "webview" video mode: game on screen, buttons, quick menu over the game, speed, slots.

To verify now, **on Windows**:

1. **Native** video mode: with the quick menu (Esc) open, the game must stay visible under the interface.
   This is the question that decides the architecture.
2. FPS (F3) ≈ 59.7 at 1×.
3. Clean audio, with no delay growing over time.
4. Controller recognised ("Controller connected" toast).
5. Smoothness and latency compared between "Native" and "In the webview" (selector in the quick menu).

After Windows comes Phase 0 step 4: **Android**.
It needs `tauri android init`, a Kotlin plugin with a `SurfaceView` under the WebView, the `.so` cores in jniLibs
(loading still to be completed in `crates/emudome-platform/src/android.rs`) and a few touch buttons.

## Commands

```powershell
./scripts/fetch-cores.ps1        # downloads mgba_libretro.dll into cores/ (not versioned)
cd app; npm install; npm run tauri dev
```

- Checks: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` (zero warnings),
  `npm run build` in `app/` (also runs `vue-tsc`).
- The SDL3 build needs CMake. If it is not on PATH, set `CMAKE` for that command only (VS Build Tools 2022 ships one).
- Environment variables:
  - `EMUDOME_VIDEO=native|webview` forces the video path.
  - `EMUDOME_CORES_DIR` tells where to look for the cores.
  - `RUST_LOG=debug` for the logs.
- The executable accepts a game path as an argument and opens it at startup.
- Homebrew test ROM: `tools/test-rom/emudome-test.gba`. Magenta screen, green while A is held (X key).

## Architecture

```
crates/libretro-sys      libretro API types and constants (added as they are needed)
crates/emudome-core      libretro host: LoadedCore (libloading), Session (emulation thread),
                         Command/Event, Shared (RGBA frame, audio queue, input, stats), speed, states, .sav,
                         ErrorCode (errors for the UI as code + params)
crates/emudome-platform  EVERYTHING that depends on the operating system, one module per OS
app/src-tauri            Tauri shell: window, video.rs (wgpu), audio.rs (cpal), gamepad.rs (SDL3), commands
app/src                  Vue interface: App.vue, QuickMenu, GameCanvas (webview mode), Toasts, emu.ts (typed bridge),
                         i18n/ (index.ts vue-i18n setup and locale choice, en.json, it.json, labels.ts codes → text)
```

- The interface never touches the core: it calls the Tauri commands, which send `Command` to the `Session`.
  The core runs on its own thread; the libretro callbacks use a `thread_local` context.
- **Video, two paths:**
  - **Native**: wgpu draws on the window, under a transparent child webview ("ui").
    Needs Tauri's `unstable` feature (`window.add_child`).
  - **Webview**: the RGBA frames reach a `<canvas>` through the `emu://frame?since=N` protocol.
  - The default per OS is `NATIVE_VIDEO_DEFAULT` in `emudome-platform`: Linux uses the webview.
- Audio: shared queue, linear resampling to the device's rate,
  ±0.5% correction on the queue's fill level. Muted when the speed is not 1×.
- Keyboard: for now it goes through the webview (`input` command). Controllers go through an SDL3 thread
  with RetroArch-style positional mapping; the Home button opens the menu.
  Events: `emu://gamepad-connected`, `emu://gamepad-disconnected` (`{ name }`), `emu://menu`.

## Lessons already learned (do not repeat)

- The wgpu surface must be created **on the main thread**: from other threads the Tauri window handle blocks.
  Drawing then continues on a dedicated thread.
- Create the wgpu instance **without a display handle** (`new_without_display_handle_from_env`).
  With Tauri's handle the surface fails with `MismatchingDisplayHandle`.
- On Linux/GTK the native surface **covers** the webview. Webview mode is used there.
- wgpu 30 changed its API compared to the examples around:
  - `queue.present(frame)`
  - `CurrentSurfaceTexture`
  - `color_space` in `SurfaceConfiguration`
  - `Some(&layout)` in `bind_group_layouts`
- cpal 0.18: `build_output_stream` takes the `StreamConfig` by value.

## Project rules

- **No `#[cfg(target_os)]` outside `crates/emudome-platform`.** macOS exists as a stub but is **not** compiled
  (no Apple license). iOS is excluded. The code must stay ready to add macOS without rewrites.
- **Never include, download or point to Nintendo files** (games, BIOS, firmware, keys). The user adds the games only.
  The cores use their built-in open source replacements. 3DS games must be decrypted.
- License GPL-3.0-or-later; the cores are separate projects with their own licenses.
- Features live in the shell, not in the cores: same interface for GBA, NDS and 3DS.
  Where a core cannot support a feature, the interface shows it disabled with the reason.
- Settings will be the strong point (Phase 1): levels Default → Global → Console → Game → Session,
  plus a "per device" axis. UI generated from a single schema in Rust. Do not write scattered settings by hand.

## Interface and tone

- Design system: **EmuDome Design System** (a copy of GameDome's that evolves on its own),
  https://claude.ai/artifact/TEz4yQgd1w27HRQLt4beAy. The main tokens are already in `app/src/styles.css`.
- Modern console style, dark theme:
  - neutral graphite, the colour comes from the covers;
  - purple (`accent`) only on the main action;
  - glass for panels and menus;
  - Lucide icons, no emoji.
- In game **no interface**: everything goes through the quick menu (Esc / Home button).
- **Every menu is fully usable with the controller** (d-pad/stick to move, A confirms, B goes back), as well as with mouse and keyboard.
  No screen may require mouse or keyboard.
- Texts (in every language): address the user directly ("tu" in Italian), short sentences, verb first on buttons,
  no exclamation marks. Errors say what is happening and what to do.

## References

- Analysis and integrations (cores, platforms, settings catalogue, risks, roadmap):
  https://claude.ai/artifact/A8hWdUgrW6qieC7XxQm6ou
- Interface prototype (draft, paused): https://claude.ai/artifact/HeL9MWZ9Pd41dGLGCZWzaj
- Roadmap:
  - Phase 0: video, GBA, Windows and Android
  - Phase 1: desktop shell (library, settings engine, complete saves, shaders with librashader)
  - Phase 2: NDS
  - Phase 3: 3DS
  - Phase 4: full Android
  - Phase 5: extras

## Known gaps of the prototype (intentional, for later)

- `RETRO_ENVIRONMENT_GET_VARIABLE` always answers `false`: core options come with the settings engine.
- Audio is muted in fast forward and slow motion.
- The cores are not included in the build; the package takes them from `cores/` as a resource.
- No game library, no button remapping, no rewind: they are Phase 1.

## Handoff with Cowork

Analysis, decisions and design are handled in a separate Cowork session, which reads this folder but does not see the Claude Code conversation.
After every Phase 0 test, update **`docs/fase0-esito.md`** (create it if missing) in this format, without deleting the earlier entries:

```
## <date and time> · <what was tested>
| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | ok / ko / not tested | what is visible, screenshots if any |
| 2. FPS at 1× | measured value | headless or in app |
| 3. Audio | ok / ko / not tested | crackles, delay |
| 4. Controller | ok / ko / not tested | model |
| 5. Native vs webview | differences noticed | smoothness, latency |

Errors and fixes: files touched, error, cause, how it was solved.
Proposed next step: one line.
```

Rules: write only verified facts, mark "not tested" where the test is missing, do not change the sections of this file without telling the user.
