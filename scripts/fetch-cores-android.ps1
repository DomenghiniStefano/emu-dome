# Downloads the libretro cores for Android into cores/android/<abi>/, pinned by cores.lock: every
# archive and file is sha256-checked, and a mismatch deletes the core's files and fails. The Gradle
# build (crates/emudome-platform/android/build.gradle.kts) packages only the files cores.lock lists.
# Usage (PowerShell, from the repo root):
#   ./scripts/fetch-cores-android.ps1 [-Cores mgba,melondsds,azahar|all] [-Abi arm64-v8a]
# Default: the cores cores.lock ships. Saved as lib<core>_libretro_android.so (Android packages only
# lib*.so; the name emudome-platform looks for). Needs Node (the logic is scripts/cores.mjs).

param(
    [string[]]$Cores = @(),
    # The ABIs cores.lock pins: arm64-v8a.
    [string]$Abi = "arm64-v8a"
)

$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "fetch", "--platform", "android-$Abi")
if ($Cores.Count -gt 0) { $arguments += @("--cores", ($Cores -join ",")) }
& node @arguments
exit $LASTEXITCODE
