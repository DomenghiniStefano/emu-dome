# Puts the libretro cores into the repo's cores/ folder (Windows x86_64), pinned by cores.lock:
# every archive and file is sha256-checked, and a mismatch deletes the core's files and fails.
# Cores pinned as upstream builds are downloaded; mGBA is built from its pinned source (as
# scripts/build-mgba.ps1 does: Visual Studio 2022 Build Tools, the Android NDK's clang-cl, git).
# Usage (PowerShell, from the repo root):  ./scripts/fetch-cores.ps1 [-Cores mgba,melondsds,azahar|all]
# Default: the cores cores.lock ships. Needs Node (the logic is scripts/cores.mjs, shared by every
# platform). The cores are separate projects with their own licences: see THIRD_PARTY.md.

param(
    # Core ids from cores.lock: mgba (GB, GBC, GBA), melondsds (NDS), azahar (3DS); "all" for every one.
    [string[]]$Cores = @()
)

# Not "Stop": a build writes warnings on stderr, which Windows PowerShell 5.1 turns into a
# terminating error when the output is redirected. The exit code says whether it worked.
$ErrorActionPreference = "Continue"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "fetch", "--platform", "windows-x86_64")
if ($Cores.Count -gt 0) { $arguments += @("--cores", ($Cores -join ",")) }
& node @arguments
exit $LASTEXITCODE
