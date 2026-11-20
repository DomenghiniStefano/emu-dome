The libretro cores go here (e.g. mgba_libretro.dll), with their licences in notices/<platform>/.
They are not in the repo: download them with scripts/fetch-cores.ps1, scripts/fetch-cores.sh or
scripts/fetch-cores-android.ps1, which take exactly what cores.lock pins and check every sha256.
SOURCES.md is generated from cores.lock (node scripts/cores.mjs generate) and ships with the cores.
