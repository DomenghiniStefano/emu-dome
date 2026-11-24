# Releases

How an EmuDome release is made, and what is not in place yet (Trello #83, #44, #50). Nothing here has been run end
to end yet: no release exists.

## Today, as the code is

- **Branches and tags.** `develop` is the integration branch (`CLAUDE.md`, «Current state»); `stage` the test
  builds (below). There is no `main` branch on the remote and no tag yet. Nobody pushes to `main` until the first
  release, 1.0.0.
- **Version.** One number, written in one place: `[workspace.package] version` in the root `Cargo.toml` (today
  `0.0.1`). Every crate takes it (`version.workspace = true`); `app/package.json` and `tauri.conf.json` have none
  (Tauri falls back to Cargo's for the runtime, the bundles and the Windows file version), and
  `scripts/versions.mjs` (a step of `scripts/check`) fails when one is written anywhere else. The Tauri CLI takes
  the Android `versionName` and `versionCode` only from its config (without it Gradle falls back to `1.0` / code 1),
  so `android-build` passes the version with `--config` (`versions.mjs tauri-config`); never upload a bare
  `pnpm tauri android build`. The `versionCode` is `major * 1000000 + minor * 1000 + patch`, so every upload must
  raise the version (`node scripts/versions.mjs bump [patch|minor|major]`, which also refreshes `Cargo.lock`): Google
  Play refuses a code it has already seen. The 0.0.x versions are test builds for Google Play's internal testing
  track; 1.0.0 is the first release.
- **Identifier.** `it.domehub.emudome` (`identifier` in `tauri.conf.json`; the Android `applicationId` and
  `namespace` in `app/src-tauri/gen/android/app/build.gradle.kts`, the generated activity's package
  `it.domehub.emudome`, the platform library's `it.domehub.emudome.platform`). It names the data folders on every
  OS and is the package name on Google Play, which can never change once an app is published there. It was
  `com.domenghini.emudome` before (Trello #90): a desktop build copies the old folders once at its first
  start (`emudome_core::data_move`, the old ones are left in place); Android data does not carry over (another
  package's private storage cannot be read). The Android OAuth client for Drive sync (#49) is registered for
  `it.domehub.emudome` with the debug keystore's SHA-1, and gets the release keystore's SHA-1 before a release.
- **Packages.** The package formats and the files in each are in `app/src-tauri/package/<platform>.json`,
  generated from `cores.lock` by `node scripts/cores.mjs generate` (`targets`, `resources`): only the files of
  `cores.lock` and their licences, `cores/SOURCES.md`, EmuDome's `LICENSE` (as `LICENSE.txt`) and `THIRD_PARTY.md`
  (the licences page reads both, `about.lic`). Their `beforeBundleCommand` (`scripts/cores.mjs verify --package`)
  checks the cores against their sha256 first.
  - Windows: `pnpm package:windows` (from `app/`) builds the NSIS installer, **per user**
    (`bundle.windows.nsis.installMode: "currentUser"` in `tauri.conf.json`: no administrator rights, installed
    in the user's own local app data, Tauri's default for that mode). The data folders
    (`%APPDATA%\it.domehub.emudome`, `%LOCALAPPDATA%\it.domehub.emudome`) are outside the install folder, so
    uninstalling keeps saves, settings and the library. Tauri's uninstaller offers a «Delete the application
    data» box, unticked by default (Tauri's template; not seen on a real install yet): only a user who ticks it
    loses them (it deletes both data folders). An update (below) runs the same installer in passive mode, over
    the old version.
  - Linux: `pnpm package:linux` on Linux builds an AppImage and a deb (`targets` of `package/linux-x86_64.json`);
    the webview video path is the Linux default (`emudome-platform`). Never built yet: this machine has no Linux,
    and mGBA for Linux is still unpinned (#92), so `verify --package` refuses it and a Linux release stays blocked
    until #92.
  - Android: `./scripts/android-build.ps1 -Release` (`sh scripts/android-build.sh --release`) builds a signed
    release APK (arm64-v8a) from a clean clone; `-Bundle` (`--bundle`) builds the signed App Bundle (AAB) Google
    Play takes instead (`pnpm tauri android build --aab`, Gradle's `bundleArm64Release`; always a release, never
    with `-Install`). The keystore lives outside the repository, named by `EMUDOME_KEYSTORE`,
    `EMUDOME_KEYSTORE_PASSWORD`, `EMUDOME_KEY_ALIAS`, `EMUDOME_KEY_PASSWORD`
    (`app/src-tauri/gen/android/app/build.gradle.kts`). Only `-DryRun` has been run for `-Bundle`.
- **Cores.** Packages carry only the files `cores.lock` pins (version, URL or source build, sha256). mGBA for
  Windows and Android is built by EmuDome from the pinned source archive (Trello #88, `scripts/build-mgba.ps1`):
  the build is reproducible, but gives the pinned sha256 only with the toolchain `cores.lock` names (Visual Studio
  2022 Build Tools with MSVC 14.44, the Android NDK 27.1 and its clang-cl, the Windows SDK, CMake); another
  toolchain is refused. So the built files are published once, in a GitHub release of this repository named
  `cores-<core>-<first 7 characters of the commit>` (today `cores-mgba-7a12d6d`, Trello #95, and
  `cores-melondsds-f394adb`), and `cores.lock`
  names its download folder (`built.release`). The fetch scripts download them from there, sha256-checked, and
  build from source only when the download fails or with `-Build` / `--build`. That release hosts binaries only:
  the source offer is the pinned upstream source archive with the build scripts (`cores/SOURCES.md`). melonDS DS
  for Android is built the same way (`scripts/build-melondsds.ps1`, NDK 27.1, 16 KB pages), and
  so are the libraries its CMake would git-clone, each pinned in `cores.lock` (`build.dependencies`: commit, source
  archive, sha256, licence) and handed to CMake offline; its Windows and Linux files are still upstream's release
  assets. mGBA for
  Linux is still the libretro buildbot's `latest/`, which has no stable URL: packaging refuses it unless
  `EMUDOME_ALLOW_UNSTABLE_CORES=1`, allowed for local test packages only, never for a release, until it is built on
  Linux (Trello #92, a release blocker for Linux only). Azahar is not packaged (Phase 3).
- **Signing.** No Windows code signing for now (#50): the installer is unsigned and Windows SmartScreen warns on
  first run. Android release builds are signed with the project's release keystore, kept outside the repository.
  Desktop updates are signed with the project's update key (below).
- **Updates** (#44, #50: GitHub Releases, stable only). The desktop app updates itself with Tauri's
  updater plugin (`emudome-platform/src/updater.rs`, `emudome_core::updates`, `app/src-tauri/src/updates.rs`):
  it reads `latest.json` from the newest GitHub release that is not a pre-release
  (`https://github.com/DomenghiniStefano/emu-dome/releases/latest/download/latest.json`; the cores' releases are
  pre-releases, so they never count), downloads the package it names, checks its signature against the public
  key built into the app, and runs it. `general.updates` (per device) says when: automatic (look at start and
  every 6 hours, download in the background, then X06 offers «Riavvia e aggiorna»), ask first (look, offer; the
  download starts on «Riavvia e aggiorna»), off (only «Controlla ora» in Settings › Informazioni looks). Never in
  game: the card waits for the library, and installing refuses while a game is open. Android updates come from
  Google Play (the rows are desktop only).
  - The **public key** is a build-time value, `EMUDOME_UPDATER_PUBKEY` (like the Google client id): a build
    without it cannot update itself and says so (`general.updates` disabled, «This version of EmuDome can't
    update itself»). `tauri.conf.json` keeps `plugins.updater.pubkey` empty on purpose. `EMUDOME_UPDATER_ENDPOINT`
    may point a test build at another manifest (https only in a release build).
  - The **private key** never enters the repository nor the app: only the release build's environment has it
    (`TAURI_SIGNING_PRIVATE_KEY`, its content or the path of a file outside the repository, and
    `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). `pnpm package:windows:release` / `package:linux:release` add
    `package/updater.json` (`createUpdaterArtifacts`: the installer's `.sig` beside it), whose
    `beforeBuildCommand` runs `node scripts/updater-manifest.mjs check-env` first: both keys set, a key file
    outside the repository, no value printed. Losing the private key means the installed apps can never update
    again (a new key needs a manual reinstall): keep it backed up, outside the repository.
  - `latest.json` is written by `node scripts/updater-manifest.mjs write` from the built packages and their
    `.sig` files, and uploaded to the release with them.

## Test builds (Google Play's internal testing)

The 0.0.x versions go to testers through Google Play's internal testing track (no review, up to 100 testers
added by their Google account's address). `.github/workflows/stage.yml` does it on every push to `stage`, through `play-release.yml` (the job `stage.yml`
and `main.yml` share):
`sh scripts/android-build.sh --bundle` on an Ubuntu runner, then the AAB uploaded to the `internal` track
(r0adkll/upload-google-play, MIT, pinned to a commit), the AAB also kept 14 days as the run's artifact.

1. On `develop`, `node scripts/versions.mjs bump` (the patch; Play refuses a versionCode it has seen) and
   commit `chore: <version>`; `./scripts/check.ps1` green on it (the merge rule).
2. Merge `develop` into `stage` and push `stage`: the run builds and uploads. A failed run uploads nothing.

Once, by the owner, before the first run:
- Play Console (the DomeHub account): create the app `it.domehub.emudome`, turn on Play App Signing, and upload the
  first AAB by hand (Play takes uploads through its API only for an app that has one): `./scripts/android-build.ps1
  -Bundle` with the upload keystore.
- The upload keystore lives outside the repository (licence rule 6). Its SHA-1 also goes to the Android OAuth
  client (Drive sync, #49), with Play App Signing's own certificate.
- A Google Cloud service account with access to the Play Developer API, invited in Play Console (Users and
  permissions) with the release rights on the app.
- GitHub: an environment `play-internal`, its deployments limited to `stage`, with the secrets the workflow's
  header lists (the keystore as base64). Secrets never reach a workflow run from a fork's pull request.
- The testers' addresses in the internal testing track's list (and in the OAuth consent screen's test users while
  it is in «Testing», for Drive).

## The update key (once)

1. Outside the repository: `pnpm tauri signer generate -w <a folder outside the repo>/emudome-updater.key` (from
   `app/`), with a password. It writes the private key (`emudome-updater.key`) and the public key
   (`emudome-updater.key.pub`). Back the private key and its password up; never commit, paste or upload them.
2. Keep the public key's content for `EMUDOME_UPDATER_PUBKEY` (it is public: it may go in the release notes or a
   machine's environment, never needs hiding, but the build reads it from the environment, not from the repo).

## The release flow

1. On `develop`, raise the version (`node scripts/versions.mjs bump`); `./scripts/check.ps1` green on that commit
   (the merge rule).
2. Merge `develop` into `main` (fast-forward) and push it: `main.yml` builds the Android package and, after your
   approval, tags that commit `v<version>`. The tag is the release: every package of it is built from the tagged
   commit, never from a working copy.
3. Build each package **from a clean clone** of the tag (`git clone --branch v<version>`, no leftovers in
   `target/`, `node_modules/` or `cores/`):
   - `./scripts/check.ps1` (Windows) / `sh scripts/check` (Linux) green in that clone;
   - the cores fetched from `cores.lock` (`./scripts/fetch-cores.ps1`, `sh scripts/fetch-cores.sh`,
     `./scripts/fetch-cores-android.ps1`; mGBA and Android's melonDS DS downloaded from their `cores-<core>-<commit>`
     releases), each checked
     against its sha256, with no `EMUDOME_ALLOW_UNSTABLE_CORES`;
   - the update keys in the environment of that shell only (PowerShell: `$env:EMUDOME_UPDATER_PUBKEY = Get-Content
     <folder>\emudome-updater.key.pub -Raw`, `$env:TAURI_SIGNING_PRIVATE_KEY = '<folder>\emudome-updater.key'`,
     `$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<password>'`);
   - Windows: `pnpm package:windows:release` (the NSIS installer and its `.sig`, in
     `target/release/bundle/nsis/`); Linux: `pnpm package:linux:release` on Linux (blocked until #92); Android:
     `./scripts/android-build.ps1 -Bundle` for Google Play (and `-Release` for an APK to attach);
   - `node scripts/updater-manifest.mjs write --version <version> --notes "<what changed>" --out latest.json
     windows-x86_64=<the setup .exe> [linux-x86_64=<the .AppImage>]`.
4. Publish (#50):
   - **GitHub Releases** of the repository, on the tag, **not** a pre-release (the updater only sees the latest
     full release): the installers with their `.sig` files under the names `latest.json` gives, `latest.json`,
     the APK, `cores/SOURCES.md` and the licences next to them (the source offer for the cores,
     `THIRD_PARTY.md`). Upload `latest.json` last: from then on every installed app sees the new version;
   - **domehub.it**, the same files;
   - **Google Play**, under the DomeHub developer account: `.github/workflows/main.yml` on the push to `main`
     builds the AAB and, once the owner approves the deployment (the environment `play-production`: `main` only,
     the owner as required reviewer, its own service account with the production rights), uploads it to the
     production track and tags the commit `v<version>` (a lightweight tag, made by the run; a version already
     tagged stops it before the build). Run by hand (Actions > main > Run workflow) it can go to the closed
     testing track `alpha` instead, tagging nothing: a personal Play account needs a closed test with at least
     12 testers for 14 days before its first production release. The desktop packages are not in the workflow
     yet (the steps above).
5. Merge `main` back into `develop` if anything was committed on the way.

## A new pin of a core EmuDome builds (today mGBA, and melonDS DS for Android)

A new commit, or a toolchain change, gives new bytes: they are built once, checked, published, then pinned.

1. In `cores.lock`, change the core's `source` (commit, version, `archive`, and `archive_sha256`: download the
   archive and hash it) and `license` (`url` at the new commit, its `sha256`); remove `built.release` from the
   entries being re-pinned (the check refuses a release named after another commit). A core whose CMake fetches
   libraries at configure time (melonDS DS: `cmake/FetchDependencies.cmake`) lists each in `build.dependencies`
   with the name FetchContent gives it, the commit its ref names at the new commit (a branch: its head when the
   upstream release was built), the archive `https://github.com/<repo>/archive/<commit>.tar.gz`, its sha256 and
   its licence (checked GPL-compatible first); a library missing from the list fails the configure, since the
   build passes `FETCHCONTENT_FULLY_DISCONNECTED`.
2. Build each platform on a machine with the toolchain to record: `./scripts/build-mgba.ps1 -Platform
   windows-x86_64` and `-Platform android-arm64-v8a`, `./scripts/build-melondsds.ps1` (or
   `./scripts/fetch-cores.ps1 -Build`). A notice the build writes (`output`, melonDS DS's
   `melondsds-LICENSE.txt`) lands in `out/` too, with its sha256 printed. The sha256 differs
   from the old pin, so the install is refused and the file stays in `<work dir>/<core>/<platform>/out/`: put its
   sha256 in the entry's `library.sha256` and the toolchain printed by the build in `built.toolchain`, then build
   again; now it installs. Build twice from a clean work folder to see the same sha256.
3. Publish a GitHub release of this repository with the tag `cores-<core>-<first 7 characters of the commit>`
   (e.g. `cores-mgba-7a12d6d`), never a `v<version>` tag (mark it as a pre-release, so that it never shows as the
   app's latest release). Its assets are the installed files under their file names: every platform's library,
   the licence (`license.file`, from `cores/notices/<platform>/`), each notice, and a `SHA256SUMS.txt` of them.
   Download the assets again and compare their sha256 with the lock.
4. Add `"release": "https://github.com/DomenghiniStefano/emu-dome/releases/download/<tag>/"` to each `built`
   entry, run `node scripts/cores.mjs generate` and `node scripts/cores.mjs check --self-test`, update
   `THIRD_PARTY.md` (commit, release), then fetch into an empty `cores/` (a clean clone) and see the download, not
   a build, give the pinned files.

A published release is never replaced or edited: a new build is a new pin and a new tag.

## Android: 16 KB pages

Google Play takes an app targeting Android 15+ only if it supports 16 KB memory pages (updates without it are
refused from 2027-02-01), and a device with 16 KB pages runs an app with a 4 KB-aligned library only in its
compatibility mode, with a warning on first launch (seen on a Galaxy S23, Android 17: «Questa app non è
compatibile con dimensioni pagina di 16 kB»). Two halves, both needed for every library in the APK:

- **In the zip**: libraries stored uncompressed and 16 KB-aligned. AGP 8.5.1+ does it with
  `packaging.jniLibs.useLegacyPackaging = false` (written in `app/src-tauri/gen/android/app/build.gradle.kts`;
  AGP 9.3.1 today). Check: `zipalign -c -P 16 -v 4 <apk>` (SDK build-tools 35+) ends with «Verification
  successful».
- **In the ELF**: every `LOAD` segment `Align 0x4000`, and `GNU_RELRO` either ending on a 16 KB boundary or filling
  its `LOAD` segment. Check: `llvm-readelf -lW <lib>.so` (the NDK's) on each library taken out of the APK.
  - `libemudome_app.so` (ours): linked with `-z max-page-size=16384 -z common-page-size=16384`
    (`.cargo/config.toml`), because NDK 27 links with 4 KB pages by default.
  - `libc++_shared.so`: the NDK 27.1 one is already 16 KB-aligned.
  - `libmgba_libretro_android.so`: built by EmuDome with `-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON`
    (`scripts/core-build.mjs`), so the pinned `cores-mgba-7a12d6d` file is already 16 KB-aligned (`LOAD` align
    0x4000, `GNU_RELRO` ending at 0x164000). Nothing to re-pin.
  - `libmelondsds_libretro_android.so`: upstream's v1.4.0 release asset was not aligned (built by its CI with
    NDK r27d, `LOAD` align 0x1000), so EmuDome builds it from the pinned v1.4.0 source and its pinned dependencies
    with `-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON` (`scripts/build-melondsds.ps1`), published as
    `cores-melondsds-f394adb`: `LOAD` align 0x4000, `GNU_RELRO` filling its `LOAD`.

## Still open

- No release was built yet: the per-user installer, the uninstaller keeping the data, an update from one version
  to the next and the AAB are still to be tried on real machines (#44).
- The Linux packages were never built (no Linux here); mGBA for Linux built from source (#92), then published in
  the same kind of release.
- Windows code signing: not for now (#50).
- The update's TLS goes through the system (`native-tls`: SChannel, OpenSSL on Linux): Tauri's default rustls
  would add `webpki-root-certs` (CDLA-Permissive-2.0), not on `deny.toml`'s list; not decided yet.
