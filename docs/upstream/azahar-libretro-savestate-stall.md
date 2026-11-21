# Upstream report draft: Azahar libretro core, a save state stops the emulation for ~0.25 s

Draft for the owner to file on https://github.com/azahar-emu/azahar/issues (Trello #98). Not posted anywhere yet. The
text below the line is the issue as it would be filed; the notes after it are for us only. It is a performance
request, not a bug: file it only if the owner wants to.

---

## Title

libretro: `retro_serialize_size` takes ~0.25 s (New 3DS) on the emulation thread, so every save state drops ~15 frames

## Version

- Azahar 2126.2, commit `219d743a1e698101fe1ab8bef43e6563126c3c27`
- Build: the release's libretro core for Windows, `azahar-libretro-windows-x86_64-2126.2.zip` (`azahar_libretro.dll`)
- Windows 11, AMD Ryzen 7 9700X, NVIDIA RTX 3060 (driver 617.42)

## Setup

- A libretro frontend calling `retro_serialize_size` then `retro_serialize` between two `retro_run` calls, on the
  thread that runs the core (as the API requires).
- Content: Craftus Reloaded 0.3 (MIT, https://github.com/RSDuck/craftus_reloaded), `craftus_reloaded.3dsx`, in a
  generated world. Other options at their defaults (`citra_is_new_3ds` = `New 3DS`).

## Observed

Time of `retro_serialize_size` + `retro_serialize`, measured around the two calls (release build of the frontend,
states of ~760 KB):

| Renderer | System model | Time per save state |
| --- | --- | --- |
| Vulkan | New 3DS | 233-305 ms (10 saves) |
| Software | New 3DS | 231-237 ms (5 saves) |
| Vulkan | Old 3DS | 155-159 ms (3 saves) |

The frames right after a save take their usual 1-3 ms on Vulkan: the rasterizer cache flush and drop in
`System::serialize` (`gpu->ClearAll(true)`) costs nothing visible here. The time does not depend on the renderer and
scales with the emulated memory (New 3DS 256 MB FCRAM, Old 3DS 128 MB), so it looks like `SaveStateBuffer()`
itself: the Boost archive of the whole system into a `std::ostringstream`, the copy out of it (`sstream.str()`), and
`CompressDataZSTDDefault` over the result. We have not split the time between these three.

At 60 fps that is 15-18 frames with no picture and no audio for every save state, and for every automatic save a
frontend makes (periodic auto saves, rewind is out of reach).

## Expected / suggestions

A save state well under a frame, or at least a few frames. Ideas, none tried:

- The libretro entry points could skip the zstd pass: libretro frontends compress state files themselves (RetroArch
  does), and the state stays a valid input to `retro_unserialize` if the header marks it uncompressed.
- Serialize into a buffer reserved once at the state's last size, instead of a growing `std::ostringstream` plus the
  copy made by `str()`.
- Memory blocks written as raw bytes, untouched pages skipped.

---

## Notes for us (not part of the issue)

- Where the time goes, from the source at 219d743: `retro_serialize_size` (`src/citra_libretro/citra_libretro.cpp`)
  drains the kernel's async operations (`DrainAsyncOperations`), then calls `System::SaveStateBuffer()`
  (`src/core/savestate.cpp`) and keeps the result; `retro_serialize` only copies it. So the shell calls the slow
  part exactly once per save already (`LoadedCore::save_state_into`), and nothing on our side waits for the GPU.
- The shell cannot move it: only the emulation thread may call the core. What the shell does about it (Trello #98):
  the audio output refills its queue after the gap and fades back in, instead of crackling for seconds; the pacing
  starts over after the stall without a burst of catch-up frames.
- `saves.autoq` defaults to 5 minutes: on 3DS that is one ~0.25 s hitch every 5 minutes. The «Annulla» of a load
  serializes too (one more stall before each load), but loads are refused on Vulkan with title ID 0 (#93).
- Measured with `headless_gpu --timing` and the session's `quick save: ... taken in` log line; the full numbers are in
  `docs/fase0-esito.md`, entry «3DS quick-save stall and 2× in the world».
