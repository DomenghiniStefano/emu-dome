# Upstream report draft: Azahar libretro core, Vulkan, loading a state of content with title ID 0 crashes

Draft for the owner to file on https://github.com/azahar-emu/azahar/issues (Trello #93). Not posted anywhere yet. The
text below the line is the issue as it would be filed; the notes after it are for us only.

---

## Title

libretro (Vulkan): loading a save state of homebrew (title ID 0) crashes with an access violation in the pipeline cache

## Version

- Azahar 2126.2, commit `219d743a1e698101fe1ab8bef43e6563126c3c27`
- Build: the release's libretro core for Windows, `azahar-libretro-windows-x86_64-2126.2.zip` (`azahar_libretro.dll`,
  sha256 `aa353985a1d3fd2d34593b419b21b82a62c58a945f3b11601bfeafe32e0b1813`)
- Windows 11, AMD Ryzen 7 9700X, NVIDIA RTX 3060 (driver 617.42, Vulkan 1.4)

## Setup

- A libretro frontend that provides a Vulkan context (`RETRO_HW_CONTEXT_VULKAN`).
- Core option `citra_graphics_api` = `Vulkan`; the other options at their defaults.
- Content: free homebrew in `.3dsx` form, for example Craftus Reloaded 0.3 (MIT,
  https://github.com/RSDuck/craftus_reloaded), `craftus_reloaded.3dsx`, sha256
  `7beb2eb124342cbc41980f401ececba3eaa53fdc33a5da92f41517544d1e0f47`. Any content that runs with title ID 0 should
  do; `.3dsx` files always do.

## Steps to reproduce

1. Load the core with Vulkan and start the `.3dsx`.
2. Let it run for a few seconds (the title screen is enough).
3. Save a state (`retro_serialize`).
4. Load that state (`retro_unserialize`).

## Expected

The game continues from the state, as it does with the software renderer.

## Observed

The process crashes at the first `retro_run` after the load: access violation in `azahar_libretro.dll` at offset
`+0x385f5d`, reading address `0x650`, on the thread that calls `retro_run`. It happens every time.

The same crash also happens at boot, without any state, when `citra_use_disk_shader_cache` is `disabled`.

With `citra_graphics_api` = `Software` the same state loads correctly.

## Suspected cause

From reading `src/video_core/renderer_vulkan/vk_pipeline_cache.cpp` (not debugged): `PipelineCache::SwitchCache`
returns early when the new title ID equals the current program ID. For content with title ID 0 that is always true, so
the switch never runs, and the driver pipeline cache it would create is never created. After a state load (which
rebuilds the renderer) or with the disk shader cache disabled, nothing else creates it, and the first pipeline build
dereferences it while it is still null. Content with a non-zero title ID takes the other branch, so retail games
should not be affected (not verified: we have no free content with a title ID to test with).

## Suggested fix

Make sure the driver pipeline cache exists whatever the title ID: either create it when the renderer is built, before
any `SwitchCache` call, or let `SwitchCache` create it when it is missing even when the title ID does not change
(skipping only the reload of the disk cache). A null check before the cache is used would at least turn the crash into
a slower first frame.

---

## Notes for us (not part of the issue)

- Measured on 2026-11-21 with our own headless host (`headless_gpu`, a minimal libretro frontend), see
  `docs/fase0-esito.md`, entry «3DS on Azahar with Vulkan through the shell's host». It also crashed with the shell's own
  pipeline copy turned off, so it is not the frontend's frames. It was **not** tried in RetroArch: if the owner has
  RetroArch at hand, trying the steps there first makes the report stronger («any libretro frontend»).
- The crash offset and address are those of the build pinned in `cores.lock`; a newer build will have other offsets.
- Before filing, check against the 2126.2 source that the file path and the `PipelineCache::SwitchCache` name are
  exact (our log records `vk_pipeline_cache.cpp` and `SwitchCache`; the folder and class name were written from the
  usual layout of the Citra code base), and search the issue tracker for an existing report.
- Until Azahar fixes it, EmuDome refuses every state load and rewind for such content on Vulkan
  (`core_profile::state_load_block`, `stateLoadUnsupported`). When a fixed version is pinned, remove the rule and this
  line in `docs/known-gaps.md`.
