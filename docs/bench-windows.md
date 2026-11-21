# Bench on Windows (Trello #19)

The bench mode (`app/src-tauri/src/bench.rs`, knobs in `app/src-tauri/src/tuning.rs`) and the
first raw runs (`docs/bench-runs/`, one PC: RTX 3060, 240 Hz display, release build, test ROM).
This work was paused unfinished; this page says what the runs show, what was decided from them
(nothing yet) and what of the original work-in-progress was not brought over.

## What the runs show

Video (`docs/bench-runs/video-tables.md`, 30 s per run):

- Every configuration keeps the core at 59.7 FPS; the differences are in how evenly new game
  frames reach the screen and in CPU.
- DX12 with frame latency 2 (`AutoVsync` or `Fifo`, both presentation systems) is the most even
  vsync path: new-frame interval p95 about 16.8-17.0 ms and 23-34 intervals off by more than 25 %,
  against p95 about 20.5-20.8 ms and 42-139 off with DX12 latency 1 or Vulkan (either latency).
  CPU of the process: DX12 7-9 % of one core, Vulkan 10-11 %, GL 50-56 %.
- `Immediate` and `Mailbox` present hundreds to thousands of times per second and cost
  19-97 % of one core: not worth it at 60 FPS. GL does not offer `Mailbox`.
- The webview path presents as evenly as Vulkan, with 8-10 % CPU in the app process but
  44-49 % in the whole process tree (the WebView2 processes).
- `m-` runs (quick menu open): the menu pauses the game, so they only show that presenting
  under the menu continues; the core FPS there is not a measure.

Audio (`runs__a-*.json`, 440 Hz test tone, queue target 30 ms, default device at 44.1 or 48 kHz,
2 or 8 channels):

- The requested device buffer (256, 512, 1024 frames, default) does not change the callback size
  in practice (441 or 480 frames, i.e. 10 ms): the shared-mode device decides.
- Queue fill stays at about 29 ms on average, device latency 32-33 ms.
- Out of 15 runs, 5 had one underrun (20-294 frames, 1-2 audible breaks of the tone); no pattern
  by buffer, rate or channels. The runs are too short and too few to rank the settings.

## Proposed, not adopted

Both change the app's normal behaviour, so they wait for the owner's decision; the knobs reproduce
them without code changes:

- DX12 as the first backend on Windows (`WGPU_BACKEND=dx12`); today wgpu picks Vulkan.
- Frame latency 2 (`EMUDOME_FRAME_LATENCY=2`); today the app uses 1 (`DEFAULT_FRAME_LATENCY`).

## Not ported from the original work-in-progress

- `emudome_platform::preferred_gpu_backends()` (DX12 first on Windows, every backend as a fallback)
  and the frame latency default of 2: they changed the default behaviour (see above).
- A `frames_run` counter in the core's `Shared`: `Stats::frames` counts the same frames now, and the
  bench reads that.
- The launcher script that started the runs and the script that built `video-tables.md`: they were
  never in the repo. A rerun needs the bench mode only (`docs/bench-runs/README.md`).

## Still open for #19

- Repeat the audio runs longer (minutes, not 30 s) and with the game's own audio, to tell the
  underruns apart from noise.
- Runs on other PCs and on Linux / Android (the bench reads CPU time there too, untested).
- Decide the two proposals above.
