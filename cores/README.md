The libretro cores go here (e.g. mgba_libretro.dll; Android's in android/<abi>/), with their licences in
notices/<platform>/. They are not in the repo: download them with scripts/fetch-cores.ps1,
scripts/fetch-cores.sh or scripts/fetch-cores-android.ps1, which take exactly what cores.lock pins and check
every sha256. Cores EmuDome builds from source (mGBA for Windows and Android, melonDS DS for Android) are
downloaded from this repository's cores-<core>-<commit> releases, and built with scripts/build-mgba.ps1 /
scripts/build-melondsds.ps1 only when that download fails or with -Build.
SOURCES.md is generated from cores.lock (node scripts/cores.mjs generate) and ships with the cores.
