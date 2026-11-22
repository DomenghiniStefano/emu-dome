# Sources of the libretro cores

<!-- Generated from cores.lock by `node scripts/cores.mjs generate`. Do not edit by hand. -->

EmuDome ships these libretro cores unmodified, as separate programs it loads at run time.
Some are the upstream builds, downloaded; the others EmuDome builds from the unmodified
upstream source and publishes in its own GitHub releases (said per build below). Each keeps
its own licence, whose text is in `notices/`. The Corresponding Source of each binary is the
upstream commit named below (for a build of EmuDome, the source archive of that commit and
the build scripts named with it). If a link stops working, open
an issue at https://github.com/DomenghiniStefano/emu-dome/issues and the project will provide that source.

## mGBA (GB, GBC, GBA)

- Licence: MPL-2.0 AND BSD-3-Clause; its text is `notices/mgba-LICENSE.txt`, and the licences of what it bundles are next to it.
- Source: https://github.com/libretro/mgba, commit `7a12d6d4b9acb14c0ae62c9166b6a2f3d08007f6` (master, at the commit the libretro buildbot built on 2026-11-16; version 0.11-212-7a12d6d).
  Source archive of that commit: https://github.com/libretro/mgba/archive/7a12d6d4b9acb14c0ae62c9166b6a2f3d08007f6.tar.gz (sha256 `5cbf639e527fb586bf33e14d59037eab35f78f45154e0b79c784fff474c3bc37`).
  The build scripts and their CMake options are in https://github.com/DomenghiniStefano/emu-dome (`cores.lock`, `scripts/core-build.mjs`, `scripts/build-mgba.ps1`, `scripts/build-mgba.sh`).
- Builds:
  - windows-x86_64: `mgba_libretro.dll`, built by EmuDome from the source archive above with `scripts/build-mgba.ps1` / `scripts/build-mgba.sh` (clang-cl 18.0.2 (Android NDK 27.1.12297006), MSVC 14.44.35207 libraries and linker, Windows SDK 10.0.26100.0, CMake 3.31.6-msvc6; sha256 `8fbb4137f6dd3b28f4d92c832d2d7460a9aff5009b3a7c20f75320e4ed53b66d`), published as https://github.com/DomenghiniStefano/emu-dome/releases/download/cores-mgba-7a12d6d/mgba_libretro.dll
  - linux-x86_64: `mgba_libretro.so`, from https://buildbot.libretro.com/nightly/linux/x86_64/latest/mgba_libretro.so.zip (sha256 `85b8b8191b937ee1b40f14588686d7caf46c4cce70ea85c8597a741ffb113e9c`)
  - android-arm64-v8a: `libmgba_libretro_android.so`, built by EmuDome from the source archive above with `scripts/build-mgba.ps1` / `scripts/build-mgba.sh` (Android NDK 27.1.12297006, API 26, CMake 3.31.6-msvc6; sha256 `ee99a5cd60fb99bff962f464b1a32f3af317c31b0b8f55847101a05fe289b563`), published as https://github.com/DomenghiniStefano/emu-dome/releases/download/cores-mgba-7a12d6d/libmgba_libretro_android.so

## melonDS DS (NDS)

- Licence: GPL-3.0-or-later; its text is `notices/melondsds-LICENSE.txt`, and the licences of what it bundles are next to it.
- Source: https://github.com/JesseTG/melonds-ds, commit `f394adbacb5722ee97c1b37c8064da9a25818310` (v1.4.0; version 1.4.0).
  Source archive of that commit: https://github.com/JesseTG/melonds-ds/archive/f394adbacb5722ee97c1b37c8064da9a25818310.tar.gz (sha256 `1b377c6d84355b2449e8f31893839fa0b138b51a98bf72edb8bc3ca9169186a6`).
  The build scripts and their CMake options are in https://github.com/DomenghiniStefano/emu-dome (`cores.lock`, `scripts/core-build.mjs`, `scripts/build-melondsds.ps1`, `scripts/build-melondsds.sh`).
  It compiles in these libraries, each built from the source archive of its pinned commit:
  - melonDS (GPL-3.0-or-later): https://github.com/JesseTG/melonDS, commit `16f127dbb587a73371827ce79689c5d237c4b59b`; https://github.com/JesseTG/melonDS/archive/16f127dbb587a73371827ce79689c5d237c4b59b.tar.gz (sha256 `618354a6853be72f7777c36914139405fb17e1058454f4af8eea3ac04871676e`).
  - libretro-common (MIT): https://github.com/libretro/libretro-common, commit `fa8a1b572a444fcb552007019f8caf68281108aa`; https://github.com/libretro/libretro-common/archive/fa8a1b572a444fcb552007019f8caf68281108aa.tar.gz (sha256 `6803ff96bada0ba968cd5d9f417a217ebeca527a0e8b849145a2183ad779faff`).
  - embed-binaries (MIT): https://github.com/andoalon/embed-binaries, commit `078b62beba97e8192c99bfb16d5e17220cfc7598`; https://github.com/andoalon/embed-binaries/archive/078b62beba97e8192c99bfb16d5e17220cfc7598.tar.gz (sha256 `3b1017d10077de692e19c3772d824010705533ec441583587fac4a959ae77bd5`).
  - glm (MIT): https://github.com/g-truc/glm, commit `6f14f4792a0cde5d0cf2c910506724d61cb95834`; https://github.com/g-truc/glm/archive/6f14f4792a0cde5d0cf2c910506724d61cb95834.tar.gz (sha256 `9d96623ff3fb4ffd8cacd0e3aab39c14bb173f0bdf62518b04087000a251349f`).
  - zlib (Zlib): https://github.com/madler/zlib, commit `da607da739fa6047df13e66a2af6b8bec7c2a498`; https://github.com/madler/zlib/archive/da607da739fa6047df13e66a2af6b8bec7c2a498.tar.gz (sha256 `b9258cf6254e7f7c37f1cd61dba943a1c5ea3cff5718c789834dac359094f5f7`).
  - libslirp (BSD-3-Clause AND MIT): https://github.com/JesseTG/libslirp-mirror, commit `e61dbd459c8c06607b3a84694489427e8ec60f17`; https://github.com/JesseTG/libslirp-mirror/archive/e61dbd459c8c06607b3a84694489427e8ec60f17.tar.gz (sha256 `2269005f612868c31027ea992bd88f0f7c89c69e7f2eef495479cb2a2f9e2ed3`).
  - pntr (Zlib): https://github.com/robloach/pntr, commit `a47fe167703b0ac3bfad00785b20b0dce6712509`; https://github.com/robloach/pntr/archive/a47fe167703b0ac3bfad00785b20b0dce6712509.tar.gz (sha256 `628ee25a6eae708b1ae143ad52337bd69745f34d1e7000cd8d96d7628b5b24c6`).
  - fmt (MIT): https://github.com/fmtlib/fmt, commit `1be298e1bd68957e4cd352e1f676f00e07dcfb57`; https://github.com/fmtlib/fmt/archive/1be298e1bd68957e4cd352e1f676f00e07dcfb57.tar.gz (sha256 `d041f836d9b2a3c1c0fe3cce81ffe6a4a257028faa2f4af0dd20d7e58b493766`).
  - yamc (MIT): https://github.com/yohhoy/yamc, commit `4e015a7e8eb0d61c34e6928676c8c78881a72d73`; https://github.com/yohhoy/yamc/archive/4e015a7e8eb0d61c34e6928676c8c78881a72d73.tar.gz (sha256 `f322d79bfd7dda607c69c867a1cc57192173f02d973e6c9bae02c3ad984c114e`).
  - span-lite (BSL-1.0): https://github.com/martinmoene/span-lite, commit `f63fe2974943b5f555b928b10d910e457fbb93d4`; https://github.com/martinmoene/span-lite/archive/f63fe2974943b5f555b928b10d910e457fbb93d4.tar.gz (sha256 `ec8a2538f8bdba08476194f79d40fbbed0f1aaac51954e797b4066d5f8822a9a`).
  - date (MIT): https://github.com/HowardHinnant/date, commit `179a6b921ccd6179b5b04dce812a89f71a51b4df`; https://github.com/HowardHinnant/date/archive/179a6b921ccd6179b5b04dce812a89f71a51b4df.tar.gz (sha256 `4d162515b38cd522cd7b9bbf8b04ece43574ee39107f6a054adc2706d5ea86f7`).
- Builds:
  - windows-x86_64: `melondsds_libretro.dll`, from https://github.com/JesseTG/melonds-ds/releases/download/v1.4.0/melondsds_libretro-win32-x86_64-Release.zip (sha256 `f03af0318fee91a83880f438b369a1368fea6b5c1524a09dde849ca8fa68603f`)
  - linux-x86_64: `melondsds_libretro.so`, from https://github.com/JesseTG/melonds-ds/releases/download/v1.4.0/melondsds_libretro-linux-x86_64-Release.zip (sha256 `ca6b0be914a9d3f822a885547879b279e3fa25326e41d0dc9b8a670fc79df0d0`)
  - android-arm64-v8a: `libmelondsds_libretro_android.so`, built by EmuDome from the source archive above with `scripts/build-melondsds.ps1` / `scripts/build-melondsds.sh` (Android NDK 27.1.12297006, API 26, CMake 3.31.6-msvc6; sha256 `73e357cee10c2bce5f61088461bd55bf110c7572fea8b5e31a87cef4cc2236f0`), published as https://github.com/DomenghiniStefano/emu-dome/releases/download/cores-melondsds-f394adb/libmelondsds_libretro_android.so
