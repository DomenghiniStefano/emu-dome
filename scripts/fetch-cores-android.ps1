# Downloads the libretro cores for Android into cores/android/<abi>/, the jniLibs folder
# packaged in the APK (see crates/emudome-platform/android/build.gradle.kts).
# Usage (PowerShell, from the repo root):  ./scripts/fetch-cores-android.ps1 [-Abi arm64-v8a]
# The cores are not in the repo: they are separate projects with their own licenses (mGBA: MPL 2.0).
#
# Android only packages native libraries named lib*.so, so `mgba_libretro_android.so`
# is saved as `libmgba_libretro_android.so` (the name emudome-platform looks for).

param([string]$Abi = "arm64-v8a")

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "cores/android/$Abi"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$base = "https://buildbot.libretro.com/nightly/android/latest/$Abi"
$cores = @("mgba")   # Phase 2: "melonds_ds"   Phase 3: "azahar"

foreach ($core in $cores) {
    $file = "$core`_libretro_android.so"
    $zip = Join-Path $dest "$file.zip"
    Write-Host "Downloading $core ($Abi)..."
    Invoke-WebRequest -Uri "$base/$file.zip" -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $dest -Force
    Remove-Item $zip
    Move-Item -Force (Join-Path $dest $file) (Join-Path $dest "lib$file")
}
Write-Host "Done. Cores in $dest"
Get-ChildItem $dest -Filter *.so | ForEach-Object { Write-Host " - $($_.Name)" }
