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
fallback; the loopback flow does not apply to Android).

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
