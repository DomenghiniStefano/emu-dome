# Bench runs (Trello #19)

Raw reports of the first bench runs on Windows, taken on the night of 2026-11-16 to 17
(Rome time) with the homebrew test ROM, release build, one PC (RTX 3060, 240 Hz display).
They are the measures behind the matrix of Trello #19; what they say, and what is still open,
is in [`../bench-windows.md`](../bench-windows.md).

One file per run, `runs__<run>.json`, as the bench mode wrote it, plus a `launcher` block added by
the script that started the runs (process id, whether the window took the focus, CPU of the whole
process tree including WebView2). Run names:

| Prefix | What varies |
| --- | --- |
| `v-` | video: backend (`vulkan`, `dx12hwnd`, `dx12visual`, `gl`, `webview`), present mode, frame latency (`l1`, `l2`) |
| `r2-`, `r3-` | repeats of the most promising `v-` runs |
| `m-` | video with the quick menu opened 5 s after the first frame (`EMUDOME_BENCH_MENU=1`) |
| `a-` | audio: queue target (`t30` = 30 ms), device buffer, rate and channels, with the 440 Hz test tone |
| `try-` | first trials of the bench itself |

`video-tables.md` is the `v-` and `m-` runs as tables.

Sanitized for publication: the report paths (`tuning.bench.out`, `tuning.data_dir`) pointed into a
temporary folder of the PC that ran them; they now read `<bench-dir>\...`. The WebView2 profile
folders the runs created were never kept. Nothing else was changed (the files were only re-indented).

## Running a bench again

Bench mode is off unless `EMUDOME_BENCH` is set. With it, the app opens the game given on the
command line, waits for the first frame and a warm-up, measures for the given seconds, writes the
JSON report and exits; the window does not take the focus. The knobs are listed in
`crates/emudome-core/src/tuning.rs`. Example (PowerShell, release build):

```powershell
$env:EMUDOME_BENCH = '30'                       # seconds measured
$env:EMUDOME_BENCH_OUT = "$PWD\bench\v-dx12-fifo-l2.json"
$env:EMUDOME_DATA_DIR = "$PWD\bench\data"       # keeps every data file (saves, settings, library, webview profile) out of the real ones
$env:WGPU_BACKEND = 'dx12'                      # read by wgpu itself
$env:EMUDOME_PRESENT_MODE = 'fifo'
$env:EMUDOME_FRAME_LATENCY = '2'
.\target\release\emudome.exe tools\test-rom\emudome-test.gba
```

Keep the bench folder outside the repo: the data folder fills with the webview's profile.
The launcher script and the table generator used for these runs were not part of the repo.
