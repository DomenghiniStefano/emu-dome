# Downloads the libretro cores into the repo's cores/ folder (Windows x86_64), pinned by cores.lock:
# every archive and file is sha256-checked, and a mismatch deletes the core's files and fails.
# Usage (PowerShell, from the repo root):  ./scripts/fetch-cores.ps1 [-Cores mgba,melondsds,azahar|all]
# Default: the cores cores.lock ships. Needs Node (the logic is scripts/cores.mjs, shared by every
# platform). The cores are separate projects with their own licences: see THIRD_PARTY.md.

param(
    # Core ids from cores.lock: mgba (GB, GBC, GBA), melondsds (NDS), azahar (3DS); "all" for every one.
    [string[]]$Cores = @()
)

$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "cores.mjs"
$arguments = @($script, "fetch", "--platform", "windows-x86_64")
if ($Cores.Count -gt 0) { $arguments += @("--cores", ($Cores -join ",")) }
& node @arguments
exit $LASTEXITCODE
