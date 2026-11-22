# Customization: how far can EmuDome go

Feasibility study, 2026-11-17. It answers, for every area the user could customize: **what**, **where it is
stored** (format and settings level), **how it is edited**, **how it works in our stack**, **what it costs**
(S = days, M = one to three weeks, L = a month or more), **what limits the cores and platforms impose**, and
**prior art**. It is not a visual design proposal: where a design decision is needed it is listed as a question
for Cowork (section 12).

Conventions:

- Levels follow the Phase 1 settings engine: **Default → Global → Console → Game → Session**, plus the
  **per-device axis** (see section 10 for what "device" means in each area).
- "Today" means the code on `develop` at 8775c17. Facts not confirmed in a reliable source are marked
  **to verify**. Links to prior art are in section 13; numbers in square brackets point there.

## 0. Where we start from (what exists today)

| Area | Today | Where |
|---|---|---|
| Touch controls | Since Trello #29: layouts per console and orientation (defaults, the user's own from the C06 editor kept per device in `touch.tedit`), tap / hold / turbo, the 3DS Circle Pad, three built-in skins, `touch.*` settings, E14. Before it: a fixed overlay in CSS, no settings | `app/src/components/touch/`, `crates/emudome-core/src/touch.rs` |
| Controller routing | Pure `Router::step`; `Bindings { modifier, combos }` with `DEFAULT_MODIFIER = L3` and `DEFAULT_COMBOS`; Guide toggles the menu; menu navigation with repeat | `crates/emudome-core/src/hotkeys.rs`, `docs/combo-tasti.md` |
| Controller mapping | SDL3, positional RetroArch-style (right face button = A). Mask merged from keyboard / gamepad / touch | `app/src-tauri/src/gamepad.rs`, `crates/emudome-core/src/input.rs` |
| Keyboard | The webview forwards raw keys (`key` command); Rust routes them: fixed shortcuts and the game keys of `pad.remap` | `crates/emudome-core/src/keyboard.rs`, `app/src-tauri/src/hotkeys.rs` |
| Video | Two paths: native wgpu under a transparent webview, or canvas via `emu://frame`. Software frames only (RGBA copy) | `app/src-tauri/src/video.rs`, `NATIVE_VIDEO_DEFAULT` in `emudome-platform` |
| Audio | cpal, linear resampling, ±0.5 % queue correction, muted when speed ≠ 1× | `app/src-tauri/src/audio.rs` |
| Core options | `GET_VARIABLE` → `false`; `SET_VARIABLES` accepted and ignored. No `SET_CORE_OPTIONS_V2`, no `SET_HW_RENDER`, no pointer, no rumble, no sensors | `crates/emudome-core/src/callbacks.rs` |
| Saves | `.sav` every 30 s, on exit and when Android goes to the background; 3 slots in the menu, 1–9 from the controller; all in `app_data/saves`, `app_data/system` | `crates/emudome-core/src/session.rs`, `app/src-tauri/src/lib.rs` |
| Interface | vue-i18n (en, it), language from the OS; design tokens in `styles.css`; no theme switch | `app/src/i18n/`, `app/src/styles.css` |
| Settings | **None persisted.** Only env vars (`EMUDOME_VIDEO`, `EMUDOME_CORES_DIR`, `RUST_LOG`) | — |

Two facts from the code shape everything below:

1. **There is no settings store yet.** Every item in this document is a consumer of the Phase 1 engine; the
   engine is the first thing to build, and its data model (section 10) must already fit touch layouts and
   bindings, which are not scalar values.
2. **The host accepts only software-rendered frames.** melonDS DS's OpenGL renderers and Azahar's
   Vulkan/OpenGL renderers need `RETRO_ENVIRONMENT_SET_HW_RENDER` [C1]. Without it NDS runs on the software
   renderer (fine, it is melonDS DS's default) but **3DS is limited to Azahar's "Software" graphics API**, which
   is too slow for most games on phones (**to verify** with a benchmark). Internal resolution, texture filters and
   upscaling options for NDS/3DS (section 5) exist only with hardware rendering. This is a Phase 2/3 item, but it
   decides how much "video customization" is real for those consoles.

---

## 1. Touch controls

### What the user could customize

- **Per control:** position, size, opacity, shape (circle, rounded rect, pill), skin/image, label, visibility,
  hit area beyond the drawn shape ("extended edges"), z-order when hit areas overlap.
- **Per layout:** global opacity, auto-hide after N seconds of no touch (PPSSPP does it [P1]), "show only when
  no controller is connected", haptic strength, and which game screen area it can cover.
- **Variants:** portrait and landscape; per console (GBA, NDS, 3DS have different buttons); per game (e.g. a
  game that needs only A); per device (phone vs tablet vs foldable: a layout drawn for 20:9 does not fit 4:3).
- **Extra controls** (all optional, added from a palette):

| Control | What it does | Notes |
|---|---|---|
| Turbo button | Repeats a button at N Hz while held, or toggles turbo on a button | Runs in the shell's input layer, not in the core |
| Toggle / latch | Press once = held until pressed again | E.g. "hold B to run" |
| Multi-button | One touch presses several buttons (A+B, L+R) | RetroArch `a|b` descriptors [P2], Delta's input lists [P3] |
| Macro / sequence | Timed sequence of presses ("down, down-right, right, A") | Needs a per-frame scheduler in the input layer; frame-exact only if fed from the emulation thread |
| Hotkey buttons | Save/load state, next slot, fast forward (hold or toggle), slow motion, pause, screenshot, swap screens, next layout, menu | Same `Hotkey` enum as the controller router: one action set for all input sources |
| Analog stick | Circle pad (3DS), C-stick (New 3DS), floating or fixed centre, deadzone, "stick as d-pad" | Needs `RETRO_DEVICE_ANALOG` in the host (missing today) |
| Touch-screen passthrough | The NDS/3DS bottom screen itself is the touch surface | Needs `RETRO_DEVICE_POINTER` (missing today), see below |
| Microphone button (NDS) | Blow/noise/real mic while held | melonDS DS has `melonds_mic_input` (Silence, Blow, Noise, Microphone) and a Hold/Toggle/Always button mode [C3] |
| Gyro toggle (3DS) | Enables the phone's gyroscope as 3DS motion | See section 2 |

### How it is stored

- A **layout** is a document, not a setting: a JSON file (`*.layout.json`) with controls in **normalized
  coordinates** relative to a reference area (safe-area-inset viewport), each control `{ id, kind, inputs[],
  x, y, w, h, shape, skin?, opacity?, extendedEdges?, behaviour }`. Normalized coordinates are what RetroArch
  overlays use (`overlayN_normalized = true`, 0..1 [P2]) and what makes one layout survive different resolutions.
- A layout declares the **game screen frames** it expects (like Delta's `screens` with `inputFrame` /
  `outputFrame` [P3]): this is how a layout and the NDS/3DS screen layout (section 3) stay consistent.
- The **settings** only reference layouts: `touch.layout.portrait = "builtin:gba-default"` at Console level,
  overridable at Game level, and resolved on the **device axis by form-factor bucket** (phone-tall, phone-wide,
  tablet, foldable-open; the bucket is computed from the viewport aspect ratio, not from the model name).
- Scalars that are not per layout (global opacity, haptics, auto-hide) are ordinary settings at Global level.
- **Skin packs**: a zip (`*.emudome-skin`) with `manifest.json` (id, name, author, licence, consoles,
  orientations, form factors), the layout JSON files and images (SVG or PNG). Same idea as `.deltaskin` [P3];
  we do not read `.deltaskin` or RetroArch `.cfg` directly in Phase 1 (an importer is possible later, both
  formats are documented [P2][P3]).

### How it is edited

- **In-app layout editor** (Android first, also usable on desktop with a mouse for testing): enter from the
  quick menu → "Edit controls". The game stays paused under the editor, so placement is judged on the real
  screen.
- Editor features, cheapest first: drag; resize (handle or pinch); per-control opacity; snap to a grid and to
  other controls' edges/centres; symmetric mirror (move the d-pad, the face buttons mirror); safe-area guides
  (notch, rounded corners, gesture bar) from `env(safe-area-inset-*)`; undo/redo (a command stack over the
  layout document, cheap because the document is small); "reset to the console default"; duplicate a layout to
  the other orientation; add/remove controls from a palette.
- **Import/export/share**: export a layout or skin pack as a file (Android share sheet, desktop save dialog);
  import by opening the file. A text code (base64 of the JSON) is a cheap extra for sharing in chats.

### Feasibility in our stack

- The overlay is already Vue in the webview over the native surface; making it data-driven is a rewrite of
  `TouchControls.vue` into a renderer of the layout document plus a control component per `kind`. Multi-touch
  already works (pointer capture per control). **Feasible, no platform risk.**
- **Touch passthrough** (NDS/3DS bottom screen): the touch lands in the webview, so the overlay must forward
  pointer events on the screen rectangle to Rust, which maps them to the core's `RETRO_DEVICE_POINTER`
  coordinates (−0x7fff..0x7fff over the **whole frame the core outputs** [C1]). Requirements:
  - the host must answer `input_state(port, RETRO_DEVICE_POINTER, …)` (missing today);
  - the shell must know where the bottom screen is inside the frame. If the core composes the layout (melonDS DS
    and Azahar do, section 3), the shell must mirror the core's layout maths; if the shell composes (option B in
    section 3), it knows by construction. This is the strongest argument for shell-side screen layout.
  - melonDS DS has `melonds_touch_mode` (Joystick / Pointer / Auto) [C3]; Azahar has
    `citra_enable_touch_touchscreen` and `citra_enable_mouse_touchscreen` [C4].
- **Haptics**: `@tauri-apps/plugin-haptics` (Android/iOS) offers `vibrate(ms)` and `impactFeedback(style)` [T1].
  Its docs warn that cheap phones may not vibrate correctly [T1]. Latency from the webview → plugin bridge is
  probably fine for a "click" but **to verify**. Desktop: no-op.
- **Turbo, macros, toggles** live in Rust next to the `Router`: the overlay sends "control X pressed/released",
  Rust turns it into button bits per frame. Keeping it in Rust means the same macro works from a controller.
- Performance: a few dozen absolutely positioned elements; no risk. Images as SVG keep sharp at any size
  (icons must follow the global stroke-icon rule: opaque strokes, fade the container).

### Effort

| Piece | Size |
|---|---|
| Data-driven overlay + layout JSON + built-in defaults (GBA, NDS, 3DS × portrait/landscape) | M |
| Editor: drag, resize, snap, reset, undo, per-control opacity | M |
| Form-factor buckets, safe areas, per-orientation | S |
| Turbo / toggle / multi-button / hotkey buttons | S |
| Macros with a timeline editor | M |
| Analog stick control + `RETRO_DEVICE_ANALOG` in the host | S |
| Touch passthrough + `RETRO_DEVICE_POINTER` | S (shell-side layout) / M (mirroring core layouts) |
| Skin packs (zip, manifest, import/export, share) | M |
| Importers for `.deltaskin` / RetroArch overlay `.cfg` | M each, later |

### Risks and limits

- **Licensing of skins**: user skins may contain Nintendo artwork (button glyphs, console photos). We never ship
  such art; a third-party pack is the user's responsibility. Built-in skins are our own.
- A layout drawn for one aspect ratio looks wrong on another: normalized coordinates plus "anchors" (each
  control anchored to a corner or edge) are needed, otherwise controls drift on wide screens. **Anchoring is the
  decision that makes or breaks portability** of shared layouts.
- Macros that run while the game is paused or fast-forwarded must be defined in frames, not milliseconds.
- RetroAchievements hardcore mode (section 8) forbids some of this (turbo is usually allowed, macros: **to
  verify** against the RA rules).

### Prior art

RetroArch overlays: file-only, normalized descriptors, multi-button, `analog_left`, `dpad_area`, per-button
images, `next_target` to switch overlays; in-app editing has been a long-standing user request [P2][P4].
Delta `.deltaskin`: zip + `info.json`, per device (`iphone`/`ipad`, `standard`/`edgeToEdge`/`splitView`) and
per orientation, `items` with `inputs` (combos and `quickSave`, `fastForward`, `toggleFastForward`), `screens`
with `inputFrame`/`outputFrame` that split the NDS frame in two, `extendedEdges`, `translucent`, `debug` [P3].
PPSSPP: in-app drag editor with reset, style, opacity, auto-hide, gesture mapping on the empty background, custom
combo buttons with shapes [P1][P5]. Lemuroid: auto-sized radial pad, customization limited to size and vertical
position [P6]. AetherSX2/NetherSX2 and Azahar Android: in-app drag editors (**to verify** feature lists).

---

## 2. Physical controllers and keyboard

### What the user could customize

- **Mapping**: physical button → RetroPad button, per console (the RetroPad is the common language, see below),
  per game, per controller.
- **Positional vs label** mapping (owner decided: customizable). Positional = the right face button is A (what
  we do now, RetroArch style); label = the button printed "A" is A (Xbox layout puts A at the bottom).
- **Combo modifier** (L3 by default) and **the hotkey table** (`Bindings`), including disabling combos.
- **Turbo**: per button, rate in Hz, hold or toggle.
- **Analog**: deadzone (inner/outer), sensitivity curve, "left stick as d-pad" threshold (today: half travel),
  trigger threshold (today: half travel), stick-to-circle-pad for 3DS, right stick as C-stick or as touch
  pointer (Azahar's `citra_analog_function` [C4]; melonDS DS joystick cursor speed/deadzone/curve [C3]).
- **Gyro/motion** (3DS games, Boktai-style sensors): source = controller gyro, phone gyro, or a stick; sensitivity.
- **Rumble**: on/off, strength (melonDS DS has `melonds_rumble_intensity` and motor choice for the Rumble Pak [C3]).
- **Multiple players**: port assignment (who is player 1). GBA/NDS/3DS are single-player per instance, so this
  matters only for link/netplay (section 8): low priority.
- **Keyboard mapping**: same model as a controller; keyboard is just another device.
- **Menu navigation** remapping (accessibility, section 7).

### How it is stored

- **Bindings document** per device class, sparse: `{ device: <id>, map: { "a": ["east"], "b": ["south"], … },
  modifier: "l3", combos: { "save_state": "r1", … }, turbo: { "a": { "hz": 15, "mode": "hold" } }, analog: {…} }`.
- **Device identity**: SDL3 gives a GUID (vendor/product/version) per model; the per-device axis keys on the
  **model GUID** (all 8BitDo Pro 2 in Xbox mode share one profile), with an optional per-unit override (serial
  where SDL exposes it; **to verify** availability). A "generic" profile is the fallback, built from SDL's
  standard gamepad layout, so an unknown pad works without setup.
- Levels: Default (code, today's `DEFAULT_COMBOS`) → Global → Console (NDS adds Y/X, 3DS adds ZL/ZR/C-stick)
  → Game. Session overrides are not useful for mappings and can be skipped.
- Merge rule: **per action**, not per table: a Game-level file that remaps only A keeps the Global Start/Select.
  The Default level is the code table, so updating EmuDome's defaults reaches users who changed one button.
- Positional/label: one Global setting `input.face_layout = positional | label`, overridable per device
  (Nintendo-layout pads like the 8BitDo in Switch mode may want "label").

### How it is edited

- **Button mapping screen** (UI generated from the schema): a list of RetroPad buttons for the current console,
  "press a button" capture, conflict warnings (two actions on one button), reset per button and per page, and a
  live tester (which bits reach the game, which combos fire). Works with the controller itself (the router
  already has a UI-input mode).
- Hotkey table: same screen, second tab; shows which combos collide with Steam/Game Bar when the modifier is
  Guide (`docs/combo-tasti.md`, section 3).
- Import/export: the bindings document as a JSON file; in Phase 1 file-only is acceptable for advanced options
  (curves, per-unit overrides).

### Feasibility in our stack

- The `Router` already separates "buttons now" from "game mask" and takes a `Bindings`; extending `Bindings`
  with the map, turbo and analog settings is local to `emudome-core` and stays testable without SDL. **Feasible.**
- **Label mapping**: SDL3 has `SDL_GetGamepadButtonLabel`, which reports the label (A/B/X/Y, Cross/Circle…) of
  the face buttons for known models [S1]. Positional uses `SDL_GAMEPAD_BUTTON_SOUTH/EAST/…`. Both are a lookup
  table in `gamepad.rs`; unknown controllers fall back to positional.
- **Gyro**: SDL3 exposes gamepad sensors (`SDL_SetGamepadSensorEnabled`, `SDL_SENSOR_GYRO/ACCEL`) for DualShock 4,
  DualSense, Switch Pro, Steam Controller and others [S1] (per-model list **to verify**). The phone's own gyro
  needs Android sensor APIs in `emudome-platform/src/android.rs`. The core gets it through
  `RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE` (missing today); Azahar has `citra_enable_motion` and
  `citra_motion_sensitivity` [C4].
- **Rumble**: `RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE` → `SDL_RumbleGamepad`; phone vibration via the haptics
  plugin. S.
- **Android controllers**: SDL3 is not built for Android yet (fase0-esito, 21:53). Either build SDL3 for Android
  (SDL has an Android Java layer: integration with Tauri's activity **to verify**) or read `KeyEvent`/`MotionEvent`
  in the Kotlin plugin and pass them to Rust. The mapping model is the same either way; the input backend is a
  `emudome-platform` concern.
- **Keyboard**: today the webview receives the keys and forwards them raw to Rust (`key` command, routed in
  `emudome_core::keyboard`), so it still loses keys while focus is elsewhere and cannot see raw scancodes reliably. Moving keyboard capture to Rust (window events) is **to verify** with
  Tauri's child-webview setup; the mapping model does not depend on it.
- **Steam Input** can remap a pad before we see it; we cannot detect a Steam remap. Document it, do not fight it.

### Effort

| Piece | Size |
|---|---|
| Bindings document + per-action merge + device GUID axis | M |
| Mapping UI with capture, conflicts, tester (schema-generated) | M |
| Positional/label switch | S |
| Turbo per button | S |
| Deadzones/curves/thresholds | S |
| Rumble | S |
| Gyro (controller via SDL3) | M |
| Gyro (phone, Android sensors) | M |
| Android controller backend | M (separate stream, Phase 0 step 4 / Phase 4) |

### Risks and limits

- Label mapping is only as good as SDL's controller database; third-party pads in "XInput mode" report Xbox
  labels whatever is printed on them.
- The Guide button may never reach us (Steam, Game Bar, Android `KEYCODE_HOME`) — already handled by L3 + Start.
- The modifier being remappable means a user can pick a button the game needs; the UI must warn (Select is a
  game button, L3 is not).

### Prior art

RetroArch: per-core/per-game remap files (`.rmp`), autoconfig profiles per controller model, hotkey enable button
[R1][R2]. DuckStation: input profiles in `inputprofiles/`, per-game settings in `gamesettings/` that can either
pick a profile or carry their own bindings [P7][P8]. Dolphin: per-game GameINI files named by game ID, user
copies override shipped defaults [P9]. Azahar: controller shortcuts with up to two inputs, scoped per controller,
per controller type or for all (PR #1717, see `docs/combo-tasti.md`). Steam Input: per-game configurations and
Guide chords (see `docs/combo-tasti.md`).

---

## 3. Video

### What the user could customize

- **Screen layout (NDS/3DS)**: stacked (top/bottom, bottom/top), side by side (either order), single screen
  (top or bottom), large + small ("focus"), hybrid (one large plus both small), rotation (left, right, upside
  down — book-style games), gap between screens, size ratio of the small screen, swap screens (toggle or hold),
  a **list of layouts to cycle** with a hotkey (L3 + R3 in the combo table).
- **Custom layout**: each screen as a free rectangle (Azahar standalone has `custom_layout` with
  `custom_top_x/y/width/height` and `custom_bottom_*` [C5]; Delta's `outputFrame` [P3]).
- **Scaling**: fit, integer scale, stretch, aspect ratio (native, 4:3, 16:9, custom), position (centre, top in
  portrait to leave room for touch controls), background colour or blurred fill around the game (the global "no
  empty space" rule makes this relevant).
- **Shaders**: a librashader preset (`.slangp`) per console/game; parameters of the preset; LCD/colour
  correction (mGBA has `mgba_color_correction` and `mgba_interframe_blending` [C2]).
- **Filters**: nearest / linear / sharp-bilinear on the final scale; melonDS DS has per-screen filtering for
  hybrid layouts [C3].
- **Internal resolution / renderer** (NDS/3DS only, needs hardware rendering, section 0): melonDS DS 1×–8×
  OpenGL [C3]; Azahar 1×–18×, texture filters, custom textures [C4].
- **Window**: windowed/fullscreen/borderless, remember size and position, start fullscreen; Android: immersive
  mode, orientation lock per console/game.
- **Video path**: native vs webview (today's selector), and backend (Vulkan/DX12/GL) as an advanced option.
- **Frame pacing**: vsync, a frame delay option to cut latency (**to verify** usefulness).

### How it is stored

- Scalars (`video.scale_mode`, `video.aspect`, `video.filter`, `video.shader_preset`, `video.path`,
  `video.backend`) at Global → Console → Game; Session for "try a shader now without saving".
- Screen layout: `video.screens.cycle = ["top_bottom", "side_by_side", "hybrid_top", …]` at Console/Game,
  plus `video.screens.custom` (rectangles, normalized) — and, on Android, **per orientation** and on the
  **form-factor axis** (same buckets as touch layouts; a touch layout may embed its own screen frames, which then
  win).
- Backend and video path: **per device = per machine/GPU** (a Global value stored in a machine-local file, not
  synced, because a GPU-specific choice must not travel to another PC).
- Shader presets: files in `shaders/` (user-added slang presets), referenced by path.

### How it is edited

- Scalars: schema-generated UI.
- Screen layout: a **layout picker** in the quick menu (cycle list) and, later, a **screen editor** that reuses
  the touch editor's canvas to place the two screen rectangles (same drag/snap/undo code).
- Shaders: a **preset picker** (list + live preview on the running game) and a parameter page generated from the
  preset's `#pragma parameter` list (librashader parses them [V1]).

### Feasibility in our stack

- **Where to compose NDS/3DS screens — the key choice:**
  - *A. Let the core compose*: melonDS DS draws its own layouts (`melonds_screen_layout1..8`, up to 8 in the cycle,
    hybrid ratio, gap up to 126 px, secondary scale [C3]); Azahar draws `citra_layout_option` (Top-Bottom, Single,
    Large/Small, Side by Side), `citra_swap_screen`, `citra_large_screen_proportion` [C4]. Cheap (just core
    options), but: two different layout vocabularies, the touch pointer mapping must mirror each core's maths,
    no free custom rectangles in Azahar libretro, and each layout change may resize the frame.
  - *B. Shell composes*: ask both cores for a fixed stacked frame (top/bottom, no gap) and let our wgpu
    compositor cut it into two textures and place them anywhere (this is what Delta does with `inputFrame` [P3]).
    One vocabulary for both consoles, free custom layouts, exact touch mapping, the touch overlay and the screen
    layout share one document, and a second window/second monitor becomes possible. Cost: we write the layout
    engine and must handle the 3DS's different screen widths (400 vs 320) and hardware-rendered frames later.
  - **Recommendation: B**, with A as a fallback while B is not ready (Phase 2 can ship NDS with core layouts).
    Shaders under B apply per screen (**to verify** what librashader expects for two viewports).
- **Webview path**: the canvas can do layout B with CSS/2D canvas too, but shaders are only realistic on the
  native path (librashader's wgpu runtime). A "shader requires Native video" disabled-with-reason entry follows
  the project rule.
- **librashader**: pure Rust, wgpu runtime listed as fully supported from 0.11 [V1]; limits: some shaders cannot be
  translated to WGSL, no FSR on wgpu, some need `FLOAT32_FILTERABLE`, input mipmaps are never generated [V1]; the
  parser is stricter than RetroArch's [V1]. **Compatibility of librashader's wgpu dependency with our wgpu 30 is
  to verify** (a version mismatch means waiting or patching).
- Integer scale, aspect, filters, background: a few uniforms in `video.rs`. S.
- Rotation: a transform in the compositor (B) or `RETRO_ENVIRONMENT_SET_ROTATION` from the core.
- **Bench**: `docs/bench-windows.md` does not exist yet (not on `develop`, not in the `feat/bench` worktree):
  backend comparison results are **pending**.

### Effort

| Piece | Size |
|---|---|
| Scale/aspect/filter/integer/background | S |
| Native ↔ webview + backend as settings (machine-local) | S |
| Shell-side screen compositor (B) incl. cycle list and swap | M |
| Screen editor reusing the touch editor | S (after the touch editor) |
| librashader integration + preset picker + parameters | M (L if the wgpu version gap needs work) |
| Hardware rendering (`SET_HW_RENDER`, GL/Vulkan interop with wgpu) | **L**, Phase 2/3, needed for 3DS |

### Risks and limits

- Hardware rendering is the biggest technical risk in the whole customization story: without it, 3DS video
  options are mostly unavailable and 3DS speed on phones is doubtful.
- Shaders on Android cost battery and may cost frames on mid-range GPUs (Adreno 618 in the test phone).
- Core-composed layouts (A) change the output geometry: `SET_GEOMETRY` must be honoured without a flicker.

### Prior art

melonDS DS layouts and cycle list [C3]; Azahar libretro layouts [C4] and standalone custom layout keys [C5];
DeSmuME hybrid layouts (equal height, emphasize vertical) [C6]; Delta screen frames and CoreImage filters [P3];
RetroArch shader presets and per-core/per-game overrides [R1].

---

## 4. Audio

### What the user could customize

Master volume, mute, per-game volume; output device (desktop); latency/buffer size (low latency vs no
crackle); dynamic rate control strength (today ±0.5 %); audio during fast forward and slow motion (today muted;
options: mute, play pitched, play time-stretched); resampler quality; core-side audio options (mGBA low-pass
filter and level [C2]; melonDS DS interpolation and bit depth [C3]; Azahar HLE/LLE [C4]); NDS/3DS microphone
source (none, noise, blow, real mic [C3][C4]).

### Storage and editing

Scalars at Global → Console → Game; output device and buffer size on the **per-device (machine) axis**. Edited
from the schema-generated settings UI; volume also in the quick menu.

### Feasibility

All inside `audio.rs` (cpal): buffer size through `StreamConfig::buffer_size` (Fixed) where the backend allows it
(AAudio/WASAPI; **to verify** per backend); volume = a gain before the queue. Fast-forward audio: playing every
frame's samples at higher speed needs a time-stretch (e.g. a WSOLA crate, **to verify**) or simply pitched
playback (drop/resample). Real microphone needs `RETRO_ENVIRONMENT_GET_MICROPHONE_INTERFACE` plus capture via
cpal and the Android `RECORD_AUDIO` permission.

### Effort and limits

Volume/mute/per-game: S. Buffer/latency: S. Fast-forward audio: M. Microphone: M. Limits: Bluetooth headsets add
latency we cannot remove; the 8-channel device seen on Windows (fase0-esito) needs a channel-layout setting only
if users complain.

### Prior art

RetroArch audio latency and dynamic rate control [R1]; PPSSPP and DuckStation expose audio stretching during
fast forward (**to verify**).

---

## 5. Core options

### What exists in the cores

**mGBA** (`mgba_*`, categories system, video, audio, input, performance) [C2]:
`mgba_gb_model`, `mgba_use_bios`, `mgba_skip_bios`, `mgba_gb_colors`, `mgba_gb_colors_preset`, `mgba_sgb_borders`,
`mgba_color_correction` (OFF/GBA/GBC/Auto), `mgba_interframe_blending` (OFF, mix, mix_smart, lcd_ghosting,
lcd_ghosting_fast), `mgba_audio_low_pass_filter`, `mgba_audio_low_pass_range` (5–95 %),
`mgba_allow_opposing_directions`, `mgba_solar_sensor_level` (sensor, 0–10), `mgba_force_gbp`,
`mgba_idle_optimization`, `mgba_frameskip` (+ threshold, interval). About 17 options, a third of them
Game Boy-only (EmuDome does not target GB/GBC: hide them, or decide to support GB/GBC — owner question).

**melonDS DS** (`melonds_*`, categories `system`, `time`, `video`, `audio`, `screen`, `firmware`, `network`,
`cpu`, `osd`) [C3]:
- system: console mode (Auto/DS/DSi), BIOS/firmware mode (Native/Built-In), DS and DSi firmware file, DSi NAND,
  boot mode (Direct/Native), virtual SD cards (DSi and homebrew, read-only, sync to host), battery interval and
  low-battery threshold, Slot-2 device (Rumble Pak, Memory Expansion, Solar Sensor, Motion Pak), rumble
  intensity and motors, host light sensor;
- video: render mode (Software, OpenGL Classic, OpenGL Compute), internal resolution 1×–8×, polygon splitting,
  high-res coordinates, threaded software renderer;
- screen: cursor mode and timeout, touch mode (Joystick/Pointer/Auto), joystick cursor deadzone/speed/curve,
  number of layouts (1–8) and layouts #1–#8 (17 choices), hybrid ratio, hybrid small screen, filtering, gap,
  secondary screen scale;
- audio: mic input mode, mic button mode, bit depth, interpolation;
- firmware overrides: DNS/Wi-Fi server, language, username, favourite colour, birthday, alarm;
- cpu: JIT on/off, block size, branch/literal optimizations, fast memory;
- `time` and `osd` categories exist; their options were not read (**to verify**).
Roughly 70 options.

**Azahar** (`citra_*`) [C4]: CPU JIT, CPU clock 25–400 %; New/Old 3DS model, region, system language;
audio emulation HLE/LLE/LLE multithreaded; mic input type; graphics API (Auto/Vulkan/OpenGL/Software), hardware
shaders, shader JIT, accurate multiplication, disk shader cache, resolution 1×–18×, texture filter (Anime4K,
Bicubic, ScaleForce, xBRZ, MMPX), texture sampling, custom/dump textures; layout, prominent screen, swap mode,
large screen proportion, stereo 3D mode and depth; virtual SD, save location (libretro vs Azahar default);
right analog function, analog deadzone, mouse/touch touchscreen, pointer timeout, motion support and sensitivity.
About 35 options.

### How to expose them generically

1. **Host API** (in `callbacks.rs`): answer `RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION` with 2, accept
   `SET_CORE_OPTIONS_V2` and `SET_CORE_OPTIONS_V2_INTL` (definitions with key, label, categorized label, info,
   category, values, default), keep `SET_VARIABLES` (v0) as a fallback, implement `GET_VARIABLE` and
   `GET_VARIABLE_UPDATE`, `SET_CORE_OPTIONS_DISPLAY` and `SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK` (cores hide
   options that do not apply, e.g. OpenGL resolution in software mode) [C1][C7]. Answer `GET_LANGUAGE` so cores
   with translations (`_INTL`) give Italian labels where they have them (**to verify** per core).
2. **Option manifest**: cores declare options in `retro_set_environment`/`retro_init`, before a game is loaded
   [C7], so the shell can **introspect a core without a game** and cache a manifest
   (`core-options/<core>-<version>.json`). The settings UI is built from the manifest, so the options page works
   before a game runs.
3. **Schema integration**: the Rust settings schema is static; core options are a **dynamic namespace**
   `core.<core_id>.<key>` whose type is "enum of the manifest's values". The engine stores the value string,
   validates it against the manifest, and drops values a newer core no longer offers (falls back to the default
   and logs it).
4. **Levels**: Default = the core's default; Global and Console are the same for a single-console core;
   Game is the important one (per-game JIT off, internal resolution, Slot-2 device); Session for experiments.
5. **Curation layer**: a small table in the shell per core: which options are hidden (they duplicate a shell
   feature: layouts if we choose shell-side layout, frameskip, GB-only options), which are promoted to the main
   video/audio pages, which need a restart (mGBA marks them "(Restart)" in the label [C2]; melonDS DS and Azahar:
   **to verify**), and our translated label/help when the core's text is poor. Everything else appears under
   "Advanced → Core options", grouped by the core's categories.
6. **Disabled with a reason**: options that depend on a missing host feature (OpenGL renderer without
   `SET_HW_RENDER`, real microphone without the mic interface) are shown disabled with the reason (project rule).

### Effort, risks

Host API + manifest + generic page: **M**. Curation table: S per core. Risks: option keys change between core
versions (store per core version, migrate); a few options only apply after `retro_reset` or reload — the UI must
say "applies on restart"; BIOS/firmware file pickers (melonDS DS lists files it found in the system folder) must
never point the user to download Nintendo files.

### Prior art

RetroArch core options with categories and per-game/per-content-directory overrides (`.opt` files) [R1][C7].

---

## 6. Saves

### What the user could customize / have

- **In-game saves** (SRAM, EEPROM, flash; NDS save chip; 3DS extdata/savedata in the virtual SD/NAND): location
  per game/console, autosave interval (today 30 s), backup on every write with N versions kept.
- **RTC**: real time vs offset vs fixed date (melonDS DS has a `time` category, **to verify** its options);
  GBA RTC (Pokémon): mGBA's handling through libretro **to verify**.
- **Save states**: number of slots (today 3 in the menu, 1–9 by combo), thumbnails, timestamps, names/notes,
  auto-save on exit + resume on launch, "undo load" (keep the state replaced by a load) and "undo save" (keep the
  overwritten slot), lock a slot.
- **Folders**: one tree per console and game (`saves/<console>/<game-id>/`), or user-chosen roots (e.g. a synced
  folder).
- **Import/export**: per-game zip (in-game save + states + metadata), raw `.sav` import from other emulators.
- **Backups/versioning**: rotating copies of the in-game save (`.sav.1`, `.sav.2`…) on every launch and before
  every import; states kept per version of the core.
- **Sync between devices**: see below.
- **BIOS/firmware**: user-provided only.

### How it is stored

- Not "settings": files on disk under the data directory, with **settings that point to them**
  (`saves.root`, `saves.autosave_interval`, `states.slots`, `states.autosave_on_exit`, `saves.backups_kept`),
  at Global/Console/Game. The root folder is on the per-device axis (a path is machine-local).
- Per-game metadata (`meta.json`: game id, core and core version per state, timestamps, thumbnail file names).
- Game identity: a content hash (CRC32 + SHA-1 of the ROM file, 3DS: title ID from the header). The file name is
  not an identity (renaming a ROM must not lose its saves).

### Feasibility

- In-game saves: done for GBA via `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM)`. melonDS DS and Azahar manage
  some of their own files: Azahar keeps saves and system files in an `Azahar/` folder inside the frontend's save
  directory (or its standalone user dir, `citra_use_libretro_save_path`) [C4]; melonDS DS keeps virtual SD images
  and may write its own save file (**to verify** whether it uses `SAVE_RAM` or its own `.sav`). So "per-game
  folder" works by giving each game its own `GET_SAVE_DIRECTORY` (**to verify** that both cores accept a
  per-game save dir without side effects).
- Thumbnails: the shell has the last frame (`Shared`), so a PNG next to each state is S.
- Auto-save/resume, undo load: S each.
- **Compatibility with other emulators**: GBA `.sav` is raw save memory, shared by mGBA standalone and most
  emulators (an RTC footer may differ: **to verify**). NDS raw `.sav` is what melonDS standalone uses (**to
  verify** melonDS DS's file). 3DS saves live in the NAND/SD directory tree: compatible with Azahar standalone
  when we point the core at the same layout. **Save states are never portable** between cores or core versions
  (Delta notes melonDS core updates broke states [P10]): we label states with the core version and warn on load.
- **Sync**: three tiers:
  1. *User-managed folder*: the saves root on a folder synced by Syncthing/Dropbox/OneDrive/Google Drive desktop.
     Zero code beyond "choose folder" and safe writes (write to temp + rename). Conflicts are the sync tool's.
  2. *Built-in cloud provider* (Delta's model: Dropbox and Google Drive through a sync framework with
     pluggable back-ends [P10]): OAuth per provider, per-file version tracking, conflict UI. L. Delta's history
     shows the risk: a bug uploaded corrupted saves and associated states with the wrong games [P10].
  3. *Own sync server*: L+, plus running a service. Not recommended.
  Conflict rule for any tier: never overwrite silently; keep both, ask the user, show date and play time.
- Android: the data dir is app-private; "choose folder" means Storage Access Framework (content URIs, not
  paths), which the core cannot open directly — the shell would copy in/out. **M on Android**, S on desktop.
- **BIOS/firmware** (project rule: never shipped, downloaded or pointed to): a "System files" page that lists
  what each core *can* use (mGBA `gba_bios.bin`, optional [C8]; melonDS DS `bios7.bin`, `bios9.bin`,
  `firmware.bin`, DSi files, optional for DS mode thanks to built-in replacements [C3]; Azahar system files
  dumped from the user's console, some games need Mii data [C4]), lets the user pick a file they own, checks it
  by hash, copies it into the `system` folder, and shows "built-in replacement in use" otherwise. No links, no
  hashes that read as a download guide beyond validation.

### Effort

States UX (thumbnails, autosave/resume, undo, more slots, notes): M. Per-game folders + metadata + migration of
today's flat folder: S. Import/export zip: S. Backups: S. Sync tier 1: S (desktop) / M (Android). Tier 2: L.
System files page: S.

### Prior art

RetroArch: savefile/savestate folders sorted by core and content, auto-save/load state, undo load/save, state
thumbnails [R1]. Delta: Dropbox/Google Drive sync via Harmony, conflict file after a bug [P10]. melonDS DS: virtual
SD sync to host [C3].

---

## 7. Library and interface

### What the user could customize

- **Theme**: within the EmuDome Design System only — dark (default), maybe a light or OLED-black variant,
  accent colour from a small curated set. Not free colour pickers (the design system owns the look).
- **Language**: today en/it from the OS; add a manual override (setting `ui.language`), more locales later.
- **Library**: grid/list, sort (name, last played, play time, console), filters (console, favourites), group by
  console, hidden games, folders to scan (per device), game title overrides.
- **Covers**: see below.
- **Per-game profile**: the Game level of the settings engine — a game page shows "this game overrides: …" with a
  reset per item.
- **Quick menu**: order and visibility of entries, which actions get a top-level button (save, load, speed,
  layout, screenshot…).
- **Accessibility**: text size (scale factor), high contrast, reduced motion (also from the OS
  `prefers-reduced-motion`), remappable UI navigation (which buttons confirm/back: Nintendo vs Xbox convention),
  hold-to-repeat timing (today 400/120 ms), screen-reader labels (already through i18n keys).

### Storage and editing

All Global scalars except library folders (per device) and per-game title/cover overrides (Game). Quick menu
layout as an ordered list setting. Edited through the schema UI; the quick menu could have its own "edit" mode
(design question).

### Feasibility

All webview work on existing pieces (vue-i18n, CSS tokens). Theme = token sets switched by a root attribute; the
design system must define them. Library = new: scanner, database (SQLite or a JSON index), hash identification.

**Covers — licensing**: box art of commercial games is copyrighted by the publishers. libretro-thumbnails is a
community collection without a clear image licence (its README sets contribution rules only [L1]). Options, safest
first: (a) **user-supplied images** (pick a file, or drag one in); (b) **generated covers** (title + console tile in
the design system's style, no third-party art); (c) **opt-in scraping** from a third-party service the user
configures with their own account (ScreenScraper, IGDB…; terms and commercial-use rules **to verify** before any
integration); never bundle or host covers ourselves. 3DS games carry an icon (SMDH) inside the ROM and NDS ROMs
have a banner icon: extracting it from the user's own file is fine and gives a real image with no download
(**to verify** decoding effort).

### Effort

Theme variants: S (after design). Language override: S. Library (scan, hash, DB, views, sort): L (Phase 1 core
item, outside "customization" proper). Covers (a)+(b)+ROM icons: M. Quick menu customization: S. Accessibility
settings: S–M.

---

## 8. Cheats, patches, achievements, multiplayer

### Cheats

- Formats: mGBA accepts GameShark/Action Replay/CodeBreaker types (encrypted GSA, decrypted CodeBreaker, decrypted
  AR MAX reported working through libretro [X1]); melonDS DS supports "RetroArch cheats" and "native cheats"
  [C3b] (Action Replay DS codes, **to verify**); **Azahar's libretro core does not support RetroArch cheats** [C4]
  — 3DS Gateway cheats (`[Name]` + hex pairs) exist in standalone Citra/Azahar [X2] but there is no libretro path:
  3DS cheats would need a core patch or upstream work.
- Host side: `retro_cheat_reset` / `retro_cheat_set(index, enabled, code)` — the API has no field for the code
  type, a known limitation [X1].
- Storage: per game, `cheats/<game-id>.json` (name, code, enabled, type hint). Import from RetroArch `.cht`
  (simple key=value format) later.
- Editor: list with toggles, add/edit code, import file. M. Cheat databases are third-party content: user import
  only.

### Patches (soft-patching)

- IPS, UPS, BPS applied in memory before `retro_load_game`, for cores that take the ROM as a buffer
  (`need_fullpath = false`; mGBA does, melonDS DS lists softpatching as supported [C3b]). Azahar needs a path;
  3DS mods are LayeredFS (romfs/exefs replacement), a different mechanism (`citra_custom_textures` is related but
  separate [C4]).
- Storage: patch files next to the ROM with the same name (RetroArch convention) or chosen per game (Game level
  setting `content.patches = [..]`, ordered). The game id must then be the **patched** hash for saves (a ROM hack
  is a different game) — product decision.
- Effort: S–M (the three formats are small; in-house code or a crate, **to verify** crate quality).

### RetroAchievements

- Integration library: rcheevos with the `rc_client` API; the emulator provides HTTP itself; the guide recommends
  developing with hardcore off [A1].
- Support per console: GBA (mGBA listed as most recommended) and NDS/DSi (melonDS DS listed; DSi without states)
  are supported; **3DS is not a RetroAchievements console** [A2].
- Hardcore mode forbids save-state loading, rewind, slow motion, frame advance (from an unofficial copy of the
  compliance page [A3], **to verify** on the official one); a new emulator needs RA's approval after public
  release (reported by Epilogue's changelog [A4]); one compliance page states a minimum public availability
  period (**to verify**).
- Feasibility: memory exposure via `retro_get_memory_*` / `RETRO_ENVIRONMENT_SET_MEMORY_MAPS` (both cores list
  "memory monitoring" [C3b][C8]). Effort: L (client, login, UI, hardcore gating across every feature). Later
  phase.

### Netplay and local multiplayer (notes only)

- GBA link cable: mGBA standalone supports multiple local instances; through libretro **to verify** (likely not).
- NDS: melonDS DS lists "Netplay (LAN only)" [C3b]; Nintendo WFC replacement servers via `melonds_*` DNS options
  [C3] give online play in some games without any work from us.
- 3DS: Azahar standalone has multiplayer rooms; libretro status **to verify**.
- libretro rollback netplay needs fast, deterministic save states each frame: costly for NDS/3DS. Recommend: no
  netplay until Phase 5; online play through WFC servers is a free win for NDS.

---

## 9. Editors overall

| Editor | Worth building? | Phase 1 form | Later |
|---|---|---|---|
| Settings pages (incl. core options) | Yes, generated from the schema | Generated UI | — |
| Button mapping (controller, keyboard, hotkeys) | Yes, core feature on desktop | Capture + conflicts + tester | Curves, per-unit overrides |
| Touch layout editor | Yes, core feature on Android | Drag/resize/snap/reset/undo | Macros timeline, skin packs, importers |
| Screen layout editor (NDS/3DS) | Yes, after the touch editor (shares the canvas) | Cycle list picker; file-only custom rectangles | Visual editor (Phase 2) |
| Shader preset picker | Yes, cheap and visible | Picker + params | Preset chains |
| Cheat editor | Medium | File import + toggles | Code editor |
| Quick menu editor | Low | Ordered-list setting | Visual |
| Theme editor | No | Fixed variants from the design system | — |

**One data model**: every editor writes either a **scalar at a level** (through the settings engine) or a
**document** (layout, bindings, cheats) that a scalar references. The settings engine owns resolution
(Default → Global → Console → Game → Session, with the device axis); documents are versioned JSON with a
`schema` field and migrations. Editors never write files directly: they call one Tauri command family
(`settings_get/set/reset`, `doc_get/put/list/import/export`), so undo, validation and sync are in one place.

---

## 10. Data model sketch

```
<data dir>/
  settings/
    global.toml                 # Global level, portable (synced)
    console/{gba,nds,3ds}.toml  # Console level
    game/<game-id>.toml         # Game level (sparse)
    device/<machine-id>.toml    # machine-local: video path/backend, audio device/buffer, library folders,
                                # saves root (never synced)
  input/
    bindings/<device-guid>.json # controller/keyboard bindings per model (+ "generic.json")
  touch/
    layouts/<layout-id>.layout.json
    skins/<pack-id>/manifest.json + assets
  core-options/<core>-<version>.json   # cached manifests from SET_CORE_OPTIONS_V2
  cheats/<game-id>.json
  saves/<console>/<game-id>/{game.sav, state-<n>.state, state-<n>.png, meta.json, backups/}
  system/                       # user-provided BIOS/firmware only
  shaders/                      # user-added slang presets
```

```rust
// Sketch, not an API proposal: names to be fixed with the engine.
enum Level { Default, Global, Console(ConsoleId), Game(GameId), Session }
enum DeviceAxis { Machine(MachineId), Controller(Guid), FormFactor(Bucket) } // which one depends on the key

struct SettingDef {            // the single Rust schema the UI is generated from
    key: &'static str,         // "video.scale_mode", "touch.layout.portrait", "core.melonds.melonds_render_mode"
    kind: Kind,                // Bool, Int{min,max}, Enum{values}, DocRef{doc_type}, List, …
    levels: LevelMask,         // where it may be set (e.g. video.backend: Global + Machine only)
    device_axis: Option<AxisKind>,
    applies: Applies,          // Live | OnReset | OnRestart
    requires: Option<Feature>, // e.g. HwRender, NativeVideo → shown disabled with the reason
    synced: bool,
}

struct TouchLayout { schema: u32, id: String, console: ConsoleId, orientation: Orientation,
                     form_factor: Option<Bucket>, reference: Insets, screens: Vec<ScreenFrame>,
                     controls: Vec<Control> }
struct Control { id: String, kind: ControlKind /* Button, Dpad, Stick, Touchscreen, Hotkey, Macro, Turbo */,
                 inputs: Vec<InputRef>, rect: NormRect, anchor: Anchor, shape: Shape,
                 skin: Option<AssetRef>, opacity: Option<f32>, extended_edges: Insets, behaviour: Behaviour }

struct Bindings { schema: u32, device: Guid, face_layout: FaceLayout /* Positional | Label */,
                  map: BTreeMap<RetroPadButton, Vec<PhysicalInput>>, modifier: PhysicalInput,
                  combos: BTreeMap<Hotkey, PhysicalInput>, turbo: BTreeMap<RetroPadButton, Turbo>,
                  analog: AnalogSettings }
```

Which level each area lives at:

| Area | Default | Global | Console | Game | Session | Device axis |
|---|---|---|---|---|---|---|
| Touch layout reference | built-in | ✓ | ✓ | ✓ | — | form factor + orientation |
| Touch scalars (opacity, haptics, auto-hide) | ✓ | ✓ | ✓ | ✓ | — | — |
| Bindings / hotkeys | `DEFAULT_COMBOS` | ✓ | ✓ | ✓ | — | controller model |
| Video scalars, shader | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Screen layout (NDS/3DS) | ✓ | — | ✓ | ✓ | ✓ | form factor + orientation |
| Video path, backend | per OS | ✓ | — | — | ✓ | machine |
| Audio volume, FF audio | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Audio device, buffer | ✓ | — | — | — | — | machine |
| Core options | core default | (=Console) | ✓ | ✓ | ✓ | — |
| Save/state behaviour | ✓ | ✓ | ✓ | ✓ | — | saves root: machine |
| UI (theme, language, accessibility, quick menu) | ✓ | ✓ | — | — | — | — |
| Cheats, patches | — | — | — | ✓ (document) | — | — |

Format choice: TOML for hand-edited sparse settings (comments survive with `toml_edit`), JSON for documents
produced by editors. Both versioned with `schema`.

---

## 11. Roadmap proposal

**Phase 1 (desktop shell) — customization that must be there:**

1. Settings engine with levels, device axis, schema-generated UI, documents store (M–L; everything depends on it).
2. Core options v2 host API + manifests + generic "Core options" page + curation for mGBA (M).
3. Bindings: map, positional/label, modifier and hotkey table remappable, turbo, deadzones, mapping UI (M).
4. Video scalars (scale, aspect, integer, filter, background), video path/backend as settings (S).
5. Audio volume, per-game volume, buffer size, fast-forward audio choice (S–M).
6. Saves: per-game folders + migration, thumbnails, more slots, auto-save/resume, undo load, backups, import/export,
   system-files page (M).
7. librashader presets + picker (M; verify the wgpu version gap first).
8. Language override, accessibility basics (text size, reduced motion, navigation convention) (S).

**Phase 1.5 / Android track (Phase 0 step 4 → Phase 4):** data-driven touch overlay, built-in layouts per
console × orientation × form factor, the in-app touch editor, haptics, Android controller backend, phone gyro.
The touch overlay rewrite should start in Phase 1 because the NDS layout (Phase 2) depends on it.

**Phase 2 (NDS):** `RETRO_DEVICE_POINTER`, shell-side screen compositor and cycle list, screen editor, mic button,
melonDS DS curation, cheats (mGBA + melonDS DS), soft-patching.

**Phase 3 (3DS):** `SET_HW_RENDER` (L), analog circle pad/C-stick, motion, Azahar curation, 3D modes.

**Phase 5 (extras):** skin packs and importers, macros timeline, built-in cloud sync, RetroAchievements (GBA/NDS),
netplay notes revisited, covers scraping (opt-in).

---

## 12. Open questions

### For Cowork (design)

1. Touch editor interaction: how selection, resize handles, snap feedback and the control palette look and
   behave on a 6" phone; where undo/redo/reset live; how the paused game shows under the editor.
2. Built-in touch skins: one style or a few (outline, filled, minimal)? How pressed state is shown without
   translucent strokes (global icon rule).
3. Visual language for "overridden at this level" in settings pages (Global vs Console vs Game) and the reset
   affordance.
4. Mapping screen: how capture ("press a button"), conflicts and the live tester are shown, with controller-only
   navigation.
5. Screen layout picker and editor for NDS/3DS: how the cycle list is chosen; how the empty area around the
   screens is filled ("no empty space" rule: blur of the game, colour, or design-system effect).
6. Shader picker with live preview: where the preview lives (full game behind a glass panel vs thumbnail).
7. Theme variants the design system offers (OLED black? light? accent set?).
8. Generated covers: what a cover looks like with no art (title, console, icon from the ROM).
9. Quick menu: fixed set vs user-ordered; how the edit mode looks.
10. Disabled-with-reason pattern for options a core or platform cannot do.

### For the owner (product)

Answered on 2026-11-17: see `docs/decisions.md`, section "Product".

1. Default face layout: positional (today) or label? Per controller model default (Nintendo-layout pads → label)?
2. Shell-side vs core-side NDS/3DS screen layout (recommendation: shell-side, section 3)?
3. GB/GBC support (mGBA does it for free): show or hide those options?
4. Sync ambition: user-managed folder only (recommended for Phase 1) or built-in Dropbox/Google Drive later?
5. Covers: user images + generated + ROM icons only, or opt-in third-party scraping?
6. RetroAchievements: wanted at all, and in which phase (it constrains save states, rewind, slow motion in
   hardcore)?
7. Hardware rendering priority: 3DS on phones depends on it — accept that 3DS waits for it?
8. ROM hacks: same game (shared saves) or a different game (own saves) when a patch is applied?
9. Skin packs from third parties: support import from day one, and accept `.deltaskin`/RetroArch overlay imports?
10. Macros: allowed always, or off in an "achievements" mode?

---

## 13. Sources

Cores and libretro API
- [C1] libretro API header (`libretro.h`: `SET_HW_RENDER`, `RETRO_DEVICE_POINTER`, core options v2, sensor,
  rumble, microphone interfaces): https://github.com/libretro/RetroArch/blob/master/libretro-common/include/libretro.h
- [C2] mGBA libretro core options: https://raw.githubusercontent.com/libretro/mgba/master/src/platform/libretro/libretro_core_options.h
- [C3] melonDS DS option definitions: https://github.com/JesseTG/melonds-ds/tree/main/src/libretro/config/definitions
  (`screen.hpp`, `video.hpp`, `system.hpp`, `audio.hpp`, `firmware.hpp`, `cpu.hpp`, `categories.hpp`) and keys in
  https://raw.githubusercontent.com/JesseTG/melonds-ds/main/src/libretro/config/constants.hpp ; project page
  https://github.com/JesseTG/melonds-ds
- [C3b] Libretro Docs, melonDS DS (features: cheats, softpatching, netplay LAN only, achievements):
  https://docs.libretro.com/library/melonds_ds/
- [C4] Libretro Docs, Azahar (options, features, save paths, no RetroArch cheats): https://docs.libretro.com/library/azahar/
- [C5] Azahar discussion on custom layout keys (`custom_top_x` …; community): https://github.com/azahar-emu/azahar/discussions/860
- [C6] DeSmuME hybrid layouts PR: https://www.github.com/TASEmulators/desmume/pull/773
- [C7] libretro-common, core options samples (v1/v2, categories): https://raw.githubusercontent.com/libretro/libretro-common/master/samples/core_options/README.md
- [C8] Libretro Docs, mGBA (BIOS, features): https://docs.libretro.com/library/mgba/

Touch controls and skins
- [P1] PPSSPP control settings: https://www.ppsspp.org/docs/settings/controls
- [P2] RetroArch overlay format: https://docs.libretro.com/development/retroarch/input/overlay/
- [P3] Delta skins documentation: https://noah978.gitbook.io/delta-docs/skins
- [P4] libretro forum, request for in-app overlay editing: https://forums.libretro.com/t/please-allow-touch-controller-overlays-to-be-edited-via-touch/45806
- [P5] r/EmulationOnAndroid on PPSSPP custom buttons (community): https://redlib.ssps.io/r/EmulationOnAndroid/comments/1d1l5je/thanks_to_ppsspps_customizable_touch_controls
- [P6] Lemuroid 1.5.0 touch controls: https://swordfish90.github.io/lemuroid-1-5-0.html
- [T1] Tauri haptics plugin: https://v2.tauri.app/plugin/haptics/ and https://v2.tauri.app/reference/javascript/haptics/

Per-game configuration
- [P7] DuckStation per-game controller configuration commit: https://git.morozoff.pro/aiden/duckstation/commit/bda6869084ec152a4e95a29dfec986be67d993e6
- [P8] LaunchBox forum, DuckStation per-game input profiles (community): https://forums.launchbox-app.com/topic/62237-duckstation-per-game-controls/
- [P9] Dolphin GameINI (Batocera wiki, links the Dolphin wiki): https://wiki.batocera.org/emulators:dolphin ; https://wiki.dolphin-emu.org/index.php?title=GameINI
- [R1] RetroArch reference configuration: https://raw.githubusercontent.com/libretro/RetroArch/master/retroarch.cfg
- [R2] Libretro Docs, input and controls: https://docs.libretro.com/guides/input-and-controls/
- [S1] SDL3 gamepad API (button labels, sensors): https://wiki.libsdl.org/SDL3/CategoryGamepad (**to verify** exact function list)

Video
- [V1] librashader (wgpu runtime, limits, presets): https://docs.rs/crate/librashader/latest/source/README.md and https://lib.rs/crates/librashader

Saves and sync
- [P10] Delta sync FAQ and Harmony: https://faq.deltaemulator.com/using-delta/delta-sync , https://rileytestut.com/blog/2018/02/25/delta-features-syncing ,
  incompatible melonDS states https://faq.deltaemulator.com/using-delta/nintendo-ds/incompatible-save-states ,
  corrupted-upload notice https://www.patreon.com/posts/87592011

Cheats, achievements, library
- [X1] libretro forum, mGBA cheat compatibility (community): https://forums.libretro.com/t/mgba-cheat-code-compatibility/34713 ;
  cheat type limitation: https://forums.libretro.com/t/wip-adding-cheat-support-to-more-cores/6604
- [X2] Citra Gateway cheat format (source): https://git.openpunk.com/OpenPunk/citra/src/commit/32b88d47192a0b980d0d9cabd40f7636a08b522a/src/core/cheats/gateway_cheat.h
- [A1] rcheevos `rc_client` integration: https://github.com/RetroAchievements/rcheevos/wiki/rc_client-integration
- [A2] RetroAchievements emulator support: https://docs.retroachievements.org/general/emulator-support-and-issues.html
- [A3] Hardcore compliance requirements (unofficial copy, **to verify** against retroachievements.org): https://chew-ra-docs.pages.dev/general/hardcore-compliance-requirements
- [A4] Epilogue changelog (RA approval after public release): https://www.epilogue.co/changelog/playback/1.8.0
- [L1] libretro-thumbnails (README, contribution rules): https://github.com/libretro-thumbnails/libretro-thumbnails

Project documents
- `docs/combo-tasti.md` (controller combos, Steam/Game Bar/Android conflicts), `docs/fase0-esito.md` (Windows and
  Android results), `docs/bench-windows.md` (**pending**, not written yet).
