# Builds the Android APK (arm64-v8a) from a clean clone in one command, and with -Install puts it on
# the phone connected over USB. Usage (PowerShell, from the repo root):
#   ./scripts/android-build.ps1 [-Install] [-Release] [-DryRun]
# Twin for Linux / Git Bash: sh scripts/android-build.sh [--install] [--release] [--dry-run].
#
# Steps: finds the Android SDK and NDK, checks the JDK, pnpm, the Rust target and (Windows)
# symlinks; installs the frontend dependencies if app/node_modules is missing; fetches the Android
# cores pinned by cores.lock (sha256-checked, kept when already in place; mGBA is built from its
# pinned source when missing, with the NDK cores.lock names and git: scripts/core-build.mjs); runs
# `pnpm tauri android build --ci --apk --split-per-abi --target aarch64` (`--debug` unless
# -Release), which builds the frontend, the Rust library and the Gradle project in
# app/src-tauri/gen/android; prints the APK's path and size; -Install runs `adb install -r` with
# the SDK's adb. -DryRun stops after the tool checks and prints what it would run.
#
# Needs: Node and pnpm (app/package.json), a JDK 17 (`java` on PATH or JAVA_HOME), the Android SDK
# with platform 37, build-tools and an NDK (ANDROID_HOME / ANDROID_SDK_ROOT, else
# %LOCALAPPDATA%\Android\Sdk; NDK_HOME / ANDROID_NDK_HOME / ANDROID_NDK_ROOT, else the newest in
# <sdk>\ndk), `rustup target add aarch64-linux-android` once, and on Windows **developer mode on**
# (Settings > System > For developers): the Tauri CLI links libemudome_app.so into jniLibs with a
# symlink, which Windows refuses otherwise ("Creation symbolic link is not allowed").
# -Release also needs the signing variables (EMUDOME_KEYSTORE, EMUDOME_KEYSTORE_PASSWORD,
# EMUDOME_KEY_ALIAS, EMUDOME_KEY_PASSWORD; see app/src-tauri/gen/android/app/build.gradle.kts):
# the keystore lives outside the repository, never in it.
# Gradle runs without a daemon (nothing left running after the build) unless GRADLE_OPTS already
# names org.gradle.daemon. ANDROID_HOME, NDK_HOME and GRADLE_OPTS are set for the build and put
# back as they were at the end, so the calling PowerShell session keeps its own.

param(
    # Install the APK on the device adb sees (one phone over USB, USB debugging on).
    [switch]$Install,
    # A signed release APK instead of the debug one.
    [switch]$Release,
    # Only the tool checks; print the commands instead of running them.
    [switch]$DryRun
)

# 'Continue': Windows PowerShell 5.1 turns a native command's stderr lines into error records when
# the output is redirected, and 'Stop' would end the script on the first build log line. Every
# native call below checks its exit code instead.
$ErrorActionPreference = 'Continue'
$root = Split-Path $PSScriptRoot -Parent
$app = Join-Path $root 'app'
$target = 'aarch64-linux-android'
$abi = 'arm64-v8a'

class BuildStop : System.Exception {
    BuildStop([string]$message) : base($message) {}
}

function Stop-Build([string]$message) {
    throw [BuildStop]::new($message)
}

function Find-Sdk {
    foreach ($dir in @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT, (Join-Path $env:LOCALAPPDATA 'Android\Sdk'))) {
        if ($dir -and (Test-Path (Join-Path $dir 'platform-tools'))) { return $dir }
    }
    return $null
}

function Find-Ndk([string]$sdk) {
    foreach ($dir in @($env:NDK_HOME, $env:ANDROID_NDK_HOME, $env:ANDROID_NDK_ROOT)) {
        if ($dir -and (Test-Path (Join-Path $dir 'toolchains\llvm\prebuilt'))) { return $dir }
    }
    $ndkRoot = Join-Path $sdk 'ndk'
    if (-not (Test-Path $ndkRoot)) { return $null }
    $newest = Get-ChildItem $ndkRoot -Directory |
        Where-Object { Test-Path (Join-Path $_.FullName 'toolchains\llvm\prebuilt') } |
        Sort-Object { try { [version]$_.Name } catch { [version]'0.0' } } |
        Select-Object -Last 1
    if ($null -eq $newest) { return $null }
    return $newest.FullName
}

# Whether this account may create symlinks (Windows developer mode, or an elevated shell). Tried in
# a new folder under the user's temp directory, removed afterwards.
function Test-Symlink {
    $dir = Join-Path ([System.IO.Path]::GetTempPath()) ('emudome-symlink-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $dir | Out-Null
    try {
        $file = Join-Path $dir 'target.txt'
        New-Item -ItemType File -Path $file | Out-Null
        & cmd /c mklink (Join-Path $dir 'link.txt') $file 2>&1 | Out-Null
        return ($LASTEXITCODE -eq 0)
    } finally {
        Remove-Item -Recurse -Force -LiteralPath $dir -Confirm:$false
    }
}

# Runs a native command from a folder; stops the build with $why when it fails.
function Invoke-Step([string]$dir, [string]$command, [string[]]$arguments, [string]$why) {
    if ($DryRun) {
        Write-Host "would run (in $dir): $command $($arguments -join ' ')"
        return
    }
    Push-Location $dir
    try {
        & $command @arguments
        $rc = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    if ($rc -ne 0) { Stop-Build $why }
}

$saved = @{ ANDROID_HOME = $env:ANDROID_HOME; NDK_HOME = $env:NDK_HOME; GRADLE_OPTS = $env:GRADLE_OPTS }
$exitCode = 0
try {
    # --- Tools

    $sdk = Find-Sdk
    if ($null -eq $sdk) {
        Stop-Build 'Android SDK not found: set ANDROID_HOME, or install it with Android Studio (%LOCALAPPDATA%\Android\Sdk).'
    }
    $ndk = Find-Ndk $sdk
    if ($null -eq $ndk) {
        Stop-Build "Android NDK not found: set NDK_HOME, or install one under $sdk\ndk (Android Studio, SDK Manager, NDK)."
    }
    $env:ANDROID_HOME = $sdk
    $env:NDK_HOME = $ndk
    Write-Host "Android SDK: $sdk"
    Write-Host "Android NDK: $ndk"

    if ($null -eq (Get-Command java -ErrorAction SilentlyContinue) -and -not $env:JAVA_HOME) {
        Stop-Build 'no JDK: install a JDK 17 and put java on PATH (or set JAVA_HOME).'
    }
    if ($null -eq (Get-Command pnpm -ErrorAction SilentlyContinue)) {
        Stop-Build 'pnpm not found: see app/package.json (packageManager).'
    }
    $targets = & rustup target list --installed
    if ($LASTEXITCODE -ne 0) { Stop-Build 'rustup not found: install Rust with rustup.' }
    if ($targets -notcontains $target) {
        Stop-Build "the Rust target is not installed: rustup target add $target"
    }
    if (-not (Test-Symlink)) {
        Stop-Build ('Windows refuses to create symlinks for this account, and the Tauri CLI links ' +
            'libemudome_app.so into jniLibs with one. Turn on developer mode (Settings > System > ' +
            'For developers) and run this again.')
    }
    Write-Host 'Tools: JDK, pnpm, Rust target, symlinks ok.'
    if ($Release) {
        # Gradle checks them too (and that the keystore is outside the repo); asked here first so a
        # missing one stops the build before the long Rust compilation.
        $signing = @('EMUDOME_KEYSTORE', 'EMUDOME_KEYSTORE_PASSWORD', 'EMUDOME_KEY_ALIAS', 'EMUDOME_KEY_PASSWORD')
        $missing = $signing | Where-Object { -not [Environment]::GetEnvironmentVariable($_) }
        if ($missing) {
            Stop-Build ("release signing: set $($missing -join ', ') (a keystore outside the repository; " +
                'see app/src-tauri/gen/android/app/build.gradle.kts).')
        }
    }

    # --- Frontend dependencies, cores

    if (-not (Test-Path (Join-Path $app 'node_modules'))) {
        Invoke-Step $app 'pnpm' @('install', '--frozen-lockfile') 'pnpm install failed.'
    }

    $fetch = @((Join-Path $PSScriptRoot 'cores.mjs'), 'fetch', '--platform', "android-$abi")
    if ($DryRun) {
        Write-Host "would run (in $root): node $($fetch -join ' ')"
    } else {
        & node @fetch
        if ($LASTEXITCODE -ne 0) {
            if ($Release) { Stop-Build 'the Android cores could not be fetched: a release APK needs them (see above).' }
            Write-Host ('android-build: warning: the Android cores could not be fetched (see above); ' +
                'this debug APK goes without them.') -ForegroundColor Yellow
        }
    }

    # --- Build

    if (-not $env:GRADLE_OPTS -or $env:GRADLE_OPTS -notmatch 'org\.gradle\.daemon') {
        $env:GRADLE_OPTS = (@($env:GRADLE_OPTS, '-Dorg.gradle.daemon=false') | Where-Object { $_ }) -join ' '
    }
    $arguments = @('tauri', 'android', 'build', '--ci', '--apk', '--split-per-abi', '--target', 'aarch64')
    if (-not $Release) { $arguments += '--debug' }
    Invoke-Step $app 'pnpm' $arguments "the build failed (pnpm $($arguments -join ' '))."

    $kind = if ($Release) { 'release' } else { 'debug' }
    $apk = Join-Path $app "src-tauri\gen\android\app\build\outputs\apk\arm64\$kind\app-arm64-$kind.apk"
    if ($DryRun) {
        Write-Host "would produce: $apk"
    } else {
        if (-not (Test-Path $apk)) { Stop-Build "the build ended but $apk is not there." }
        $size = (Get-Item $apk).Length
        Write-Host ('APK: {0} ({1:N1} MB)' -f $apk, ($size / 1MB))
    }

    # --- Install

    if ($Install) {
        $adb = Join-Path $sdk 'platform-tools\adb.exe'
        if (-not (Test-Path $adb)) { Stop-Build "adb not found in $sdk\platform-tools." }
        Invoke-Step $root $adb @('install', '-r', $apk) `
            'adb install failed: is the phone connected over USB, with USB debugging on and this PC allowed?'
        if (-not $DryRun) { Write-Host 'Installed. Open EmuDome on the phone.' }
    }
} catch [BuildStop] {
    Write-Host "android-build: $($_.Exception.Message)" -ForegroundColor Red
    $exitCode = 1
} finally {
    foreach ($name in $saved.Keys) {
        # $null removes a variable that was not set before.
        [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process')
    }
}
exit $exitCode
