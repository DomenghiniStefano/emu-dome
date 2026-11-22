# Puts the libretro cores for Android into cores/android/<abi>/, pinned by cores.lock: every
# archive and file is sha256-checked, and a mismatch deletes the core's files and fails. Cores
# pinned as upstream builds are downloaded. mGBA and melonDS DS, which EmuDome builds for Android from
# their pinned source, are downloaded from EmuDome's own GitHub releases of those builds (cores.lock
# `built.release`); only when that fails, or with -Build, they are built here (as
# scripts/build-mgba.ps1 -Platform android-arm64-v8a and scripts/build-melondsds.ps1 do: the Android
# NDK and git). The Gradle build (crates/emudome-platform/android/build.gradle.kts)
# packages only the files cores.lock lists.
# Usage (PowerShell, from the repo root):
#   ./scripts/fetch-cores-android.ps1 [-Cores mgba,melondsds,azahar|all] [-Abi arm64-v8a] [-Build]
# Default: the cores cores.lock ships. Saved as lib<core>_libretro_android.so (Android packages only
# lib*.so; the name emudome-platform looks for). Needs Node (the logic is scripts/cores.mjs).

param(
    [string[]]$Cores = @(),
    # The ABIs cores.lock pins: arm64-v8a.
    [string]$Abi = "arm64-v8a",
    # Build the cores cores.lock builds from source (mGBA, melonDS DS) instead of downloading them, even when they
    # are in place: to reproduce the pinned sha256, or to make a new pin.
    [switch]$Build
)

# Not "Stop": a build writes warnings on stderr, which Windows PowerShell 5.1 turns into a
# terminating error when the output is redirected. The exit code says whether it worked.
$ErrorActionPreference = "Continue"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "fetch", "--platform", "android-$Abi")
if ($Cores.Count -gt 0) { $arguments += @("--cores", ($Cores -join ",")) }
if ($Build) { $arguments += "--build" }
& node @arguments
exit $LASTEXITCODE
