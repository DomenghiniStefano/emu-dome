# Third-party components

EmuDome is licensed GPL-3.0-or-later (`LICENSE`). This file lists every third-party component it
uses, with its licence as read from the component's own source. Third-party components keep their
own licences: their LICENSE files and copyright notices are never removed or altered.

The licence check in `scripts/check` (`cargo deny check licenses` with `deny.toml`, and
`scripts/npm-licenses.mjs`) fails when a crate or npm package has a licence outside the allow-list.
The cores, fonts, icons and test ROM below are not packages: they are reviewed by hand here.

Last audit: 2026-11-18. Nothing incompatible with GPL-3.0-or-later was found; the notes column says
where a point needs care.

## Emulation cores (libretro)

The cores are not in the repository (`cores/` is git-ignored, except `cores/README.md` and the
generated `cores/SOURCES.md`). They are pinned in `cores.lock` (repo root): per core the upstream
repository, the exact commit, the licence file at that commit, and per platform either the download
URL with the sha256 of the archive and of every file taken from it, or (`built`) the toolchain of a
build EmuDome makes from the core's pinned source archive, with the sha256 of the built library and
the EmuDome GitHub release that build is published in (`built.release`).
`scripts/fetch-cores.ps1`, `scripts/fetch-cores.sh` and `scripts/fetch-cores-android.ps1` (all
through `scripts/cores.mjs`) download them (a built core from that release; built from source only
when the download fails, or with `-Build` / `--build`), check every hash and fail, deleting the core's
files, on any mismatch. EmuDome loads them at run time as separate programs through the libretro
API, unmodified.

**What a package ships** (only the cores with `ship: true` in `cores.lock`):

- Windows / Linux: `pnpm package:windows` / `pnpm package:linux` (from `app/`) merge
  `app/src-tauri/package/<platform>.json`, generated from `cores.lock`, into the Tauri config. It lists
  each shipped core's library, its licence file at the pinned commit (`cores/notices/`), the
  licences of what it bundles where its release (or, for a built core, its source) has them,
  `cores/SOURCES.md` (the source offer: core, licence, repository, commit, download URLs or source
  archive and build scripts), EmuDome's `LICENSE` and this file (`THIRD_PARTY.md`: the licences page,
  `about.lic`, shows both). Its `targets` are the package formats: NSIS on Windows, AppImage and deb on
  Linux (Trello #44). Nothing else from
  `cores/` goes in. A listed file that is missing stops the build (Tauri), one with the wrong hash
  stops it before bundling (`cores.mjs verify --package`). A plain `pnpm tauri build` makes no
  installer (`bundle.active` is false in `tauri.windows.conf.json` / `tauri.linux.conf.json`).
- Android: the Gradle library (`crates/emudome-platform/android/build.gradle.kts`) reads
  `cores.lock` and copies only the listed `lib*_libretro_android.so` into `jniLibs/<abi>/`, and the
  same licence files and `SOURCES.md` into the APK's assets, sha256-checked (wrong hash: the build
  fails; missing: a release build fails, a debug build warns).

| Core | Console | Shipped | Upstream source, pinned | Licence | Notes |
| --- | --- | --- | --- | --- | --- |
| mGBA | GB, GBC, GBA | yes | https://github.com/libretro/mgba (libretro's tracking fork of https://github.com/mgba-emu/mgba), commit `7a12d6d4b9acb14c0ae62c9166b6a2f3d08007f6`, version `0.11-212-7a12d6d` | MPL-2.0 AND BSD-3-Clause | `LICENSE` at that commit is the MPL-2.0 text; source headers: "subject to the terms of the Mozilla Public License, v. 2.0". The libretro core compiles in inih (`src/third-party/inih`, BSD-3-Clause, its `LICENSE.txt` shipped as `mgba-inih-LICENSE.txt`); the other bundled libraries of mGBA's README (LZMA SDK, SQLite3, ...) are not in the libretro build. Compatible with GPL-3.0-or-later. Binaries: Windows x86_64 and Android arm64-v8a **built by EmuDome** from the source archive of that commit (`source.archive` in `cores.lock`, sha256-pinned) with `scripts/build-mgba.ps1` / `scripts/build-mgba.sh` (Trello #88), the buildbot's CMake options, unmodified; the commit is the one the libretro buildbot built on 2026-11-16 and the builds report the same version string. Those builds are published, with the licence and the inih notice, in EmuDome's GitHub release `cores-mgba-7a12d6d` (https://github.com/DomenghiniStefano/emu-dome/releases/tag/cores-mgba-7a12d6d, Trello #95), which the fetch scripts download from. Linux x86_64: still the buildbot nightly of 2026-11-16 (URL not stable, see below) until it is built on Linux (Trello #92). |
| melonDS DS | NDS | yes | https://github.com/JesseTG/melonds-ds, tag `v1.4.0`, commit `f394adbacb5722ee97c1b37c8064da9a25818310` (on melonDS, https://github.com/melonDS-emu/melonDS) | GPL-3.0-or-later | `LICENSE` at that commit is the GPL-3.0 text; source headers: "either version 3 of the License, or (at your option) any later version". Binaries: Windows x86_64 and Linux x86_64 are the upstream release assets. Android arm64-v8a is **built by EmuDome** from the source archive of that commit with `scripts/build-melondsds.ps1` / `scripts/build-melondsds.sh` (owner, 2026-11-22: upstream's Android asset is linked with 4 KB pages, refused by Google Play and run in compatibility mode on 16 KB-page devices), upstream's Android CMake option plus `-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON`, unmodified; the libraries its CMake would git-clone at configure time are pinned in `cores.lock` (`build.dependencies`: commit, source archive, sha256) and fed to CMake offline: melonDS (https://github.com/JesseTG/melonDS, branch `jtg/fix-uninitialized-opengl` at `16f127dbb587a73371827ce79689c5d237c4b59b`, GPL-3.0-or-later), libretro-common (MIT), embed-binaries (MIT, a build tool), GLM (MIT), zlib (Zlib), libslirp (BSD-3-Clause AND MIT), pntr (Zlib), {fmt} (MIT), yamc (MIT), span-lite (BSL-1.0), date (MIT); `cores/SOURCES.md` lists each commit and archive. That build is published, with the licences, in EmuDome's GitHub release `cores-melondsds-f394adb`, which the fetch scripts download from. The `melondsds-LICENSE.txt` of each build (the release asset's on Windows and Linux; on Android the one its CMake writes from the same template, `cmake/GenerateAttributions.cmake`), shipped as `melondsds-bundled-LICENSES.txt`, carries the licences of what it builds in (melonDS GPL-3.0, Dolphin code under the GPL-2.0 text (Dolphin is GPL-2.0-or-later), FatFs, {fmt}, FreeBIOS, GLM, libretro-common, date, Open Sans, pcap, pntr, Slirp, span-lite, teakra, Tiny AES, xxhash, yamc, zlib and more): reviewed as compatible. |
| Azahar | 3DS | no (Phase 3) | https://github.com/azahar-emu/azahar, tag `2126.2`, commit `219d743a1e698101fe1ab8bef43e6563126c3c27`; full source with submodules: the release's `azahar-unified-source-2126.2.tar.xz` | GPL-2.0-or-later AND GPL-3.0-or-later | `license.txt` at that commit is the **GPL-2.0** text, followed by a table of icon licences (CC BY-ND 3.0 / CC0, Qt frontend icons, not in the libretro core). Source headers are mixed: "Licensed under GPLv2 or any later version" (most Citra-era files, e.g. `src/core/hle/service/service.cpp`, `src/citra_libretro/citra_libretro.cpp`) and "Licensed under GPLv3 or any later version" (e.g. `src/core/core.cpp`). The combined core can be conveyed under GPL-3.0-or-later: compatible. Binaries: the upstream release assets. Fetchable (`-Cores azahar`), not packaged yet. |

When a build ships the cores, it must ship each core's licence and offer its source (GPL section 6;
MPL-2.0 section 3.2): `cores/SOURCES.md` and `cores/notices/` do that. Changes to a core are kept as
recognisable patches or marked commits (none today).

**mGBA is built from source.** Neither mgba-emu/mgba nor libretro/mgba publishes libretro builds,
and the buildbot keeps only `latest/`, so its pinned bytes stopped being downloadable (Android,
2026-11-21). EmuDome builds mGBA itself from the pinned source archive (`scripts/core-build.mjs`:
archive sha256-checked, CMake with the buildbot's options, the result installed only if its sha256 is
the one `cores.lock` pins for that toolchain; licence and notices taken from the same archive).
The build needs the toolchain `cores.lock` names to give the pinned bytes, so the built files are
published once in a GitHub release of this repository, `cores-<core>-<first 7 characters of the
commit>` (Trello #95; `built.release` in `cores.lock`, which `cores.mjs check` holds to that name),
and every other machine downloads them from there, sha256-checked like any other core. That release
only hosts the binaries: their Corresponding Source is the pinned upstream source archive with
EmuDome's build scripts, offered in `cores/SOURCES.md`. How a new pin is built and published:
`docs/release.md`, «A new pin of a core EmuDome builds». **Linux x86_64** keeps the buildbot nightly until it is
built on Linux (Trello #92): `cores.mjs verify --package` refuses that entry (`url_stable: false`)
unless `EMUDOME_ALLOW_UNSTABLE_CORES=1` is set for a local test package; it also lacks the inih
notice, which only the source build installs.

`scripts/check` runs `node scripts/cores.mjs check --self-test`: it fails when `cores.lock`, the
bundle lists, `cores/SOURCES.md`, the Gradle build and this file (each core's commit) disagree.

## Notable components

| Component | Used for | Licence | Notes |
| --- | --- | --- | --- |
| Tauri 2 (`tauri`, `tauri-build`, `tauri-plugin-dialog`, `wry`) | app shell, webview | Apache-2.0 OR MIT | `tao` (windowing) is Apache-2.0. |
| `@tauri-apps/api`, `@tauri-apps/plugin-dialog` | interface to the shell | Apache-2.0 OR MIT | |
| `tauri-plugin-updater`, minisign-verify | EmuDome updating itself (desktop only, `emudome-platform/src/updater.rs`, Trello #44) | Apache-2.0 OR MIT; minisign-verify MIT | Built with `native-tls` instead of its default rustls: TLS through SChannel on Windows and the system's OpenSSL on Linux (`openssl` Apache-2.0, `openssl-sys` MIT; OpenSSL 3 itself is Apache-2.0). The default would bring `webpki-root-certs` (CDLA-Permissive-2.0), not on `deny.toml`'s list. Not compiled for Android (updates from Google Play). |
| wgpu, naga, ash | native video; ash: the raw Vulkan types of wgpu's backend (`gpu_interop.rs`) | MIT OR Apache-2.0 | |
| cpal | audio output | Apache-2.0 | Apache-2.0 is compatible with GPL-3.0 (not with GPL-2.0-only). |
| SDL3 (`sdl3-sys`, `sdl3-src`) | controllers; built from source and linked statically | Zlib | SDL bundles HIDAPI, offered under GPL-3.0, a BSD-style licence or the original HIDAPI licence (`src/hidapi/LICENSE.txt` in SDL): every option is compatible. |
| notify | library folder watching | CC0-1.0 | |
| reqwest 0.12, rustls, ring, rustls-native-certs | Google Drive sync: HTTPS to Google (`emudome-sync`) | MIT OR Apache-2.0; rustls and rustls-native-certs Apache-2.0 OR ISC OR MIT; ring Apache-2.0 AND ISC | ring builds C and assembly with the system's C compiler; code from BoringSSL is Apache-2.0 or ISC (`LICENSE`, `LICENSE-BoringSSL`, `LICENSE-other-bits` in the crate). reqwest 0.12 because 0.13's rustls brings `webpki-root-certs` (CDLA-Permissive-2.0, not in `deny.toml`'s allow-list). |
| keyring | the Drive sign-in's refresh token in the system's credential store (`emudome-platform`) | MIT OR Apache-2.0 | Windows: Credential Manager; Linux: Secret Service over zbus (MIT), pure Rust. |
| libloading | loading the cores | ISC | |
| Vue 3, vue-i18n | interface | MIT | |
| Lucide icons (`@lucide/vue`) | interface icons | ISC | Lucide includes icons derived from Feather (MIT); both notices are in the package. |
| Reka UI (`reka-ui`) | design system primitives | MIT | Ships `tslib` (0BSD) through `aria-hidden`: see the note under the generated tables. |
| `class-variance-authority`, `clsx`, `tailwind-merge` | primitives' variants and `cn()` | Apache-2.0, MIT, MIT | |
| Tailwind CSS (`tailwindcss`, `@tailwindcss/vite`) | the design system's utilities, at build time | MIT | Dev tool: only the CSS it generates ships. |
| Vitest, `@vue/test-utils`, happy-dom | frontend tests | MIT | Dev tools, never shipped. |
| librashader (`librashader-*` 0.12.1) | screen effects (`.slangp` presets) on the native video | MPL-2.0 OR GPL-3.0-only | Used under MPL-2.0, compatible with GPL-3.0-or-later (the allow-list in `deny.toml` matches it through MPL-2.0). Builds Khronos glslang and SPIRV-Cross (C++) from source through `glslang-sys` / `spirv-cross-sys`, declared MIT OR Apache-2.0; the bundled C++ sources carry BSD-3-Clause, MIT and Apache-2.0 notices (glslang's generated parser is GPL-3.0-or-later with the Bison exception): all compatible. |

### Screen effect shaders

The built-in screen effects (`fx.fxpick`) are files in `app/src-tauri/effects/`, embedded in the
binary and written to the app's local data folder at startup. The `.slangp` presets are EmuDome's
own (GPL-3.0-or-later). The shader files are copied unchanged (line endings converted to LF) from
libretro's slang-shaders, https://github.com/libretro/slang-shaders, commit
`9230b1f24a6d0f28febe61711c0b255c4926b231` (2026-11-15). Each keeps its copyright and licence
notice in its header; only files whose header grants a GPL-3.0-compatible licence were taken
(`lcd-grid-v2`, which has no licence notice, was left out).

| File (under `effects/shaders/`) | Effect | Upstream path | Author | Licence |
| --- | --- | --- | --- | --- |
| `zfast_lcd.slang` | `lcd` | `handheld/shaders/zfast_lcd.slang` | Greg Hogan (SoltanGris42), 2017 | GPL-2.0-or-later |
| `lcd3x.slang` | `lcd_grid` | `handheld/shaders/lcd3x.slang` | Gigaherz | Public domain |
| `zfast_crt/zfast_crt_finemask.slang`, `zfast_crt/zfast_crt_impl.inc` | `crt` | `crt/shaders/zfast_crt/` | Greg Hogan (SoltanGris42), 2017 | GPL-2.0-or-later |
| `zfast_crt/zfast_crt_curvature.slang` | `crt_curved` | `crt/shaders/zfast_crt/` | Greg Hogan (SoltanGris42), 2017 | GPL-2.0-or-later |
| `xbr-lv2-standalone.slang` | `soft_pixels` | `edge-smoothing/xbr/shaders/xbr-lv2-standalone.slang` | Hyllian, 2011-2022 | MIT |

GPL-2.0-or-later is used under its "or any later version" option (GPL-3.0). The MIT notice
travels inside the file. Presets the user loads (`fx.fxfile`) are the user's own files and are
never shipped.

### Fonts

No font file is shipped. `app/src/styles.css` names `'Geist'` first in `font-family`, but nothing
loads it (no `@font-face`, no font file, no remote stylesheet): it is used only when the user has it
installed, otherwise the system font is used. If Geist is ever bundled, it is under the SIL Open Font
License 1.1: list it here and ship its `OFL.txt` next to the font files.

### Icons and images

- **App icon and mark** (`app/src/assets/brand/emudome-mark.svg` and `emudome-mark-small.svg`, the two
  masters; `app/src-tauri/icons/`, every size, `app-icon.png` and `icon.ico`; the Android launcher icons
  `app/src-tauri/gen/android/app/src/main/res/mipmap-*/` with the adaptive background in
  `values/ic_launcher_background.xml`): EmuDome's own work, part of EmuDome under GPL-3.0-or-later, drawn
  on 2026-11-21 in the Cowork session and approved by the owner (Trello #54). The cupola is the Dome
  family's mark, the owner's own (the same path as DomeHub's and GameDome's); the glyph carved out of it
  is a handheld console drawn for EmuDome (symmetric rounded corners, a d-pad, two round buttons, a play
  sign on the screen), not derived from any console's outline or any company's logo. The Android set is
  delivered as drawn, not regenerated by `pnpm tauri icon` (which would redo its padding); the
  template's own icons (the Tauri logo, the Android robot) were removed earlier.
- **Interface icons**: all from Lucide (`@lucide/vue`, ISC, see above), imported as components. The
  only icon drawn by hand is the three-line menu glyph in `app/src/components/game/TouchControls.vue`
  (three straight lines), part of EmuDome.
- No other image is shipped (the brand SVG masters are in the repository; the interface does not show
  them yet). Game covers are generated, read from the user's games or chosen by the
  user on their device, and never included.

### Test ROMs

`tools/test-rom/emudome-test.gba` is homebrew written for EmuDome from `tools/test-rom/test.s` and
`build.py`, under GPL-3.0-or-later (`tools/test-rom/LICENSE`). Its header leaves the logo area
empty: it contains no Nintendo material.

`tools/test-rom-nds/emudome-test.nds` is homebrew written for EmuDome from `tools/test-rom-nds/test.s`
and `build.py` (its own small assembler), under GPL-3.0-or-later (`tools/test-rom-nds/LICENSE`). Its
header leaves the logo area and the logo checksum empty, and it needs no BIOS or firmware file: it
contains no Nintendo material.

### Android project

`app/src-tauri/gen/android/` is the Android Gradle project, generated once by the Tauri CLI
(`pnpm tauri android init`, tauri-cli 2.12.4, Apache-2.0 OR MIT) from its template and versioned
with EmuDome's changes (release signing, launcher icons); `scripts/android-build.ps1` builds it.

| Component | Used for | Licence | Notes |
| --- | --- | --- | --- |
| Gradle Wrapper (`gradle/wrapper/gradle-wrapper.jar`, `gradlew`, `gradlew.bat`) | downloads and runs the Gradle version in `gradle-wrapper.properties` | Apache-2.0 | Build tool, in the repository, not in the APK. The jar carries its licence (`META-INF/LICENSE`, `SPDX-License-Identifier: Apache-2.0` in its manifest). |
| Android Gradle Plugin, Kotlin Gradle plugin | build | Apache-2.0 | Downloaded by Gradle at build time; build tools, not in the APK. |
| LLVM libc++ (`libc++_shared.so`) | the C++ runtime the app library links (librashader's glslang / SPIRV-Cross) | Apache-2.0 WITH LLVM-exception | Copied from the Android NDK into the APK's `lib/arm64-v8a/` by the Tauri CLI. Compatible with GPL-3.0-or-later. |
| AndroidX (`webkit`, `appcompat`, `activity-ktx`, `lifecycle-process`), Material Components, Kotlin standard library | the app's Android side (Tauri's activity and WebView) | Apache-2.0 | Downloaded by Gradle from Google's Maven and Maven Central at build time and compiled into the APK. Versions in `app/build.gradle.kts`. Compatible with GPL-3.0-or-later. |

### Platform components (not shipped by EmuDome)

WebView2 (Windows), WebKitGTK (Linux) and the Android WebView belong to the operating system; the
Android NDK/SDK and the C toolchains are build tools. None of them is redistributed in the repository.

## Generated tables

Every crate in `Cargo.lock` (all targets: Windows, Linux, Android and the macOS/iOS crates Cargo
resolves) and every npm package installed in `app/`, with the licence it declares. Written by
`node scripts/third-party-tables.mjs --write` (after `pnpm install` in `app/`), never by hand: `scripts/check`
regenerates them and fails when what is below differs. They run to the end of this file.
Licences written `A/B` are the old Cargo spelling of `A OR B`.

Licences found, all compatible with GPL-3.0-or-later: MIT, Apache-2.0, Apache-2.0 WITH
LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, MPL-2.0, Unicode-3.0, CC0-1.0, Unlicense (often
offered as one choice among several), and, for dev tools only, BlueOak-1.0.0 (`minimatch`,
`@isaacs/nonrecursive-glob-match`, used by ESLint; permissive, no FSF verdict, never shipped).

`tslib` 2.9.1 (**0BSD**) ships in the app through `reka-ui` → `aria-hidden`. 0BSD grants every
permission with no condition at all (ISC without the notice requirement); the owner allowed it on
2026-11-18, and `scripts/npm-licenses.mjs` lists it.

### Rust crates (709)

| Crate | Version | Licence |
| --- | --- | --- |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| aes | 0.8.4 | MIT OR Apache-2.0 |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| alloc-no-stdlib | 3.0.0 | BSD-3-Clause |
| alloc-stdlib | 0.3.0 | BSD-3-Clause |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 |
| alsa | 0.11.0 | Apache-2.0/MIT |
| alsa-sys | 0.4.0 | MIT |
| android_log-sys | 0.3.2 | MIT OR Apache-2.0 |
| android_logger | 0.15.1 | MIT OR Apache-2.0 |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 |
| anstream | 1.0.0 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 |
| anyhow | 1.0.104 | MIT OR Apache-2.0 |
| arc-swap | 1.9.5 | MIT OR Apache-2.0 |
| array-concat | 0.5.5 | MIT |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 |
| async-channel | 2.5.0 | Apache-2.0 OR MIT |
| async-executor | 1.14.1 | Apache-2.0 OR MIT |
| async-fs | 2.2.0 | Apache-2.0 OR MIT |
| async-io | 2.6.0 | Apache-2.0 OR MIT |
| async-lock | 3.5.0 | Apache-2.0 OR MIT |
| async-process | 2.5.0 | Apache-2.0 OR MIT |
| async-recursion | 1.2.0 | MIT OR Apache-2.0 |
| async-signal | 0.2.14 | Apache-2.0 OR MIT |
| async-task | 4.7.1 | Apache-2.0 OR MIT |
| async-trait | 0.1.92 | MIT OR Apache-2.0 |
| atk | 0.18.2 | MIT |
| atk-sys | 0.18.2 | MIT |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| base64 | 0.21.7 | MIT OR Apache-2.0 |
| base64 | 0.22.1 | MIT OR Apache-2.0 |
| base64 | 0.23.1 | MIT OR Apache-2.0 |
| bincode | 2.0.1 | MIT |
| bincode_derive | 2.0.1 | MIT |
| bit-set | 0.10.0 | Apache-2.0 OR MIT |
| bit-set | 0.11.1 | Apache-2.0 OR MIT |
| bit-set | 0.8.0 | Apache-2.0 OR MIT |
| bit-vec | 0.10.2 | Apache-2.0 OR MIT |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT |
| bitflags | 1.3.2 | MIT/Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| blake3 | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| block-padding | 0.3.3 | MIT OR Apache-2.0 |
| block2 | 0.6.2 | MIT |
| blocking | 1.7.0 | Apache-2.0 OR MIT |
| brotli | 9.1.3 | BSD-3-Clause AND MIT |
| brotli-decompressor | 6.3.2 | BSD-3-Clause/MIT |
| bs58 | 0.5.1 | MIT/Apache-2.0 |
| bumpalo | 3.20.4 | MIT OR Apache-2.0 |
| bytecount | 0.6.9 | Apache-2.0/MIT |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT |
| byteorder | 1.5.0 | Unlicense OR MIT |
| byteorder-lite | 0.1.0 | Unlicense OR MIT |
| bytes | 1.12.1 | MIT |
| cairo-rs | 0.18.5 | MIT |
| cairo-sys-rs | 0.18.2 | MIT |
| camino | 1.2.6 | MIT OR Apache-2.0 |
| cargo_metadata | 0.19.2 | MIT |
| cargo_toml | 1.0.1 | Apache-2.0 OR MIT |
| cargo-platform | 0.1.9 | MIT OR Apache-2.0 |
| cbc | 0.1.2 | MIT OR Apache-2.0 |
| cc | 1.11.2 | MIT OR Apache-2.0 |
| cesu8 | 1.1.0 | Apache-2.0/MIT |
| cfb | 0.14.0 | MIT |
| cfg_aliases | 0.2.2 | MIT |
| cfg-expr | 0.15.8 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 |
| chrono | 0.4.45 | MIT OR Apache-2.0 |
| cipher | 0.4.4 | MIT OR Apache-2.0 |
| cmake | 0.1.58 | MIT OR Apache-2.0 |
| codespan-reporting | 0.13.1 | Apache-2.0 |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 |
| combine | 4.6.9 | MIT |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT |
| constant_time_eq | 0.4.2 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| cookie | 0.18.2 | MIT OR Apache-2.0 |
| core_detect | 1.0.0 | MIT/Apache-2.0 |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| core-foundation | 0.9.4 | MIT OR Apache-2.0 |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| core-graphics | 0.25.0 | MIT OR Apache-2.0 |
| core-graphics-types | 0.2.0 | MIT OR Apache-2.0 |
| coreaudio-rs | 0.14.2 | MIT/Apache-2.0 |
| cpal | 0.18.2 | Apache-2.0 |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| crc | 3.4.0 | MIT OR Apache-2.0 |
| crc-catalog | 2.5.0 | MIT OR Apache-2.0 |
| crc32fast | 1.5.5 | MIT OR Apache-2.0 |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 |
| crunchy | 0.2.4 | MIT |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| cssparser | 0.37.0 | MPL-2.0 |
| cssparser | 0.38.0 | MPL-2.0 |
| cssparser-macros | 0.7.1 | MPL-2.0 |
| ctor | 1.0.14 | Apache-2.0 OR MIT |
| darling | 0.24.1 | MIT |
| darling_core | 0.24.1 | MIT |
| darling_macro | 0.24.1 | MIT |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 |
| data-encoding | 2.11.1 | MIT |
| dbus | 0.9.12 | Apache-2.0/MIT |
| dbus-secret-service | 4.2.0 | MIT OR Apache-2.0 |
| deranged | 0.5.9 | MIT OR Apache-2.0 |
| derive_more | 2.1.1 | MIT |
| derive_more-impl | 2.1.1 | MIT |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| dirs | 7.0.0 | MIT OR Apache-2.0 |
| dirs-next | 1.0.2 | MIT OR Apache-2.0 |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 |
| dirs-sys-next | 0.1.2 | MIT OR Apache-2.0 |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 |
| dlib | 0.5.3 | MIT |
| dlopen2 | 0.8.2 | MIT |
| dlopen2_derive | 0.4.3 | MIT |
| document-features | 0.2.12 | MIT OR Apache-2.0 |
| dom_query | 0.28.0 | MIT |
| dom_query | 0.29.0 | MIT |
| dpi | 0.1.2 | Apache-2.0 AND MIT |
| dtoa | 1.0.11 | MIT OR Apache-2.0 |
| dtoa-short | 0.3.5 | MPL-2.0 |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 |
| either | 1.19.0 | MIT OR Apache-2.0 |
| embed_plist | 1.2.2 | MIT OR Apache-2.0 |
| embed-resource | 3.0.12 | MIT |
| encoding_rs | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| endi | 1.1.1 | MIT |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 |
| env_filter | 0.1.4 | MIT OR Apache-2.0 |
| env_filter | 2.0.0 | MIT OR Apache-2.0 |
| env_logger | 0.11.11 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| erased-serde | 0.4.11 | MIT OR Apache-2.0 |
| errno | 0.3.15 | MIT OR Apache-2.0 |
| event-listener | 5.4.2 | Apache-2.0 OR MIT |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| field-offset | 0.3.6 | MIT OR Apache-2.0 |
| filetime | 0.2.30 | MIT/Apache-2.0 |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 |
| fixedbitset | 0.5.9 | MIT OR Apache-2.0 |
| flate2 | 1.1.11 | MIT OR Apache-2.0 |
| fnv | 1.0.7 | Apache-2.0 / MIT |
| foldhash | 0.1.5 | Zlib |
| foldhash | 0.2.0 | Zlib |
| foreign-types | 0.3.2 | MIT/Apache-2.0 |
| foreign-types | 0.5.0 | MIT/Apache-2.0 |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 |
| foreign-types-shared | 0.1.1 | MIT/Apache-2.0 |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 |
| fs2 | 0.4.3 | MIT/Apache-2.0 |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 |
| futures-core | 0.3.34 | MIT OR Apache-2.0 |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 |
| futures-io | 0.3.34 | MIT OR Apache-2.0 |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 |
| futures-task | 0.3.34 | MIT OR Apache-2.0 |
| futures-util | 0.3.34 | MIT OR Apache-2.0 |
| gdk | 0.18.2 | MIT |
| gdk-pixbuf | 0.18.5 | MIT |
| gdk-pixbuf-sys | 0.18.0 | MIT |
| gdk-sys | 0.18.2 | MIT |
| gdkwayland-sys | 0.18.2 | MIT |
| gdkx11 | 0.18.2 | MIT |
| gdkx11-sys | 0.18.2 | MIT |
| generic-array | 0.14.7 | MIT |
| getrandom | 0.2.17 | MIT OR Apache-2.0 |
| getrandom | 0.3.4 | MIT OR Apache-2.0 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| gio | 0.18.4 | MIT |
| gio-sys | 0.18.1 | MIT |
| gl_generator | 0.14.0 | Apache-2.0 |
| glib | 0.18.5 | MIT |
| glib-macros | 0.18.5 | MIT |
| glib-sys | 0.18.1 | MIT |
| glob | 0.3.4 | MIT OR Apache-2.0 |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib |
| glslang | 0.8.1 | MIT OR Apache-2.0 |
| glslang-sys | 0.8.1+275822a | MIT OR Apache-2.0 |
| glutin_wgl_sys | 0.6.1 | Apache-2.0 |
| gobject-sys | 0.18.0 | MIT |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 |
| gtk | 0.18.2 | MIT |
| gtk-sys | 0.18.2 | MIT |
| gtk3-macros | 0.18.2 | MIT |
| half | 2.7.1 | MIT OR Apache-2.0 |
| halfbrown | 0.4.0 | Apache-2.0/MIT |
| hashbrown | 0.12.3 | MIT OR Apache-2.0 |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| heck | 0.4.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 |
| hex | 0.4.3 | MIT OR Apache-2.0 |
| hkdf | 0.12.4 | MIT OR Apache-2.0 |
| hmac | 0.12.1 | MIT OR Apache-2.0 |
| html5ever | 0.39.0 | MIT OR Apache-2.0 |
| html5ever | 0.40.1 | MIT OR Apache-2.0 |
| http | 1.5.0 | MIT OR Apache-2.0 |
| http-body | 1.1.0 | MIT |
| http-body-util | 0.1.5 | MIT |
| httparse | 1.10.1 | MIT OR Apache-2.0 |
| hyper | 1.12.0 | MIT |
| hyper-rustls | 0.27.10 | Apache-2.0 OR ISC OR MIT |
| hyper-tls | 0.6.0 | MIT/Apache-2.0 |
| hyper-util | 0.1.21 | MIT |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 |
| ico | 0.6.0 | MIT |
| icu_collections | 2.4.0 | Unicode-3.0 |
| icu_locale_core | 2.4.0 | Unicode-3.0 |
| icu_normalizer | 2.4.0 | Unicode-3.0 |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 |
| icu_properties | 2.4.0 | Unicode-3.0 |
| icu_properties_data | 2.4.0 | Unicode-3.0 |
| icu_provider | 2.4.0 | Unicode-3.0 |
| ident_case | 1.0.1 | MIT/Apache-2.0 |
| idna | 1.1.0 | MIT OR Apache-2.0 |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT |
| image | 0.25.10 | MIT OR Apache-2.0 |
| image-webp | 0.2.4 | MIT OR Apache-2.0 |
| indexmap | 1.9.3 | Apache-2.0 OR MIT |
| indexmap | 2.14.2 | Apache-2.0 OR MIT |
| infer | 0.22.0 | MIT |
| inotify | 0.11.5 | ISC |
| inotify-sys | 0.1.8 | ISC |
| inout | 0.1.4 | MIT OR Apache-2.0 |
| ipnet | 2.12.2 | MIT OR Apache-2.0 |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| javascriptcore-rs | 1.1.2 | MIT |
| javascriptcore-rs-sys | 1.1.1 | MIT |
| jiff | 0.2.46 | Unlicense OR MIT |
| jiff-core | 0.1.8 | Unlicense OR MIT |
| jiff-static | 0.2.46 | Unlicense OR MIT |
| jiff-tzdb | 0.1.10 | Unlicense OR MIT |
| jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT |
| jni | 0.21.1 | MIT/Apache-2.0 |
| jni | 0.22.4 | MIT OR Apache-2.0 |
| jni-macros | 0.22.4 | MIT OR Apache-2.0 |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 |
| jobserver | 0.1.35 | MIT OR Apache-2.0 |
| js-sys | 0.3.107 | MIT OR Apache-2.0 |
| json-patch | 4.2.0 | MIT/Apache-2.0 |
| jsonptr | 0.7.1 | MIT OR Apache-2.0 |
| keyboard-types | 0.8.3 | MIT OR Apache-2.0 |
| keyring | 3.6.3 | MIT OR Apache-2.0 |
| khronos_api | 3.1.0 | Apache-2.0 |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 |
| kqueue | 1.2.1 | MIT |
| kqueue-sys | 1.1.2 | MIT |
| libappindicator | 0.9.0 | Apache-2.0 OR MIT |
| libappindicator-sys | 0.9.0 | Apache-2.0 OR MIT |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| libdbus-sys | 0.2.7 | Apache-2.0/MIT |
| libloading | 0.7.4 | ISC |
| libloading | 0.8.9 | ISC |
| libloading | 0.9.1 | ISC |
| libm | 0.2.16 | MIT |
| librashader | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-cache | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-common | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-pack | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-preprocess | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-presets | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-reflect | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-runtime | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| librashader-runtime-wgpu | 0.12.1 | MPL-2.0 OR GPL-3.0-only |
| libredox | 0.1.25 | MIT |
| link-section | 0.19.4 | Apache-2.0 OR MIT |
| linked-hash-map | 0.5.6 | MIT/Apache-2.0 |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| litemap | 0.8.3 | Unicode-3.0 |
| litrs | 1.0.1 | MIT OR Apache-2.0 |
| lock_api | 0.4.14 | MIT OR Apache-2.0 |
| log | 0.4.35 | MIT OR Apache-2.0 |
| mach2 | 0.6.0 | BSD-2-Clause OR MIT OR Apache-2.0 |
| markup5ever | 0.39.0 | MIT OR Apache-2.0 |
| markup5ever | 0.40.0 | MIT OR Apache-2.0 |
| md-5 | 0.10.6 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| memoffset | 0.9.1 | MIT |
| mime | 0.3.18 | MIT OR Apache-2.0 |
| minisign-verify | 0.3.0 | MIT |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| mio | 1.2.5 | MIT |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| muda | 0.20.0 | Apache-2.0 OR MIT |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT |
| naga | 30.0.2 | MIT OR Apache-2.0 |
| naga-types | 30.0.2 | MIT OR Apache-2.0 |
| native-tls | 0.2.18 | MIT OR Apache-2.0 |
| ndk | 0.9.0 | MIT OR Apache-2.0 |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 |
| new_debug_unreachable | 1.0.6 | MIT |
| nix | 0.29.0 | MIT |
| nom | 8.0.0 | MIT |
| nom_locate | 5.0.0 | MIT |
| notify | 8.2.2 | CC0-1.0 |
| notify-types | 2.3.1 | MIT OR Apache-2.0 |
| num | 0.4.3 | MIT OR Apache-2.0 |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| num-bigint | 0.4.8 | MIT OR Apache-2.0 |
| num-complex | 0.4.6 | MIT OR Apache-2.0 |
| num-conv | 0.2.2 | MIT OR Apache-2.0 |
| num-derive | 0.4.2 | MIT OR Apache-2.0 |
| num-integer | 0.1.47 | MIT OR Apache-2.0 |
| num-iter | 0.1.46 | MIT OR Apache-2.0 |
| num-rational | 0.4.2 | MIT OR Apache-2.0 |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| objc2 | 0.6.5 | MIT |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-audio-toolbox | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-avf-audio | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-cloud-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-audio | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-audio-types | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-data | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-location | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-services | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-text | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-encode | 4.1.0 | MIT |
| objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT |
| objc2-foundation | 0.3.2 | MIT |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-metal | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-osa-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-user-notifications | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-web-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 |
| openssl | 0.10.82 | Apache-2.0 |
| openssl-macros | 0.1.1 | MIT/Apache-2.0 |
| openssl-probe | 0.2.2 | MIT OR Apache-2.0 |
| openssl-sys | 0.9.118 | MIT |
| option-ext | 0.2.0 | MPL-2.0 |
| ordered-float | 5.5.1 | MIT |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 |
| osakit | 0.3.1 | MIT OR Apache-2.0 |
| pango | 0.18.3 | MIT |
| pango-sys | 0.18.0 | MIT |
| parking | 2.2.1 | Apache-2.0 OR MIT |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 |
| persy | 1.8.1 | MPL-2.0 |
| petgraph | 0.8.3 | MIT OR Apache-2.0 |
| phf | 0.13.1 | MIT |
| phf | 0.14.0 | MIT |
| phf_codegen | 0.13.1 | MIT |
| phf_codegen | 0.14.0 | MIT |
| phf_generator | 0.13.1 | MIT |
| phf_generator | 0.14.0 | MIT |
| phf_macros | 0.13.1 | MIT |
| phf_macros | 0.14.0 | MIT |
| phf_shared | 0.13.1 | MIT |
| phf_shared | 0.14.0 | MIT |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| piper | 0.2.5 | MIT OR Apache-2.0 |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 |
| platform-dirs | 0.3.0 | MIT |
| plist | 1.10.3 | MIT |
| png | 0.18.1 | MIT OR Apache-2.0 |
| polling | 3.11.0 | Apache-2.0 OR MIT |
| pollster | 0.4.0 | Apache-2.0/MIT |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT |
| potential_utf | 0.1.6 | Unicode-3.0 |
| powerfmt | 0.2.1 | MIT OR Apache-2.0 |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 |
| precomputed-hash | 0.1.1 | MIT |
| presser | 0.3.1 | MIT OR Apache-2.0 |
| proc-macro-crate | 1.3.1 | MIT OR Apache-2.0 |
| proc-macro-crate | 2.0.2 | MIT OR Apache-2.0 |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 |
| proc-macro-error | 1.0.4 | MIT OR Apache-2.0 |
| proc-macro-error-attr | 1.0.4 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| profiling | 1.0.18 | MIT OR Apache-2.0 |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| quick-error | 2.0.1 | MIT/Apache-2.0 |
| quick-xml | 0.42.0 | MIT |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| r-efi | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| rand | 0.10.4 | MIT OR Apache-2.0 |
| rand | 0.8.8 | MIT OR Apache-2.0 |
| rand_chacha | 0.3.1 | MIT OR Apache-2.0 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 |
| rand_core | 0.6.4 | MIT OR Apache-2.0 |
| range-alloc | 0.1.5 | MIT OR Apache-2.0 |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| raw-window-metal | 1.1.0 | MIT OR Apache-2.0 |
| rayon | 1.12.0 | MIT OR Apache-2.0 |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 |
| redox_syscall | 0.5.18 | MIT |
| redox_users | 0.4.6 | MIT |
| redox_users | 0.5.3 | MIT |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 |
| regex | 1.13.1 | MIT OR Apache-2.0 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 |
| reqwest | 0.12.28 | MIT OR Apache-2.0 |
| reqwest | 0.13.5 | MIT OR Apache-2.0 |
| rfd | 0.17.2 | MIT |
| ring | 0.17.14 | Apache-2.0 AND ISC |
| rpkg-config | 0.1.2 | Zlib OR MIT OR Apache-2.0 |
| rspirv | 0.13.0+sdk-1.4.341.0 | Apache-2.0 |
| rtrb | 0.4.1 | MIT OR Apache-2.0 |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| rustc-hash | 1.1.0 | Apache-2.0/MIT |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| rustls | 0.23.46 | Apache-2.0 OR ISC OR MIT |
| rustls-native-certs | 0.8.5 | Apache-2.0 OR ISC OR MIT |
| rustls-pki-types | 1.16.0 | MIT OR Apache-2.0 |
| rustls-webpki | 0.103.17 | ISC |
| rustversion | 1.0.23 | MIT OR Apache-2.0 |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| same-file | 1.0.6 | Unlicense/MIT |
| schannel | 0.1.30 | MIT |
| schemars | 0.8.22 | MIT |
| schemars | 0.9.0 | MIT |
| schemars | 1.2.3 | MIT |
| schemars_derive | 0.8.22 | MIT |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| sdl3-src | 3.4.20 | Zlib |
| sdl3-sys | 0.7.3+SDL-3.4.20 | Zlib |
| secret-service | 4.0.0 | MIT OR Apache-2.0 |
| security-framework | 3.7.0 | MIT OR Apache-2.0 |
| security-framework-sys | 2.17.0 | MIT OR Apache-2.0 |
| selectors | 0.38.0 | MPL-2.0 |
| selectors | 0.41.0 | MPL-2.0 |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 |
| serde_spanned | 0.6.9 | MIT OR Apache-2.0 |
| serde_spanned | 1.1.3 | MIT OR Apache-2.0 |
| serde_urlencoded | 0.7.1 | MIT/Apache-2.0 |
| serde_with | 3.24.0 | MIT OR Apache-2.0 |
| serde_with_macros | 3.24.0 | MIT OR Apache-2.0 |
| serde-untagged | 0.1.9 | MIT OR Apache-2.0 |
| serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 |
| serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 |
| servo_arc | 0.4.3 | MIT OR Apache-2.0 |
| servo_arc | 0.5.0 | MIT OR Apache-2.0 |
| sha1 | 0.10.7 | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| shlex | 2.0.1 | MIT OR Apache-2.0 |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 |
| simd_cesu8 | 1.2.0 | Apache-2.0 OR MIT |
| simd-adler32 | 0.3.10 | MIT |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 |
| siphasher | 1.0.4 | MIT OR Apache-2.0 |
| slab | 0.4.12 | MIT |
| slotmap | 1.1.1 | Zlib |
| smallvec | 1.16.5 | MIT OR Apache-2.0 |
| smartstring | 1.0.1 | MPL-2.0+ |
| socket2 | 0.6.5 | MIT OR Apache-2.0 |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 |
| soup3 | 0.5.0 | MIT |
| soup3-sys | 0.5.0 | MIT |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 |
| spirv-cross-sys | 0.7.4+aa217ae | MIT OR Apache-2.0 |
| spirv-cross2 | 0.7.2 | MIT OR Apache-2.0 |
| spirv-cross2-derive | 0.1.0 | MIT OR Apache-2.0 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 |
| static_vcruntime | 3.0.0 | MIT OR Apache-2.0 OR Zlib |
| string_cache | 0.11.0 | MIT OR Apache-2.0 |
| string_cache | 0.9.0 | MIT OR Apache-2.0 |
| string_cache_codegen | 0.11.2 | MIT OR Apache-2.0 |
| string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 |
| strsim | 0.11.1 | MIT |
| strumbra | 0.6.0 | MIT |
| subtle | 2.6.1 | BSD-3-Clause |
| swift-rs | 1.0.9 | MIT OR Apache-2.0 |
| syn | 1.0.109 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| sync_wrapper | 1.0.2 | Apache-2.0 |
| synstructure | 0.14.0 | MIT |
| system-configuration | 0.7.0 | MIT OR Apache-2.0 |
| system-configuration-sys | 0.6.0 | MIT OR Apache-2.0 |
| system-deps | 6.2.2 | MIT OR Apache-2.0 |
| tao | 0.37.2 | Apache-2.0 |
| tao-macros | 0.1.4 | MIT OR Apache-2.0 |
| tar | 0.4.46 | MIT OR Apache-2.0 |
| target-lexicon | 0.12.16 | Apache-2.0 WITH LLVM-exception |
| tauri | 2.12.5 | Apache-2.0 OR MIT |
| tauri-build | 2.7.4 | Apache-2.0 OR MIT |
| tauri-codegen | 2.7.4 | Apache-2.0 OR MIT |
| tauri-macros | 2.7.4 | Apache-2.0 OR MIT |
| tauri-plugin | 2.7.2 | Apache-2.0 OR MIT |
| tauri-plugin-dialog | 2.8.4 | Apache-2.0 OR MIT |
| tauri-plugin-fs | 2.7.1 | Apache-2.0 OR MIT |
| tauri-plugin-updater | 2.14.1 | Apache-2.0 OR MIT |
| tauri-runtime | 2.12.2 | Apache-2.0 OR MIT |
| tauri-runtime-wry | 2.12.3 | Apache-2.0 OR MIT |
| tauri-utils | 2.10.2 | Apache-2.0 OR MIT |
| tauri-winres | 0.3.7 | MIT |
| tempfile | 3.27.0 | MIT OR Apache-2.0 |
| tendril | 0.5.1 | MIT OR Apache-2.0 |
| termcolor | 1.4.1 | Unlicense OR MIT |
| thiserror | 1.0.69 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| time | 0.3.55 | MIT OR Apache-2.0 |
| time-core | 0.1.9 | MIT OR Apache-2.0 |
| time-macros | 0.2.32 | MIT OR Apache-2.0 |
| tinystr | 0.8.5 | Unicode-3.0 |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT |
| tokio | 1.54.1 | MIT |
| tokio-macros | 2.7.2 | MIT |
| tokio-native-tls | 0.3.1 | MIT |
| tokio-rustls | 0.26.6 | MIT OR Apache-2.0 |
| tokio-util | 0.7.20 | MIT |
| toml | 0.8.2 | MIT OR Apache-2.0 |
| toml | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_datetime | 0.6.3 | MIT OR Apache-2.0 |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.19.15 | MIT OR Apache-2.0 |
| toml_edit | 0.20.2 | MIT OR Apache-2.0 |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_writer | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 |
| tower | 0.5.3 | MIT |
| tower-http | 0.6.11 | MIT |
| tower-layer | 0.3.3 | MIT |
| tower-service | 0.3.3 | MIT |
| tracing | 0.1.44 | MIT |
| tracing-attributes | 0.1.31 | MIT |
| tracing-core | 0.1.36 | MIT |
| tray-icon | 0.25.1 | MIT OR Apache-2.0 |
| try-lock | 0.2.5 | MIT |
| ts-rs | 12.0.1 | MIT |
| ts-rs-macros | 12.0.1 | MIT |
| typeid | 1.0.3 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| uds_windows | 1.2.1 | MIT |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 |
| unsigned-varint | 0.8.0 | MIT |
| untrusted | 0.9.0 | ISC |
| unty | 0.0.4 | MIT OR Apache-2.0 |
| url | 2.5.8 | MIT OR Apache-2.0 |
| urlpattern | 0.6.0 | MIT |
| utf_types | 1.0.0 | Apache-2.0 OR MIT |
| utf8_iter | 1.1.0 | Apache-2.0 OR MIT |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT |
| uuid | 1.29.2 | Apache-2.0 OR MIT |
| vcpkg | 0.2.15 | MIT/Apache-2.0 |
| vec_extract_if_polyfill | 0.1.0 | MIT |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| version-compare | 0.2.1 | MIT |
| virtue | 0.0.18 | MIT |
| vswhom | 0.1.0 | MIT |
| vswhom-sys | 0.1.3 | MIT |
| walkdir | 2.5.0 | Unlicense/MIT |
| want | 0.3.2 | MIT |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| wasm-bindgen | 0.2.130 | MIT OR Apache-2.0 |
| wasm-bindgen-futures | 0.4.80 | MIT OR Apache-2.0 |
| wasm-bindgen-macro | 0.2.130 | MIT OR Apache-2.0 |
| wasm-bindgen-macro-support | 0.2.130 | MIT OR Apache-2.0 |
| wasm-bindgen-shared | 0.2.130 | MIT OR Apache-2.0 |
| wasm-streams | 0.4.2 | MIT OR Apache-2.0 |
| wasm-streams | 0.5.0 | MIT OR Apache-2.0 |
| wayland-sys | 0.31.11 | MIT |
| web_atoms | 0.2.6 | MIT OR Apache-2.0 |
| web_atoms | 0.3.0 | MIT OR Apache-2.0 |
| web-sys | 0.3.107 | MIT OR Apache-2.0 |
| web-time | 1.1.0 | MIT OR Apache-2.0 |
| webkit2gtk | 2.0.2 | MIT |
| webkit2gtk-sys | 2.0.2 | MIT |
| webview2-com | 0.39.1 | MIT |
| webview2-com-macros | 0.8.1 | MIT |
| webview2-com-sys | 0.39.1 | MIT |
| wgpu | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-core | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-core-deps-apple | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-core-deps-emscripten | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-core-deps-windows-linux-android | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-hal | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-naga-bridge | 30.0.2 | MIT OR Apache-2.0 |
| wgpu-types | 30.0.2 | MIT OR Apache-2.0 |
| winapi | 0.3.9 | MIT/Apache-2.0 |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| winapi-util | 0.1.11 | Unlicense OR MIT |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| window-vibrancy | 0.8.1 | Apache-2.0 OR MIT |
| windows | 0.62.2 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.53.1 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.42.2 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.53.1 | MIT OR Apache-2.0 |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.42.2 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.53.1 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 |
| windows-core | 0.62.2 | MIT OR Apache-2.0 |
| windows-future | 0.3.2 | MIT OR Apache-2.0 |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 |
| windows-registry | 0.6.1 | MIT OR Apache-2.0 |
| windows-result | 0.4.1 | MIT OR Apache-2.0 |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 |
| windows-sys | 0.45.0 | MIT OR Apache-2.0 |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 |
| windows-sys | 0.59.0 | MIT OR Apache-2.0 |
| windows-sys | 0.60.2 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| windows-targets | 0.42.2 | MIT OR Apache-2.0 |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 |
| windows-version | 0.1.7 | MIT OR Apache-2.0 |
| winnow | 0.5.40 | MIT |
| winnow | 1.0.5 | MIT |
| winreg | 0.56.0 | MIT |
| wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| writeable | 0.6.4 | Unicode-3.0 |
| wry | 0.57.2 | Apache-2.0 OR MIT |
| x11 | 2.21.0 | MIT |
| x11-dl | 2.21.0 | MIT |
| xattr | 1.7.0 | MIT OR Apache-2.0 |
| xdg-home | 1.3.0 | MIT |
| xml-rs | 0.8.29 | MIT |
| yoke | 0.8.4 | Unicode-3.0 |
| yoke-derive | 0.8.4 | Unicode-3.0 |
| zbus | 4.4.0 | MIT |
| zbus_macros | 4.4.0 | MIT |
| zbus_names | 3.0.0 | MIT |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerofrom | 0.1.8 | Unicode-3.0 |
| zerofrom-derive | 0.1.8 | Unicode-3.0 |
| zeroize | 1.9.2 | Apache-2.0 OR MIT |
| zeroize_derive | 1.5.0 | Apache-2.0 OR MIT |
| zerotrie | 0.2.5 | Unicode-3.0 |
| zerovec | 0.11.9 | Unicode-3.0 |
| zerovec-derive | 0.11.7 | Unicode-3.0 |
| zigzag | 0.1.0 | MIT |
| zlib-rs | 0.6.8 | Zlib |
| zmij | 1.0.23 | MIT |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib |
| zvariant | 4.2.0 | MIT |
| zvariant_derive | 4.2.0 | MIT |
| zvariant_utils | 2.1.0 | MIT |

### npm packages (249, 53 shipped)

| Package | Version | Licence | Ships in the app |
| --- | --- | --- | --- |
| @babel/helper-string-parser | 7.29.14 | MIT | yes |
| @babel/helper-validator-identifier | 7.29.7 | MIT | yes |
| @babel/parser | 7.29.14 | MIT | yes |
| @babel/types | 7.29.14 | MIT | yes |
| @cacheable/memory | 2.2.0 | MIT | no (dev tool) |
| @cacheable/utils | 2.5.0 | MIT | no (dev tool) |
| @eslint-community/eslint-utils | 4.10.1 | MIT | no (dev tool) |
| @eslint-community/regexpp | 4.12.3 | MIT | no (dev tool) |
| @eslint/config-array | 0.23.6 | Apache-2.0 | no (dev tool) |
| @eslint/config-helpers | 0.7.0 | Apache-2.0 | no (dev tool) |
| @eslint/core | 1.2.1 | Apache-2.0 | no (dev tool) |
| @eslint/js | 10.0.1 | MIT | no (dev tool) |
| @eslint/object-schema | 3.0.5 | Apache-2.0 | no (dev tool) |
| @eslint/plugin-kit | 0.7.3 | Apache-2.0 | no (dev tool) |
| @floating-ui/core | 1.8.0 | MIT | yes |
| @floating-ui/dom | 1.8.0 | MIT | yes |
| @floating-ui/utils | 0.2.12 | MIT | yes |
| @floating-ui/vue | 2.0.1 | MIT | yes |
| @humanfs/core | 0.19.2 | Apache-2.0 | no (dev tool) |
| @humanfs/node | 0.16.8 | Apache-2.0 | no (dev tool) |
| @humanfs/types | 0.15.0 | Apache-2.0 | no (dev tool) |
| @humanwhocodes/module-importer | 1.0.1 | Apache-2.0 | no (dev tool) |
| @humanwhocodes/retry | 0.4.3 | Apache-2.0 | no (dev tool) |
| @internationalized/date | 3.12.4 | Apache-2.0 | yes |
| @internationalized/number | 3.6.9 | Apache-2.0 | yes |
| @intlify/core-base | 11.4.20 | MIT | yes |
| @intlify/devtools-types | 11.4.20 | MIT | yes |
| @intlify/message-compiler | 11.4.20 | MIT | yes |
| @intlify/shared | 11.4.20 | MIT | yes |
| @isaacs/nonrecursive-glob-match | 1.1.0 | BlueOak-1.0.0 | no (dev tool) |
| @jridgewell/gen-mapping | 0.3.13 | MIT | no (dev tool) |
| @jridgewell/remapping | 2.3.5 | MIT | no (dev tool) |
| @jridgewell/resolve-uri | 3.1.3 | MIT | no (dev tool) |
| @jridgewell/sourcemap-codec | 1.6.0 | MIT | yes |
| @jridgewell/trace-mapping | 0.3.31 | MIT | no (dev tool) |
| @keyv/bigmap | 1.3.1 | MIT | no (dev tool) |
| @keyv/serialize | 1.1.1 | MIT | no (dev tool) |
| @lucide/vue | 1.73.0 | ISC | yes |
| @nodelib/fs.scandir | 2.1.5 | MIT | no (dev tool) |
| @nodelib/fs.stat | 2.0.5 | MIT | no (dev tool) |
| @nodelib/fs.walk | 1.2.8 | MIT | no (dev tool) |
| @one-ini/wasm | 0.2.1 | MIT | no (dev tool) |
| @oxc-project/types | 0.156.0 | MIT | no (dev tool) |
| @rolldown/binding-win32-x64-msvc | 1.2.19 | MIT | no (dev tool) |
| @rolldown/pluginutils | 1.1.1 | MIT | no (dev tool) |
| @swc/helpers | 0.5.24 | Apache-2.0 | yes |
| @tailwindcss/node | 4.3.5 | MIT | no (dev tool) |
| @tailwindcss/oxide | 4.3.5 | MIT | no (dev tool) |
| @tailwindcss/oxide-wasm32-wasi | 4.3.5 | MIT | no (dev tool) |
| @tailwindcss/oxide-win32-x64-msvc | 4.3.5 | MIT | no (dev tool) |
| @tailwindcss/vite | 4.3.5 | MIT | no (dev tool) |
| @tanstack/virtual-core | 3.19.11 | MIT | yes |
| @tanstack/vue-virtual | 3.13.52 | MIT | yes |
| @tauri-apps/api | 2.12.4 | Apache-2.0 OR MIT | yes |
| @tauri-apps/cli | 2.12.4 | Apache-2.0 OR MIT | no (dev tool) |
| @tauri-apps/cli-win32-x64-msvc | 2.12.4 | Apache-2.0 OR MIT | no (dev tool) |
| @tauri-apps/plugin-dialog | 2.8.4 | MIT OR Apache-2.0 | yes |
| @types/chai | 5.2.3 | MIT | no (dev tool) |
| @types/deep-eql | 4.0.2 | MIT | no (dev tool) |
| @types/esrecurse | 4.3.1 | MIT | no (dev tool) |
| @types/estree | 1.0.10 | MIT | no (dev tool) |
| @types/json-schema | 7.0.15 | MIT | no (dev tool) |
| @types/node | 22.20.5 | MIT | no (dev tool) |
| @types/web-bluetooth | 0.0.21 | MIT | yes |
| @types/whatwg-mimetype | 3.0.2 | MIT | no (dev tool) |
| @types/ws | 8.18.2 | MIT | no (dev tool) |
| @typescript-eslint/eslint-plugin | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/parser | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/project-service | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/scope-manager | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/tsconfig-utils | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/type-utils | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/types | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/typescript-estree | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/utils | 8.72.2 | MIT | no (dev tool) |
| @typescript-eslint/visitor-keys | 8.72.2 | MIT | no (dev tool) |
| @vitejs/plugin-vue | 6.0.11 | MIT | no (dev tool) |
| @vitest/mocker | 5.1.0 | MIT | no (dev tool) |
| @vitest/spy | 5.1.0 | MIT | no (dev tool) |
| @volar/language-core | 2.4.30 | MIT | no (dev tool) |
| @volar/language-service | 2.4.30 | MIT | no (dev tool) |
| @volar/source-map | 2.4.30 | MIT | no (dev tool) |
| @volar/typescript | 2.4.30 | MIT | no (dev tool) |
| @vue/compiler-core | 3.5.45 | MIT | yes |
| @vue/compiler-dom | 3.5.45 | MIT | yes |
| @vue/compiler-sfc | 3.5.45 | MIT | yes |
| @vue/compiler-ssr | 3.5.45 | MIT | yes |
| @vue/devtools-api | 6.6.4 | MIT | yes |
| @vue/eslint-config-typescript | 14.9.0 | MIT | no (dev tool) |
| @vue/language-core | 3.3.14 | MIT | no (dev tool) |
| @vue/reactivity | 3.5.45 | MIT | yes |
| @vue/runtime-core | 3.5.45 | MIT | yes |
| @vue/runtime-dom | 3.5.45 | MIT | yes |
| @vue/server-renderer | 3.5.45 | MIT | yes |
| @vue/shared | 3.5.45 | MIT | yes |
| @vue/test-utils | 2.6.0 | MIT | no (dev tool) |
| @vueuse/core | 14.4.0 | MIT | yes |
| @vueuse/metadata | 14.4.0 | MIT | yes |
| @vueuse/shared | 14.4.0 | MIT | yes |
| abbrev | 5.0.0 | ISC | no (dev tool) |
| acorn | 8.19.0 | MIT | no (dev tool) |
| acorn-jsx | 5.3.2 | MIT | no (dev tool) |
| ajv | 6.15.0 | MIT | no (dev tool) |
| alien-signals | 3.2.1 | MIT | no (dev tool) |
| aria-hidden | 1.2.6 | MIT | yes |
| assertion-error | 2.0.1 | MIT | no (dev tool) |
| balanced-match | 4.0.4 | MIT | no (dev tool) |
| boolbase | 1.0.0 | ISC | no (dev tool) |
| brace-expansion | 5.0.13 | MIT | no (dev tool) |
| braces | 3.0.3 | MIT | no (dev tool) |
| buffer-image-size | 0.6.4 | MIT | no (dev tool) |
| cacheable | 2.5.0 | MIT | no (dev tool) |
| chai | 6.3.0 | MIT | no (dev tool) |
| class-variance-authority | 0.7.1 | Apache-2.0 | yes |
| clsx | 2.1.1 | MIT | yes |
| commander | 14.0.3 | MIT | no (dev tool) |
| config-chain | 1.1.13 | MIT | no (dev tool) |
| cross-spawn | 7.0.6 | MIT | no (dev tool) |
| csstype | 3.2.3 | MIT | yes |
| debug | 4.4.3 | MIT | no (dev tool) |
| deep-is | 0.1.4 | MIT | no (dev tool) |
| defu | 6.1.7 | MIT | yes |
| detect-libc | 2.1.2 | Apache-2.0 | no (dev tool) |
| editorconfig | 3.0.2 | MIT | no (dev tool) |
| enhanced-resolve | 5.27.0 | MIT | no (dev tool) |
| entities | 7.0.1 | BSD-2-Clause | yes |
| es-module-lexer | 2.3.2 | MIT | no (dev tool) |
| escape-string-regexp | 4.0.0 | MIT | no (dev tool) |
| eslint | 10.15.0 | MIT | no (dev tool) |
| eslint-config-prettier | 10.1.8 | MIT | no (dev tool) |
| eslint-plugin-vue | 10.12.1 | MIT | no (dev tool) |
| eslint-scope | 9.1.2 | BSD-2-Clause | no (dev tool) |
| eslint-visitor-keys | 3.4.3, 5.0.1 | Apache-2.0 | no (dev tool) |
| espree | 11.2.0 | BSD-2-Clause | no (dev tool) |
| esquery | 1.7.0 | BSD-3-Clause | no (dev tool) |
| esrecurse | 4.3.0 | BSD-2-Clause | no (dev tool) |
| estraverse | 5.3.0 | BSD-2-Clause | no (dev tool) |
| estree-walker | 2.0.2, 3.0.3 | MIT | yes |
| esutils | 2.0.3 | BSD-2-Clause | no (dev tool) |
| expect-type | 1.4.0 | Apache-2.0 | no (dev tool) |
| fast-deep-equal | 3.1.3 | MIT | no (dev tool) |
| fast-glob | 3.3.3 | MIT | no (dev tool) |
| fast-json-stable-stringify | 2.1.0 | MIT | no (dev tool) |
| fast-levenshtein | 2.0.6 | MIT | no (dev tool) |
| fastq | 1.20.3 | ISC | no (dev tool) |
| fdir | 6.5.0 | MIT | no (dev tool) |
| file-entry-cache | 11.1.5 | MIT | no (dev tool) |
| fill-range | 7.1.1 | MIT | no (dev tool) |
| find-up | 5.0.0 | MIT | no (dev tool) |
| flat-cache | 6.1.23 | MIT | no (dev tool) |
| flatted | 3.4.4 | ISC | no (dev tool) |
| glob | 13.0.7 | BlueOak-1.0.0 | no (dev tool) |
| glob-parent | 5.1.2, 6.0.2 | ISC | no (dev tool) |
| globals | 17.14.0 | MIT | no (dev tool) |
| graceful-fs | 4.2.12 | BlueOak-1.0.0 | no (dev tool) |
| happy-dom | 21.0.4 | MIT | no (dev tool) |
| hashery | 1.5.1 | MIT | no (dev tool) |
| hookified | 1.15.1, 2.2.0 | MIT | no (dev tool) |
| ignore | 5.3.2, 7.0.14 | MIT | no (dev tool) |
| imurmurhash | 0.1.4 | MIT | no (dev tool) |
| ini | 1.3.8 | ISC | no (dev tool) |
| is-extglob | 2.1.1 | MIT | no (dev tool) |
| is-glob | 4.0.3 | MIT | no (dev tool) |
| is-number | 7.0.0 | MIT | no (dev tool) |
| isexe | 2.0.0 | ISC | no (dev tool) |
| jiti | 2.7.0 | MIT | no (dev tool) |
| js-beautify | 2.0.3 | MIT | no (dev tool) |
| js-cookie | 3.0.8 | MIT | no (dev tool) |
| json-schema-traverse | 0.4.1 | MIT | no (dev tool) |
| json-stable-stringify-without-jsonify | 1.0.1 | MIT | no (dev tool) |
| keyv | 5.6.0 | MIT | no (dev tool) |
| levn | 0.4.1 | MIT | no (dev tool) |
| lightningcss | 1.33.0 | MPL-2.0 | no (dev tool) |
| lightningcss-win32-x64-msvc | 1.33.0 | MPL-2.0 | no (dev tool) |
| locate-path | 6.0.0 | MIT | no (dev tool) |
| lru-cache | 11.5.3 | BlueOak-1.0.0 | no (dev tool) |
| magic-string | 0.30.21, 1.4.5 | MIT | yes |
| merge2 | 1.4.1 | MIT | no (dev tool) |
| micromatch | 4.0.8 | MIT | no (dev tool) |
| minimatch | 10.2.6, 11.0.8 | BlueOak-1.0.0 | no (dev tool) |
| minipass | 7.1.3 | BlueOak-1.0.0 | no (dev tool) |
| ms | 2.1.3 | MIT | no (dev tool) |
| muggle-string | 0.4.1 | MIT | no (dev tool) |
| nanoid | 3.3.20 | MIT | yes |
| natural-compare | 1.4.0 | MIT | no (dev tool) |
| nopt | 10.0.1 | ISC | no (dev tool) |
| nth-check | 2.1.1 | BSD-2-Clause | no (dev tool) |
| obug | 3.0.0 | MIT | no (dev tool) |
| ohash | 2.0.12 | MIT | yes |
| optionator | 0.9.4 | MIT | no (dev tool) |
| p-limit | 3.1.0 | MIT | no (dev tool) |
| p-locate | 5.0.0 | MIT | no (dev tool) |
| path-browserify | 1.0.1 | MIT | no (dev tool) |
| path-exists | 4.0.0 | MIT | no (dev tool) |
| path-key | 3.1.1 | MIT | no (dev tool) |
| path-scurry | 2.0.2 | BlueOak-1.0.0 | no (dev tool) |
| picocolors | 1.1.1 | ISC | yes |
| picomatch | 2.3.2, 4.0.7 | MIT | no (dev tool) |
| postcss | 8.5.33 | MIT | yes |
| postcss-selector-parser | 7.2.1 | MIT | no (dev tool) |
| prelude-ls | 1.2.1 | MIT | no (dev tool) |
| prettier | 3.10.1 | MIT | no (dev tool) |
| proto-list | 1.2.4 | ISC | no (dev tool) |
| punycode | 2.3.1 | MIT | no (dev tool) |
| qified | 0.10.1 | MIT | no (dev tool) |
| queue-microtask | 1.2.3 | MIT | no (dev tool) |
| reka-ui | 2.11.0 | MIT | yes |
| reusify | 1.1.0 | MIT | no (dev tool) |
| rolldown | 1.2.19 | MIT | no (dev tool) |
| run-parallel | 1.2.0 | MIT | no (dev tool) |
| semver | 7.8.6 | ISC | no (dev tool) |
| shebang-command | 2.0.0 | MIT | no (dev tool) |
| shebang-regex | 3.0.0 | MIT | no (dev tool) |
| source-map-js | 1.2.2 | BSD-3-Clause | yes |
| std-env | 4.3.0 | MIT | no (dev tool) |
| tailwind-merge | 3.7.0 | MIT | yes |
| tailwindcss | 4.3.5 | MIT | no (dev tool) |
| tapable | 2.3.4 | MIT | no (dev tool) |
| tinybench | 6.2.3 | MIT | no (dev tool) |
| tinyexec | 1.3.1 | MIT | no (dev tool) |
| tinyglobby | 0.2.17 | MIT | no (dev tool) |
| to-regex-range | 5.0.1 | MIT | no (dev tool) |
| ts-api-utils | 2.5.0 | MIT | no (dev tool) |
| tslib | 2.9.1 | 0BSD | yes |
| type-check | 0.4.0 | MIT | no (dev tool) |
| typescript | 5.9.3 | Apache-2.0 | no (dev tool) |
| typescript-eslint | 8.72.2 | MIT | no (dev tool) |
| undici-types | 6.21.0 | MIT | no (dev tool) |
| uri-js | 4.4.1 | BSD-2-Clause | no (dev tool) |
| util-deprecate | 1.0.2 | MIT | no (dev tool) |
| vite | 8.3.8 | MIT | no (dev tool) |
| vitest | 5.1.0 | MIT | no (dev tool) |
| vscode-jsonrpc | 10.0.0 | MIT | no (dev tool) |
| vscode-languageserver-protocol | 3.18.5 | MIT | no (dev tool) |
| vscode-languageserver-textdocument | 1.0.16 | MIT | no (dev tool) |
| vscode-languageserver-types | 3.18.5 | MIT | no (dev tool) |
| vscode-uri | 3.2.0 | MIT | no (dev tool) |
| vue | 3.5.45 | MIT | yes |
| vue-component-type-helpers | 3.3.14 | MIT | no (dev tool) |
| vue-eslint-parser | 10.4.2 | MIT | no (dev tool) |
| vue-i18n | 11.4.20 | MIT | yes |
| vue-tsc | 3.3.14 | MIT | no (dev tool) |
| whatwg-mimetype | 3.0.0 | MIT | no (dev tool) |
| which | 2.0.2 | ISC | no (dev tool) |
| why-is-node-running | 3.2.1 | MIT | no (dev tool) |
| word-wrap | 1.2.5 | MIT | no (dev tool) |
| ws | 8.23.0 | MIT | no (dev tool) |
| xml-name-validator | 5.0.0 | Apache-2.0 | no (dev tool) |
| yocto-queue | 0.1.0 | MIT | no (dev tool) |
