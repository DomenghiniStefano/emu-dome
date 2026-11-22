# Builds the melonDS DS libretro core (NDS) for Android from the source cores.lock pins (v1.4.0 and
# every library its CMake would fetch, each pinned to a commit and a sha256-checked archive), and
# installs it into cores/android/<abi>/ only if its sha256 is the one cores.lock pins.
# EmuDome builds it because upstream's Android release is linked with 4 KB pages, which Google Play
# and 16 KB-page devices refuse (docs/release.md, "Android: 16 KB pages"); Windows and Linux still
# use upstream's release files.
# Usage (PowerShell, from the repo root):
#   ./scripts/build-melondsds.ps1 [-Platform android-arm64-v8a] [-Jobs 2] [-WorkDir <short path>]
# Needs Node, git (melonDS DS's CMake looks for it) and the Android NDK (cores.lock names the version
# the pin came from). CMake: CMAKE, else the Android SDK's, else PATH. Every step: scripts/core-build.mjs.
# fetch-cores-android.ps1 downloads the pinned build from EmuDome's GitHub release instead, and calls
# this same build only when that download fails or with -Build.

param(
    [ValidateSet("android-arm64-v8a")]
    [string]$Platform = "android-arm64-v8a",
    # Parallel compile jobs: 2 by default, so the build leaves the machine usable.
    [int]$Jobs = 2,
    # Download and build folder (default <temp>\emudome-cb). Keep it short.
    [string]$WorkDir = ""
)

# Not "Stop": CMake writes its warnings on stderr, and Windows PowerShell 5.1 would turn them into a
# terminating error whenever the output is redirected. The exit code says whether the build worked.
$ErrorActionPreference = "Continue"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "build", "--platform", $Platform, "--cores", "melondsds", "--jobs", "$Jobs")
if ($WorkDir -ne "") { $arguments += @("--work-dir", $WorkDir) }
& node @arguments
exit $LASTEXITCODE
