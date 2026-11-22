# Feasibility checks

Desk research done in the Cowork session, without running the app. Each section says what was checked, how, what
is settled and what is still **to verify** with code on a real machine. Owner decisions these checks serve are in
`docs/decisions.md`, section "Product". The customization study they come from is `docs/customization.md`.

## 2026-11-17 · librashader with our wgpu

**Question**: can shaders (librashader's wgpu runtime) go into Phase 1, given that we are on wgpu 30
(`Cargo.lock`: wgpu 30.0.2)?

**How**: a scratch crate with `librashader = { version = "0.12.1", default-features = false, features =
["runtime-wgpu"] }`, then `cargo tree -i wgpu` and `cargo check` (Linux, Rust 1.97).

**Result**:

- librashader **0.12.1** (latest stable on crates.io) depends on `wgpu = "30.0.0"` and `naga 30`: it resolves to
  the **same wgpu 30.0.2** we use, so there is one wgpu in the tree. `cargo check` passes. The version gap feared in
  `docs/customization.md` section 3 does not exist today.
- MSRV: with wgpu, librashader follows wgpu's MSRV (README).
- Limits from the README, unchanged: input mipmaps are never generated (no known preset needs them), the preset
  parser is stricter than RetroArch's, the chain ends at a caller-provided texture and viewport (we blit it).
- A `librashader-*` 0.40.0-beta.1 series exists on crates.io: a future major version; the wgpu version it will pin
  is unknown. Keeping wgpu in step with librashader becomes a rule when upgrading either.

**To verify**:

- ~~Cross-compiling the C++ of `librashader-reflect` for Android arm64~~: **done 2026-11-17** (Windows host, NDK
  27.1.12297006, `aarch64-linux-android26-clang` as `CC`/`CXX`/linker, `llvm-ar`): a scratch crate with
  `librashader 0.12.1` (`runtime-wgpu`) + `wgpu 30` builds for `aarch64-linux-android` in 5.5 min at 2 jobs, no
  warnings; `glslang-sys 0.8.1` and `spirv-cross-sys 0.7.4` compile, one wgpu (30.0.2) in the tree. Still untested:
  linking into the app's `.so` and running a preset on the device.
- Runtime cost on the Adreno 618 (A71) of a typical preset (e.g. an LCD-grid shader) at GBA resolution.
- Shaders exist only on the native video path (webview mode shows them disabled with the reason).

**Verdict**: feasible for Phase 1 on desktop; on Android it builds, the runtime cost on the A71 is the open point.

## 2026-11-17 · Google Drive sync

**Question**: can EmuDome sign in to Google and sync saves, states, settings, library index and (optionally) games
between PC and phone, as decided on 2026-11-17?

**Scope: `drive.file`.** Google lists `drive.file` as **non-sensitive** and recommended: only basic OAuth app
verification, no security assessment [D1]. It covers files the app creates and files the user opens with the app
through a picker [D1]. Findings from 3DSync, a 3DS save-sync tool that uses the same scope [D4]:

- grants are stored **per user and per Google Cloud project**, not per token: one Cloud project with a desktop client
  and an Android client should see the same files (two clients in the same project: **to verify**, 3DSync tested
  across projects only);
- **files the user puts in the Drive folder by hand (web, Drive app) are invisible** to the app until picked one by
  one. Consequence for EmuDome: games and saves reach Drive **only through EmuDome's own upload**; the app text must
  say so (design card for Cowork). A full `drive` scope would lift this but is restricted (security assessment):
  not worth it;
- in "Testing" publishing status refresh tokens expire after 7 days: the consent screen must be "In production"
  (same lesson as Rapportino).

**Desktop sign-in (Windows, Linux).** Google's installed-app flow: system browser + **loopback redirect** on a random
local port + **PKCE (S256)**; custom URI schemes and copy/paste are no longer supported; embedded webviews are
forbidden, so the sign-in cannot run in Tauri's webview [D2]. Desktop apps are public clients: the client secret is
optional with PKCE [D2]. The refresh token is stored in the OS credential store (`keyring` crate: Windows Credential
Manager, Secret Service on Linux; **to verify** on a Linux without a keyring daemon).

**Android sign-in.** `AuthorizationClient.authorize()` (Google Identity Services) with the `drive.file` scope
returns an access token directly; on a later call it returns a new one without user interaction while the grant
lasts [D3]. No refresh token on the device (Google advises against storing it there) and no backend needed: the
Kotlin plugin calls `authorize()` when the Rust side needs a token. A Credential Manager sign-in first is optional
(it picks the default account) [D3]. Needs Play Services: devices without them get no Drive sync (**to verify**
fallback; the loopback flow does not apply to Android). The Android OAuth client (#49) is tied to the package name
and the signing certificate: it is registered for `it.domehub.emudome` (the identifier since 2026-11-21, #90) with
the debug keystore's SHA-1, and the release keystore's SHA-1 is added before a release.

**Drive side.** Plain Drive REST v3 from Rust (`reqwest`; generated crates such as `google-drive3` are an option):
resumable uploads for large files (games, 3DS states), `md5Checksum` on every file to compare without downloading,
`appProperties` (private to the app) to carry game id, device, play time and core version, `changes.list` with a
saved page token to learn what changed since the last sync in one call.

**Sync model** (from `docs/decisions.md`): pull before launch, push after exit and periodically; conflict = both
sides changed since the last synced checksum → ask, keep both. The last synced checksum per file lives in a local
sync index (SQLite or JSON, together with the library database).

**Sizes** (estimates from console memory, **to measure** with the headless example per core):

| What | Size |
|---|---|
| In-game saves | GBA up to 128 KB, NDS up to a few MB, 3DS save data/extdata a few MB |
| Save states | GBA under 1 MB; NDS several MB (4 MB main RAM + VRAM); 3DS likely tens to 100+ MB (128 MB FCRAM, compression unknown) |
| Games | GBA up to 32 MB, NDS up to 512 MB, 3DS up to 4 GB and more |

3DS states are the size problem after games: a setting "sync states: all / last only / none" per console is likely.

**To verify with code**: one Cloud project, desktop + Android clients, a file created on one visible on the other;
loopback sign-in from Tauri on Windows and Linux; `authorize()` from the Tauri Android activity; resumable upload of
a 1 GB file with interruption; `changes.list` latency.

**Done in code (2026-11-21, Trello #20, `crates/emudome-sync`)**, tested without Google (an in-memory Drive and a
local mock HTTP server), not wired into the app's game flow yet:
- the installed-app sign-in pieces: PKCE S256 (RFC 7636's test vector), the authorization URL (`drive.file`,
  offline access, consent), the loopback receiver on `127.0.0.1:<port 0>` with the `state` check, the code exchange
  and the refresh (the client secret sent when the build has one: Google's desktop clients still want it with
  PKCE), revocation at sign-out; client id and secret are build-time values. The refresh token is kept in the
  system's credential store (`keyring`: Windows Credential Manager; on Linux the Secret Service over zbus, pure
  Rust: **still to verify** on a Linux desktop, and what happens without a keyring daemon: `cloudCredentialsUnavailable`);
- Drive REST v3 behind a trait: list / find / folders / multipart and resumable uploads (chunks of 8 MiB, resumed
  after a lost chunk by asking the session where it stands) / download / `appProperties` / server-side copy /
  `about` quota and account / `changes.list`;
- the sync index, the conflict rule (exhaustively tested), the «in use» marker, pull before launch and push after
  exit for one game's folder, conflicts settled keeping the other side as a copy, with tests that every interrupted
  pull or push leaves whole files on both sides and converges on the next run;
- TLS: reqwest 0.12 with rustls and ring and the system's roots (reqwest 0.13's rustls pulls a CDLA-licensed crate
  the licence check refuses). Android has no roots that way yet (Phase 4, with its sign-in).

Sizes measured so far (docs/fase0-esito.md): GBA test ROM state 528 KB, `.sav` 128 KB; NDS (melonDS DS) state
18.3 MB; 3DS (Azahar, homebrew) states 544-771 KB, zstd-compressed, so far below the 100 MB feared (a retail game:
to measure). Everything above 5 MiB goes up resumable.

Still to verify against Google (owner's UAT with `drive_probe`): the sign-in end to end, the folder visible on
drive.google.com, a resumable upload of a large file with the network cut halfway, the quota.

**Verdict**: feasible without a server. Main constraint: Drive content comes only from EmuDome uploads.

## 2026-11-17 · Game identity

**Question**: how to recognise the same game on two devices, with different file names, trimmed dumps and patches.

| Console | Identity | Notes |
|---|---|---|
| GB/GBC | SHA-1 of the file | Header (title at 0x134, header checksum 0x14D, global checksum 0x14E) for display and sanity only |
| GBA | SHA-1 of the file | Header game code (0xAC, 4 chars) and version (0xBC) for display and grouping; homebrew often has no code |
| NDS | SHA-1 of the first **used size** bytes | Used size at header 0x80 ("program size without cartridge final padding"); DSi-enhanced titles: 0x210 (including DSi areas) [N1]. Trimming removes only the padding after it, so trimmed and untrimmed dumps hash the same. Game code 0x0C, version 0x1E |
| 3DS | Program/title ID from the NCCH header, plus version | NCSD media ID at 0x108 [N2]; NCCH program ID offset **to verify** on 3dbrew. Region variants have different IDs. Updates and DLC are separate titles. `.cia` files carry the ID in their own metadata (**to verify**) |

- ROM hacks (decided: a different game): the identity is the hash of the patched content, computed after applying
  the patch in memory; metadata keeps the original's identity.
- Display name: GB/GBC/GBA from the file name (headers are short and a ROM hack keeps its base game's, e.g.
  «POKEMON FIRE»; owner's UAT 2026-11-17), NDS banner / 3DS SMDH title, user override later; never the identity.
- A crate exists for NDS headers (`ds-rom` on crates.io [N3]); the GB/GBA/NDS reads are a few lines each.

**Verdict**: feasible, small (S). Hashing a 512 MB NDS file costs about a second on a phone: done once at import
and cached by path + size + modification time.

## 2026-11-17 · Controllers on Android

**Question**: SDL3 on Android (same backend as desktop) or Android's own input events?

- SDL's Android input lives in its Java layer (`SDLActivity`, `SDLControllerManager`). Using it inside another
  app's activity means copying and splitting SDL's Java classes; projects that did it report it as the hardest
  part [A1], and Godot's SDL3 joystick work left Android unmerged because SDL's Activity/View classes do not fit
  its own activity [A2]. Tauri owns our activity: **not recommended**.
- Recommended: read `KeyEvent` / `MotionEvent` from gamepads in the Kotlin plugin, by overriding
  `dispatchKeyEvent` / `dispatchGenericMotionEvent` in Tauri's `MainActivity` (so the WebView does not eat the
  d-pad), and pass button and axis state to Rust over JNI into the same `Router::step`. Android already
  normalises gamepads (`KEYCODE_BUTTON_A/B/X/Y`, `AXIS_X/Y/Z/RZ`, triggers). Rumble and gamepad motion sensors
  through `InputDevice`'s vibrator and sensor managers (Android 12+).
- The mapping model (positional/label, bindings per controller model) is unchanged; the device id becomes
  Android's vendor/product id.

**To verify**: which physical button Android reports as `KEYCODE_BUTTON_A` on Nintendo-layout pads (8BitDo in
Switch mode, Switch Pro) — this decides the positional table on Android; Guide/Home delivery; focus with the
WebView on top.

**Verdict**: feasible (M), as a `emudome-platform` Android input backend.

**Built** (Trello #47, 2026-11-22; not run on a device yet): `emudome_platform::pads::PadSource` is the one
controller source of the app's controller thread, SDL3 on the desktop and Android's input events on Android. The
activity is not edited: the platform library's `GamepadInput.kt` wraps the activity window's `Window.Callback`
(the DecorView asks the window for it at every event), so a gamepad's events are taken before the WebView, and
registers an `InputDeviceListener`. Only gamepad / joystick devices are read (`SOURCE_GAMEPAD` or
`SOURCE_JOYSTICK`, not virtual): the phone's own back key keeps the system bridge's route. Kotlin forwards raw key
codes and axis values; the tables are Rust (`pads/android_map.rs`, tested on every system): `KEYCODE_BUTTON_A` is
read as the bottom button (as SDL's own Android backend reads it), the right stick and the triggers are chosen
from the device's motion ranges (a pair centred on 0 is a stick, a pair from 0 up a pair of triggers). The three
points above are still to verify on a device.

## Brief for Claude Code · hardware rendering spike (`SET_HW_RENDER`)

Owner decision 2026-11-17: a spike right after Phase 0, before the rest of Phase 1, to learn whether 3DS holds up on
phones. Trello card #16. No Nintendo files: use 3DS/NDS homebrew only.

Questions, in order; stop when the answer is clear:

0. **Baseline with what exists**: Azahar libretro with its "Software" graphics API on the PC and on the A71 through
   today's host (software frames). FPS in a homebrew 3D scene. If the core refuses to start without
   `SET_HW_RENDER`, note it. Also check that libretro nightlies ship Azahar and melonDS DS for Android arm64.
1. **GL with readback (cheap path)**: implement `RETRO_ENVIRONMENT_SET_HW_RENDER` for OpenGL (desktop) / GLES 3
   (Android) with a context of our own (WGL/GLX/EGL in `emudome-platform`), the core renders into our FBO, we
   `glReadPixels` into the existing RGBA frame. At 1x (3DS 400x240 + 320x240) the readback is small: this measures
   whether the **emulation itself** is fast enough on the A71, which is the real question. Same for melonDS DS's GL
   renderer (check whether it supports GLES at all).
2. **Zero copy (only if 1 is promising)**: Vulkan context negotiation (`retro_hw_render_interface_vulkan`), sharing
   wgpu's `VkDevice`/queue through `wgpu-hal` (`as_hal`, `create_texture_from_hal`) so the core's image is sampled by
   our compositor without a copy; extensions the core asks for at device creation (`open_with_callback` or
   equivalent in wgpu 30: **to verify**). GL-to-wgpu sharing (external memory) only if Vulkan is not an option.

Report in `docs/fase0-esito.md` (or a new `docs/hw-render.md`), facts only, with FPS per device and renderer.

### 2026-11-17 · Step 0, desk part (no code run)

**Nightlies** (buildbot listings read 2026-11-17): both cores ship for the three platforms.

| Platform | Azahar | melonDS DS |
|---|---|---|
| android/latest/arm64-v8a | `azahar_libretro.so.zip` (no `_android` suffix) | `melondsds_libretro_android.so.zip` |
| windows/x86_64/latest | `azahar_libretro.dll.zip` | `melondsds_libretro.dll.zip` |
| linux/x86_64/latest | `azahar_libretro.so.zip` | `melondsds_libretro.so.zip` |

The fetch scripts guessed `melonds_ds`; fixed to `melondsds`, and the Android script maps Azahar's file name.

**Azahar** (`src/citra_libretro/citra_libretro.cpp`, master): the renderer follows the core option
`citra_graphics_api` = `auto` | `vulkan` | `opengl` | `software` (default `auto`). OpenGL asks for
`OPENGL_CORE` 4.3 on desktop and `OPENGLES3` 3.2 on Android; Vulkan asks for `VULKAN` 1.1 with context negotiation
and `GET_HW_RENDER_INTERFACE`; **Software never calls `SET_HW_RENDER`**. `auto` is always Vulkan on Android and
asks `GET_PREFERRED_HW_RENDER` elsewhere. A refused `SET_HW_RENDER` makes the load fail. **Consequence**: our host
answers `GET_VARIABLE` with `false`, so Azahar runs `auto` and fails to load everywhere; step 0 needs
`GET_VARIABLE` to answer `citra_graphics_api=Software` (the first core option the settings engine must feed).

**melonDS DS** (JesseTG/melonds-ds, main): software is the default renderer, said to fall back to software when GL
is unavailable (fallback code not read: **to verify**). Its GL renderer asks for `OPENGL_CORE` 3.2 (4.3 for
"Compute"); OpenGL is off by default in Android builds, so **no GLES and no Vulkan**: on phones melonDS DS is
software only.

**Prior art**: `retro-rs` (Rust) implements step 1 with a surfman GL context, the core drawing into the surface's
FBO and `glReadPixels` back to RGBA. No Rust frontend with `SET_HW_RENDER` on wgpu was found.

**Order from here**: (0) answer `citra_graphics_api=software`, measure Azahar software and melonDS DS on the PC and
the A71; (1) GL readback (desktop GL 4.3, Android GLES 3.2), useful for Azahar only; (2) if GLES readback is slow on
the A71, go to Vulkan negotiation, which Azahar already supports in full.

### 2026-11-17 · Step 0, PC measurements (Windows)

Host: `GET_VARIABLE` now answers core options (`crates/emudome-core/src/core_options.rs`). The exact value is
**`citra_graphics_api` = `Software`**, capital S: Azahar's definition lists `Auto`, `Vulkan`, `OpenGL`, `Software`
and compares case-sensitively; anything else (including its own declared default `auto`) means auto. Without the option
Azahar's `retro_load_game` returns false (seen); with it the core starts and never calls `SET_HW_RENDER`.
Both cores ask `GET_CORE_OPTIONS_VERSION` and register their options with `SET_CORE_OPTIONS_V2`.

AMD Ryzen 7 9700X, release build of the windowless `headless` example, software frames, nightly cores of 2026-11-17.

| Core | Renderer | Content | Started | 1x | Max (unthrottled) |
|---|---|---|---|---|---|
| melonDS DS | software (default) | Breaking Bad DS 1.0.6, 3D lab scene | yes, built-in firmware + FreeBIOS | 59.8 fps (target 59.83) | 222 fps (3D scene), 541 fps (2D intro) |
| Azahar 2127.0-rc3 | Software (forced) | Craftus Reloaded 0.3, title screen (citro3d) | yes | 17.4 fps over 10 s (target 60) | 16 fps |

Azahar's software renderer runs at about a quarter of full speed on a fast desktop CPU, on a title screen only (the
menu needs the touch screen, which the host does not feed, so no in-world scene was measured). Software is therefore
not a usable 3DS path even on the PC, let alone the A71: step 1 (GL readback) is needed to learn anything about phones.
melonDS DS in software has ample headroom on the PC. A71 measurements: not done (no phone access in this session).

## Hardware rendering (`SET_HW_RENDER`)

**2026-11-19**: OpenGL (melonDS DS, Azahar), GL readback vs GL-to-Vulkan zero copy, resolution scaling and the
recommendation per OS are in `docs/feasibility-hw-render.md`, which now carries the whole answer to #16.

**2026-11-18 · Step 1, done with Vulkan instead of GL (Windows, headless).** Trello #16, for #46.

**Question**: can EmuDome's host give a libretro core a hardware render context and present the core's frames through
our wgpu renderer?

**Answer: yes for Vulkan, verified on Windows with Azahar.** The core renders on a Vulkan device shared with wgpu, wgpu
samples the core's image directly (no copy), and the picture is correct. OpenGL was not built: Azahar does not need it
(Vulkan is its renderer on every platform, Android included) and melonDS DS falls back to software when GL is refused.

**How**: a spike crate, `crates/hw-render-spike` (binary, `publish = false`), with its own minimal libretro host so the
shell's host is untouched. It is its own Cargo workspace, outside the root one (Trello #72): run it from the repo root
with `cargo run --release --manifest-path crates/hw-render-spike/Cargo.toml -- <core> <content> --opt key=value
[--frames N] [--png out.png] [--core-log] [--adapter NAME]`. Windowless: no window, no surface. The generic
`SET_HW_RENDER` types and constants were added to `crates/libretro-sys`; the Vulkan structs of `libretro_vulkan.h`
are in the spike (`vk_iface.rs`, with ash's handle types). Content: the same homebrew as step 0 (Craftus Reloaded 0.3,
MIT; Breaking Bad DS 1.0.6, Apache-2.0). No Nintendo files.

The path, as implemented (`present.rs`):

1. Azahar, with `citra_graphics_api=Vulkan`, calls `SET_HW_RENDER` (context type 6 = Vulkan, version 1.1) and
   `SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE` (version 2, but only `create_device` is set: no `create_instance`, no
   `create_device2`) inside `retro_load_game`, which returns at once: the game itself loads in `context_reset`.
2. The host creates a wgpu Vulkan instance and adapter, takes the raw handles with `as_hal::<Vulkan>()`, and calls the
   core's `create_device` with wgpu's instance and GPU, wgpu's required device extensions
   (`Adapter::required_device_extensions`) and wgpu's Vulkan 1.0 features (`physical_device_features(..).get_core()`).
   Azahar creates the `VkDevice` (adding its own extensions) and leaves it to the frontend (`destroy_device` is null).
3. wgpu wraps that device: `wgpu_hal::vulkan::Adapter::device_from_raw` (ownership to wgpu-hal) then
   `wgpu::Adapter::create_device_from_hal`. One device, one `VkQueue`, shared.
4. The host answers `GET_HW_RENDER_INTERFACE` with a `retro_hw_render_interface_vulkan` (v5) built from those handles,
   then calls `context_reset`.
5. Every `retro_run`, Azahar submits its own command buffers (under `lock_queue`), calls `set_image` with one output image
   (400x480, `R8G8B8A8_UNORM`, `SHADER_READ_ONLY_OPTIMAL`, no semaphores, the same `VkImage` every frame) and
   `video_refresh(RETRO_HW_FRAME_BUFFER_VALID, 400, 480)`.
6. The host wraps that `VkImage` once as a wgpu texture (`texture_from_raw` with a no-op drop callback and
   `TextureMemory::External`, then `create_texture_from_hal` with initial state `RESOURCE`), records one raw image barrier
   (core's colour/transfer writes to our fragment-shader reads: Azahar's render pass has no outgoing dependency and wgpu
   inserts nothing for a read after a read), and draws it with an ordinary wgpu render pass. The barrier goes in its own
   encoder: wgpu panics when raw (`as_hal_mut`) and wgpu commands are mixed in one encoder ("Mixing the wgpu encoding API
   with the raw encoding API is not permitted", seen). wgpu's `queue.submit` is wrapped in the same lock the core takes.
7. Optionally the result is copied to a buffer and mapped (readback), to check the pixels and measure the cost.

**Measured** (AMD Ryzen 7 9700X, NVIDIA GeForce RTX 3060, driver 617.42, Vulkan 1.4; Azahar 2127.0-rc4 and melonDS DS
1.4.0 nightlies of 2026-11-18; release build; unthrottled; Craftus title screen, the only 3DS scene reachable without
touch input):

| Run | Result |
| --- | --- |
| Azahar Vulkan: device creation by the core / `context_reset` (game load), 7 runs | 143-204 ms (699 ms on the very first run) / 178-474 ms (441 ms on the first) |
| Azahar Vulkan, `retro_run` + wgpu draw of the core image + wait for the GPU, each frame (4 runs of 300-600 frames) | 1590-2823 fps (0.35-0.63 ms per frame); the draw + submit + wait part alone 0.12-0.15 ms |
| Same + readback of the 400x480 frame to the CPU | 1196-2447 fps; draw + readback 0.17-0.34 ms per frame |
| Read-back picture | correct title screen (PNG checked by eye), 140 colours, 600 distinct frames out of 600 (the picture is live) |
| Azahar Software, same tool, same scene | 28.4 fps over 300 frames (35 ms per `retro_run`); 36.5 fps over 120 frames |
| Azahar with `citra_graphics_api=OpenGL`, host refusing GL | `SET_HW_RENDER` type 3 (OpenGL core 4.3) refused, `retro_load_game` fails ("Failed to set HW renderer") |
| melonDS DS with `melonds_render_mode=opengl`, host refusing GL | asks for type 3 (OpenGL core 3.2), loads anyway in software: 626 fps, same as its default |
| Teardown | clean: our image wrapper dropped, then `retro_unload_game` / `retro_deinit`, then the device |

"Run only" timings (no frontend wait) swung between 3100 and 11800 fps across runs: Azahar's submissions are
asynchronous, so they are not a measure of the frame; the rows above with a GPU wait are. The title screen keeps the 3DS
CPU mostly idle, so these numbers say the **render path and its interop cost** are negligible (well under 1 ms per frame);
they say nothing about emulation speed in a real scene. The Software figure differs from step 0's 17.4 fps (different
nightly, rc3 vs rc4, and a different host); the comparison that holds is inside this tool: Vulkan is two orders of
magnitude faster than Software on this scene.

Also verified: the spike type-checks for `aarch64-linux-android` (`cargo check --target aarch64-linux-android -p
hw-render-spike`, NDK 27.1; today `--manifest-path crates/hw-render-spike/Cargo.toml` instead of `-p`): the code has nothing Windows-specific.

**What does not work or is not handled** (each one stops the spike with a message rather than guessing):

- OpenGL / GLES contexts: not built. The host refuses them (Azahar then fails to load, melonDS DS falls back to software).
- `set_command_buffers`, `set_signal_semaphore`, semaphores passed to `set_image`, an image owned by another queue
  family, more than one sync index: Azahar uses none of them.
- Wrapping assumes the image's size equals the size given to `video_refresh` (true for Azahar; the image extent is not
  in `retro_vulkan_image`).
- No integrated GPU could be tested: the machine's AMD iGPU is not exposed to Vulkan (wgpu lists only the RTX 3060).

**Recommended architecture for Phase 2/3**:

- **3DS: Vulkan everywhere** (Windows, Linux, Android), never GL. The shell offers Azahar only `Vulkan` or `Software` for
  `citra_graphics_api` (any other value makes it ask for a context the shell refuses).
- **The core's device is the compositor's device.** `video.rs` keeps the wgpu instance, adapter and surface; for a
  hardware session the device comes from the core's `create_device` (as in the spike) and the compositor's pipeline is
  rebuilt on it; for software cores wgpu's own device stays. The surface works with any device of the same adapter;
  `VK_KHR_swapchain` is already in the extensions passed to the core (it was in wgpu's list).
- **Zero copy on the native path**: the compositor samples the wrapped core image in the pass that draws the game today
  (the barrier from step 6 first). Readback only where the CPU needs pixels: the webview video path (Linux uses it),
  slot thumbnails, screenshots. Measured cost at 1x is a fraction of a millisecond.
- **Threading**: the core stays on the emulation thread, the compositor on its render thread; both use one `VkQueue`
  (Azahar creates exactly one), so every wgpu `submit` **and surface `present`** take the interface's queue lock.
  Sync: two sync indices with a fence each, `wait_sync_index` waiting on the compositor's fence for that index. Queue order
  plus the barrier keeps frames whole: the compositor shows whichever finished frame precedes its submission.
- **In code**: the environment side (`SET_HW_RENDER`, negotiation, `GET_HW_RENDER_INTERFACE`, the interface callbacks) in
  `emudome-core` as plain FFI with no wgpu dependency; the device creation and wrapping next to `video.rs` in the app.
  No `#[cfg(target_os)]` is needed (none in the spike).
- **NDS**: melonDS DS stays software (fast enough on the PC, and its GL renderer has no GLES or Vulkan version). GL
  upscaling for melonDS DS on desktop would need a GL context of our own (WGL/GLX/EGL in `emudome-platform`) plus readback
  or GL/Vulkan memory sharing: not built, not measured.

**Open risks** (to verify before Phase 3 relies on this):

1. **Device features**: Azahar's `create_device` (v1) enables only the Vulkan 1.0 features and the extensions it is
   handed; the feature structs wgpu chains on `pNext` (timeline semaphore, robustness2, image robustness, descriptor
   indexing...) are not enabled, while wgpu's private capabilities come from what the adapter supports, so wgpu may use
   timeline semaphores on a device where the feature is off. It ran correctly on NVIDIA, but **no validation layer is
   installed on this machine**, so the run is not validated. Fixes, in order: run under `VK_LAYER_KHRONOS_validation`;
   if it complains, ask wgpu for fewer capabilities or create the device in the frontend with
   `open_with_callback` plus Azahar's extensions and features (bypassing `create_device`), or get `create_device2`
   into Azahar upstream.
2. **Robustness**: wgpu's Vulkan 1.0 features include `robustBufferAccess` = 1, and Azahar merges what it is given, so its
   device runs with robust buffer access (possible cost on the GPU; the libretro header asks frontends not to add
   robustness features in `create_device2`). Not measured.
3. **Phones**: not run on the A71. Azahar's Android core asks for Vulkan too, so the same path applies; whether
   3DS emulation is fast enough on the phone is still the open question of #16.
4. **Real scenes**: only a title screen; CPU-heavy 3DS scenes, save states with a hardware context, resolution scaling
   (`citra_resolution_factor`: bigger images, bigger readback) and `context_destroy` / re-creation were not tested.
5. **Queue sharing under load**: the lock was exercised on one thread only (the spike submits and runs the core on the
   same thread); contention with a render thread is untested.

**Next step**: run the same spike binary on the A71 (`aarch64-linux-android`, pushed with `adb` to `/data/local/tmp`,
windowless Vulkan) with Azahar's Android core and Craftus, to get the phone's 3DS numbers; in the same pass, run the PC
case once under the Khronos validation layer.

## Sources

- [D1] Google, Choose Google Drive API scopes: https://developers.google.com/workspace/drive/api/guides/api-specific-auth
- [D2] Google, OAuth 2.0 for desktop apps: https://developers.google.com/accounts/docs/OAuth2InstalledApp
- [D3] Android, Authorize access to Google user data: https://developer.android.com/identity/authorization
- [D4] 3DSync issue #67, drive.file findings: https://github.com/vllni/3DSync/issues/67
- [N1] Ekona, DS cartridge header: https://scenegate.github.io/Ekona/docs/specs/cartridge/header.html
- [N2] 3dbrew, NCSD: https://www.3dbrew.org/wiki/NCSD
- [N3] ds-rom crate: https://docs.rs/ds-rom
- [A1] SDL forum, SDL for controllers only on Android: https://discourse.libsdl.org/t/adding-sdl-support-for-controllers-only-on-android/27358
- [A2] Godot PR #109645, SDL3 joystick driver: https://github.com/godotengine/godot/pull/109645
- librashader README (crate 0.12.1, read from the downloaded source)
