# Downloads the libretro cores EmuDome uses into the repo's cores/ folder.
# Usage (PowerShell, from the repo root):  ./scripts/fetch-cores.ps1
# The cores are not in the repo: they are separate projects with their own licenses (mGBA: MPL 2.0).

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "cores"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$base = "https://buildbot.libretro.com/nightly/windows/x86_64/latest"
$cores = @("mgba")   # Phase 2: "melonds_ds"   Phase 3: "azahar"

foreach ($core in $cores) {
    $zip = Join-Path $env:TEMP "$core`_libretro.dll.zip"
    Write-Host "Downloading $core..."
    Invoke-WebRequest -Uri "$base/$core`_libretro.dll.zip" -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $dest -Force
    Remove-Item $zip
}
Write-Host "Done. Cores in $dest"
Get-ChildItem $dest -Filter *.dll | ForEach-Object { Write-Host " - $($_.Name)" }
