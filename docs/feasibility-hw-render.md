# Feasibility: hardware rendering (`SET_HW_RENDER`)

Trello #16. This file collects the whole answer; the step 0 and step 1 entries stay in `docs/feasibility.md` («Brief for Claude Code · hardware rendering spike» and «Hardware
rendering (`SET_HW_RENDER`)»), and the numbers from there are repeated here only where they are needed.

## Question

The shell's host takes software frames only (`video_refresh` with a CPU buffer). Azahar (3DS: Vulkan or OpenGL
renderer) and melonDS DS (NDS: OpenGL renderer, the only way to upscale) need `RETRO_ENVIRONMENT_SET_HW_RENDER`: the
frontend provides a GPU context, the core renders into it, and `video_refresh` signals `RETRO_HW_FRAME_BUFFER_VALID`.
Can EmuDome do it with its wgpu 30 video, on Windows first, and what about Linux and Android? What does it cost, and
which path is best?

## Answer in short

- **Feasible, both APIs, verified on Windows** with real cores and free homebrew: Vulkan (Azahar) and desktop OpenGL
  (melonDS DS and Azahar), the frames reaching a wgpu draw with no visible window.
- **3DS: Vulkan, zero copy, on every OS.** The core creates its device from wgpu's instance and adapter, wgpu wraps it,
  the compositor samples the core's image directly. Cheapest path measured (0.09-0.34 ms per frame for the draw, 1x-4x).
- **NDS: software stays the default.** melonDS DS's OpenGL renderer works through our host (upscaling 1x-8x), and on
  Windows its frames reach wgpu **without a copy** through memory shared between GL and Vulkan. It is optional
  (desktop-only upscaling); melonDS DS has no GLES and no Vulkan renderer, so phones keep software.
- **Readback** (GPU to CPU) is the fallback and the webview path's only option; it is cheap at 1x and grows with the
  resolution (OpenGL `glReadPixels`: 0.3-1.4 ms at 1x, 0.8-3.6 ms at 4x, 2.5 ms at 8x on this machine).
- **Work**: Vulkan in the shell M-L (about 6-10 days); OpenGL for melonDS DS on desktop M (about 4-7 days with zero
  copy, 2-3 with readback only). Phones: not measured (no A71 access in these sessions).

## Findings, with sources

### The libretro API

- `retro_hw_render_callback` [L1]: the core fills `context_type` (OpenGL, OpenGL core, GLES 2/3, Vulkan, D3D...),
  `version_major/minor`, `depth`, `stencil`, `bottom_left_origin`, `cache_context`, `debug_context`, and its own
  `context_reset` / `context_destroy`; the frontend fills `get_current_framebuffer` (the GL framebuffer object the
  core must draw its frame into) and `get_proc_address` (GL entry points). `SET_HW_RENDER` is called inside
  `retro_load_game`; the frontend creates the context afterwards and calls `context_reset`, where the core creates its
  GPU objects. `context_destroy` comes before the context goes away (video restart, unload). Refusing
  `SET_HW_RENDER` makes the core fall back or fail, as the core decides.
- OpenGL frames: the core draws into `get_current_framebuffer()`, then calls `video_refresh(RETRO_HW_FRAME_BUFFER_VALID,
  w, h, 0)`. The frame is the bottom-left `w x h` of the framebuffer, upside down when `bottom_left_origin` is set
  (GL convention). The framebuffer must be as large as the core's `max_width x max_height` [L1].
- Vulkan [L2]: no framebuffer. The core may negotiate device creation
  (`SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`, `create_device` / `create_device2`), asks
  `GET_HW_RENDER_INTERFACE` for `retro_hw_render_interface_vulkan` (instance, GPU, device, queue, `lock_queue`,
  `set_image`, sync indices, semaphores) and hands one `VkImage` per frame with `set_image`.
- `SET_HW_SHARED_CONTEXT`: the core keeps its own GL state and the frontend must not rely on the context's state
  across `retro_run` [L1]. Azahar asks for it.

### What the cores ask for

| Core | Renderer option | `SET_HW_RENDER` | Notes |
| --- | --- | --- | --- |
| Azahar 2127.0-rc4 | `citra_graphics_api` = `Vulkan` | Vulkan 1.1, negotiation v2 (`create_device` only) | Device made by the core from our instance/GPU [A1]; `cache_context` true |
| Azahar | `citra_graphics_api` = `OpenGL` | OpenGL core 4.3 (GLES 3.2 on Android builds), `bottom_left_origin`, `SET_HW_SHARED_CONTEXT` | Load fails if refused [A1] |
| Azahar | `citra_resolution_factor` `1`-`18` | | Internal resolution [A2] |
| melonDS DS 1.4.0 | `melonds_render_mode` = `opengl` (`compute`: GL 4.3) | OpenGL core 3.2, `depth` true, `bottom_left_origin` | Never caches the framebuffer id; falls back to software if refused [M1] |
| melonDS DS | `melonds_opengl_resolution` `1`-`8` | | Values read from the DLL's strings |

melonDS DS has no GLES or Vulkan renderer; OpenGL is off in its Android builds (`docs/feasibility.md`, step 0).

### How other frontends do it

- RetroArch does not translate between APIs: its video driver *is* the core's API. Cores that need OpenGL core make it
  switch to its `glcore` driver (setting «Allow cores to change video drivers»; melonDS and Citra are among the cores
  listed as forcing `glcore`) [R1][R2]. EmuDome's compositor is wgpu (Vulkan/DX12), so a GL core needs readback
  or interop instead: this spike measures both.
- `retro-rs` (Rust) does GL with a surfman context and `glReadPixels` (step 0 note in `docs/feasibility.md`). No Rust
  frontend presenting a libretro core's GL or Vulkan frames through wgpu was found.

### wgpu 30 pieces that make it possible

- Vulkan: `Adapter::as_hal`, `wgpu_hal::vulkan::Adapter::device_from_raw` (wrap a device the core created),
  `Device::texture_from_raw` + `create_texture_from_hal` (wrap the core's `VkImage`), and
  `wgpu_hal::vulkan::Adapter::open_with_callback`, which lets the frontend **add device extensions** to wgpu's own
  device (used here for `VK_KHR_external_semaphore_win32`; the open question «`open_with_callback` or equivalent in
  wgpu 30: to verify» of the brief is answered: it exists and works).
- wgpu enables `VK_KHR_external_memory_win32` / `_fd` by itself when the GPU has them (seen in the extension list).
- OpenGL: wgpu's GL backend (`Backends::GL`, in wgpu's default features, already in the app's dependency tree) creates
  a desktop **core-profile** WGL context on a window it never shows (no `WS_VISIBLE`) on Windows, and an EGL context
  (GLES) on Linux/Android. `wgpu_hal::gles::Adapter::adapter_context().lock()` makes it current and returns a `glow`
  context: the spike uses it as the core's context, with no window code of its own.

## The prototype

`crates/hw-render-spike` (binary, `publish = false`, its own minimal libretro host; the shell is untouched; its own Cargo
workspace, outside the root one, so the checks never build it). The Vulkan
half existed (2026-11-18); this round added OpenGL.

```
cargo run --release --manifest-path crates/hw-render-spike/Cargo.toml -- <core> <content> --opt key=value ...
          [--frames N] [--png out.png] [--gl] [--no-interop]
```

- `host.rs`: with `--gl` the host accepts `OPENGL` / `OPENGL_CORE` contexts, answers `GET_PREFERRED_HW_RENDER` with
  OpenGL core, hands `get_current_framebuffer` / `get_proc_address` (from `gl.rs`), and records `SET_GEOMETRY` /
  `SET_SYSTEM_AV_INFO`. Without `--gl`, GL is refused as before.
- `gl.rs`: the context (wgpu-hal's, see above); function addresses through `wglGetProcAddress` then `opengl32.dll`
  (or `eglGetProcAddress`), picked at run time by which library loads, so no per-OS code; the framebuffer the core
  draws into (colour + depth/stencil, sized to the core's maximum geometry); `glReadPixels`; and the import of Vulkan
  memory and semaphores (`GL_EXT_memory_object_win32`, `GL_EXT_semaphore_win32`, `glTexStorageMem2DEXT`, entry points
  loaded by hand: `glow` does not wrap them).
- `interop.rs`: the compositor side for a GL core: wgpu's own Vulkan device opened with `open_with_callback`
  (+ `VK_KHR_external_semaphore_win32`), one exportable `VkImage` (dedicated, `OPAQUE_WIN32`) that GL uses as the
  framebuffer's colour texture, two exportable binary semaphores, the image wrapped as a wgpu texture, and a wgpu draw
  that flips and crops the GL frame. Per frame on the zero-copy path: GL waits `vk_done` (previous draw finished) →
  `retro_run` → GL signals `gl_done` with the image moved to `SHADER_READ_ONLY` → an empty Vulkan submit waits
  `gl_done` (Vulkan orders a semaphore wait before every later submission) → wgpu draw → an empty submit signals
  `vk_done`. On the readback path: `glReadPixels` → `queue.write_texture` → the same draw.
- `main.rs` `run_gl`: phases of 300 frames, unthrottled: (A) `retro_run` + `glFinish`; (B) readback + upload + draw +
  wait for the GPU; (C) zero copy + draw + wait; then 60 frames of B and of C with the drawn target read back (live
  pictures, PNG). The Vulkan phases are unchanged from 2026-11-18.
- No visible window, no focus taken: Vulkan has no surface; the WGL window is wgpu-hal's hidden one.
- Content: Craftus Reloaded 0.3 (MIT, 3DS homebrew, title screen) and Breaking Bad DS 1.0.6 (Apache-2.0, NDS
  homebrew, intro), the same as step 0, kept outside the repo. No Nintendo files. Cores: libretro nightlies of
  2026-11-19 (Azahar 2127.0-rc4, melonDS DS 1.4.0).
- No new crate in `Cargo.lock`: the spike adds `glow` 0.17 (MIT OR Apache-2.0 OR Zlib) as a direct dependency, already
  in the tree through wgpu-hal, and turns on wgpu-hal's `gles` feature (wgpu already builds it).

## Measured

Machine: AMD Ryzen 7 9700X, NVIDIA GeForce RTX 3060 (driver 617.42; GL 4.6.0 core profile, Vulkan 1.4), Windows 11,
release build. Unthrottled; each phase 300 frames (well under 2 s). Time per frame is wall time on the emulation
thread, which is also how long that thread is blocked (no separate CPU-time counter was taken). Two runs per row where
two numbers are given. The scenes are a title screen (3DS) and an intro (NDS): they measure the **render path and its
transfer cost**, not emulation speed in a real game.

### OpenGL: readback vs zero copy

FPS for the whole loop (core + transfer + wgpu draw + wait for the GPU). `glReadPixels` is the read alone.

| Core, resolution, frame | A: run + `glFinish` | B: readback + upload + draw | C: zero copy + draw | `glReadPixels` |
| --- | --- | --- | --- | --- |
| melonDS DS GL 1x, 256x384 | 647 / 613 fps | 522 / 490 fps | 678 / 654 fps | 0.32 / 0.35 ms |
| melonDS DS GL 2x, 512x768 | 632 fps | 437 fps | 652 fps | 0.50 ms |
| melonDS DS GL 4x, 1024x1536 | 588 / 578 fps | 354 / 350 fps | 600 / 594 fps | 0.81 / 0.82 ms |
| melonDS DS GL 8x, 2048x3072 | 419 / 417 fps | 168 fps | 401 fps | 2.54 ms |
| Azahar GL 1x, 400x480 | 501 / 474 fps | 378 / 367 fps | 379 / 400 fps | 1.33 / 1.41 ms |
| Azahar GL 2x, 800x960 | 327 fps | 271 fps | 326 fps | 1.86 ms |
| Azahar GL 4x, 1600x1920 | 163 / 151 fps | 146 / 142 fps | 186 / 184 fps | 3.44 / 3.56 ms |

- Zero copy costs about what the core alone costs (C ≈ A); readback adds 0.4 ms (melonDS 1x) to 3.6 ms (melonDS 8x)
  per frame, all of it blocking the emulation thread.
- One pass (melonDS 4x and 8x, back to back) measured B at 45 fps with `glReadPixels` at 11-12 ms and C at 258-294
  fps; the next pass gave the figures above. The cause was not found (the GPU was idle before and after); it shows the
  synchronous readback is the path most exposed to GPU contention.
- Pictures: both paths give 60 distinct frames out of 60 with the same content (colour counts equal, or within two for
  melonDS DS's animated intro); PNGs checked by eye, upright, and the Azahar one identical to its Vulkan render.
- `context_reset`: 60-308 ms (Azahar, which loads the game there), 267 ms (melonDS DS). GL adapter request: under 2 ms (the
  context itself is created inside `wgpu::Instance::new`, not timed); compositor Vulkan device: 66-410 ms.
- Framebuffer sizes follow the cores' maximum geometry: 1024x576 per 1x for melonDS DS (8192x4608 at 8x), and
  **7200x4800 for Azahar at every factor** (it announces its 18x maximum): 138 MB of VRAM for a 400x480 frame.

### Vulkan (Azahar): resolution scaling

| Resolution, frame | `retro_run` only | Run + zero-copy wgpu draw + wait | Run + draw + readback | Draw (+ readback) per frame |
| --- | --- | --- | --- | --- |
| 1x, 400x480 | 3299 / 5605 fps | 3530 / 3254 fps | 1954 / 1810 fps | 0.08-0.09 (0.19-0.20) ms |
| 2x, 800x960 | 2628 fps | 2642 fps | 912 fps | 0.14 (0.48) ms |
| 4x, 1600x1920 | 1331 / 1475 fps | 1922 / 1815 fps | 271 / 262 fps | 0.34 (1.77-1.81) ms |

«`retro_run` only» has no wait for the GPU, so it is not a frame time (Azahar submits asynchronously); the middle
column is the comparable one. Readback costs 5x the zero-copy draw at 4x.

### Same scene, Vulkan vs OpenGL (Azahar, 1x)

Whole loop with zero copy: Vulkan 3254-3530 fps (about 0.3 ms per frame), OpenGL 379-400 fps (about 2.6 ms). Same
picture. Both are far above 60 fps on this desktop; the factor of about 8 is a reason to pick Vulkan for Azahar, not a
measure of real games.

### Not measured

- Phones (A71): no access in these sessions. Whether 3DS emulation is fast enough on the phone stays the open
  question.
- Linux and Android runs of the OpenGL path (the code has no Windows-only `cfg`, but the zero-copy half uses Win32
  handles only; the Linux equivalents are the `_fd` extensions, not written).
- Real game scenes, save states with a hardware context, context loss/re-creation, a separate render thread.
- AMD and Intel GPUs (the machine's AMD iGPU is not exposed to Vulkan or used by WGL here).
- Validation layers: not installed on this machine; the GL/Vulkan layout and semaphore use is not validated.
- Async readback (pixel buffer objects, one frame of latency): not built.

## Recommendation per OS

| OS | 3DS (Azahar) | NDS (melonDS DS) |
| --- | --- | --- |
| **Windows** | Vulkan, core device = compositor device, zero copy (step 1 design in `docs/feasibility.md`). Readback only for screenshots and slot thumbnails | Software by default. Optional OpenGL upscaling: core-profile GL context + zero copy through `GL_EXT_memory_object_win32` / `GL_EXT_semaphore_win32` into wgpu's Vulkan device; readback as the fallback when the driver lacks the extensions |
| **Linux** (webview video path today) | Vulkan as on Windows; while the webview path is used, the frame must reach the CPU anyway: wgpu readback (0.2 ms at 1x, 1.8 ms at 4x) into the existing RGBA frame | Software by default. OpenGL: our own EGL context with `EGL_OPENGL_API` (desktop GL, surfaceless/pbuffer, no window) and `glReadPixels` straight into the RGBA frame; no Vulkan needed on the webview path. wgpu's GL backend gives GLES only there, so it cannot be the core's context |
| **Android** (`SurfaceView`, wgpu Vulkan) | Vulkan, zero copy, same code as desktop (Azahar's Android core asks for Vulkan) | Software only (no GLES renderer in melonDS DS) |

- In the product the GL context should live in `emudome-platform` (WGL on a hidden window / EGL), not on
  wgpu-hal's internal GL adapter: the spike borrows wgpu-hal's to avoid window code, but that context belongs to
  wgpu's GL backend and its API is not meant for a third party driving it.
- The environment side (`SET_HW_RENDER`, negotiation, interfaces, framebuffer callbacks) belongs in `emudome-core` as
  plain FFI; device creation, wrapping and interop next to `video.rs` (built that way: `hw_render/`,
  `gpu_interop.rs`, `hw_render.rs`).
- The settings engine offers `citra_graphics_api` only `Vulkan` and `Software`, and `melonds_render_mode` `opengl` only
  where a desktop GL context exists (shown disabled with the reason elsewhere: project rule).

## Estimated work

| Item | Size | Notes |
| --- | --- | --- |
| Vulkan `SET_HW_RENDER` in the shell (3DS) | M-L, 6-10 days | Environment side in `emudome-core`; `video.rs` takes the core's device; queue lock around every submit and present; two sync indices; `context_destroy`/`reset` on video restart; readback for the webview path, screenshots, thumbnails; save states with a hardware context |
| OpenGL for melonDS DS, readback only | S-M, 2-3 days | GL context in `emudome-platform` (WGL / EGL); framebuffer; `glReadPixels` into the RGBA frame (works for both video paths) |
| OpenGL zero copy (Windows; Linux `_fd`) | M, 3-5 days more | As in `gl.rs` / `interop.rs`, plus handle ownership, driver capability checks, fallback to readback |
| Validation and other GPUs | S, 1-2 days | Khronos validation layer run; AMD/Intel runs |

## Risks

1. **Phone speed** of 3DS emulation: unmeasured, decides whether 3DS ships on Android.
2. **Vulkan device features** given to Azahar's `create_device`: wgpu may use features the core did not enable;
   unvalidated. No longer applies in the shell: the app opens the device itself (below, «Vulkan in the shell»).
3. **GL/Vulkan interop correctness**: layouts and semaphores follow the `EXT_external_objects` spec as read, but no
   validation layer was run; the spike leaks the three exported Win32 handles per session (closed only at exit); a
   driver without the GL extensions needs the readback fallback (supported by NVIDIA here; AMD/Intel not checked).
4. **Framebuffer size**: Azahar announces 7200x4800 as its maximum at every factor; sizing to the current geometry and
   growing on `SET_GEOMETRY` saves VRAM, but needs every core to read `get_current_framebuffer` each frame (melonDS DS
   does by design [M1]; Azahar not checked).
5. **Readback under contention**: one pass showed `glReadPixels` 15x slower; the zero-copy path is the safe default
   wherever the video is native.
6. **GL state sharing**: on wgpu-hal's context the spike is the only GL user; in the product the core's context must
   stay separate from anything else drawn with GL.

## The host side, ready since #78

What the hardware-rendering work builds on (Trello #78); the GPU rendering itself is the next section:

- **Hardware-rendering calls** go to the session's `HwRenderHandler` (`crates/emudome-core/src/hw_render/`), told
  apart by their raw number (44 with the experimental flag is `SET_HW_SHARED_CONTEXT`, without it
  `SET_SERIALIZATION_QUIRKS`). `SoftwareOnly` refuses them all; `vulkan::VulkanHandler` (#46) accepts Vulkan.
- **Frames** go to the session's `VideoSink` (`video_sink.rs`) as `VideoFrame::Cpu` (bytes) or `VideoFrame::Gpu`
  (`GpuFrame`: the core's image and the semaphores to wait for, raw handles as integers). A GPU sink must keep
  `Shared::frame` filled (readback on demand) for screenshots, slot thumbnails and the webview path.
- **One GPU for the whole run** (`GpuService`, `app/src-tauri/src/video.rs`): surfaces attach and detach, the device
  stays, so Android's background / resume no longer destroys the device under a running core.
- **wgpu-hal interop** lives in `app/src-tauri/src/gpu_interop.rs` only (`VulkanInterop` for the device's raw
  handles and a core image as a texture).
- **Callbacks from the core's own threads** find their context (one static slot per loaded core, not a
  `thread_local`), and a `need_fullpath` core gets the path only: a 3DS game is never read into memory.

## Vulkan in the shell (#46)

- **One device, the app's.** `GpuService` opens its Vulkan device with wgpu's extensions and features plus what
  Azahar uses when the GPU has them (`gpu_interop::open_device_for_cores`: `geometryShader`, `logicOp`,
  `samplerAnisotropy`, `fragmentStoresAndAtomics`, `shaderClipDistance`; `VK_EXT_custom_border_color`,
  `extended_dynamic_state`, `index_type_uint8`, `pipeline_creation_cache_control`, `fragment_shader_interlock`,
  `external_memory_host`, `VK_KHR_fragment_shader_barycentric`, `image_format_list`, `timeline_semaphore`, with their
  feature structs). Azahar's libretro build picks what to use from what the GPU *supports*, not from what the device
  enabled, so the device enables everything it can see. The core is handed this device in
  `retro_hw_render_interface_vulkan`; its negotiation interface is read for the API version it wants
  (`get_application_info`: 1.1) and `create_device` is never called, so the risk «Vulkan device features given to
  Azahar's `create_device`» above no longer applies.
- **Frames.** `video_refresh(RETRO_HW_FRAME_BUFFER_VALID)` → the handler's `GpuFrame` (the image from `set_image`) →
  `hw_render::GpuFrameSink`: a raw barrier (the core's colour writes → fragment reads), a draw of the core's image into
  a game texture EmuDome owns, a raw barrier ordering the core's next frame after that read, then a readback into
  `Shared::frame`, all on the emulation thread inside the callback (the submit under the queue lock). The readback
  uses two buffers in turn and waits for the GPU at most 3 ms (`READBACK_WAIT`): a frame not ready then is published
  by the next call, one frame late, so a GPU busy with another program never holds the emulation thread for long.
  The core's image is wrapped anew each frame: Azahar destroys and recreates it on every state load, possibly with
  the same handle.
- **Queue lock.** The core takes it around its `vkQueueSubmit` (`lock_queue` / `unlock_queue`); the shell around
  every submit and present (the renderer's paint and present, librashader's preset load, the copy).
- **Software fallback.** `CoreProfile::hw_render.renderer(vulkan)` picks `citra_graphics_api`: `Vulkan` when the app's
  device is on Vulkan (opened for cores), else `Software`, reported as `hwRenderFallback` (`emu://health`).
- **Not yet:** the native renderer sampling the game texture directly (every frame is read back today); OpenGL.

## Sensors (plan, not implemented)

Who needs them: mGBA (GBA cartridges with a tilt sensor or gyro, the solar sensor: `mgba_solar_sensor_level` has a
`sensor` value that reads the light level) and Azahar (3DS accelerometer and gyroscope); melonDS DS none.

1. **libretro side**: answer `RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE` (25) with `retro_sensor_interface`: one
   `set_sensor_state(port, action, rate)` and one `get_sensor_input(port, id)` per callback slot, like the other
   callbacks. `get_sensor_input` reads the frozen input (`input::Snapshot`, taken at `input_poll`), never live values.
2. **State**: `InputState` gains the seven readings (accelerometer x/y/z, gyroscope x/y/z, illuminance) as `f32` bits
   in atomics, and the set of sensors the core switched on (`set_sensor_state`), which the shell reads to start and
   stop the device's sensors (battery on Android).
3. **Sources**, in order: the phone's own sensors on Android (through `emudome-platform`, SensorManager), the
   controller's gyro and accelerometer on desktop (SDL3 gamepad sensors: DualShock 4, DualSense, Switch Pro), else
   tilt from the right stick (a pure mapping, unit-tested). Light: the phone's light sensor, else the `consoles.solar`
   slider as today.
4. **Settings**: one choice per console for the source (device / controller / right stick), from the Rust schema;
   rows disabled with the reason where the platform has no such sensor.
5. **Tests**: the sensor interface answered per slot, the stick-to-tilt mapping, the snapshot; on a device, a tilt
   cartridge test ROM with a free licence.

## Sources

- [L1] libretro.h (`retro_hw_render_callback`, `SET_HW_RENDER`, `SET_HW_SHARED_CONTEXT`, `RETRO_HW_FRAME_BUFFER_VALID`):
  https://github.com/libretro/RetroArch/blob/master/libretro-common/include/libretro.h
- [L2] libretro_vulkan.h (negotiation interface, `retro_hw_render_interface_vulkan`):
  https://github.com/libretro/RetroArch/blob/master/libretro-common/include/libretro_vulkan.h
- [A1] Azahar libretro frontend code (OpenGL core 4.3 / GLES 3.2 request, `SetHWSharedContext`, Vulkan negotiation,
  `context_reset` loading the game): https://github.com/azahar-emu/azahar/blob/master/src/citra_libretro/citra_libretro.cpp
- [A2] Azahar core options (`graphics_api` values, `resolution_factor` 1-18):
  https://github.com/azahar-emu/azahar/blob/master/src/citra_libretro/core_settings.cpp
- [M1] melonDS DS OpenGL renderer (OpenGL core 3.2 / 4.3 compute, depth, `bottom_left_origin`, «don't cache it; some
  frontends hand out a different one each frame», software fallback):
  https://github.com/JesseTG/melonds-ds/blob/main/src/libretro/render/opengl.cpp ; option keys:
  https://github.com/JesseTG/melonds-ds/blob/main/src/libretro/config/constants.hpp
- [R1] libretro blog, «Changing behavior of "gl" and "glcore" video drivers» (2020-10-13): kronos, citra,
  beetle-psx-hw, mupen64plus_next (Linux) and melonDS «will always try to force the "glcore" driver»; setting «Allow
  cores to change video drivers», on by default: https://www.libretro.com/?p=49082
- [R2] libretro blog, RetroArch 1.7.7, the `glcore` driver (OpenGL 3.2 core, chosen under Settings > Drivers > Video):
  https://www.libretro.com/?p=47030
- [K1] Khronos `EXT_external_objects`, `EXT_external_objects_win32` (memory objects, semaphores, layout enums; values
  checked against `gl.xml` in `khronos_api` 3.1.0): https://registry.khronos.org/OpenGL/extensions/EXT/EXT_external_objects.txt
- wgpu 30 / wgpu-hal 30 source (`gles/wgl.rs` hidden window and core-profile context, `vulkan/adapter.rs`
  `open_with_callback` and external memory extensions), read from the crates in the cargo registry.
