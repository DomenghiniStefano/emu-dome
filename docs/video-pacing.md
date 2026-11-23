# Video pacing: present-driven or audio-driven (spike note, Trello #79)

Status: a written spike, nothing implemented beyond the first step below. Which way to go is not decided yet.

## What the native renderer does now (#79)

The emulation thread runs at the core's own rate (59.7275 Hz for GBA, 59.8261 Hz for NDS) from its own timer
(`session.rs` and `session/emu_loop.rs`, sleep to the next frame). The video thread presents **only when something changed**: a new game frame
(`Shared::frame_wake`), a resize, a new surface, another game, the inset, the effect. With nothing new it sleeps
(`Wake`, at most 100 ms); a failed present waits 8 ms before the next try; a minimized window (0 × 0) draws nothing.

So every game frame is presented once and a paused game, a still menu or a minimized window cost no GPU work. What it
does not fix: the two clocks still differ. At 60 Hz with Fifo, a 59.73 Hz game misses one vsync every ~3.7 s (one
frame shown twice: a small judder on a scrolling game); at 120 / 144 Hz the frames are shown 2 or 2-3 vsyncs each,
unevenly. Under Mailbox / Immediate the present no longer blocks on vsync, but there is nothing to present between
game frames, so the old full-core spin is gone.

The present mode is the `video.vsync` setting: on, `AutoVsync` (FifoRelaxed or Fifo); off, `AutoNoVsync` (Immediate,
else Mailbox, else Fifo). It changes only how each frame is presented, never the emulation thread's timer.

## The two ways to make the clocks agree

**Present-driven (video sync):** the display's vsync is the master clock. The core is run once per vsync (at 60 Hz:
the game runs 0.46 % fast for GBA) and the audio is resampled to follow (the ±0.5 % fill-level correction in
`audio.rs` already covers 0.46 %). RetroArch's default. Smooth scrolling at 60 Hz; needs the refresh rate (wgpu does
not give it: the OS does, per monitor, and it changes when the window moves), a variable-refresh display, or a rate
far from the game's (120 / 144 Hz: run on every 2nd vsync and the speed is off by ~0.5 %, or frame-pacing logic). On
a 50 Hz or odd-rate display the game would run at the wrong speed: needs a fallback to the next one.

**Audio-driven (audio sync):** the audio device's clock is the master: the emulation thread runs a frame whenever the
ring has room for it (blocking on the ring instead of a timer). Exact game speed and pitch, no audio correction;
video stays as today (each frame presented once, the occasional repeated vsync remains). The ring and its fill level
(`emudome_core::audio`) are already the right shape for it; the emulation thread would wait on the fill level, with
the timer as the fallback when there is no audio device or the game is muted / at another speed.

## Suggestion

Keep today's timer (it is the fallback of both). Try audio-driven first: it is local to `session.rs` and the ring, it
removes the ±0.5 % resampling drift, and it does not need the display's rate. Present-driven is the bigger win for
smooth scrolling on 60 Hz desktops and needs the refresh rate per monitor from `emudome-platform`: a second spike, after
a measurement of how often the repeated frame is visible in play (the bench's `video` section counts repeated and
skipped game frames per present).
