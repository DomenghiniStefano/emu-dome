#!/usr/bin/env sh
# Builds the melonDS DS libretro core (NDS) for Android from the source cores.lock pins (v1.4.0 and
# every library its CMake would fetch, each pinned to a commit and a sha256-checked archive), and
# installs it into cores/android/<abi>/ only if its sha256 is the one cores.lock pins.
# EmuDome builds it because upstream's Android release is linked with 4 KB pages, which Google Play
# and 16 KB-page devices refuse (docs/release.md, "Android: 16 KB pages"); Windows and Linux still
# use upstream's release files.
# Usage, from the repo root, on Linux, WSL or Git Bash:
#   sh scripts/build-melondsds.sh [--platform android-arm64-v8a] [--jobs 2] [--work-dir DIR]
# Needs Node, git (melonDS DS's CMake looks for it) and the Android NDK (NDK_HOME, or
# <sdk>/ndk/<version>). Every step: scripts/core-build.mjs. cores.lock's built.toolchain names the
# toolchain the pin came from (Windows); the NDK of another host may give other bytes.
set -eu
platform=android-arm64-v8a
jobs=2
work_dir=''
while [ $# -gt 0 ]; do
  case "$1" in
    --platform) platform=$2; shift 2 ;;
    --jobs) jobs=$2; shift 2 ;;
    --work-dir) work_dir=$2; shift 2 ;;
    *) echo "unknown argument: $1 (usage: --platform P --jobs N --work-dir DIR)" >&2; exit 2 ;;
  esac
done
script="$(dirname "$0")/cores.mjs"
if [ -n "$work_dir" ]; then
  exec node "$script" build --platform "$platform" --cores melondsds --jobs "$jobs" --work-dir "$work_dir"
fi
exec node "$script" build --platform "$platform" --cores melondsds --jobs "$jobs"
