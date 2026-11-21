# Phase 0 on Windows: test results

## 2026-11-16 17:17 · Core downloaded, windowless test, Windows build

Core: `cores/mgba_libretro.dll`, 3,045,973 bytes (mGBA 0.11-212-7a12d6d, libretro nightly).

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | App built, not started yet |
| 2. FPS at 1× | ≈ 60 frames/s | Windowless (`headless`), counting frames: 60 in the first second, then 12 every 200 ms. The core declares 59.7275. The example's statistic prints `0.0` because it reads it right as the first second ends (`session.rs:293`, updated every full second). 2× ≈ 120.4; Max ≈ 7277. In app (F3): not tested |
| 3. Audio | not tested | |
| 4. Controller | not tested | |
| 5. Native vs webview | not tested | |

Also verified without a window: the A button turns the center pixel from magenta to green; state saved to and loaded from slot 1; an empty slot gives the expected error; `.sav` written (131,072 bytes).

Checks: `npm run build` ok, `cargo build -p emudome` ok, `cargo clippy --workspace` zero warnings, `cargo fmt --all --check` clean.

Errors and fixes: none, no file touched.

Risks read in the code (not tested): the transparent "ui" webview over the DX12 surface could cover the game. To try at runtime: `WGPU_DX12_PRESENTATION_SYSTEM=Visual`, then `WGPU_BACKEND=vulkan` or `gl`. The panels' glass (`backdrop-filter`) does not blur the native game underneath.

Proposed next step: start the app on the user's PC and check items 1, 3, 4 and 5 by eye.

## 2026-11-16 18:05 · App started on Windows, quick menu in native mode

`target/debug/emudome.exe` (debug) with Vite on 5173, ROM `tools/test-rom/emudome-test.gba`.
From the log: wgpu uses **Vulkan** (NVIDIA GeForce RTX 3060, driver 617.42), not DX12. Audio opened at 44,100 Hz, **8 channels**, F32.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | ok | User's screenshot (`emudome_4nJB8EU43j.png`, ShareX): in "Native" mode, menu open and game paused, the game's magenta shows under the interface and through the panel. Vulkan backend, no environment variables |
| 2. FPS at 1× | 59.7 | In app with F3, native mode, reported by the user |
| 3. Audio | ok | Heard on headphones, reported by the user. Delay over time not measured. With 8 channels the code (`audio.rs:94-102`) writes left/right on the first two and zeroes the others |
| 4. Controller | ko | The "Controller connected" toast appears, but the buttons do nothing. Model to be noted |
| 5. Native vs webview | no difference in smoothness noticed | In "In the webview" mode the window turns transparent (bug) |

Notes on the interface (from the screenshot, outside the 5 items):
- the panel's content has no right margin (speed, slots, video selector) and ends under a dark band on the edge;
- an empty band remains between the Video section and the buttons at the bottom;
- the glass does not blur the game underneath (`backdrop-filter` only works inside the page): expected.

Errors and fixes: none.
Proposed next step: find the cause of the silent controller and of the transparent window in webview mode.

## 2026-11-16 · Fixes for the controller and webview mode (waiting for the test)

Errors and fixes:
- **Silent controller.** File: `app/src-tauri/Cargo.toml`. Error: "Controller connected" toast but no button read. Probable cause (from the gilrs 0.11.2 documentation, not yet confirmed): the default backend on Windows, Windows.Gaming.Input, reads the buttons only when the foreground window belongs to the process, and the focus is in WebView2. Fix: `gilrs` with `default-features = false, features = ["xinput"]`. Limit: XInput only sees Xbox controllers and compatible ones.
- **Transparent window in webview mode.** Files: `app/src-tauri/src/lib.rs`, `app/src/App.vue`. Cause deduced from the code, not observed: the "ui" webview is created `.transparent(true)` and the WebView2 background stays `(0,0,0,0)` in webview mode too. Fix: `VideoMode::webview_background()`, applied at startup and in `set_video_mode` (opaque `#0e0f13` in webview, transparent in native); an error while switching mode now shows as a toast.

Checks: `cargo fmt --all --check` ok, `cargo clippy --workspace` zero warnings, `vue-tsc` ok, `cargo build -p emudome` ok. App restarted for the test.

## 2026-11-16 20:36 · Controllers with SDL3 instead of gilrs (waiting for the button test)

User's decision: no temporary XInput, switch to SDL3 right away (gamepad only, static, no DLL). The XInput fix of the previous entry was never tested and is replaced.

| Item | Result | Notes |
| --- | --- | --- |
| 4. Controller | recognised, buttons to be tested | Log: `controller collegato: Steam Controller (tipo steam, vid 28de, pid 1304)`, read directly through HIDAPI without Steam. Same result with the windowless example `gamepad_probe` |

Errors and fixes:
- Files: `app/src-tauri/Cargo.toml`, `app/src-tauri/src/gamepad.rs` (rewritten), `Cargo.lock`, new `app/src-tauri/examples/gamepad_probe.rs`. `sdl3-sys` 0.7 (SDL 3.4.20) with `build-from-source-static`, joystick and HIDAPI only. Hints: `SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1`, `SDL_HINT_JOYSTICK_HIDAPI_STEAM=1`.
- The build needs CMake: it is not on PATH, but VS Build Tools 2022 has one. For now the build runs with the `CMAKE` variable set for that command only; the machine was not changed.
- Build folder with a long path: SDL's MSBuild fails (FTK1011, 260-character limit). With the repo's `target\` it does not happen.
- Still open: L2/R2 not mapped (the bit is missing in `input.rs`); the `emu://gamepad-scollegato` event has no toast in the interface; the left stick no longer moves the d-pad.

Proposed next step: hold A on the Steam Controller and see the screen turn green.

## 2026-11-16 20:45 · Button test with SDL3

Controllers: Steam Controller (vid 28de, pid 1304) and 8BitDo in Xbox mode (log: `XInput Controller #1`, type xboxone, vid 2dc8, pid 310a). Connection and disconnection show in the log.

| Item | Result | Notes |
| --- | --- | --- |
| 4. Controller | ok | Reported by the user: button **B** (on the right, Xbox layout) → green screen, i.e. the GBA's A, as the RetroArch-style positional mapping wants. A and X: no effect, expected (the ROM reacts only to the GBA's A, `test.s:24`). Y: the screen flickers; unverified hypothesis: mGBA uses the RetroPad's Y as turbo A. The Steam/Home button opens the menu |
| 5. Native vs webview | ko (low priority) | After the WebView2 background fix the user reports that the transparent window problem in webview mode remains. Cause still to be found |

Errors and fixes: none in this test.
To decide: positional mapping (GBA's A on the right button) or by label (GBA's A on the button marked A); it belongs in the settings engine.
Proposed next step: Phase 0 step 4, Android.

## 2026-11-16 21:15 · Project language switched to English (no test run)

Owner's decision: code, comments, logs and documentation in English; the interface through vue-i18n (`app/src/i18n/`: `en.json` default and fallback, `it.json` with the previous Italian texts), language from the OS locale. Nothing was tested in the app: items 1–5 not tested in this entry.

Errors and fixes:
- Backend errors are now structured: `ErrorCode` in `crates/emudome-core/src/error.rs`, serialized as `{ code, params }`, mirrored in `app/src/emu.ts` and turned into text by `errorText` (`app/src/i18n/labels.ts`). Every command returns it; the `error` event carries `{ type: "error", error: ErrorCode }`. Loading an empty slot is now `slotEmpty` (was the raw text "slot vuoto"); a state the core refuses is `stateInvalid`.
- Controller events renamed: `emu://gamepad` → `emu://gamepad-connected`, `emu://gamepad-scollegato` → `emu://gamepad-disconnected`, both with `{ name }` (null when SDL knows no name). The disconnection now shows a toast, closing the item left open on 20:36.
- Log messages are in English: the log lines quoted in the entries above (`controller collegato: …`) now read `controller connected: …`. Line references in earlier entries (`session.rs:293`, `audio.rs:94-102`) point to the code as it was then.

Checks: `cargo fmt --all` clean, `cargo clippy --workspace --all-targets -- -D warnings` ok, `cargo build -p emudome` ok, `npm run build` ok.
Proposed next step: Phase 0 step 4, Android.

## 2026-11-16 21:53 · Android: Samsung Galaxy A71 (SM-A715F, Android 13), debug APK

APK `app-arm64-debug.apk` (arm64-v8a), core `libmgba_libretro_android.so` (libretro nightly, 2,472,240 bytes) packaged in the APK,
ROM `tools/test-rom/emudome-test.gba` copied to `/data/user/0/com.domenghini.emudome/files/` and opened through the launch intent
(`am start ... --es game <path>`). wgpu uses **Vulkan** (Adreno 618). Portrait, touch overlay on.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | ok | SurfaceView under the transparent WebView: the magenta game shows under the touch buttons and, with the quick menu open, through the panel's glass (tinted, not blurred). Screenshots in the agent's scratchpad: game, A held (green), menu open, webview mode, after background/resume |
| 2. FPS at 1× | core 59.39–59.80, presented 59.87–59.99 | From the 10 s log line `fps: core …, presented …` (target 59.73); first window after start 52.3 presented, after resume 57.2 |
| 3. Audio | partly | AAudio stream opened (48,000 Hz, 2 ch, F32) and `started` in `dumpsys audio`. The test ROM makes no sound: quality and delay not tested |
| 4. Controller | not tested | SDL3 not built for Android; controllers on Android come later |
| 5. Native vs webview | both work | Switch from the quick menu works; webview mode draws the canvas smaller (CSS scale). Smoothness and latency not compared by eye |

Also verified: touch A turns the screen green; the quick menu opens from the touch menu button; going to the home screen and back
re-creates the surface without a crash (same process).

Errors and fixes:
- `tauri android build`: "Creation symbolic link is not allowed for this system" (the CLI symlinks `libemudome_app.so` into jniLibs; Windows
  developer mode is off). Workaround: the Rust part compiles anyway; copy the `.so` into `gen/android/app/src/main/jniLibs/arm64-v8a/`
  and run `gradlew assembleArm64Debug -x rustBuildArm64Debug`. Real fix: enable developer mode.
- `video.rs`: `CreateSurfaceError { inner: Hal(MissingDisplayHandle) }`: wgpu's GLES backend needs a display handle even when Vulkan is used.
  Fix: `ExternalSurface` also implements `HasDisplayHandle` (`DisplayHandle::android()`).
- Build stopped once with a full disk (C: at 0 bytes); space freed by the manager.

Proposed next step: test audio with a game that makes sound, then landscape layout and Android controllers.

## 2026-11-17 22:09 · Hardware rendering spike, step 0: core options, Azahar and melonDS DS on the PC (headless)

Windows 11, AMD Ryzen 7 9700X. Windowless `headless` example, release build, software frames, no audio device.
Cores: libretro nightlies of 2026-11-17 (`azahar_libretro.dll` = Azahar 2127.0-rc3, `melondsds_libretro.dll`), fetched with
`./scripts/fetch-cores.ps1 -Cores melondsds,azahar`. Content, homebrew only, downloaded outside the repo:
- NDS: **Breaking Bad DS** 1.0.6 (William278), Apache-2.0, 3D lab scene,
  https://github.com/WiIIiam278/breaking-bad-ds/releases/download/1.0.6/breaking-bad-ds.nds
- 3DS: **Craftus Reloaded** 0.3 (RSDuck), MIT (textures CC BY-SA), citro3d,
  https://github.com/RSDuck/craftus_reloaded/releases/download/0.3/craftus_reloaded.3dsx

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless only |
| 2. FPS at 1× | melonDS DS 59.8; Azahar 17.4 | melonDS DS software renderer in the 3D lab scene (after tapping A through the intro), 10 s window; Max 222 fps there, 541 fps on the 2D intro. Azahar `citra_graphics_api=Software` on the Craftus title screen, 10 s window (first 5 s run: 19.4); Max 16 fps, so it is saturated |
| 3. Audio | not tested | headless |
| 4. Controller | not tested | headless; A tapped from the example |
| 5. Native vs webview | not tested | headless |

Also verified: melonDS DS starts with no BIOS or firmware (its built-in firmware and FreeBIOS); save state, load state and
pause work on both cores. Azahar without the option: `retro_load_game` returns false (its default `auto` asks for a hardware
renderer). Craftus's menu needs the touch screen, which the host does not send, so no in-world 3DS scene was measured.

Errors and fixes:
- `crates/emudome-core/src/callbacks.rs`: `GET_VARIABLE` always answered false, so Azahar picked `auto` and refused to load.
  Fix: core options stored per session (`core_options.rs`, definitions from `SET_VARIABLES` / `SET_CORE_OPTIONS` v1 and v2),
  `GET_CORE_OPTIONS_VERSION` = 2, the shell's values in `SessionOptions::core_options`, the app forces
  `citra_graphics_api=Software` for Azahar (`FORCED_CORE_OPTIONS` in `app/src-tauri/src/lib.rs`).
- The option value is `Software`, not `software`: Azahar compares case-sensitively.

Proposed next step: step 1, `SET_HW_RENDER` with OpenGL and readback, since Azahar's software renderer is at a quarter of full
speed on the PC.

## 2026-11-18 19:55 · GBA pacing on the 240 Hz PC (Trello #57)

Owner's personal PC: Windows 11, NVIDIA GeForce RTX 3060 (Vulkan), 240 Hz monitor, Logitech G432 (44100 Hz, 8 channels).
App debug build (`pnpm tauri build --debug --no-bundle`, develop cd93712), mGBA core from `cores/`, data folder redirected
with `EMUDOME_DATA_DIR` to a scratch folder. Content: the homebrew test ROM, and the game of the first report (a copy outside
the repo, removed afterwards). Measures from the `pacing:` (session, 10 s windows) and `fps:` (video thread) log lines.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | ok | bench mode with the quick menu opened: presented 233-240 /s with the game paused |
| 2. FPS at 1× | 59.73 | `pacing:` 59.71-59.75 frames/s every 10 s window, native and webview, test ROM and the game of the report, 0 resets in steady play; the 1 s `core` counter swings 58.93-60.33 around it |
| 3. Audio | not tested | device opened, nothing listened to |
| 4. Controller | not tested | |
| 5. Native vs webview | no difference in pacing | webview: `pacing:` 59.72-59.74 (one 10 s window at 58.58 with 2 resets) |

Errors and fixes:
- The 60.66 of the report was not the game running fast. The `core` value of the `fps:` line is the last 1 s window of the
  session, and while the game is paused (quick menu open) the session runs no frames and never updates it: before the fix, with
  the quick menu open the line kept printing the same `core 59.73` for 40 s, with no `pacing:` line at all, while `presented`
  stayed at about 240. A 1 s window reads up to 60.33 in normal play, so a frozen 60.66 fits a window taken just before the
  pause.
- Fix, `crates/emudome-core/src/session.rs`: the 1 s measure is `FpsMeter`, restarted on pause, resume and speed change; paused,
  `emu_fps` is 0; the 10 s `pacing:` window restarts there too; stray spaces removed from the `pacing:` line.
  `app/src-tauri/src/video.rs`: the `fps:` line says `(paused)`. After the fix, quick menu open: `fps: core 0.00 (paused)`.

Proposed next step: none for pacing; listen to the audio at 1× for drift during a long session.

## 2026-11-18 · Hardware rendering spike, step 1: Azahar on Vulkan through wgpu (headless, Windows)
Spike binary `crates/hw-render-spike`; full write-up in `docs/feasibility.md`, section "Hardware rendering
(`SET_HW_RENDER`)". Ryzen 7 9700X, RTX 3060 (NVIDIA 617.42), Azahar 2127.0-rc4 and melonDS DS 1.4.0 nightlies of
2026-11-18, Craftus Reloaded title screen and Breaking Bad DS (homebrew only).

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless; the core's Vulkan image is drawn by a wgpu render pass into an offscreen target and read back: correct picture |
| 2. FPS at 1× | not measured at 1×; unthrottled 1590-2823 fps with the wgpu draw, 28.4 fps with Software | Azahar Vulkan on the device it created through `create_device`, wrapped by wgpu; title screen only, so the render path is measured, not real-scene emulation |
| 3. Audio | not tested | headless |
| 4. Controller | not tested | headless |
| 5. Native vs webview | not tested | readback for the webview path costs 0.05-0.2 ms per 400x480 frame on top of the draw |

Errors and fixes: `crates/hw-render-spike/src/present.rs`, wgpu panicked "Mixing the wgpu encoding API with the raw
encoding API is not permitted" when the raw image barrier and the wgpu render pass shared one encoder; the barrier now
has its own encoder, submitted first in the same `submit`.
Proposed next step: the same spike on the A71 (Azahar Android core, Vulkan) and one PC run under the Khronos validation layer.

## 2026-11-19 23:10 · NDS with melonDS DS through the shell's host: two screens, touch (headless, Windows)
Trello #45, core side. melonDS DS 1.4.0 nightly of 2026-11-19, `tools/test-rom-nds/emudome-test.nds` (our homebrew:
top screen magenta / green with A, bottom screen blue / the touched point as a colour), `headless` example
(`--screens`, `--touch`), debug build of the host. Built-in BIOS and firmware only.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless; the frame is 256 x 384, top screen above the bottom one (forced `top-bottom`, gap 0), cut into two screens by `screens::Split` (PNGs of each screen checked by eye) |
| 2. FPS at 1× | 60.0 frames/s over 3 s (core target 59.83) | Max 528 frames/s (debug host); test ROM, so the CPU load is not a game's |
| 3. Audio | not tested | headless, silent ROM |
| 4. Controller | not tested | A from the input state: top screen green while held, magenta after |
| 5. Native vs webview | not tested | both paths draw the whole frame with the one fit rule (`screens::fit`), which the default layout matches exactly |

Touch: six window points of a 1920 x 1080 window through `screens::pointer_at` (default layout, bottom screen at
704,540 2x) reached the ROM as the expected touch-screen pixel (corners 0,0 and 255,191 included); a point on the top
screen is a release. Save state 18.3 MB, saved and loaded; rewind 10 s = 100 states, 19.2 MB; screenshot ok.
Homebrew writes no `.sav` (melonDS DS keeps one shared SD image, `saves/melonDS DS/dldi_sd_card.bin`).

Errors and fixes: melonDS DS refused the first test ROM ("Expected logo CRC16 of 0xCF56"): it checks the logo checksum of
non-homebrew ROMs only, so the test ROM is homebrew (ARM9 at 0x1000) and carries no logo and no logo checksum; it also
wants at least 0x1000 bytes of header ("ROM is only 1536 bytes").
Proposed next step: open a DS homebrew game in the app (native and webview) and check the stylus with the mouse (UAT).

## 2026-11-21 13:00 · Hot paths: per-frame time of the emulation thread, audio lock contention (headless, Windows)
Trello #79. mGBA 0.11-212-7a12d6d (`cores/mgba_libretro.dll`), `tools/test-rom/emudome-test.gba` (state 528 448 bytes),
release build of the `headless` example with its new `--timing 20`: 20 s at 1x, a quick save into slot 5 every 2 s,
the emulation thread's work per frame recorded (commands, `retro_run`, rewind, saves; not the pacing sleep).
Before = develop 7d6fb85 plus the timer only; after = branch `perf/hot-paths` (I/O worker, rewind encoder thread,
lock-free audio ring, frames swapped in). Same PC, same command.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless; the renderer now presents only on a change, see below |
| 2. FPS at 1× | 1195-1199 frames recorded in each ~20 s run, before and after (core target 59.73) | per-frame work in the tables below |
| 3. Audio | not tested | headless, silent ROM; lock contention measured by a unit test (below) |
| 4. Controller | not tested | |
| 5. Native vs webview | not tested | |

Work per frame, µs (rewind 30 s on, `speed.rew` default):

| Build | mean | p50 | p90 | p99 | p99.9 | max | frames > 8 ms | > 16.7 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| before, run 1 | 613 | 563 | 889 | 1644 | 17903 | 17908 | 9 | 4 |
| before, run 2 | 598 | 557 | 849 | 988 | 16277 | 17218 | 9 | 1 |
| before, run 3 | 616 | 564 | 902 | 1026 | 16227 | 16754 | 9 | 1 |
| after, run 1 | 428 | 445 | 706 | 806 | 5023 | 8053 | 1 | 0 |
| after, run 2 | 420 | 456 | 725 | 855 | 1267 | 2191 | 0 | 0 |
| after, run 3 | 426 | 454 | 720 | 817 | 1263 | 1648 | 0 | 0 |

Rewind off: before mean 317, p99 327, max 16117, 9 frames > 8 ms; after mean 205, p99 453, max 4798, none > 8 ms.
The frames over 8 ms before are the quick saves (undo read, state write, thumbnail PNG, all on the emulation thread):
~16-18 ms each, a skipped frame at 60 Hz. After, a quick save costs the serialize and a hand-off. Rewind's cost on
GBA went from ~300 to ~210 µs per frame (the serialize stays on the emulation thread; the difference is encoded on
its own thread). The two outliers after (8.0 ms and 4.8 ms max, one frame each) were not tied to a save; not
investigated.

Rewind cost for the owner / Cowork decision on its default (rewind stays on by default for now): on GBA ~0.2 ms per
frame and 0.7 MB for 30 s. NDS: not tested, there is no melonDS DS core in `cores/`; from the 2026-11-19 entry an NDS
state is 18.3 MB, so each state taken (every 4 frames above 8 MB: `rewind::interval_for`) costs an 18 MB serialize on
the emulation thread plus the encode on the rewind thread: to measure with `--timing 20 --rewind 30` on a DS game.

Audio lock contention (`cargo test -p emudome-core --release --lib -- audio::tests::callback_wait --ignored
--nocapture`): longest wait of a simulated 5 ms audio callback over 200 callbacks while the producer spends 4 ms per
push (a slow phone's WSOLA): one mutex around queue and work (the old design) 4.001 ms; the lock-free ring 0.003 ms.

Errors and fixes: none in the measurement. Files touched: see Trello #79 (session I/O worker `session/io.rs`,
`rewind.rs`, `audio.rs` in both crates, `video_sink.rs`, `frame.rs`, `video.rs`, `health.rs`).
Proposed next step: the same `--timing` run on a DS game with melonDS DS, and the owner's UAT of the native video
(CPU while paused and minimized) and of unplugging headphones during play.

## 2026-11-21 15:00 · 3DS on Azahar with Vulkan through the shell's host (headless, Windows)
Trello #46, first milestone (branch `feat/3ds-host`). Ryzen 7 9700X, RTX 3060 (NVIDIA 617.42, Vulkan 1.4), Windows 11.
Core: Azahar 2126.2 (commit 219d743), fetched with `./scripts/fetch-cores.ps1 -Cores azahar` (sha256 as in
`cores.lock`). Content, homebrew only, outside the repo: **Craftus Reloaded** 0.3 (RSDuck, MIT; textures CC BY-SA),
`craftus_reloaded.3dsx`, sha256 7beb2eb124342cbc41980f401ececba3eaa53fdc33a5da92f41517544d1e0f47. Tools: `headless` (software) and the new `headless_gpu` example (same
test, the app's GPU: Azahar renders on the app's Vulkan device, frames copied on the GPU and read back into
`Shared::frame`). `--dir` is a short temp folder: under a long folder Azahar's NAND paths pass 248 characters and the
core aborts the process (see Errors).

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless; the owner's UAT. Frames reach `Shared::frame` (read back), which the native renderer uploads as for a software core |
| 2. FPS at 1× | Software: 43.1 (saturated). Vulkan: 58.7-60.3 at 1×, Max 685-802, 2× 121 (debug build, before another program used the GPU) | release numbers with a clean GPU: pending (below) |
| 3. Audio | not tested | headless |
| 4. Controller | not tested | touch tested: `--tap` on the bottom screen enters a Craftus world |
| 5. Native vs webview | not tested | both read the same read-back frame today |

Software renderer (release, title screen, today's develop): 1× 43.1 fps over 8 s, Max 43, 2× 45.8; `--timing 12`
work per frame p50 22.1 ms, p90 24.5 ms, p99 48.9 ms, max 241 ms, 469 of 487 frames over 16.7 ms.

Vulkan (debug build of `headless_gpu`, title screen): Azahar accepts Vulkan on the app's device (negotiation v2 read:
it asks for Vulkan 1.1; its `create_device` is not called), `context_reset` 207-562 ms (the game is loaded there);
picture identical to the software one; 1× 58.7 and 60.3 fps over 3 s (two runs), Max 685 and 802 fps, 2× 121. The
frame splits into the 400×240 top and the centred 320×240 bottom screen; `--tap 110,203` then `--tap 232,168` (New
World, Continue) enter a generated world (an in-world 3D scene).

Not measured cleanly (pending: the owner was playing a game on this PC, which shares the GPU): release runs taken
while that game ran gave 1× 50.1-56.7 fps with Max 419-440 and 2× 121, i.e. not a lack of speed but waits on a GPU busy
with another program; the synchronous readback of those runs has since been bounded (3 ms, two buffers, a late frame
published one frame late). To do with the GPU free: `--timing 20` on the title screen and in the world, release build.

Quick save size (state file, title screen): 545 KB software, 544 KB Vulkan; slot thumbnail 17 KB. A quick save with
Vulkan took up to ~0.5-0.65 s of the emulation thread (p99 / max of a `--timing 20` run with a save every 2 s, under the
GPU contention above): to measure again with the GPU free. The 100 MB cloud warning is far off on this content.

Rewind: not measured. Loading a state on Vulkan rebuilds Azahar's whole renderer (~170 ms seen in the log between
the load's start and its end), so rewind playback at 60 steps per second cannot run on 3DS with Vulkan; and on content
without a title ID (all `.3dsx` homebrew) the load crashes the process (Errors). Owner decision of 2026-11-21: rewind
defaults off on 3DS and phones.

Errors and fixes:
- Azahar 2126.2 aborts the whole process when it cannot create its NAND folders (Windows error 206, path over 248
  characters): seen with a save folder under the long scratch folder. In the app the path after
  `%APPDATA%\com.domenghini.emudome\saves\` is about 130 characters for a homebrew key; not fixed (a card).
- Azahar 2126.2 with Vulkan crashes (access violation in `azahar_libretro.dll` +0x385f5d, reading 0x650, emulation
  thread) at the first frame after a state load, and at boot with `citra_use_disk_shader_cache=disabled`, on content
  without a title ID. Cause read in its source (`vk_pipeline_cache.cpp`, `SwitchCache`): for title ID 0 the switch
  is skipped and the driver pipeline cache is never created. Not ours (it crashes with the shell's copy turned off);
  real games (non-zero title ID) take the other branch (not verified: no free homebrew with a title ID at hand).
  `perf.shcache` is left unwired because of it.
- `gpu_interop.rs`: the device creation looped in `push_next` (the feature structs still chained from the
  query); fixed by clearing their `p_next`.
- `hw_render.rs`: the core's image is wrapped anew every frame (Azahar recreates it on a state load).
Proposed next step: the release `--timing 20` runs (title and world) with the GPU free, then the owner's UAT of a 3DS
homebrew in the app (native and webview video).

## 2026-11-21 16:30 · Azahar and the Windows path limit (headless, Windows)
Trello #94 (branch `fix/azahar-path`). Azahar 2126.2 (commit 219d743), software renderer, debug build of the
`headless` example (`--secs 1`); Craftus Reloaded 0.3 (`.3dsx`, same sha256 as above). Windows 11 with the
`LongPathsEnabled` policy on (1, read only). The save folders were placed with an 8.3 short base path plus padding so
the game's folder (`<dir>\saves\3ds\<40 hex>`) had an exact length; "long-path copy" is the same executable with
`app/src-tauri/windows-app-manifest.xml` embedded by `mt.exe` (a scratch copy, as the app gets it from `build.rs`).

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless |
| 2. FPS at 1× | not measured | runs of 1 s, only whether the game starts |
| 3. Audio | not tested | headless |
| 4. Controller | not tested | headless |
| 5. Native vs webview | not tested | headless |

Deepest paths under the game's folder after a run: folder
`Azahar/nand/data/<32 zeros>/extdata/00048000/F000000B/boss` (80 characters), file
`.../F000000B/user/gamecoin.dat` (93). No long-path manifest: a game folder of 165 characters runs, 166 aborts the
process (illegal instruction, exit 132; the file would be 260 characters), as computed for a 259-character file limit.
With the guard (default `--max-path 247,259`) 166 is refused before the core loads (`savePathTooLong`, length 260,
limit 259) and 165 runs. Long-path copy (`RtlAreLongPathsEnabled` true): game folders of 166, 200, 230, 250, 259,
260, 262, 300 and 320 characters run with the whole tree in the game's folder (35 entries).

Without the manifest and with the guard bypassed (`--max-path 32762,32762`): 166 and 200 abort; 260 and 320 run, but
Azahar cannot reach its save directory and writes its whole tree to `%APPDATA%\Azahar` instead (created by these
runs, holding Craftus' `sdmc/craftus/options.ini`). So past 260 characters without long paths the saves would go to a
folder shared by every game, silently; the guard refuses those too.

Errors and fixes: `crates/emudome-core/src/save_path.rs` (the guard), `n3ds::SAVE_DEPTH`, `CoreProfile::save_depth`,
`ErrorCode::SavePathTooLong`, `crates/emudome-platform/src/paths.rs` (limits, policy), `app/src-tauri/build.rs` and
`windows-app-manifest.xml` (long-path aware app). The earlier entry's abort is now refused with a message.
Proposed next step: the owner's UAT of a 3DS homebrew in the installed app with a deep data folder (`EMUDOME_DATA_DIR`).

## 2026-11-21 18:00 · 3DS on Azahar with Vulkan: release measurements with the GPU free (#46)
Release build of the app's `headless_gpu` example (branch `feat/3ds-host` = develop's 3DS code, bounded readback),
Azahar 2126.2 (219d743, pinned), Craftus Reloaded 0.3 homebrew (MIT), Vulkan on the app's device, RTX 3060 /
NVIDIA 617.42, Ryzen 7 9700X, no other program using the GPU. `--dir` a short temp folder.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless; the frames reach `Shared::frame` through the readback |
| 2. FPS at 1× | title screen 57.2 over 10 s (last second 60.0); in the world 59.2 over 10 s (last second 60.0); target 60.00 | the 10-s averages include the start (context reset ~0.2-0.5 s) |
| 3. Audio | not tested | headless |
| 4. Controller | not tested | touch by `--tap` (enters a generated world) |
| 5. Native vs webview | not tested | |

Max (unthrottled): title 1098 fps, world 551 fps. 2×: title 105.3 emulated fps; in the world 13.5 (not explained:
Max reaches 551 there, so it is not the GPU; to look at with the speed / audio path at 2× on 3DS).

Work per frame, `--timing 20` (a quick save every 2 s, µs):
| Scene | frames | mean | p50 | p90 | p99 | p99.9 | max | > 8 ms | > 16.7 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| title | 1084 | 3578 | 1200 | 1803 | 18763 | 237607 | 271945 | 16 | 12 |
| world | 1064 | 4875 | 1551 | 3076 | 41872 | 310504 | 323915 | 46 | 29 |

The frames over 16.7 ms are mostly the quick saves: on Vulkan a state save takes ~0.24-0.32 s of the emulation thread
(Azahar serializes with the renderer's state), so a 3DS quick save drops ~15-20 frames. Outside saves the emulation
holds 60 fps with a p50 of 1.2-1.6 ms per frame.
GPU tests after the readback change (`cargo test -p emudome --lib gpu -- --ignored`): 3 passed (core image as a texture,
core image → Shared::frame, built-in effects).

Errors and fixes: none in these runs.
Proposed next step: look at the 3DS quick-save stall (serialize off the hot path is not possible: only the emulation
thread may call the core; at least warn or pause audio) and the 2× slowdown in the world.

## 2026-11-21 19:30 · 3DS quick-save stall and 2× in the world (#98)
Release build of `headless_gpu` (branch `perf/3ds-stalls`, from develop 0146fb2), Azahar 2126.2 (219d743, pinned),
Craftus Reloaded 0.3 homebrew (MIT, same sha256), Vulkan on the app's device, RTX 3060 / NVIDIA 617.42, Ryzen 7
9700X, GPU free. In the world (`--press-a 2 --tap 110,203 --tap 232,168`, checked on a PNG of the top screen), a
fresh short `--dir` per run. Runs kept under ~20 s, so `--secs 3-6` instead of 10.

| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | not tested | headless |
| 2. FPS at 1× | 59.7-60.0 in the world (5 runs, 3-6 s each) | last second 60.0 |
| 3. Audio | not tested on a device | the output's refill after a stall is unit-tested (below) |
| 4. Controller | not tested | touch by `--tap` |
| 5. Native vs webview | not tested | |

2× in the world: 120.2-120.3 frames/s over 3-6 s in 5 runs, first second 120.9-121.0, last second 120.0; frame work at
2× max 9-17 ms, at most 2 frames over 8.3 ms. Max in the world 764-1272 frames/s (2-10 s). Also after 10 s at Max,
2× ran at 120.3. The 13.5 of the earlier entry did not come back: that run read the rate of the first 1-s window
1.2 s after 10 s at Max, so one stall at that moment was enough to read low; the code paths have not changed since
(no commit to the session, readback or callbacks after it). `headless` now measures 2× over the whole `--secs`
window from the frame counter and prints the first and last second and the frame work (`--max-secs` sets the Max
window alone).

Quick save (`--timing 10`, a save every 2 s; time of `retro_serialize_size` + `retro_serialize`, from the new
`quick save: ... taken in` log line; states of 755-771 KB):
| Renderer | Model | Saves | Time per save | Frames after it |
| --- | --- | --- | --- | --- |
| Vulkan | New 3DS (default) | 5 + 4 | 233-305 ms | 1.0-3.2 ms each (6 frames after each save) |
| Software | New 3DS | 5 | 231-237 ms | (the software world runs at ~4 fps: not comparable) |
| Vulkan | Old 3DS (`citra_is_new_3ds`) | 3 | 155-159 ms | |

The time is the same with the software renderer and scales with the emulated memory (New 3DS 256 MB FCRAM, Old
3DS 128 MB): it is Azahar's `System::SaveStateBuffer()` (Boost archive of the whole system, then zstd), all of it run
inside `retro_serialize_size`; `retro_serialize` only copies. The Vulkan cache flush in it (`gpu->ClearAll(true)`)
costs nothing visible: the frames after a save take their usual 1-3 ms. The shell already calls the slow part once
per save, and only the emulation thread may call it, so the stall stays: a 3DS quick save drops ~15-18 frames.
Draft for upstream: `docs/upstream/azahar-libretro-savestate-stall.md` (not posted).

What the shell does now: the audio output, once its queue runs dry, stays silent until the queue is back at its
target (60 ms) and fades back in over 5 ms. Before, it played each sample as it arrived, and the ±0.5 % control took
~12 s to refill 60 ms, so after a stall the queue hovered at empty with a fade and a step on every late frame (a
crackle). Unit test `audio::tests::a_stall_is_one_clean_gap_and_the_queue_refills_to_its_target` (a 0.3-s stall: one
dry spell, no step above 0.02, the queue back near its target); it fails on the old code (dry spells repeated). The
pacing already starts over after the stall without catch-up frames.

Errors and fixes: none in these runs.
Proposed next step: the owner hears a 3DS quick save in the app (one short clean gap, no crackle after it); decide
whether 3DS keeps `saves.autoq` at 5 min (one ~0.25 s hitch each time).
