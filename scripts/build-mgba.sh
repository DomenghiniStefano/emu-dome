#!/usr/bin/env sh
# Builds the mGBA libretro core (GB, GBC, GBA) from the source cores.lock pins, and installs it into
# cores/ (Linux) or cores/android/<abi>/ (Android) only if its sha256 is the one cores.lock pins.
# EmuDome builds mGBA itself because no upstream publishes a fixed libretro build of it (Trello #88).
# Usage, from the repo root, on Linux or WSL:
#   sh scripts/build-mgba.sh [--platform linux-x86_64|android-arm64-v8a] [--jobs 2] [--work-dir DIR]
# Needs Node, git, cmake and a C compiler (ninja or make); for Android the NDK (NDK_HOME, or
# <sdk>/ndk/<version>). Every step and toolchain: scripts/core-build.mjs. cores.lock does not pin
# a Linux build of its own yet (Trello #92): until it does, linux-x86_64 is refused there.
set -eu
platform=linux-x86_64
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
  exec node "$script" build --platform "$platform" --cores mgba --jobs "$jobs" --work-dir "$work_dir"
fi
exec node "$script" build --platform "$platform" --cores mgba --jobs "$jobs"
