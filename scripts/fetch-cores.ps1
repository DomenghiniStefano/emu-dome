# Puts the libretro cores into the repo's cores/ folder (Windows x86_64), pinned by cores.lock:
# every archive and file is sha256-checked, and a mismatch deletes the core's files and fails.
# Cores pinned as upstream builds are downloaded. mGBA, which EmuDome builds from its pinned source,
# is downloaded from EmuDome's own GitHub release of that build (cores.lock `built.release`); only
# when that fails, or with -Build, it is built here (as scripts/build-mgba.ps1 does: Visual Studio
# 2022 Build Tools, the Android NDK's clang-cl, git).
# Usage (PowerShell, from the repo root):  ./scripts/fetch-cores.ps1 [-Cores mgba,melondsds,azahar|all] [-Build]
# Default: the cores cores.lock ships. Needs Node (the logic is scripts/cores.mjs, shared by every
# platform). The cores are separate projects with their own licences: see THIRD_PARTY.md.

param(
    # Core ids from cores.lock: mgba (GB, GBC, GBA), melondsds (NDS), azahar (3DS); "all" for every one.
    [string[]]$Cores = @(),
    # Build the cores cores.lock builds from source (mGBA) instead of downloading them, even when they
    # are in place: to reproduce the pinned sha256, or to make a new pin.
    [switch]$Build
)

# Not "Stop": a build writes warnings on stderr, which Windows PowerShell 5.1 turns into a
# terminating error when the output is redirected. The exit code says whether it worked.
$ErrorActionPreference = "Continue"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "fetch", "--platform", "windows-x86_64")
if ($Cores.Count -gt 0) { $arguments += @("--cores", ($Cores -join ",")) }
if ($Build) { $arguments += "--build" }
& node @arguments
exit $LASTEXITCODE
