# Sources of the libretro cores

<!-- Generated from cores.lock by `node scripts/cores.mjs generate`. Do not edit by hand. -->

EmuDome ships these libretro cores unmodified, as separate programs it loads at run time.
Each keeps its own licence, whose text is in `notices/`. The Corresponding Source of each
binary is the upstream commit named below. If a link stops working, open an issue at
https://github.com/DomenghiniStefano/emu-dome/issues and the project will provide that source.

## mGBA (GB, GBC, GBA)

- Licence: MPL-2.0; its text is `notices/mgba-LICENSE.txt`.
- Source: https://github.com/libretro/mgba, commit `7a12d6d4b9acb14c0ae62c9166b6a2f3d08007f6` (master, as built by the libretro buildbot on 2026-11-16; version 0.11-212-7a12d6d).
  Download it as https://github.com/libretro/mgba/archive/7a12d6d4b9acb14c0ae62c9166b6a2f3d08007f6.zip
- Builds:
  - windows-x86_64: `mgba_libretro.dll`, from https://buildbot.libretro.com/nightly/windows/x86_64/latest/mgba_libretro.dll.zip (sha256 `a7ae453b739d3f9117537e8ca7275ecebd923d9035b52435f556470bf58a268f`)
  - linux-x86_64: `mgba_libretro.so`, from https://buildbot.libretro.com/nightly/linux/x86_64/latest/mgba_libretro.so.zip (sha256 `85b8b8191b937ee1b40f14588686d7caf46c4cce70ea85c8597a741ffb113e9c`)
  - android-arm64-v8a: `libmgba_libretro_android.so`, from https://buildbot.libretro.com/nightly/android/latest/arm64-v8a/mgba_libretro_android.so.zip (sha256 `df1dd7fe5eb28cdb071bddb45fe4f3f0127512440a41402a5e380007d693099e`)

## melonDS DS (NDS)

- Licence: GPL-3.0-or-later; its text is `notices/melondsds-LICENSE.txt`, and the licences of what it bundles are next to it.
- Source: https://github.com/JesseTG/melonds-ds, commit `f394adbacb5722ee97c1b37c8064da9a25818310` (v1.4.0; version 1.4.0).
  Download it as https://github.com/JesseTG/melonds-ds/archive/f394adbacb5722ee97c1b37c8064da9a25818310.zip
- Builds:
  - windows-x86_64: `melondsds_libretro.dll`, from https://github.com/JesseTG/melonds-ds/releases/download/v1.4.0/melondsds_libretro-win32-x86_64-Release.zip (sha256 `f03af0318fee91a83880f438b369a1368fea6b5c1524a09dde849ca8fa68603f`)
  - linux-x86_64: `melondsds_libretro.so`, from https://github.com/JesseTG/melonds-ds/releases/download/v1.4.0/melondsds_libretro-linux-x86_64-Release.zip (sha256 `ca6b0be914a9d3f822a885547879b279e3fa25326e41d0dc9b8a670fc79df0d0`)
  - android-arm64-v8a: `libmelondsds_libretro_android.so`, from https://github.com/JesseTG/melonds-ds/releases/download/v1.4.0/melondsds_libretro-android-Release.zip (sha256 `555568396dd5cf230030e4dd0ff0b0c6ca7118231fe562bc50680fefa7ad8594`)
