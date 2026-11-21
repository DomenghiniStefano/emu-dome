#!/usr/bin/env sh
# Downloads the libretro cores for Linux x86_64 into the repo's cores/ folder, pinned by cores.lock:
# every archive and file is sha256-checked, and a mismatch deletes the core's files and fails.
# A core cores.lock marks `built` for the platform is downloaded from EmuDome's own GitHub release of
# that build (`built.release`), and built from its pinned source (as scripts/build-mgba.sh does) only
# when that fails or with --build; for linux-x86_64 mGBA is still the buildbot's file until Trello #92.
# Usage, from the repo root:
#   sh scripts/fetch-cores.sh [--cores mgba,melondsds,azahar|all] [--platform linux-x86_64] [--build]
# Default: the cores cores.lock ships, for linux-x86_64. Needs Node (the logic is scripts/cores.mjs,
# shared by every platform). The cores are separate projects with their own licences: THIRD_PARTY.md.
# For running only (not packaging), the distribution's package also works (e.g. libretro-mgba on
# Debian/Ubuntu): EmuDome finds it by itself in /usr/lib/x86_64-linux-gnu/libretro.
set -eu
platform=linux-x86_64
cores=''
build=''
while [ $# -gt 0 ]; do
  case "$1" in
    --cores) cores=$2; shift 2 ;;
    --platform) platform=$2; shift 2 ;;
    --build) build=--build; shift ;;
    *) echo "unknown argument: $1 (usage: --cores a,b --platform P --build)" >&2; exit 2 ;;
  esac
done
script="$(dirname "$0")/cores.mjs"
set -- fetch --platform "$platform"
if [ -n "$cores" ]; then set -- "$@" --cores "$cores"; fi
if [ -n "$build" ]; then set -- "$@" "$build"; fi
exec node "$script" "$@"
