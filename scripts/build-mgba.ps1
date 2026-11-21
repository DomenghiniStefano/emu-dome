# Builds the mGBA libretro core (GB, GBC, GBA) from the source cores.lock pins, and installs it into
# cores/ (Windows) or cores/android/<abi>/ (Android) only if its sha256 is the one cores.lock pins.
# EmuDome builds mGBA itself because no upstream publishes a fixed libretro build of it (Trello #88).
# Usage (PowerShell, from the repo root):
#   ./scripts/build-mgba.ps1 [-Platform windows-x86_64|android-arm64-v8a] [-Jobs 2] [-WorkDir <short path>]
# Needs Node, git, and for Windows Visual Studio 2022 (or its Build Tools) with the C++ tools, for
# Android the NDK (cores.lock names the version the pin came from). CMake: CMAKE, else PATH, else
# the one inside Visual Studio / the Android SDK. Every step and toolchain: scripts/core-build.mjs.
# fetch-cores.ps1 / fetch-cores-android.ps1 call the same build when mGBA is not in place.

param(
    [ValidateSet("windows-x86_64", "android-arm64-v8a")]
    [string]$Platform = "windows-x86_64",
    # Parallel compile jobs: 2 by default, so the build leaves the machine usable.
    [int]$Jobs = 2,
    # Download and build folder (default <temp>\emudome-cb). Keep it short: MSBuild stops at 260 characters.
    [string]$WorkDir = ""
)

# Not "Stop": CMake writes its warnings on stderr, and Windows PowerShell 5.1 would turn them into a
# terminating error whenever the output is redirected. The exit code says whether the build worked.
$ErrorActionPreference = "Continue"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "build", "--platform", $Platform, "--cores", "mgba", "--jobs", "$Jobs")
if ($WorkDir -ne "") { $arguments += @("--work-dir", $WorkDir) }
& node @arguments
exit $LASTEXITCODE
