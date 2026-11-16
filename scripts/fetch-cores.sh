#!/usr/bin/env bash
# Downloads the libretro cores for Linux x86_64 into the repo's cores/ folder.
# Alternatively: the distribution's package (e.g. libretro-mgba on Debian/Ubuntu),
# which EmuDome finds by itself in /usr/lib/x86_64-linux-gnu/libretro.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/cores"
mkdir -p "$dest"
base="https://buildbot.libretro.com/nightly/linux/x86_64/latest"
for core in mgba; do
  echo "Downloading $core..."
  curl -fsSL "$base/${core}_libretro.so.zip" -o "/tmp/${core}.zip"
  unzip -o -q "/tmp/${core}.zip" -d "$dest"
  rm "/tmp/${core}.zip"
done
ls -1 "$dest"/*.so
