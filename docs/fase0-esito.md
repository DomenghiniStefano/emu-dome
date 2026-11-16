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
