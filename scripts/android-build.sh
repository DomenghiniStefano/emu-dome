#!/usr/bin/env sh
# Builds the Android APK (arm64-v8a) from a clean clone in one command, and with --install puts it
# on the phone connected over USB. Twin of scripts/android-build.ps1 (Windows PowerShell), which
# documents the steps and the requirements; this one is for Linux and Git Bash. Usage, from the
# repo root:
#   sh scripts/android-build.sh [--install] [--release] [--dry-run]
# SDK: ANDROID_HOME / ANDROID_SDK_ROOT, else ~/Android/Sdk (or %LOCALAPPDATA%\Android\Sdk under Git
# Bash); NDK: NDK_HOME / ANDROID_NDK_HOME / ANDROID_NDK_ROOT, else the newest in <sdk>/ndk.
# --release needs EMUDOME_KEYSTORE, EMUDOME_KEYSTORE_PASSWORD, EMUDOME_KEY_ALIAS and
# EMUDOME_KEY_PASSWORD (a keystore outside the repository: app/src-tauri/gen/android/app/build.gradle.kts).
# On Windows the Tauri CLI needs developer mode for its jniLibs symlink: the .ps1 checks it first.
set -eu

install=0
release=0
dry=0
while [ $# -gt 0 ]; do
  case "$1" in
    --install) install=1; shift ;;
    --release) release=1; shift ;;
    --dry-run) dry=1; shift ;;
    *) echo "unknown argument: $1 (usage: --install --release --dry-run)" >&2; exit 2 ;;
  esac
done

root=$(cd "$(dirname "$0")/.." && pwd)
app="$root/app"
target=aarch64-linux-android
abi=arm64-v8a

stop() {
  echo "android-build: $1" >&2
  exit 1
}

# Runs a command from a folder, or with --dry-run only prints it.
step() {
  dir=$1
  shift
  if [ "$dry" -eq 1 ]; then
    echo "would run (in $dir): $*"
    return 0
  fi
  (cd "$dir" && "$@")
}

sdk=''
for dir in "${ANDROID_HOME:-}" "${ANDROID_SDK_ROOT:-}" "${LOCALAPPDATA:+$LOCALAPPDATA/Android/Sdk}" \
  "${HOME:+$HOME/Android/Sdk}"; do
  if [ -n "$dir" ] && [ -d "$dir/platform-tools" ]; then
    sdk=$dir
    break
  fi
done
[ -n "$sdk" ] || stop 'Android SDK not found: set ANDROID_HOME, or install it with Android Studio.'

ndk=''
for dir in "${NDK_HOME:-}" "${ANDROID_NDK_HOME:-}" "${ANDROID_NDK_ROOT:-}"; do
  if [ -n "$dir" ] && [ -d "$dir/toolchains/llvm/prebuilt" ]; then
    ndk=$dir
    break
  fi
done
if [ -z "$ndk" ] && [ -d "$sdk/ndk" ]; then
  ndk=$(ls -d "$sdk"/ndk/*/toolchains/llvm/prebuilt 2>/dev/null | sort -V | tail -n 1)
  ndk=${ndk%/toolchains/llvm/prebuilt}
fi
[ -n "$ndk" ] || stop "Android NDK not found: set NDK_HOME, or install one under $sdk/ndk."
export ANDROID_HOME="$sdk" NDK_HOME="$ndk"
echo "Android SDK: $sdk"
echo "Android NDK: $ndk"

command -v java >/dev/null 2>&1 || [ -n "${JAVA_HOME:-}" ] ||
  stop 'no JDK: install a JDK 17 and put java on PATH (or set JAVA_HOME).'
command -v pnpm >/dev/null 2>&1 || stop 'pnpm not found: see app/package.json (packageManager).'
rustup target list --installed 2>/dev/null | grep -qx "$target" ||
  stop "the Rust target is not installed: rustup target add $target"
echo 'Tools: JDK, pnpm, Rust target ok.'
if [ "$release" -eq 1 ]; then
  # Gradle checks them too; asked here first so a missing one stops before the Rust compilation.
  missing=''
  for name in EMUDOME_KEYSTORE EMUDOME_KEYSTORE_PASSWORD EMUDOME_KEY_ALIAS EMUDOME_KEY_PASSWORD; do
    eval "value=\${$name:-}"
    [ -n "$value" ] || missing="$missing $name"
  done
  [ -z "$missing" ] || stop "release signing: set$missing (a keystore outside the repository; see app/src-tauri/gen/android/app/build.gradle.kts)."
fi

if [ ! -d "$app/node_modules" ]; then
  step "$app" pnpm install --frozen-lockfile || stop 'pnpm install failed.'
fi

if ! step "$root" node "$root/scripts/cores.mjs" fetch --platform "android-$abi"; then
  [ "$release" -eq 1 ] && stop 'the Android cores could not be fetched: a release APK needs them (see above).'
  echo 'android-build: warning: the Android cores could not be fetched (see above); this debug APK goes without them.' >&2
fi

case "${GRADLE_OPTS:-}" in
  *org.gradle.daemon*) ;;
  *) export GRADLE_OPTS="${GRADLE_OPTS:+$GRADLE_OPTS }-Dorg.gradle.daemon=false" ;;
esac
kind=debug
debug_flag=--debug
if [ "$release" -eq 1 ]; then
  kind=release
  debug_flag=''
fi
# shellcheck disable=SC2086 # $debug_flag is one word or none
step "$app" pnpm tauri android build --ci --apk --split-per-abi --target aarch64 $debug_flag ||
  stop 'the build failed.'

apk="$app/src-tauri/gen/android/app/build/outputs/apk/arm64/$kind/app-arm64-$kind.apk"
if [ "$dry" -eq 1 ]; then
  echo "would produce: $apk"
else
  [ -f "$apk" ] || stop "the build ended but $apk is not there."
  echo "APK: $apk ($(wc -c <"$apk" | tr -d ' ') bytes)"
fi

if [ "$install" -eq 1 ]; then
  adb="$sdk/platform-tools/adb"
  [ -x "$adb" ] || adb="$adb.exe"
  [ -x "$adb" ] || stop "adb not found in $sdk/platform-tools."
  step "$root" "$adb" install -r "$apk" ||
    stop 'adb install failed: is the phone connected over USB, with USB debugging on and this PC allowed?'
  [ "$dry" -eq 1 ] || echo 'Installed. Open EmuDome on the phone.'
fi
