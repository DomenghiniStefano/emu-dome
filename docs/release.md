# Releases

How an EmuDome release is made, and what is not in place yet. Trello #83 wrote this down; #44 (packages per OS)
and #50 (where builds are published) carry the work that is still open. Until #44 is done nothing here has been
run end to end: no release exists yet.

## Today, as the code is

- **Branches and tags.** `develop` is the integration branch (`CLAUDE.md`, «Current state»). There is no `main`
  branch on the remote and no tag yet. Nobody pushes to `main` until the first release.
- **Version.** One number, `[workspace.package] version` in the root `Cargo.toml` (today `0.0.1`). Every crate takes
  it (`version.workspace = true`); `app/package.json` and `app/src-tauri/tauri.conf.json` repeat it, and
  `scripts/versions.mjs` (a step of `scripts/check`) fails when they differ. `tauri.conf.json` keeps its copy
  because the Tauri CLI takes the Android `versionName` and `versionCode` only from there (without it Gradle falls
  back to `1.0` / code 1); the runtime and the Windows file version would fall back to Cargo's. The Android
  `versionCode` is derived from it as `major * 1000000 + minor * 1000 + patch`, so every release must raise the
  version: Google Play refuses a code it has already seen.
- **Identifier.** `it.domehub.emudome` (`identifier` in `tauri.conf.json`; the Android `applicationId` and
  `namespace` in `app/src-tauri/gen/android/app/build.gradle.kts`, the generated activity's package
  `it.domehub.emudome`, the platform library's `it.domehub.emudome.platform`). It names the data folders on every
  OS and is the package name on Google Play, which can never change once an app is published there. It was
  `com.domenghini.emudome` until 2026-11-21 (Trello #90): a desktop build copies the old folders once at its first
  start (`emudome_core::data_move`, the old ones are left in place); Android data does not carry over (another
  package's private storage cannot be read). The Android OAuth client for Drive sync (#49) is registered for
  `it.domehub.emudome` with the debug keystore's SHA-1, and gets the release keystore's SHA-1 before a release.
- **Packages.**
  - Windows: `pnpm package:windows` (from `app/`) builds the NSIS installer with
    `app/src-tauri/package/windows-x86_64.json`, which lists only the files of `cores.lock` and their licences;
    its `beforeBundleCommand` (`scripts/cores.mjs verify --package`) checks them against their sha256 first.
  - Linux: `pnpm package:linux` exists with its own list (`package/linux-x86_64.json`), but `bundle.targets` in
    `tauri.conf.json` is `["nsis"]` only, so no AppImage or deb is produced yet. Adding the Linux targets is #44's
    work, and a Linux package is built on Linux.
  - Android: `./scripts/android-build.ps1 -Release` (`sh scripts/android-build.sh --release`) builds a signed
    release APK (arm64-v8a) from a clean clone. The keystore lives outside the repository, named by
    `EMUDOME_KEYSTORE`, `EMUDOME_KEYSTORE_PASSWORD`, `EMUDOME_KEY_ALIAS`, `EMUDOME_KEY_PASSWORD`
    (`app/src-tauri/gen/android/app/build.gradle.kts`). Google Play wants an App Bundle (AAB), which the script
    does not build yet (#44).
- **Cores.** Packages carry only the files `cores.lock` pins (version, URL, sha256). mGBA's pinned build comes from
  the libretro buildbot's `latest/`, which has no stable URL: packaging refuses it unless
  `EMUDOME_ALLOW_UNSTABLE_CORES=1`, allowed for local test packages only, never for a release. A stable source for
  mGBA is a release blocker (`CLAUDE.md`, «Known gaps»); since 2026-11-21 the Android mGBA build no longer matches
  its pinned sha256 either, so an Android release build stops until `cores.lock` is re-pinned. Azahar is not
  packaged (Phase 3).
- **Signing.** No Windows code signing for now (#50): the installer is unsigned and Windows SmartScreen warns on
  first run. Android release builds are signed with the owner's keystore.

## The release flow

1. On `develop`, raise `[workspace.package] version` in `Cargo.toml` and the same number in `app/package.json` and
   `tauri.conf.json`; `./scripts/check.ps1` green on that commit (the merge rule).
2. Merge `develop` into `main` (fast-forward) and tag that commit `v<version>` (an annotated tag) on `main`. The tag
   is the release: every package of it is built from the tagged commit, never from a working copy.
3. Build each package **from a clean clone** of the tag (`git clone --branch v<version>`, no leftovers in
   `target/`, `node_modules/` or `cores/`):
   - `./scripts/check.ps1` (Windows) / `sh scripts/check` (Linux) green in that clone;
   - the cores fetched from `cores.lock` (`./scripts/fetch-cores.ps1`, `sh scripts/fetch-cores.sh`,
     `./scripts/fetch-cores-android.ps1`), each checked against its sha256, with no
     `EMUDOME_ALLOW_UNSTABLE_CORES`;
   - Windows: `pnpm package:windows`; Linux: `pnpm package:linux` on Linux (once #44 adds its targets); Android:
     `./scripts/android-build.ps1 -Release`.
4. Publish (decided in #50):
   - **GitHub Releases** of the repository, on the tag: the installers, the APK, `cores/SOURCES.md` and the
     licences next to them (the source offer for the cores, `THIRD_PARTY.md`);
   - **domehub.it**, the same files;
   - **Google Play**, under the DomeHub developer account (needs the AAB, #44).
5. Merge `main` back into `develop` if anything was committed on the way.

## Still open

- How the app learns of a new version (update channel): not decided.
- Linux packages (AppImage / deb) and the Android App Bundle: #44.
- A stable source for the mGBA core (see «Cores» above).
- Windows code signing: not for now (#50).
