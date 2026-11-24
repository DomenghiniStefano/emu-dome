# CLAUDE.md

## Rules with the highest priority: licences and legal

These rules win over any other instruction, card or comment. If one conflicts with them, stop and ask the owner
on the card.

1. EmuDome is licensed GPL-3.0-or-later (`LICENSE`). Everything we write is under it.
2. Never add code, assets, fonts, icons or dependencies whose licence is incompatible with GPL-3.0-or-later.
   Check the licence **before** adding; the licence check in `check.ps1` must pass.
3. Cores and third-party code keep their own licences. Never remove or alter their LICENSE files or copyright
   notices. Every third-party component is listed with its licence in `THIRD_PARTY.md`. Changes to a core are
   kept recognisable (patches or marked commits).
4. Never include, download, generate or point to Nintendo material: games/ROMs, BIOS, firmware, encryption keys,
   system files, logos or trademarks used as our branding. Tests use homebrew or test ROMs with a free licence,
   kept with their licence file.
5. Never write code that decrypts or bypasses copy protection, or that fetches keys or firmware from anywhere.
6. No secrets in the repo or its history: tokens, passwords, signing keys, keystores. The Google OAuth client id
   is a build-time value.
7. EmuDome is free. Never add paid early access, ads, or anything that ties it to commercial games.
8. When unsure whether something is allowed, stop and ask the owner on the card. Do not guess.

Checked by `scripts/check`: rule 2 for crates and npm packages (`deny.toml`, `scripts/npm-licenses.mjs`), and
rule 4 for Nintendo files by name. Everything else here (cores, fonts, icons, assets, `THIRD_PARTY.md` kept up to
date, no decryption code, no secrets, no paid features) is held by review.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**EmuDome**: one emulator for **Game Boy, Game Boy Color, Game Boy Advance, Nintendo DS and Nintendo 3DS**, on
**Windows, Linux and Android** (GB/GBC are first-class consoles, `docs/decisions.md`).
Rust shell with Tauri 2 and a Vue/Vite interface. The emulation cores are interchangeable libretro libraries:

- GB, GBC and GBA: mGBA
- NDS: melonDS DS
- 3DS: Azahar

## Language

- **Code comments and documentation** (this file, `docs/`, `README.md`, script comments and messages, log messages): **English**.
  Names in code (types, functions, variables) in English.
- **User interface**: **never hardcoded**. Every user-facing string lives in the vue-i18n message files
  (`app/src/i18n/`): `en.json` is the default and the fallback, `it.json` the Italian translation (same keys).
  The language follows the OS / browser locale; anything that is not Italian gets English.
- The Rust backend sends **structured data, never sentences**: errors are `ErrorCode` (`crates/emudome-core/src/error.rs`),
  serialized as `{ code, params }`, its TypeScript type generated into `app/src/generated/` (see «The Rust ↔
  TypeScript contract»); events carry codes and values
  (e.g. `emu://gamepad-connected` with `{ name }`). The interface turns them into text in `app/src/i18n/labels.ts`
  (`errorText`, `speedLabel`, `gamepadName`). A new error = a new `ErrorCode` variant, the generated files made
  again (`EMUDOME_BLESS=1 cargo test -p emudome-core contract`) + `errors.<code>` in **both** message files: the
  contract test and `app/src/i18n/parity.test.ts` fail until all three are there. A file or folder that cannot be
  read or created is sent as values, never as the OS's sentence (which holds the path; that goes to the log): its
  name / path and an `IoCause` (`notFound`, `permissionDenied`, `other`, from the `io::ErrorKind`; `IoCause::of_any`
  finds it in an `anyhow` chain), each code with a cause having one text per cause, `errorCauses.<set>.<cause>`:
  the code's own set, or `data`, shared by the codes about a file in EmuDome's own data folder (settings, library,
  notifications, saves, states, screenshots, BIOS copies, the sync index). A failure EmuDome tells apart further
  carries a small `reason` enum (`ImageProblem` for `coverInvalid`, `SignInProblem` for `cloudSignInFailed`), one
  text per value, `errorReasons.<code>.<reason>`; anything else (a loader's, a GPU's, the network's or the
  updater's text) goes to the log where it happens and the code carries no detail at all. `app/src/i18n/causes.ts`
  lists the codes, sets and reasons; parity checks every text and that no `errors.*` message has a `{detail}`.
  `romUnreadable` carries `file` (a document's display name), `path` (a document's URI) and `cause`; G02 shows the
  file name, the cause's text, then the path on a muted line clamped to two.
- Kept file names: `docs/fase0-esito.md` and `docs/combo-tasti.md` keep their Italian names (other documents link to them); their content is English.

## Current state: Phase 1 desktop shell, Phase 2 core work started

Phase 0 (feasibility) is done. On `develop` now: shell and themes, settings engine and pages, library, game flow,
quick menu, saves, controls, first run, rewind and screenshots (keyed by the game, #85), covers from a screenshot or
the user's image framed in «Adatta la copertina» (#85), screen effects (librashader), appearance, and the
melonDS DS core (#45), and 3DS games open on Azahar (#46, first milestone: Vulkan on the app's device, frames read
back to the CPU, encrypted games refused with E10), and the Google Drive sync (#20 backend; #31 wiring: pull
before launch, heartbeat, push after exit, «Sincronizza ora», games on Drive, the `cloud_*` commands and
`cloud://` events; the Cloud section, G03 / G04 / G05 / G14 and Drive-only games in the library), and the search everywhere
and the notification centre in the top bar (#32, X01–X03), and the release plumbing (#44: per-user installer, Linux
package targets, the Android App Bundle, EmuDome updating itself with X06, the licences page, the log file and its
export; no release built yet, see `docs/known-gaps.md`). The audit put its findings on Trello (#69–#83, P0 first): the P0 cards come
before any new feature.

Git: `develop` is the integration branch, checked out in the main worktree; work happens on branches in
`C:\Projects\emu-dome.wt\<name>` and is merged into `develop` from the main worktree. Pushing `develop` is allowed.
`stage` takes `develop` (merged, version raised) for a test build on Google Play's internal track, pushed only when
the owner asks for one (its CI uploads it, `.github/workflows/stage.yml`, `docs/release.md` «Test builds»).
No push to `main` (from 1.0.0, the release: `.github/workflows/main.yml` uploads to Play's production after the
owner's approval). **Merge rule**: `./scripts/check.ps1` green on the merge commit itself
(not only on the branch) before `git push` of `develop`; `wip` commits are squashed before merging, never merged as
they are. **Checks on work branches**: only targeted ones while working (the
touched crates' tests and clippy, typecheck, lint, the touched Vitest files), `./scripts/check.ps1 --fast` once at the
end of a branch; the full check runs only on the merge commit into `develop`. No build only to measure or to confirm
something already verified.

Phase 0 answered the architecture question: the native video under a transparent webview holds up. The full test
log, entry by entry, is `docs/fase0-esito.md`; in short:

- **Linux** (virtual X11, mGBA 0.10.2): windowless host 1× ≈ 59.7 FPS, 2× ≈ 120, Max ≈ 900; app in webview mode
  works. Native mode does not (the GTK surface covers the webview).
- **Windows**: native video under the quick menu ok (wgpu picks Vulkan), 59.7 FPS at 1×, audio heard ok,
  controllers ok through SDL3 (Steam Controller, 8BitDo). Last reported open: the window turns transparent in webview mode.
- **Android** (Galaxy A71): `SurfaceView` under the WebView ok, ≈ 59.8 FPS (Vulkan), touch controls,
  surface survives background/resume. Not tested yet: audio quality and delay (the test ROM is silent),
  Android controllers (read through Android's input events since #47, not run on a device yet), landscape layout.
- Since then, on `develop`: L3 controller combos, controller navigation in every menu, the L01 app shell and themes,
  the settings engine (2a).

## Commands

```powershell
./scripts/fetch-cores.ps1 [-Cores mgba,melondsds,azahar|all] [-Build]  # cores pinned by cores.lock into cores/, sha256-checked
                                              # (default: the shipped ones; sh scripts/fetch-cores.sh [--cores ...] [--build] on Linux);
                                              # mGBA (and Android's melonDS DS) come from our releases cores-<core>-<commit>
                                              # (cores.lock built.release), built from source (as build-mgba.ps1 /
                                              # build-melondsds.ps1) only if that download fails or with -Build
./scripts/fetch-cores-android.ps1 [-Cores ...] [-Abi arm64-v8a] [-Build]  # same, into cores/android/<abi>/ as lib*_libretro_android.so
./scripts/build-mgba.ps1 [-Platform windows-x86_64|android-arm64-v8a] [-Jobs 2]  # mGBA from the source cores.lock
                                              # pins, installed only if its sha256 is the pinned one (scripts/core-build.mjs:
                                              # VS 2022 Build Tools + the NDK's clang-cl, NDK for Android, git; sh build-mgba.sh)
./scripts/build-melondsds.ps1 [-Jobs 2]          # melonDS DS for Android from the source cores.lock pins, with every
                                              # library its CMake would git-clone pinned there (build.dependencies) and
                                              # fed to CMake offline; 16 KB pages (NDK 27.1, git; sh build-melondsds.sh)
./scripts/android-build.ps1 [-Install] [-Release] [-Bundle] [-DryRun]  # the Android APK (arm64-v8a) from a clean
                                              # clone: cores, frontend, Rust, Gradle; prints the APK; -Install = adb
                                              # install -r; -Bundle = the signed AAB for Google Play (a release)
                                              # (sh scripts/android-build.sh [--install] ... on Linux / Git Bash)
node scripts/cores.mjs generate               # after editing cores.lock: cores/SOURCES.md, app/src-tauri/package/*.json
pnpm package:windows                          # installer (from app/): only the cores.lock files, verified (THIRD_PARTY.md)
pnpm package:windows:release                  # the same, signed for the updater (`.sig`): needs the update keys in the
                                              # environment (TAURI_SIGNING_PRIVATE_KEY, EMUDOME_UPDATER_PUBKEY; never in
                                              # the repo); package:linux:release on Linux
node scripts/updater-manifest.mjs write ...   # latest.json of a release (check-env, --self-test)
                                              # releases (tag on main, clean clone, where published, the update key):
                                              # docs/release.md
node scripts/third-party-tables.mjs --write   # after a dependency change (and pnpm install): THIRD_PARTY.md's tables
node scripts/release.mjs internal|production [--version patch|minor|major|x.y.z] [--dry-run]
                                              # a release from develop: version raised and committed, full check,
                                              # develop pushed, then to stage (Play internal test) or main (Play
                                              # production after approval); 0.0.x test builds, 1.0.0 the first
node scripts/versions.mjs [bump|print]       # the version, written only in Cargo.toml's [workspace.package];
                                              # the Android build passes it to Tauri (--config)
git config core.hooksPath scripts/git-hooks   # once per clone: pre-commit runs the fast checks
./scripts/check.ps1                           # EVERY check (scripts/check): run before declaring work done
                                              # same list: `sh scripts/check` (Git Bash, Linux), `pnpm check` (from app/)
./scripts/check.ps1 --fast                    # on a work branch: no Android clippy, cargo test only on the
                                              # packages changed since develop (both said skipped); never for a merge
rustup target add aarch64-linux-android       # once: the Android clippy steps of the check
nvm use                                       # Node from .nvmrc (engines.node in app/package.json; the check stops on another)
cd app; pnpm install                          # frontend dependencies (pnpm only: npm refuses, see docs/decisions.md)
pnpm tauri dev                                # app in development (from app/)
pnpm lint                                     # ESLint (app/eslint.config.js)
pnpm format                                   # Prettier writes; format:check only checks
pnpm typecheck                                # vue-tsc
pnpm test                                     # Vitest (pure functions, primitives, the nav.ts rule)
pnpm dev, then http://localhost:<port>/#kit   # Kit page: every primitive in every state (development only)
node scripts/scan-conventions.mjs             # design system conventions (--self-test: each rule on its fixture)
cargo test --workspace
cargo test -p emudome-core settings::         # one module (settings tests live in settings/tests.rs)
cargo test -p emudome-core hotkeys::<name>    # one test
EMUDOME_BLESS=1 cargo test -p emudome-core contract   # write app/src/generated/ again after a DTO / catalogue / ErrorCode
EMUDOME_BLESS=1 cargo test -p emudome contract        # change (PowerShell: $env:EMUDOME_BLESS=1; ...); review, commit
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target aarch64-linux-android -p libretro-sys -p emudome-core -p emudome-platform -- -D warnings
                                              # Android without the NDK (the whole workspace needs it: glslang-sys)
cargo fmt --all
cargo run -p emudome-core --example headless -- <core> tools/test-rom/emudome-test.gba   # windowless core host
                                              # (--release ... --timing 20 --rewind 30: per-frame work p50/p99/max)
cargo run -p emudome --example gamepad_probe  # windowless controller probe on emudome_platform::pads (SDL3 on
                                              # the desktop; app/src-tauri/examples)
cargo run -p emudome-sync --example drive_probe -- status|sign-in [--print-url]|sign-out|list|quota|upload <f>|download <n> <out>
                                              # the real Google sign-in and Drive calls (owner's UAT only: it opens
                                              # the browser, talks to Google, keeps the refresh token in the system's
                                              # credential store; needs EMUDOME_GOOGLE_CLIENT_ID at build time)
```

- On Windows `sh` is not on the PATH that PowerShell and pnpm see, so there use `./scripts/check.ps1`.
- Build speed: with sccache installed, `RUSTC_WRAPPER=sccache` (user environment) shares the compiled dependencies
  between worktrees; each worktree keeps its own `target/` (see below). The dev profile keeps line tables only
  (`debug = true` locally for a debugger session).
- Android build (Trello #76): `./scripts/android-build.ps1` is the one command; its header lists what it needs (JDK 17,
  Android SDK with platform 37 and an NDK, `rustup target add aarch64-linux-android`, pnpm). It runs
  `pnpm tauri android build --apk --split-per-abi --target aarch64 --debug`; the APK is
  `app/src-tauri/gen/android/app/build/outputs/apk/arm64/debug/app-arm64-debug.apk`. On Windows it needs **developer
  mode on** (Settings > System > For developers): the Tauri CLI symlinks `libemudome_app.so` into jniLibs and Windows
  refuses symlinks otherwise ("Creation symbolic link is not allowed"); the script checks it first. Gradle runs
  without a daemon, and with Gradle's configuration cache off (`gen/android/gradle.properties`: the platform library's
  `emudomeCores` task is not cacheable yet).
- The Android project `app/src-tauri/gen/android` is **versioned** (generated once by `pnpm tauri android init`,
  tauri-cli 2.12.4); its `.gitignore` files keep out the build outputs and what tauri-build writes with machine paths
  (`tauri.settings.gradle`, `tauri.build.gradle.kts`, `tauri.properties`, the jniLibs `.so`, generated Kotlin). Do not run
  `tauri android init` again on it: it would overwrite our changes (release signing, launcher icons). On a Tauri
  upgrade, generate into a scratch copy and carry the differences over by hand. The cores and the SurfaceView come
  from `crates/emudome-platform/android` (included by tauri-build), never from the app project's own jniLibs.
- Android release signing: `-Release` / `assembleArm64Release` read `EMUDOME_KEYSTORE` (path of a keystore **outside**
  the repo), `EMUDOME_KEYSTORE_PASSWORD`, `EMUDOME_KEY_ALIAS`, `EMUDOME_KEY_PASSWORD` from the environment
  (`gen/android/app/build.gradle.kts`); without them, or with the keystore inside the repo, the release build stops.
  Never a keystore or a password in the repo (licence rule 6; `scripts/check` refuses keystore files).
- Android check: the whole workspace for `aarch64-linux-android` needs the NDK (`ANDROID_NDK_HOME` / `NDK_HOME`, or
  `ANDROID_HOME/ndk/<version>`): librashader's glslang-sys builds C++ for the target. Without it the check runs
  the crates with no C code and says the rest is skipped. Linux is checked only by running the check on Linux (WSL).
- `crates/hw-render-spike` is its own workspace (root `exclude`): `cargo run --release --manifest-path
  crates/hw-render-spike/Cargo.toml -- ...`; the checks never build it.
- The SDL3 build needs CMake. If it is not on PATH, set `CMAKE` for that command only (VS Build Tools 2022 ships one).
- Every worktree builds in **its own** cargo target dir (the default `target/` inside it). Never share
  `CARGO_TARGET_DIR` between worktrees: workspace crates get the same artifact names everywhere and cargo can reuse
  another worktree's stale build.
- Environment variables:
  - `EMUDOME_VIDEO=native|webview` forces the video path.
  - `EMUDOME_CORES_DIR` tells where to look for the cores (debug builds only: a release loads only the cores in its
    resource folder, each checked against `cores.lock`'s sha256, `crates/emudome-platform/src/cores.rs`).
  - `EMUDOME_THEME=dark|black|light` forces the theme (otherwise the `general.theme` setting, then the system).
  - `RUST_LOG=debug` for the logs (stderr / logcat, and the log file `<app local data>/logs/emudome.log`, the
    previous run's beside it as `emudome-previous.log`; `about.log` exports it).
  - `EMUDOME_UPDATER_PUBKEY` (build time): the public update key; without it the build cannot update itself.
    `EMUDOME_UPDATER_ENDPOINT` (build time, optional): another update manifest (docs/release.md).
  - `EMUDOME_DATA_DIR=<folder>` puts every data file there (saves, settings, library, covers, webview profile) and
    skips the one-time copy of the old identifier's folders: test and bench runs never touch the real data.
- Data folders: Tauri's app data / app local data folders for the identifier `it.domehub.emudome` (`app/src-tauri/src/dirs.rs`).
  The previous identifier was `com.domenghini.emudome` (#90): a desktop build's first start copies those folders once,
  verified, and leaves them in place (`emudome_core::data_move`); a copy that did not finish makes that run use the
  old folders and is tried again. Android data does not carry over (another package's storage).
- The executable accepts a game path as an argument and opens it at startup. On Android: the `game` extra of the
  launch intent, else the `content://` document of «Open with» (`ACTION_VIEW`, octet-stream).
- Android game folders and files (Trello #47, Storage Access Framework): «Aggiungi una cartella» is
  `ACTION_OPEN_DOCUMENT_TREE` and «Apri un file» `ACTION_OPEN_DOCUMENT` (every file offered: ROMs have no MIME type;
  Rust checks the extension, `unsupportedFormat`), both from `SystemBridge.kt` through an activity result, the read
  permission persisted. A folder is stored in `library.folders` as its tree URI (never a path; the desktop's paths
  unchanged), shown by its display name (`LibraryFolder.name`: the provider's, else the one kept in library.json's
  `folderNames` from an earlier read, else the last folder of the tree's document id, `document_tree_label`, #141),
  walked by the scan one `listTree` call per folder
  (`emudome_core::library::scan_with_documents`, games keyed by document URI + size + mtime), unreachable (E01) when
  it cannot be listed (permission given back, folder gone; `LibraryFolder.access`: `permissionLost` when the app no
  longer holds the tree's persisted permission, `SystemBridge.treePermitted`, as after an uninstall that kept the data,
  #138: E01 then says to pick it again, its picker opening on that folder (`library_choose_folder`'s `start`, the
  picker's `EXTRA_INITIAL_URI`), and picking the same folder keeps its entry and games), its persisted
  permission given back when removed; not
  watched (rescan at start / on demand). A document that does not open says why (`SystemBridge.openDocument`'s
  negative codes: not found, permission denied, other; `ContentError::Unreadable`'s `kind`, the `IoCause` of
  `romUnreadable`). Every document is read through its descriptor (`ContentFile::reader`):
  Android 17 refuses `/proc/self/fd/<fd>` for some providers (Downloads, «Permission denied»), so identification,
  hashing, icons and the bytes a core gets come from the descriptor; a `need_fullpath` core (Azahar) gets the link's
  path only when it opens, else the open is refused (`documentNeedsPath`). A file outside the library is titled by
  the document's display name (`game_file_title`), never its URI. An image (L12 «Immagine tua», the background
  image) is `ACTION_OPEN_DOCUMENT` with the PNG / JPEG / WebP types (`SystemBridge.pickImage`), opened once
  (`openOnce`, its permission not persisted) and read through its descriptor, at most 16 MiB
  (`emudome_platform::pick_image`, the desktop's file dialog behind the same call). Logcat: `android folder picked:`,
  `android file picked:`, `android image picked:`, `library scan: tree …`, `document … opened: … reopenable by path …`.
- Android's back button (Trello #14, `SystemBridge.kt` → `hotkeys::back_button`): in game with the quick menu
  closed it is the controller's menu button (`emu://menu`), anywhere else its B (`emu://nav` back). At the shell's
  root on the phone (#29) B goes to the section left last (the route's `trail`), then the Library, then leaves to the
  background (`leave_to_background` → `SystemBridge.moveToBack`, the app keeps running). Both events carry `system:
  true`, so the interface shows no focus ring for it and keeps the input in use as it was (#131). Not yet: «Open
  with» while the app already runs (needs the interface).
- Android's system bars (#131, `emudome_platform::set_immersive` → `SystemBridge.setImmersive`): shown in the shell,
  which is edge-to-edge (`enableEdgeToEdge`) and keeps inside the bars through the `safe-*` spacing tokens
  (`env(safe-area-inset-*)`, which the WebView reports); hidden (a swipe shows them for a moment) while a game is
  open, the quick menu and its editors included (a hook of `game_hooks`), and while C06 / C09 are open from Settings
  (`set_immersive` command, `composables/useImmersive.ts`), so the editors place things as a game is placed. Applied
  again each time the window gets the focus back.
- Homebrew test ROM: `tools/test-rom/emudome-test.gba`. Magenta screen, green while A is held (X key).

## Architecture

```
crates/libretro-sys      libretro API types and constants (added as they are needed)
crates/emudome-core      libretro host: LoadedCore (libloading), Session (emulation thread),
                         Command/Event, Shared (RGBA frame, audio ring, input, stats), speed, states, .sav,
                         ErrorCode (errors for the UI as code + params),
                         session/ (emu_loop.rs: the `Core` trait and `EmuLoop`, one method per `Command`, one
                         `step` per frame, every time passed in, tested on a fake core; libretro.rs: `LoadedCore`
                         + its callback slot as the `Core`; io.rs: the I/O worker),
                         testing.rs (`FakeCore`, `Gate`: a core without an emulator for the session's and the
                         shell's tests, `Session::start_with`),
                         console.rs (`ConsoleId`, `ConsoleId::ALL`; re-exported as `settings::ConsoleId`),
                         atomic.rs (`write_atomic`: the one way user data is written: temp file
                         `.<name>.<pid>-<n>.emudome-tmp`, fsync, rename, folder fsync where allowed),
                         hotkeys.rs (controller routing: game / combos / menu navigation, the combo table),
                         keyboard.rs (the keyboard's fixed keys `KEYBOARD_SHORTCUTS` and `KeyRouter`: keys in
                         game → game buttons and the same hotkey actions as the controllers; pure, unit-tested),
                         settings/ (settings engine: schema.rs types, catalog.rs the 169 entries,
                         mod.rs levels / resolve / session, store.rs files,
                         keys.rs the keys the Rust code reads, typed and checked against the catalogue when
                         compiled, and `Reader` (a value or the catalogue's default),
                         core_values.rs settings → core options, one table per core in core_values/<core>.rs;
                         pure, unit-tested)
                         video_sink.rs (`VideoSink`: where frames go, `VideoFrame::{Cpu, Gpu}`; the default
                         sink puts RGBA8 into `Shared::frame`), hw_render/ (`HwRenderHandler`: SET_HW_RENDER
                         and the calls around it; `SoftwareOnly` refuses them, `vulkan::VulkanHandler`
                         accepts Vulkan on the shell's device: interface table, `set_image`, queue lock;
                         `context_reset` / `context_destroy` run by the session outside the callbacks),
                         core_profile.rs (`CoreId` and one `CoreProfile` per core: consoles, forced
                         options, screen split, HW-render needs and the renderer option (`Renderer`:
                         Azahar Vulkan or Software), save model, how deep its own files go under the
                         game's folder (`save_depth`), settings it cannot honour, how it closes the lid
                         (`Lid`: melonDS DS its L2 + Y chord, Azahar none); nothing matches a core
                         by its name string)
                         screens.rs (NDS / 3DS screens composed by EmuDome, #28: `Split` cuts the frame,
                         `compose` places each screen, `touch_at` / `pointer_value` map a touch back;
                         `ScreenSettings` the `screens.*` values, `CustomLayout` the editor's
                         `screens.custom`, `ScreenSession` the layout shown, the swap and the lid,
                         `ScreenStatus` what the interface shows (`phone_placement`: the phone's own placement
                         is in use); `ScreenSession::set_phone_placement`, #134: the touch layout's rectangles
                         (fractions of the whole window, every screen of the console, `main` for GB / GBC / GBA,
                         checked by `placement_fits` against `console_screens`) as `Arrangement::Phone`,
                         placed as given (no `video.aspect` / `video.scale`; `screens.rot` and the swap as a
                         custom layout: each id is the console's screen whose rectangle it is, swapped each
                         screen is drawn in the other's), with an `Inset` (the quick menu open) the screens
                         together scaled uniformly into the room it leaves, never past their size nor
                         stretched, centred across and placed down as the inset aligns a game, the layout
                         cycle refused meanwhile; `phone_start` where
                         the editor starts them with no game: the cycle's first layout, not swapped, as
                         fractions; `ScreenSession::phone_start` over the open game: the layout shown now;
                         `Inset` the window's edges kept clear
                         of the game (quick menu, touch controls), its `between` the room kept between
                         two screens (the phone's strip), and `fit` the game in the rest, centred,
                         against the bottom or the top, or two screens `apart` against both ends;
                         `Layout::scale` (`Scale`, the `video.scale` setting: `Fit` as large as fits,
                         `Integer` whole scales from 1×, `Fill` stretched; whole scales on a computer
                         and as large as fits on a phone by default, #105); `auto` (`Arrangement::Auto`) is stacked in a window taller
                         than wide, side by side otherwise; pure, unit-tested)
                         touch.rs (the touch controls, #29: `ControlId` and the controls each console
                         has (`ControlId::of`), `Layout` / `Layouts` the user's layout per orientation as
                         `touch.tedit` stores it (per device, per console), what a valid one is (each
                         control once, the menu kept, size / opacity limits, hold / turbo on buttons
                         only), `Setup` what the interface reads; the defaults and the geometry are the
                         interface's; pure, unit-tested)
                         save_path.rs (#94: does a core's deepest file fit under the game's folder within
                         the platform's `PathLimits`; checked before the core loads; pure, unit-tested)
                         contract.rs (the Rust ↔ TypeScript contract: ts-rs types and fixtures generated
                         into app/src/generated/, the staleness check; see the section of that name)
                         speed_audio.rs (audio off 1×: `audio.ff` mute / resample / WSOLA time-stretch; pure)
                         library/ (game library: identity.rs console, `<console>:<hex>` id from the content,
                         title and NDS/3DS icon; scan.rs folder walk with a path+size+mtime hash cache,
                         the ROM files it skipped counted by reason (`Skipped`: encrypted 3DS,
                         unrecognised; on `library://scan` done, said by L11 only after a folder the user
                         picked, `lib/library.ts`);
                         store.rs library.json, games never deleted, the user's version pins, the
                         picked Android folders' names (`folderNames`);
                         group.rs the regional versions of one game as a family (GBA / NDS code
                         and 3DS product code without the region letter; never homebrew, GB / GBC
                         or hidden games), the version shown: the user's pin, else the one
                         launched last, else the one added first (#102); pure, unit-tested)
                         first_run.rs (first run: its steps, whether to show it; pure, unit-tested)
                         storage.rs («Gestisci lo spazio», #29: sizes walked without following links,
                         the quick saves (`auto.state` / `slotN.state` with their thumbnail and
                         record) and screenshots of every game, the temporary files, and the only
                         deletions the screen makes: picked quick saves and screenshots, the
                         temporary files, a game file inside EmuDome's games folder; unit-tested,
                         every in-game save surviving each)
                         updates.rs (EmuDome updating itself, #44: `UpdateFlow`, `general.updates` auto / ask /
                         off as a state machine, every time passed in; `UpdateStatus` for the interface;
                         pure, unit-tested)
                         notifications.rs (X03, #32: `Notice` codes + values, `Notifications` newest
                         first, at most 100, a newer kind kept unlisted; `Store` on
                         `<app local data>/notifications.json`, `$schema` 1, write_atomic, unreadable
                         never written over; unit-tested)
                         close_flow.rs (closing with a game open: the state machine; pure, unit-tested)
                         game_key.rs (`GameKey`, the one identity of a game: the library's content id;
                         saves in `saves/<console>/<hex>/` (`GameFiles`), settings `game/<console>-<hex>.json`)
                         migration.rs (the one-time move of saves named by ROM file stem and settings named
                         by path hash to the key: facts from the library only, copies in `migration-70/backup/`,
                         `migration-70/report.json`, marker `migration-70/done`; and, by the same rules, of
                         the screenshots named by ROM stem, `run_screenshots`, `migration-85/`; unit-tested)
                         screenshot.rs (screenshots keyed by the game, #85:
                         `saves/screenshots/<console>/<hex>/<YYYY-MM-DD_HH-MM-SS>.png`, outside the game's
                         saves folder (not synced, not in a core's path budget); listing, plain-name check)
                         quick_slot.rs (each game's quick slot, #110: `quick-slot.json` in its saves folder,
                         `{ "$schema": 1, "slot": n }`, synced with the saves (not a quick save for
                         `cloud.squick`); read when the game opens (`restore`: none or unreadable = slot 1,
                         an unreadable file left as it is), written by the session's I/O worker at every
                         move (`Command::SetQuickSlot`); clamped to 1–10 (`LAST`, the largest
                         `saves.slots`: «Rendi rapido» reaches slot 10, the shortcuts cycle 1–9);
                         unit-tested)
                         cover.rs (#85: a cover from the user's image or a screenshot: PNG / JPEG / WebP at
                         most 16 MiB and 8192 px a side, EXIF orientation applied, the 2:3 `CoverView` the
                         interface chose, the spare space the image blurred and dimmed, 600 × 900, JPEG 90,
                         no metadata; pure, unit-tested)
                         data_move.rs (the one-time copy of the old identifier's data folders to the new
                         ones, #90: copy + fsync + size/MD5 check per file, the old folders only read, a stamp
                         per new folder until `migration-90/done`, a new folder with data of its own never
                         written over, `migration-90/report.json`; unit-tested)
                         bios/ (the user's own BIOS / firmware, `bios.*`: mod.rs slots, check by size + MD5
                         / firmware header / DSi footer (identification only, never a source), assess,
                         status, plan; one table per core in bios/<core>.rs with its policy (melonDS DS:
                         native system files, DSi mode, DS-menu boot); files.rs probe and the copy into
                         the core's system folder via write_atomic; pure, unit-tested)
crates/emudome-platform  EVERYTHING that depends on the operating system or the form factor, one module per OS
                         (desktop.rs shared by the desktops): the main window (create_window), the file
                         picker (pick_file), the image picker (pick_image: read at most a limit, on every
                         system), logs, core paths, the external surface, the old identifier's
                         data folders (old_data_dirs: beside the new ones on desktop, none on Android),
                         paths.rs (how long a path this process can create: `path_limits`, Windows long
                         paths for the process and the `LongPathsEnabled` policy, read only);
                         storage.rs (`disk_space`: the room on a path's volume, fs2, every system; E13),
                         `open_folder` / `OPEN_FOLDER` (a folder in the system's file manager: Explorer,
                         xdg-open; none on Android), `CONTENT_LINKS_DIR`,
                         `move_to_background` (Android's back at the shell's root: `SystemBridge.moveToBack`;
                         false on the desktop), `set_immersive` (Android's system bars hidden while
                         a game is open, `SystemBridge.setImmersive`; nothing on the desktop);
                         android/ is the Gradle library (Kotlin NativeSurface: SurfaceView under the WebView;
                         SystemBridge: the back button, the launch intent's game, `content://` documents,
                         the folder / file / image pickers and the tree listing (#47);
                         GamepadInput: the controllers' key / motion events and device list;
                         jniLibs: only the cores.lock files from cores/android/<abi>/, sha256-checked);
                         pads/ (the controllers, #47: `PadSource` (open / poll / read), `PadEvent`,
                         `PadReading` (buttons by position, six axes in SDL's ranges); sdl.rs SDL3 on the
                         desktop; android.rs the JNI side of GamepadInput.kt; android_map.rs Android's key
                         codes and axes as positions, the right stick / triggers layout from the motion
                         ranges, the pads' state: pure, tested on every system),
                         content.rs (games as `content://` documents, opened without a copy: the
                         provider's file descriptor kept open for the session and read through
                         duplicates of it; a link `<cache>/content/<fd>/<name>` → `/proc/self/fd/<fd>`
                         names it, and is the path of a `need_fullpath` core when it opens; the
                         listing of a picked tree parsed; pure parts tested),
                         updater.rs (#44: Tauri's updater plugin on the desktop, registered only when built
                         with `EMUDOME_UPDATER_PUBKEY`; check / download / install; Android: unavailable),
                         log_file.rs (every log line also into `<app local data>/logs/emudome.log`, written
                         by the `emudome-log` thread, never by the thread that logs; bounded),
                         `save_file` (the save dialog), `shipped_cores` (cores.lock's shipped cores, for
                         the licences page),
                         cloud.rs (what the Drive sync needs from the system: `open_url` (the system
                         browser), `device_name`, the credential store (`keyring`: Windows Credential
                         Manager, Linux Secret Service over zbus) under the service `it.domehub.emudome`;
                         Android and macOS answer `Unsupported`, the Android sign-in plan in its doc),
                         `haptic` (a touch control's press: Android `SystemBridge.vibrate`, the
                         `VIBRATE` permission of the library's manifest; nothing on the desktop)
crates/emudome-sync      Google Drive sync (Trello #20), no OS code: oauth.rs (PKCE S256, the authorization
                         URL, code exchange, refresh, revoke; the client id and secret are build-time
                         values, `option_env!("EMUDOME_GOOGLE_CLIENT_ID" / "..._SECRET")`, none = sync not
                         configured), loopback.rs (the redirect on 127.0.0.1:<random port>, state
                         checked), auth.rs (`Auth`: refresh token in a `SecretStore`, access token
                         refreshed in memory, sign-in / sign-out), http.rs (reqwest 0.12 + rustls/ring,
                         the system's root certificates), drive/ (`DriveApi`; rest.rs Drive REST v3:
                         multipart up to 5 MiB, resumable chunks above, retries, 401 → new token,
                         `download_to` streamed to the disk, `list_all` (every app file in one listing),
                         `trash`; fake.rs `FakeDrive` in memory with failure injection), index.rs (the sync
                         index `<app data>/sync/index.json`: last synced md5 / Drive id per file, the
                         device id; write_atomic, unreadable never written over), decide.rs (the
                         conflict rule: none / push / pull / conflict, pure, tested exhaustively),
                         marker.rs («in use on <device>»: appProperties of the game's Drive folder,
                         heartbeat 2 min, stale after 10), sync.rs (`Area` (a game's saves folder ↔
                         `EmuDome/<console>/<hex>/`, settings ↔ `EmuDome/settings/`), `Syncer`: plan,
                         pull before launch, push after exit, conflicts resolved keeping the other side
                         under `conflicts/` (or both sides, «Tieni entrambe»), `cloud.squick`'s quick
                         saves filter, a cancel flag and a progress callback, the library list
                         `EmuDome/library/<device id>.json`, game files `EmuDome/games/<console>/<hex>.<ext>`;
                         data-survival tests on `FakeDrive`), service.rs (#31: `CloudService`, the sync as
                         the app runs it, no Tauri: `pull_game` / `skip_pull`, `claim` / `heartbeat`,
                         `mark_pending` + `push_game` / `push_pending` (a game stays pending in
                         `<app data>/sync/state.json` until a push goes through), `sync_all` / `sync_start`,
                         `games` / `upload_game` (quota checked) / `download_game` (temp file, MD5, free
                         name) / `remove_game` (Drive's trash), `devices`, `conflict` / `resolve`, the
                         activity (in memory), every Cloud DTO; tests on `FakeDrive`)
app/src-tauri            Tauri shell: lib.rs (wiring only: modules, `AppState`, the per-game hooks, setup, the
                         command list, `blocking()` for heavy commands), game_session.rs (`GameSession`: the
                         open game and what lives as long as it does, open / take / close / abandon; exit-path
                         tests on `FakeCore`), game.rs (opening a game, `close_game`, `abandon_game`),
                         commands.rs (the one-line in-game commands, `key`), protocol.rs (the `emu://` scheme:
                         one route per resource, unknown paths 404), events.rs (every event name),
                         hotkeys.rs (`HotkeyRunner`: the shortcuts of the pads and of the keyboard carried out,
                         one hold shared by both), screens.rs (the open game's `ScreenSession` carried out:
                         the renderer's layout, `emu://screens`, the lid chord pressed on the core, the
                         `screens_*` commands, `screens_set_phone` (the phone's placement of the open game,
                         empty withdraws it, with the inset it is fitted in: one renderer message) and `screens_phone_start` (where the editor starts it, which
                         replaced `screens_preview`); the webview canvas gets its placements from
                         `screens_placements`), tuning.rs (developer / bench knobs from the environment),
                         window.rs (the app's names for emudome_platform::create_window, and
                         `video.window` / `video.winsize` carried out: the mode at startup and on change,
                         the size only for an open game, `size_target` / `inner_size` pure and tested),
                         background_pause.rs (`general.bgpause`: the page's blur / focus through
                         `window_focus`, Android's suspend / resume; pauses and resumes only its own pause),
                         startup.rs (`startup_game`, and `startup_last_game` for `general.start`),
                         video.rs (wgpu; `GpuService`, one per app, opened on `video.api`'s backend at
                         startup (`GraphicsApi`; none there: wgpu's choice and `graphicsApiUnavailable`),
                         owns instance / adapter / device for
                         the whole run: surfaces attach and detach on it, a 3DS game leases the device
                         (`VulkanLease`) and the device is never remade under it; every submit and present
                         holds its queue lock), gpu_interop.rs (the one module under wgpu: wgpu-hal / ash,
                         the device opened with what a 3DS core uses (`open_device_for_cores`), the core's
                         image as a texture, raw barriers), hw_render.rs (a Vulkan core's frames: GPU copy
                         of the core's image into a game texture, readback into `Shared::frame`;
                         `headless_setup` for `examples/headless_gpu.rs`),
                         audio.rs (cpal), gamepad.rs (the controller thread on emudome_platform::pads,
                         the same on every system), commands,
                         settings.rs (settings commands, file locations, settings://changed),
                         effects.rs (screen effects: built-in .slangp presets from effects/, fx.* → renderer,
                         emu://effect-error), video.rs runs them through librashader (native path only)
                         bios.rs (bios_choose / bios_status, per-core system folder
                         `<app local data>/system/<core>/` for cores with a BIOS table, emu://bios-fallback)
                         theme.rs (Scuro / Nero / Chiaro: EMUDOME_THEME, else general.theme, else the system)
                         closing.rs (the close flow carried out: worker, emu://closing, exit when saved)
                         health.rs (what is not working: ErrorCodes per part, emu://health, diagnostics)
                         storage.rs (E13, #29: `storage_status`, the room on the data folder's disk and
                         whether it is low: under 500 MB; `None` when the system does not say;
                         «Gestisci lo spazio»: `storage_breakdown`, `storage_quick_saves`, the
                         `storage_delete_*` / `storage_clear_temp` / `storage_remove_game` deletions,
                         `storage_open_folder`)
                         notifications.rs (X03: the store of emudome_core::notifications, the
                         notifications_* commands, notifications://changed; `record` writes off the
                         caller's thread: gamepad.rs records a controller connected after start)
                         cloud.rs (Google Drive: the account, the refresh token through
                         emudome_platform::cloud, the `cloud_*` commands, the `cloud://` events, the
                         `cloud.*` settings read into `Prefs`; the game flow's hooks: `before_launch`
                         (pull in the open's thread, then pending), `heartbeat_hooks` (the marker while
                         the game runs), `after_exit` (the push once the exit writes are done, in the
                         background), `startup` (pending pushes, settings, library list))
                         updates.rs (#44: `UpdateFlow` carried out on the `emudome-updates` thread,
                         updates_status / _check / _later / _install, `updates://changed`, the
                         `updateReady` notification; installing refused while a game is open)
                         about.rs (#44: the licences page's rows from Cargo.toml + cores.lock +
                         THIRD_PARTY.md and their packaged texts, about_export_log through the save dialog)
                         dirs.rs (the data folders: Tauri's, EMUDOME_DATA_DIR, or the old identifier's for a
                         run whose copy did not finish), data_move.rs (that copy, first thing in setup)
                         library.rs (library commands, background scans, library://scan and library://changed,
                         play time, covers on emu://cover/<id>, library.watch with notify on desktop, Android's
                         picked document trees walked through `PlatformDocuments` (#47); user
                         covers made by `emudome_core::cover` and written with write_atomic as
                         `covers/<console>-<hex>.user-<rev>.jpg`: the picked image kept in memory on
                         emu://picked/<token>, a game's screenshots listed and served on
                         emu://screenshot/<console>-<hex>/<name>, #85)
                         touch.rs (#29: touch_layouts / touch_set_layouts (C06's «Salva»: the layout of each
                         orientation given (portrait, landscape, or both; one not given kept as stored)
                         into `touch.tedit` and its screens into `touch.tscreens` (#134: its own
                         key beside it, console level, per device, never synced, so older builds read
                         `touch.tedit` untouched; `Layouts::read` joins the two) at the console's level,
                         each given orientation's `touch.tport` / `touch.tland` to `custom`, all in one
                         `Settings::set_many` write; one invalid layout refuses the whole save),
                         touch_stick (the Circle Pad), haptic)
                         contract.rs (test only: the shell's DTOs generated into app/src/generated/app/)
app/src                  Vue interface, placed as «Interface structure» says: main.ts, App.vue (the root),
                         emu.ts (typed bridge, its types from generated/), generated/ (written by the Rust
                         contract tests, never by hand), nav.ts (controller/keyboard focus navigation),
                         components/<feature>/ one folder per feature: shell (AppShell: rail + sections,
                         outside the game; on the phone PhoneTabBar upright, the rail as a column on the
                         camera's edge on its side, lib/phoneLayout.ts decides, #29), storage (E13's
                         StorageWarning, phone only; StoragePage «Gestisci lo spazio», its
                         SpaceCard, StorageGameRow, QuickSavesChooser, ScreenshotsGallery,
                         StorageConfirm; storage.ts the pure decisions, tested), library, details, saves, settings, controls, quick-menu,
                         game (GameCanvas: webview mode, the game flow,
                         useScreenStatus: the open game's `ScreenStatus` from `emu://screens`,
                         LidClosedCard: G15 «Coperchio chiuso»),
                         touch (#29: TouchLayer, the touch controls over the game (App.vue, a touch screen):
                         the layout of the console and orientation (touchLayout.ts: the defaults anchored
                         to the window's edges, the geometry, the 8 px grid, the edges kept clear of the
                         game: the band at the bottom in portrait, the side columns in landscape; DS /
                         3DS: the safe top, the band and the L / menu / R strip between the screens
                         upright, the camera's edge on the side), the two styles «Fumé» / «Contorno»
                         (skins.ts, `touch.tskin`: ButtonVariant.PadSmoke / PadOutline colour the shape
                         PadShape.vue draws in each button's box; the d-pad one cross, padGeometry.ts),
                         tap / hold / turbo
                         (touchPress.ts), `touch.topac` / `thapt` / `thide` / `tpad`
                         (touchVisibility.ts); TouchEditor C06 over the game (from G10, the phone's
                         «Altro › Modifica la disposizione» and the `touch.tedit` row; the game
                         screens are its elements too, #134: C09's ScreenFrame, placeholders on the
                         theme's flat ground with no game, the game itself placed live over one
                         (`placement` + `inset`), each frame the console's screen it names, swapped
                         or not (`asShown`); nothing picked at first, «Altro» the layout's tools in
                         fixed places and then the picked element's; editorScreens.ts their geometry, useStartScreens
                         where they start (`screens_phone_start`); useTouchEditor the state (a finger on a screen useScreenGesture),
                         editorHistory.ts undo / redo, orientationDrafts.ts one draft per orientation
                         (kept across a turn, both saved by one «Salva», the hint to turn the phone),
                         editorToolbar.ts what fits and what goes into «Altro» and which tools a screen
                         has); E14
                         ControllerLostDialog / useControllerLoss (controllerLoss.ts the rule); all the
                         pure modules tested),
                         screen-editor (C09, Trello #28: ScreenEditor over the game at its real size, from
                         G09 and from the `screens.custom` row; a row with no console it has layouts
                         for asks for one first, settings/LayoutConsoleChooser + layoutEditors.ts, #111,
                         for C06 too; from Settings the theme's flat ground as C06, the snapping grid
                         not drawn; the panel keeps the side it opened on unless the selected screen
                         would be wholly under it, never moved by «Parti da»; useScreenEditor the state,
                         the live preview at the Session level and the steps; screenEditor.ts the
                         geometry, pure, tested),
                         first-run (FirstRun, 4 steps, once per device: marker `<app local data>/first-run/done`,
                         src-tauri/src/first_run.rs), cover (GameCover; L12 CoverPicker: CoverChoices,
                         ScreenshotGrid and «Adatta la copertina» CoverCrop, its geometry coverCrop.ts,
                         pure, tested; CoverCropDialog for the screenshot toast's «Usa come copertina»,
                         useCoverRequest), toasts, closing (the loader while a game closes),
                         cloud (#31: CloudView, the Cloud section signed out / in with its status card and
                         tabs; cloudState.ts the app's copy of `CloudStatus`, the transfer running, the
                         Drive games, the G04 queue and G05's question, started by useCloudEvents in
                         App.vue; CloudFlowOverlays G04 / G05 / G14's strip, CloudPull G03 in the starting
                         scene, `clearToLaunch` before a library game opens; CloudDriveShelf the library's
                         games only on Drive; cloud.ts the pure decisions, tested),
                         search (X01 / X02, #32: SearchPalette, mounted once in App.vue, opens over its
                         host (useSearch.ts: the shell or the quick menu, Ctrl + K or Y); searchContext.ts
                         what can be chosen there, search.ts the matching, both pure, tested;
                         useSearchSources the actions, settings and games; the quick menu's side is
                         quick-menu/useMenuSearch.ts),
                         notifications (X03: NotificationBell in the top bar, its panel; notifications.ts
                         the words, pure, tested), about (`about.lic`'s licences page, a functional
                         placeholder), updates (X06, the update card in the shell; the shared status in
                         composables/useUpdates.ts);
                         design-system/ (the whole design system, its map in design-system/README.md:
                         index.css the entry, tokens/ one file per family, styles/utilities.css the named
                         looks, components/ the primitives, lib/ cn / themeKeys / colour / accent /
                         useDensity, brand/ EmuDome's mark, kit/ the Kit page, development only; it
                         imports nothing of the app but nav.ts and generated/, kit/ excepted),
                         composables/ and lib/ (shared by 2+ features: lib/theme.ts data-theme on <html>,
                         lib/settingsModel.ts),
                         i18n/ (index.ts vue-i18n setup and locale choice, en.json, it.json, labels.ts codes → text);
                         the startup splash (#101) is plain HTML/CSS in
                         app/index.html (the mark on graphite from the first paint, the bar after 2 s;
                         its few values mirror the tokens), faded out by lib/splash.ts once App.vue has
                         decided and rendered the first run or the shell; the window's own background is
                         graphite (emudome-platform desktop.rs), Android 12+ shows the system splash
                         (gen/android res/values-v31)
```

- The interface never touches the core: it calls the Tauri commands, which send `Command` to the `Session`.
  The core runs on its own thread. The libretro callbacks find their context through the loaded core's slot
  (`callbacks.rs`: 8 static slots, one callback set per slot), so calls from a core's own threads reach it and
  a core let go keeps its slot, empty, forever. A `need_fullpath` core (Azahar) gets the game's path only: the
  host never reads its file (`GameContent`, `core.rs`).
- The open game is one `GameSession` (`app/src-tauri/src/game_session.rs`): the `Session`, `CurrentShared`, the
  key of the game whose files are in use (until its thread ended), and the per-game parts as `Hooks` started in
  order and stopped in reverse (renderer source, settings session, core options that follow the settings, the
  screens, the window's size, play time; a new per-game part is one more hook in `lib.rs`).
- Ending a game: every way out (closing the game, opening another, closing the window, quitting) takes it out
  through `GameSession::take` (the close flow, `closing.rs`) or, for `RunEvent::Exit` only, `GameSession::close`,
  and waits up to `CLOSE_TIMEOUT` (10 s) for the thread's exit writes: the `.sav` first, then the auto quick save
  with `saves.autoexit`. Android going to the background
  (`onPause`) sends `Command::Flush` (the same writes; the game keeps running, or pauses with `general.bgpause`
  until `onResume`, `background_pause.rs`). A thread that does not stop in
  time, or is abandoned after E11, is let go: the session is dead (no file, no event, no `retro_unload_game` /
  `retro_deinit`) and its core is refused with `coreNeedsRestart` until the app restarts. With Drive sync on, the
  session's end-of-thread check (`AfterEnd`) also starts the game's push in the background (`cloud::after_exit`);
  nothing waits for it, and a game let go or ended by `RunEvent::Exit` stays pending for the next start.
- Writes while the game runs (quick saves, the periodic auto quick save and `.sav` flush, «Annulla» of a save,
  screenshots, the game's quick slot) go to the session's I/O worker (`session/io.rs`, thread `emudome-io`): the emulation thread only
  serializes and copies the bytes; the worker reads what the slot held, writes atomically, encodes the PNG and sends
  the event. The emulation thread waits for it before reading a slot (load), before `Flush` and at the end of the
  session, whose exit writes stay synchronous. Rewind serializes into reused buffers and encodes its differences on
  its own thread (`emudome-rewind`); stepping back turns the newest state into the older one in place.
- **Video, two paths:**
  - **Native**: wgpu draws on the window, under a transparent child webview ("ui").
    Needs Tauri's `unstable` feature (`window.add_child`). The device is made with the first surface and
    kept: a detach or a new surface replaces the surface only (a new device only if the GPU cannot present
    to the new surface). It presents only when something changed (a new frame wakes it through
    `Shared::frame_wake`, or a message), else sleeps; a minimized window draws nothing; a failed present waits
    8 ms. A detach waits for the GPU to finish before the surface goes (Android destroys it right after). Pacing
    options (present-driven / audio-driven): `docs/video-pacing.md`. `video.vsync` picks the present mode
    (on: `AutoVsync`, off: `AutoNoVsync`), applied at once by configuring the surface again; the
    `EMUDOME_PRESENT_MODE` knob overrides it (`tuning.rs`). The row is disabled on Compatible.
  - **Webview**: the RGBA frames reach a `<canvas>` through the `emu://frame?since=N` protocol.
  - **Smoothing** (`video.filter`, `emudome_core::effects::Smoothing`): «none» nearest, «light» sharp bilinear (each
    pixel enlarged by the largest whole scale that fits along the frame's axes, blended only across what is left),
    «soft» bilinear. Native: one linear sampler, the shader picks the sample point per screen (`game_uv`, the
    `Screen` uniform's `smoothing` and `scale`), clamped inside the screen's own part of the frame so two screens
    never bleed into each other; the effect's own output is still sampled nearest. Webview: `screenSmoothing`
    (`screenDraw.ts`), the screen enlarged sharp into a canvas of its own, then drawn smoothed. Live on a change,
    «in prova» included.
  - **Screens** (Trello #28): both paths draw each screen of the frame where `emudome_core::screens::compose`
    puts it for the open game's layout (`screens.*`, the layout shown, swapped or not, and `video.aspect` and
    `video.scale` for every console: 4:3 / 16:9 stretch the composed picture, «fit» / «integer» / «fill» size it,
    a custom layout excepted): the native renderer one
    draw per screen (`Renderer::set_layout`), the webview canvas from `screens_placements` (it decides no
    geometry: `GameCanvas.vue`, `screenDraw.ts`). Touch maps back through the same layout (`pointer`).
    On a phone a two-screen game opens on `auto` (`screens.cycle`'s phone default:
    `auto`, `big_small`, `top_only`) and, like a one-screen game (#105), fills its room: `video.scale`'s
    phone default is «fit» (a `default_rules` entry; «integer» on a computer), so a GBA upright is 360 px wide on
    a 360 px phone; the touch controls' inset carries the rest (`Inset::between`, `Align::Apart`
    upright, `Align::Top` on the side), so both renderers and `pointer` get the same placements.
    A phone placement saved in C06 (#134, `touch.tscreens`) replaces the chosen layout while the touch controls are
    in use: `TouchLayer` sends it and withdraws it when the controls are
    hidden (`touch.tpad`; `touch.thide` only fades them); `useGameSession` sends a placement and the inset of the
    same change as one call (`screens_set_phone` with the inset, `Renderer::set_view`), so no frame pairs the new
    placement with the old inset; an inset alone stays `set_video_inset`. It is placed over the whole window, not in what the
    controls leave (they may overlap it); the quick menu's inset fits its screens, taken together, into the room
    left (scaled uniformly, never past their size, centred across, against the phone's sheet upright) and it comes
    back at its place when the menu closes; G09 says «Dalla disposizione touch» on the layout card and on
    «Dimensione dell'immagine» (`video.scale` does not apply to it) and offers no cycle, and L3 + R3
    shows G09's reason as a toast (`emu://hotkey` `layoutHeld`).
  - The default per OS is `NATIVE_VIDEO_DEFAULT` in `emudome-platform`: Linux uses the webview.
  - `Shared::frame` holds its pixels in an `Arc`: the sink converts the core's frame into a back buffer outside the
    lock and swaps it in; readers (texture upload, the webview's copy, PNG thumbnails and screenshots) clone the
    `Arc` under the lock and work after letting go. A buffer no reader holds is converted into again.
- tauri, wgpu and librashader are pinned to exact versions (`=x.y.z`, `app/src-tauri/Cargo.toml`, tauri also in
  `emudome-platform`) and upgrade only together, as one task ending with a native-video smoke test.
- Screen effects (`fx.*`): librashader 0.12 (its wgpu must stay our wgpu version) renders the game texture
  (`Rgba8Unorm`, the core's sRGB bytes as they are: the whole frame, every screen) into a texture of the frame's
  size times the largest screen's scale; our pass cuts each screen out of it like the plain image and mixes it
  over the plain image (`fx.fxint`, `fx.fxbright`). Rows with `needs_native_video` are disabled on Compatible (X05).
  GPU check without a window: `cargo test -p emudome --lib gpu_tests -- --ignored` (`EMUDOME_FX_DUMP=<dir>` for PNGs).
- Audio: a lock-free SPSC ring (`emudome_core::audio::AudioRing`, rtrb) from the emulation thread to the cpal
  callback, linear resampling to the device's rate, ±0.5% correction on the ring's fill level towards the queue
  target; the reader drops what is queued beyond the limit, oldest first (`audio.lat`, `audio::Latency`, read by
  every callback from an atomic: Low 30 / 75 ms, Balanced 60 / 150 ms, Safe 120 / 300 ms). The device is
  `audio.out` (the system's default, followed every 2 s, or a device by name: missing, the default plays until it
  is back; the names come from `settings_runtime_options`). A ring that runs dry (a stall: a 3DS quick save takes ~0.25 s, #98;
  a pause; a game's start) fades out and stays silent until it holds the queue target again, then fades back in
  (`Resampler` in `app/src-tauri/src/audio.rs`). Off 1× `audio.ff` decides (`speed_audio.rs`, on the emulation thread,
  before the push): muted, resampled by the speed (pitch moves), or time-stretched (pitch kept); either way about
  real time reaches the ring. At Max the speed is measured. The real-time threads (audio callback, controllers)
  read the open game through `CurrentShared` (an `ArcSwapOption`, no lock); the callback records its measurements
  only in bench mode, with `try_lock`.
- Core options: the settings' values come from `settings::core_values` (a `CoreTable` per core: setting → core
  option, `Live` or `Restart`). A change while the game runs: live options go to the core
  (`GET_VARIABLE_UPDATE`), restart ones are listed in `emu://core-restart` (`core_settings.rs`).
- Keyboard (Trello #81): the webview has the focus, so it receives the keys and forwards them raw to Rust (`key`
  command: `KeyboardEvent.code`, down / up, repeat); Rust routes them (`emudome_core::keyboard::KeyRouter`, the
  keyboard's row of `pad.remap` for the game's buttons) and carries the shortcuts out with the controllers' runner
  (`hotkeys.rs`). What stays in the page (`useGameKeys.ts`, `keyRoute.ts`): Esc (quick menu / back), F11 and the
  menus' focus keys (`nav.ts`). Opening the quick menu (`set_menu_open`) and the window losing the focus
  (`WindowEvent::Focused(false)`, and the page's `blur` through `window_focus`) let go of every keyboard button and
  end the keyboard's fast forward and held rewind, in Rust. The touch controls send buttons (`input`) on their own
  source (`input::Source::Touch`, so letting go of the keyboard never drops a finger's button), and the 3DS Circle
  Pad drawn on screen (`touch_stick`) is the left stick while moved (`InputState::set_touch_stick`, merged into the
  snapshot).
  Controllers go through one controller thread (`gamepad.rs`, the same on every system) reading
  `emudome_platform::pads::PadSource`: SDL3 on the desktop; on Android (#47) the system's own `KeyEvent` /
  `MotionEvent` of gamepad and joystick devices, taken by `GamepadInput.kt` in a wrapper of the activity window's
  `Window.Callback` before the WebView sees them (touch, the phone's own back key, keyboards and IME go on to the
  system, and so does a pad key EmuDome does not read), raw codes and axis values over JNI, mapped in Rust
  (`pads/android_map.rs`: `KEYCODE_BUTTON_A` the bottom button as SDL reads it, the right stick on `AXIS_Z` /
  `AXIS_RZ` or `AXIS_RX` / `AXIS_RY` and the triggers chosen from the device's motion ranges, a digital L2 / R2 a full
  axis; the kind from the USB vendor id, then the name). Logcat: `android pad connected: <name> <vid>:<pid>` and,
  in a debug build, a line per key and per coarse change. RetroArch-style positional mapping; triggers and the left stick past half travel
  count as L2/R2 and d-pad. Both sticks also reach the core as `RETRO_DEVICE_ANALOG` while the game has the
  input (`Routed::game_has_input`). The core reads a frozen `input::Snapshot`, taken before each frame and at
  its `input_poll`, never the live state mid-frame.
- **Controller routing lives in Rust**, as a pure function: `emudome_core::hotkeys::Router::step`
  (previous state, buttons, time → game mask + actions), unit-tested without SDL. Three modes:
  - in game: buttons go to the core, except the modifier L3, which never does;
  - L3 held: combos from the **one table** `DEFAULT_COMBOS` (`hotkeys.rs`, remappable in Phase 1),
    carried out by `HotkeyRunner` in `hotkeys.rs`, shared with the keyboard (session `Command`s, or events for
    the UI; a hold from both, Tab and L3 + R2, ends with the last release; with `speed.ffmode` = toggle Tab
    latches its hold until its next press, the quick menu or the focus loss, `keyboard::KeyRouter`);
    the full table and its TODOs are in `docs/combo-tasti.md` section 5;
  - the interface has the input (quick menu open, or no game): the game gets nothing, d-pad / A / B
    become `emu://nav` steps with key repeat (400 ms, then every 120 ms); L1 / R1 `prev` / `next`, X `more`,
    Select `select`, Y `search` (the shell opens the search everywhere, X01: a scope's `search` handler,
    bubbling like `back`), and the right stick past `pad.stickdpad` (`PadInput::stick`) `stickUp` /
    `stickDown` / `stickLeft` / `stickRight`, repeating like the d-pad (the C09 screens editor's own steps).
  The interface tells Rust the menu is open with the `set_menu_open` command.
- The library's cover grid is windowed (Trello #91, `components/library/useCoverWindow.ts`, sums in
  `coverWindow.ts`): only the rows near the screen plus 2 are in the DOM, the grid's padding stands for the rest
  (rows measured once rendered, the others estimated), and a re-render never moves what is on screen. Inside it
  the controller and the arrows move by row and column (`coverGridNav.ts`, registered with nav.ts's
  `useNavGrid`), and leave it at its edges by the spatial rule; a focused tile scrolled out of the DOM parks the
  focus on the grid (no ring) and gets it back when rendered again. Measured headless with 3005 / 10 005 mock
  games (`scripts/headless/screens/library-scale.mjs`): a d-pad step 64 / 146 ms before, under 1 ms after.
  The phone's list (`GameList.vue`, one column of `GameRow`s, two on its side: #105) is the same windowing, its
  steps `listStep` (by row in the same column, a row ↔ its button ↔ the next column's row), and the compact
  grid's `compactStep` (`CompactTile`, «Riprendi» in a corner); `library-scale.mjs --phone [--grid]` measures
  and checks them at 412 × 892 (under 1 ms a step, 3005 / 10 005), `--phone --side` the two-column list at
  780 × 360 (at least 6 games in view).
- Navigation in the interface: `app/src/nav.ts`. A screen calls `useNavScope(root, { back, autofocus })`
  and marks its first element with `data-nav-initial`; steps from `emu://nav` and from the arrows / Enter
  move the focus spatially (nearest element in that direction, looked for in the focused element's scroll
  container first; disabled, `aria-disabled`, `inert` and `visibility:hidden` elements skipped), confirm clicks,
  back calls `back`. Scopes are a tree that follows the component tree (provide / inject): the steps go to the
  newest shown child at each level, whatever the mount order. A part of a screen with steps of its own (the
  shell's sections) calls `useNavSection(root, { back, tab })`: the focus still moves across the whole screen,
  and `back` / `tab` bubble up to the screen when the section returns `false` (or has none), never past a
  scope. A screen that works by steps of its own (the C09 screens editor: the d-pad moves a screen) passes
  `own` to `useNavScope`: it sees every step first while the focus moves in its scope, and returns `false` to
  leave one to nav.ts. The focus ring (`data-nav` on `<html>`, styles.css) shows only while moving without the
  mouse.
- The shell's place is one typed `Route` (`components/shell/route.ts`: section, the game whose details show,
  the arrival a section is opened for), held by App.vue and passed down as props; no module-level «mailbox»
  refs. Toasts and errors reach the user through `provide` / `inject` (`composables/useToasts.ts`:
  `useToasts`, `useErrorSink`), never through `error` / `notice` emits.
- **Settings engine** (`emudome_core::settings`): one static schema (`catalog.rs`, 169 entries on 18 pages: key
  `<prefix>.<key>`, page (its own field: «Avanzate» keeps the `perf.` / `cores.` / `adv.` keys), kind, default
  (with `default_rules`: another default on one device or for some consoles, first match wins, read by `resolve` at
  the Default level; e.g. `speed.rew` off on phones and for 3DS, Trello #89), accepted levels, per-device flag, platform
  (a row of another device is shown disabled with the reason, except `Platform::Keyboard`, left out of the phone's
  schema altogether: `speed.ffmode`), phase, `show_if` on another row's value, `wired`: whether anything
  reads it, the unwired ones shown disabled with «arrives in a future update»); the interface builds its
  pages from `settings_schema` and never lists settings by hand. A key the interface names is a `SettingKey`
  (generated union); one the Rust code reads is a typed constant of `settings/keys.rs`, its default the catalogue's. Levels Default → Global → Console → Game →
  Session; `resolve` returns the value and the level it comes from; a level the entry does not accept is refused
  (`settingLevelRefused`). A value written at a level (or committed from the session) that equals what the weaker
  levels give there, the Default with its `default_rules` included, removes the key at that level instead of
  storing it, so a row set back to its default is no longer «Modificato»; one that differs (a game back to the
  default while its console has another value) is stored. Files: `<app data>/settings/{global,console/<id>,game/<console>-<hex>}.json` (synced later),
  per-device values in `<app local data>/local-settings/device.json` (never synced); atomic writes, unknown keys
  kept, invalid values ignored with a `settingInvalidValue` log; a file that is there but cannot be read is never
  written over (writes to it refused with `settingsReadFailed` until it reads again; the library does the same
  with `libraryReadFailed`). Every file says its layout (`"$schema": 2`, written on save; none = 0; older ones go
  through `store.rs`'s `MIGRATIONS` in memory: 1 → 2 reads the old touch styles as the new ones) and is read again before use when its mtime or size changed. The
  sync rules as they are today are tests (`settings/tests.rs`, «Sync rules»). Session = the quick menu's changes «in prova»:
  in memory, bound to the open game, committed to Game or Console (all or nothing), dropped on close.
  A game is its `GameKey` everywhere (settings, saves, library, the interface's `GameId`); a file opened by its path
  is identified first (the library's hash cache, else hashed, with no lock held). Option ids, slider ranges and chips defaults are the final ones (Trello #41, #26); search synonyms
  live in `SYNONYMS` (catalog.rs).
- **The Rust ↔ TypeScript contract** (Trello #73): the bridge's types are generated from the Rust types by ts-rs
  (`#[derive(TS)]`, MIT) into `app/src/generated/` (core types; the shell's DTOs in `generated/app/`), with
  `SettingKey` (the catalogue's keys), `HotkeyId` and `Button` from Rust lists, and the fixtures
  `error-codes.json` / `settings-catalog.json` (its `colours`: a colour entry's swatches and default, read by
  `design-system/lib/accent.ts`) / `keyboard-shortcuts.json` (read by `lib/keyboard.ts`). `emu.ts` only re-exports them. The files are committed; the contract
  test of each crate (`cargo test -p emudome-core contract`, `cargo test -p emudome contract`) fails when they are
  stale, and `EMUDOME_BLESS=1` writes them again. `app/src/i18n/parity.test.ts` holds the fixtures against the
  message files. `session::Speed` / `Event` are typed through shadow types in `contract.rs` (session.rs untouched),
  held to the real serialization by a test. Still by hand: the command names and argument lists in `emu.ts` and
  `BenchFrame` (the `bench` feature). The generated folder is
  left out of ESLint and Prettier (compared byte for byte).
- Events, every name a constant of `app/src-tauri/src/events.rs` (`<domain>://<event>`; `emu://` the emulator,
  `library://`, `settings://`, `saves://` theirs): `emu://session` (the session's `Event`s), `emu://gamepad-connected`,
  `emu://gamepad-disconnected`, `emu://gamepad-changed` (a `PadInfo`), `emu://gamepad-state` (while watched), `emu://menu`
  (`{ system }`), `emu://nav` (`{ action, system }`; `system`: Android's back button), `emu://hotkey` (`{ kind, ... }`: fps, slot, speed, turbo, paused, layoutHeld: L3 + R3 under
  the touch layout's placement),
  `emu://screens` (a `ScreenStatus`: the layout shown, the cycle, swapped, the lid supported / closed),
  `library://changed` (`LibraryChanged`: the rows that changed, the families of regional versions, or the
  whole library), `library://scan`,
  `saves://migration`, `emu://effect-error`, `notifications://changed` (the whole list, newest first),
  `cloud://status` (the whole `CloudStatus`), `cloud://progress` (`CloudProgress`: pull G03, push G14, sync,
  a game file moving), `cloud://conflict` (`CloudGameConflict`, G04), `cloud://in-use` (`CloudInUseEvent`, G05),
  `settings://changed` (`{ key, level }`), `emu://core-restart` (`{ settings }`: keys waiting for the game to be
  opened again), `emu://bios-fallback` (an `ErrorCode`: a setting asked for system files that are not in place),
  `emu://closing` (`{ phase, purpose, error }`: a game closing, see «Closing with a game open» below),
  `emu://health` (the whole list of `ErrorCode`s of what is not working, see below), `updates://changed` (an
  `UpdateStatus`, #44).
- Health (Trello #79, `app/src-tauri/src/health.rs`): backend failures that change what the user gets are one
  `ErrorCode` per part (`video.api`'s backend missing `graphicsApiUnavailable`, native video fallen back
  `nativeVideoFallback`, surface `videoSurfaceFailed`, audio output
  `audioUnavailable`, controllers `controllersUnavailable`, a thread that panicked `threadStopped`, a 3DS session that
  wrote into Azahar's own folder `coreSavedOutsideGame`, kept until restart), set and cleared
  by that part; each change sends the whole list on `emu://health`, the `diagnostics` command answers it. The shell
  (`useShellEvents.ts`) toasts each code once while it lasts. The audio thread rebuilds its stream on a stream error
  or a change of the default output (polled every 2 s when no device is forced), retrying with a growing pause
  while none can be opened; every cpal integer and float sample format is played.
- Closing with a game open (Trello #86): the window's X, a quit, `close_game` and opening another game go through
  one pure state machine, `emudome_core::close_flow` (requested → saving → saved → finish; timeout / failed → the
  user's «Wait» / «Close anyway»; a second request while saving is ignored, a quit turns a game close into a quit).
  `app/src-tauri/src/closing.rs` carries it out: `CloseRequested` / `ExitRequested` are prevented while a game is
  open, a worker waits for the session's thread (`Session::stop` / `Session::wait`, which never lets it go by
  itself), `emu://closing` reports `phase` (`saving` / `saved` / `timeout` / `failed` / `closed`) and `purpose`
  (`game` / `quit`), «Saved» stays `SAVED_VISIBLE` (600 ms), then the app exits (with the exit code asked for).
  `RunEvent::Exit`, which cannot be prevented, closes what is left in place (bounded by `CLOSE_TIMEOUT`). The
  commands that wait or read files (`open_game`, `library_open`, `close_game`, the `saves_*` that touch files) run
  on a blocking thread of the async runtime (`blocking()` in `lib.rs`), never on the event loop.
  Interface: `components/closing/` (`closing.ts` the view, `useClosing.ts`; `ClosingStrip.vue` the bottom strip
  while saving / saved, `ClosingAsk.vue` the timeout / failure question, `ClosingOverlay.vue` places them).
- «Chiudere senza salvare?»: the quick menu's «Chiudi il gioco» (the desktop's and TV's footer, the phone's strip
  «Chiudi») and the search's other game ask first when the game moved since the user's last quick save or quick
  load (any slot), or since it opened if none: `Stats::unsaved` (the `stats` command's `unsaved`), set by every
  frame run (rewinding too), a reset and an undone save or load, cleared by the user's quick save or load; the auto
  quick save the app writes does not clear it, a quick save whose write failed sets it again (`session/io.rs`).
  The text says the latest progress stays only in the auto quick save (`saves.autoexit` on) or is lost (off).
  «Salva e chiudi» quick-saves into the game's quick slot, waits for `stateSaved` (at most 10 s) and then runs the
  close flow; a failed save keeps the game open with its error. «Chiudi senza salvare» is the close flow as it was;
  «Annulla» (and B) keeps the game. The in-game save is written by the close flow either way. Interface:
  `components/game/closeAsk.ts` (the decision, pure), `useGameEnding.ts` (`requestClose`), `CloseUnsavedDialog.vue`.
  The window's X, a quit, Android going to the background and the Drive «in use» answer close without asking.

## Lessons already learned (do not repeat)

- The wgpu surface must be created **on the main thread**: from other threads the Tauri window handle blocks.
  Drawing then continues on a dedicated thread.
- Create the wgpu instance **without a display handle** (`new_without_display_handle_from_env`).
  With Tauri's handle the surface fails with `MismatchingDisplayHandle`.
- On Linux/GTK the native surface **covers** the webview. Webview mode is used there.
- wgpu 30 changed its API compared to the examples around:
  - `queue.present(frame)`
  - `CurrentSurfaceTexture`
  - `color_space` in `SurfaceConfiguration`
  - `Some(&layout)` in `bind_group_layouts`
- cpal 0.18: `build_output_stream` takes the `StreamConfig` by value.
- On Android wgpu's GLES backend wants a display handle even when Vulkan is chosen
  (`CreateSurfaceError ... MissingDisplayHandle`): `ExternalSurface` also implements `HasDisplayHandle`
  (`DisplayHandle::android()`).
- Controllers use SDL3, not gilrs: gilrs' Windows.Gaming.Input backend reads buttons only while a window of the
  process has the focus, and the focus is in WebView2.
- The static SDL3 build fails with MSBuild FTK1011 (260-character path limit) when the build folder's path is long.
  The repo's own `target\` is fine: one more reason never to move `CARGO_TARGET_DIR`.
- Azahar 2126.2 creates its emulated NAND up to 93 characters under the game's folder in `retro_load_game`; past the
  path limit (Windows without long paths: 259 for a file) it aborts the whole process, no error (Trello #94). And when
  it cannot use its save directory at all (a folder it cannot reach, e.g. over 260 characters without long paths) it
  silently falls back to `%APPDATA%\Azahar`, shared by every game. Hence the app's `longPathAware` manifest
  (`app/src-tauri/windows-app-manifest.xml`, embedded by `build.rs`), the guard before the load
  (`CoreProfile::check_save_dir`, `savePathTooLong`), the refused start when the game's folder cannot be created
  (`saveFolderUnavailable`) and the watch on Azahar's own folder after the session (`coreSavedOutsideGame`, #97). Test executables carry no manifest: on Windows they keep the
  260 limit whatever the policy says (the headless example's `--max-path`).
- A Windows audio device can open with 8 channels: `audio.rs` writes left/right on the first two and zeroes the rest.
- Branches developed in parallel break each other without a textual conflict (#70 renamed
  `AppState::game_path` to `game` and changed `GameFiles::new` to take a `GameKey`; #86 still used both and its merge
  had no conflict there). After merging `develop` into a branch, build and test the touched crates before committing
  the merge.
- A cargo feature that is off by default needs clippy both ways, with and without it (`bench` off left 8
  items unused). Items only that feature reads take `#[cfg_attr(not(feature = "…"), expect(dead_code, reason = …))]`;
  put it on the method, not on the field the method reads, or the field's expectation is unfulfilled.
- Do not start a build in a worktree while a merge or checkout is writing its files: a build once read the
  old `Cargo.toml` with the new code (`unresolved import md5`).
- Unit tests of `app/src-tauri` must not reach Tauri's runtime (an `AppHandle` method called from tested code): the
  test exe then imports Windows entry points it cannot load and dies with `STATUS_ENTRYPOINT_NOT_FOUND`
  (`health.rs`). Keep the `AppHandle` behind a closure made where the app exists.
- A catalogue entry can be marked wired while nothing reads it (`video.filter`, `video.vsync`, `speed.spstart`,
  Trello #109 / #123, wired since). Before building on a
  setting, grep for its reader (`settings/keys.rs`, the interface), not only `wired`.
- Samsung's launcher (One UI) fills an adaptive icon's transparent background with black and crops it tighter than the
  72 dp preview: the Android icon has a solid background (amber) behind the mark, and is checked on a phone.
- The test phone (Galaxy S23 at a larger display size) is 360 × 780 CSS px, not 412 × 892: phone layouts and headless
  checks use 360 × 780 and 780 × 360 first. Debug builds go on with `adb install -r`, which keeps the app's data.
- A fix tried on a phone from a test worktree is not on `develop` until it is merged: check
  `git merge-base --is-ancestor <branch> develop` before calling it done.
- A debug build and the Play build are signed with different keys: Android refuses one over the other, and the
  uninstall in between deleted every save on the phone (2026-11-24). Export the saves first; no `.debug` package
  (owner's choice). `hasFragileUserData` only keeps data for a build with the same signature.
- Android SDK packages carry a minor number since Android 16: `compileSdk = 37` is `platforms;android-37.0`
  (`platforms;android-37` does not exist and sdkmanager fails).
- A reusable workflow (`play-release.yml`) gets its environment's secrets only when the caller says
  `secrets: inherit`; without it they are empty strings, not an error. Its permissions cannot exceed the caller's.
- The Tauri CLI writes Android's versionName / versionCode only from its config's `version`; with it gone from
  `tauri.conf.json` (the version lives in Cargo.toml), the build passes it with `--config` (`versions.mjs
  tauri-config`). A bare `pnpm tauri android build` is versionCode 1: never upload it.
- Google Play's API refuses the first upload of a new app: the first AAB goes in by hand in Play Console.

## Project rules

- **No `#[cfg(target_os)]` outside `crates/emudome-platform`**, nor Tauri's `#[cfg(desktop)]` / `#[cfg(mobile)]`, nor a
  `[target.…]` table in a `Cargo.toml` (the OS gate in `scripts/check`, with its temporary exceptions). macOS exists as a stub but is **not** compiled
  (no Apple license). iOS is excluded. The code must stay ready to add macOS without rewrites.
- **Never include, download or point to Nintendo files** (games, BIOS, firmware, keys). The user adds the games only.
  The cores use their built-in open source replacements. 3DS games must be decrypted.
  Syncing the user's own games to the user's own Google Drive (planned, `docs/decisions.md`) is not EmuDome
  distributing them: it is allowed.
- License GPL-3.0-or-later; the cores are separate projects with their own licenses.
- Features live in the shell, not in the cores: same interface for GB, GBC, GBA, NDS and 3DS.
  Where a core cannot support a feature, the interface shows it disabled with the reason.
- **No `unwrap()` outside tests, no `_ =>` on an enum, no glob imports** (`use x::*`): declared in
  `[workspace.lints.clippy]` of the root `Cargo.toml`, every crate opts in with `[lints] workspace = true`.
  Where a fallback arm is unavoidable (a `#[non_exhaustive]` enum from another crate), an `#[expect]` with a `reason`.
- LF line endings everywhere (`.gitattributes`, `.editorconfig`).
- Formatting: Prettier for the frontend (`app/.prettierrc.json`), `cargo fmt` for Rust. Nobody formats by hand.
- pnpm only for the frontend (`packageManager` and `devEngines` in `app/package.json`).
- Settings will be the strong point (Phase 1): levels Default → Global → Console → Game → Session,
  plus a "per device" axis. UI generated from a single schema in Rust. Do not write scattered settings by hand.

## Engineering rules (from the audit)

The audit found the code tidy where it is easy to test and fragile at the edges: exit paths, identity, shipped
binaries, the Rust ↔ TypeScript bridge, platforms nobody compiles. Each rule below closes one of those root causes.
Apply them to every change, including small ones, and say in the card when a rule could not be met.

1. **User data is never lost.** Saves, states, settings and the library are written only through one atomic helper
   (temp file + fsync + rename). Every way a game or the app can end flushes them: closing the game, closing the
   window, quitting, Android going to the background, abandoning a hung game, the crash dialog. A file that could not
   be read is never treated as empty and never overwritten. A new persisted file or a new exit path comes with a test
   that the data survives it.
2. **One identity per game**: the library's content id (`<console>:<hex>`). Anything stored per game (saves, states,
   settings, covers, play time, sync) is keyed by it, never by a path or a file name.
3. **Shipped third-party binaries are pinned and verified**: a recorded version and checksum per core and platform,
   an explicit list of what goes in a package, its licence and source offer next to it. A change to `bundle` in
   `tauri.conf.json` or to the Gradle `jniLibs` updates `THIRD_PARTY.md` in the same commit.
4. **Design for the biggest console and the phone, not for GBA.** Before touching the core host, video, audio,
   input or save paths, check the change against 3DS (multi-GB ROMs passed by path, GPU frames, states of tens of MB,
   analog sticks, multi-threaded cores) and Android (background, process death, memory). Nothing heavy runs under a
   lock the audio callback or the renderer also takes; no disk I/O on the emulation thread or in the audio callback.
5. **One source of truth across the bridge.** What Rust knows (file extensions, setting keys and defaults, enums,
   DTOs, error codes) is never typed again in TypeScript or in another Rust file: it is generated or asked for. A
   setting the user can change is either read by the code or shown disabled with the reason.
6. **Errors never vanish.** Every `emu.*` call has an error path that reaches the user or a deliberate, commented
   ignore; no bare `void emu.x()`. A backend failure that changes what the user gets (video fallback, no audio
   device, controllers unavailable, a core refusing a game) reaches the interface as an `ErrorCode`, not only the log.
7. **Async results are applied only if they are still current**: a response to an older request (another scope,
   another game, a newer reload) is dropped.
8. **Green on Windows is not green.** A change to shared code is compiled for Android and Linux too; Tauri's
   `#[cfg(desktop)]` / `#[cfg(mobile)]` and target tables in `Cargo.toml` count as OS branching and belong to
   `crates/emudome-platform` like `#[cfg(target_os)]`.
9. **The docs say what the code does.** A commit that changes a fact written in this file, `README.md` or
   `THIRD_PARTY.md` updates it in the same commit; «Current state» and `docs/known-gaps.md` are checked at every merge.

Which of these a script will check, and when: Trello #72 (phase 1 done: Android clippy, the cfg gate with
`desktop` / `mobile` and target tables, exact pins; still open: phase 2 moves the gate's exceptions, Linux, the merge rule).
Rule 5 is checked by #73 (the generated contract below; what is still typed by hand is listed there). Rule 6's `void emu.x()` / bare `emu.x()` is refused by ESLint
(#74, `app/eslint.config.js`, its fixture in `app/src/lib/errors.lint.test.ts`): a call hands its promise to
`report` / `ignore` / `settle` (`app/src/lib/errors.ts`) or awaits it, and `catchEverything` (main.ts) reports what
escapes. Until then the others hold by review (listed below).

## What is checked, and what holds only because it is read

`./scripts/check.ps1` (`scripts/check`) is **the** list: the README, the pre-commit hook and this file point to it
instead of repeating commands. No CI for the checks, by choice (GitHub Actions only builds and uploads the Android
package: `stage`'s test builds, `main`'s releases). It runs everything and summarizes what failed, and what was
skipped because this machine lacks a tool ("green, except the skipped steps above"). It first checks the Node
version (`scripts/node-version.mjs`, against `engines.node` in `app/package.json`) and stops there on a wrong one.

**Checked**: `cargo fmt`, clippy with the workspace lints (no `unwrap`, no `_ =>` on enums, no glob imports),
clippy for Android (`aarch64-linux-android`: the crates with no C code always, the whole workspace when an NDK is
found, else skipped),
`cargo test`, the licence check (`cargo deny check licenses` with `deny.toml`, `scripts/npm-licenses.mjs`),
`THIRD_PARTY.md`'s generated tables equal to a fresh run of `scripts/third-party-tables.mjs --check` (blind spots: a
stale `app/node_modules`, the spike's workspace, the hand-written sections), one version (`scripts/versions.mjs`:
the workspace `Cargo.toml` version equal in `app/package.json` and `tauri.conf.json`, every member inheriting it),
`vue-tsc`, ESLint (the import style and no dropped `emu.*` call included), Prettier (`format:check`), Vitest (`pnpm test`), the design system conventions
(`scripts/scan-conventions.mjs`, blocking in every file: no literal colour, alpha modifier or `dark:` in classes, no
px / rem / em length or bracketed opacity, z-index, duration or curve in a class, no bare-number opacity / z-index /
duration / edge width / outline offset, no `!` override, no string variant, no raw `<button>` / `<input>` outside
`design-system/components`, no `<style>`, no `phone` / `touch` prop used only in `:class`, no hand-written widget role (`dialog`,
`radio`, `menu`…) outside `design-system/components` except the shrinking list `HAND_ROLES_PENDING`, no `<Button>` named with both
`aria-label` and `title` (an icon alone takes `label`, #104), no `<Button>` / `<LinkRow>` / `<DropdownMenuItem>` /
`<Card as="button">` with no action (a handler, `to`, submit, `as-child`, a static `disabled`, a wrapper that gives
it one, the component's root, or `<!-- no-action: why -->`, #112); the rows and what each cannot see are in
`docs/frontend-conventions.md`), and the gates: no merge conflict marker,
no OS or form-factor code outside `crates/emudome-platform` (`cfg` naming `target_os` / `target_family` / `windows` /
`unix` anywhere; `desktop` / `mobile` and `[target.…]` tables in `Cargo.toml` too, except the temporary list in the gate:
`lib.rs`, `library.rs` and `saves.rs` of `app/src-tauri`, moved in #72 phase 2; no target table left there), tauri / wgpu /
librashader pinned with `=`, `invoke` only in `app/src/emu.ts`, no emoji,
no Nintendo files by name, no signing keystore (`*.jks`, `*.keystore`, `*.p12`, `*.pfx`) in the repo, no hardcoded
visible text in `.vue` templates (`scripts/visible-text.mjs`), and the cores (`scripts/cores.mjs check --self-test`:
`cores.lock`, the bundle lists, `cores/SOURCES.md`, the Gradle build and `THIRD_PARTY.md` agree, and the versioned
Android project `app/src-tauri/gen/android` is there and adds no jniLibs folder of its own, and a core built from
source names its source archive, CMake options and existing build scripts; it cannot see whether a URL still serves
the pinned bytes, or whether a source build still gives the pinned sha256, only a fetch / build can).
Each gate's comment says what it cannot see. The pre-commit hook runs the fast ones (`cargo fmt`, conflict markers,
Prettier on the staged frontend files).

**Checked by the Rust ↔ TypeScript contract** (Trello #73, in `cargo test` and Vitest; see the section below): the
bridge's types and the `SettingKey` union are generated and not stale; every `ErrorCode` has `errors.<code>` in both
message files; en.json and it.json have the same keys, placeholders and plural cases; every catalogue key, option,
page, section and group has its texts and no text is left for a key the catalogue lost; a setting's description
names a key (F9, L3 + X…) only in its `keys.keyboard` / `keys.controller` part, which the row shows only for an input
that is there (#148, `descWithKeys`; `speed.ffmode`, the Tab key's own setting, excepted); the keys the Rust code reads
exist with the kind it reads (at compile time, `settings/keys.rs`); the catalogue entries nothing reads are an exact
list in a test and shown disabled. Seen failing once on a new `ErrorCode` variant, a renamed key, a missing key and a
key of the wrong kind.

**Held only by review** (no script can see them):
- The Rust backend sends codes and values, never sentences (the contract checks that each code has its texts and
  parity that no message shows a `{detail}`, not what a string parameter holds: `effectNotLoaded`'s `detail`, the
  shader compiler's text, is the one left, and no message shows it).
- Text built in `<script>` or TypeScript (toasts, computed labels) goes through `t()`: the visible-text gate reads
  templates only.
- Features live in the shell, not in the cores; a feature a core cannot support is shown disabled with the reason.
- Settings come from the single schema in Rust, never written by hand (keys are typed; which ones a screen picks,
  e.g. the quick menu's rows, is still written in the interface).
- A new reader of a setting marks it wired (`.not_wired()` removed in catalog.rs): the test lists the unwired ones,
  it cannot see a reader that is missing.
- A control does what it says: the `dead-button` scanner rule sees that an action is bound, not that the handler
  does something or that a parent listens to the event it emits; a feature not ready is disabled with its reason.
- Every menu is fully usable with the controller: a new screen registers `useNavScope` (`app/src/nav.ts`)
  and marks its first element; the routing itself is unit-tested (`hotkeys.rs`), the scope tree and the primitives'
  keys too (`nav.test.ts`, `design-system/components/nav.test.ts`), the screens are not.
- Texts: address the user directly, verb first on buttons, no exclamation marks; errors say what to do.
- The design system's tokens and look (graphite, accent only on the main action, glass, Lucide icons).
- Which form a button takes (#104, docs/frontend-conventions.md «Icons on buttons»): main actions and the actions of
  dialogs, cards, banners and step feet icon + words; repeated row / tile actions and toolbars the icon alone with
  `label`; links, option pickers, tabs' words, page lists, click areas and pad keys words; a dialog's quiet way out
  words. Its icon goes through Button's `icon`, not the slot.
- What the conventions scanner cannot see: a length in % / vh / vw, an icon size passed as a prop (`:size="14"`), a
  `style` binding, a look-only `phone` / `touch` prop read in `<script>` or passed down to a child.
- Clean code in the interface (see «Interface structure»): small components with one job, logic in composables and
  tested pure functions, nothing duplicated across screens, names that say what a thing is for.
- The nine «Engineering rules (from the audit)»; the OS gate cannot see a `cfg` split across lines, code
  chosen at runtime, or a target table written as dotted keys. Linux is compiled only by running the check on Linux,
  the whole workspace for Android only where an NDK is installed, and nothing links or runs on a device.

**A rule arrives with its check, in the same commit**: a rule added here either comes with its lint / test / gate
(seen failing once on a real violation or a fixture, its blind spots written next to it), or is added to the
review-only list above. A rule in neither place is the thing this section exists to stop.

## Interface and tone

- Design system: **EmuDome Design System** (a copy of GameDome's that evolves on its own),
  https://claude.ai/artifact/TEz4yQgd1w27HRQLt4beAy. Its tokens live in `app/src/design-system/tokens/` (see below).
- Modern console style, dark theme:
  - neutral graphite, the colour comes from the covers;
  - amber (`accent`, the app icon's #faa40f, dark text on it; `general.accent` offers four other
    swatches) only on the main action. For these colours the repo's tokens (`colors.css`,
    `design-system/lib/accent.ts`) are the reference, not the design system artifact;
  - glass for panels and menus;
  - Lucide icons, no emoji.
- In game **no interface**: everything goes through the quick menu (Esc / Home button).
- **Every menu is fully usable with the controller** (d-pad/stick to move, A confirms, B goes back), as well as with mouse and keyboard.
  No screen may require mouse or keyboard.
- Texts (in every language): address the user directly ("tu" in Italian), short sentences, verb first on buttons,
  no exclamation marks. Errors say what is happening and what to do.

## Interface structure (design system)

Cleanliness, components and tokens, so that changing anything later happens **in one place**,
written as clean code. The full rules and their reasons: `docs/specs/2026-11-18-design-system-foundations.md`.
Migration done: Trello #61 (foundations) → #62 (every screen moved) → #65 (one place for every look and rule:
role tokens, shared components, feature folders) → #63 (rules blocking everywhere). New screens and graphic work
are written only in this structure; `docs/frontend-conventions.md` is the full reference.

- **Tokens** in `app/src/design-system/tokens/`, one file per family: `colors.css`, `typography.css`, `spacing.css`,
  `radius.css`, `shadow.css`, `opacity.css`, `motion.css`, `layers.css`; imported by `design-system/index.css`.
  - Tailwind v4 reads them in `@theme`, with its default scales **reset**: a colour, size, radius, shadow or
    duration that is not a token generates no class at all. Preflight is off.
  - Names come from the design system (`title`, `heading`, `body`, `space-*`, `control`, `radius-*`, `glow-*`…).
    One value per step: a new value is a design-system change (decided with the owner), never a literal in a screen.
  - Themes (Scuro / Nero / Chiaro) and the runtime accent (`lib/appearance.ts`, `design-system/lib/accent.ts`) are CSS variables on
    `:root` / `[data-theme]`; Tailwind points at them (`@theme inline`), so a theme or accent change needs no class.
  - Glass, glow and other compound looks are named utilities (`design-system/styles/utilities.css`), not long class strings.
- **Primitives** in `app/src/design-system/components/<name>/`, all on **Reka UI** (one standard, no hand-made ones):
  `index.ts` re-exports, `variants.ts` holds the variants as `as const` constants (`ButtonVariant.Primary`, never
  `variant="primary"`), the parts are `.vue` files. Today: Button, Card, Chip, Tabs, Segmented, Slider, Select,
  Switch, Dialog, Popover, DropdownMenu, RadioGroup, Checkbox, Input, Heading, Badge, Kbd, Progress, EmptyState,
  LinkRow, StatTile, SectionPage, LabelledBlock, Tooltip. RadioGroup and Checkbox have no look of their own: their
  items are `as-child` on the screen's Card or Button. A screen never restyles a primitive; a missing look is a new variant.
  Buttons (#104): a Lucide icon through Button's `icon` prop (sized by the button, one step larger on the phone),
  never drawn in the slot; an icon alone takes `label` (the existing text: accessible name and Tooltip, which opens
  on the mouse's rest and on the controller's / keyboard's focus).
- **Every visual value is a token**: colours, borders (width and colour, `tokens/border.css`), margins, paddings, gaps,
  radius, shadows, opacity. Where a value has a role it has a role name (`radius-control`, `spacing-card-pad`,
  `spacing-section-x`…), so changing it everywhere is one edit. Values not decided yet are tokens under a
  «pending» comment (decided with the owner), never literals in a screen. Internal design variables only, not user settings.
- **Touch look**: one `data-density="touch"` on `<html>` (`design-system/lib/useDensity.ts`) and the `touch:` variant;
  components never take a `phone` / `touch` prop just to change their look. It is on for the phone upright and
  on its side (#29). The phone's shell keeps clear of the system's bars and the notch with the `safe-*` /
  `camera-*` spacing tokens (`env(safe-area-inset-*)`, `viewport-fit=cover` in `index.html`).
- **TV mode** (Trello #30): one `data-layout="tv"` on `<html>` (`composables/useUiMode.ts`, the decision pure in
  `lib/uiMode.ts`: `general.ui`, or at Automatica the answer to the TV question, `components/tv-mode/TvModeDialog.vue`:
  a controller connecting, or connected at startup, outside TV asks «Passare alla modalità TV?», the last one leaving
  TV asks to go back (in place of the «controller disconnected» toast, closed by a reconnection); the answer holds for
  the run, «Non chiederlo più» saves it as `general.ui`; in game the game pauses meanwhile; full screen no longer
  decides; never on a phone) and the
  `tv:` variant. TV sizes are the «TV» tokens (`typography.css`, `spacing.css`, `border.css`), drawn for 1920 × 1080
  and grown by `--tv-scale` on a larger screen. Its own parts: the shell's `TvTabs` (sections, L1 / R1 from
  anywhere), `TvButtonBar` and `TvGlyph` (the connected controller's glyphs by position and `pad.confirm`,
  `lib/padGlyphs.ts`), the library's shelves (`components/library/tv/`), Settings (`components/settings/tv/`:
  the desk's structure with the TV sizes, the pages on the left, large rows on the right; ◀ ▶
  change a short choice / switch / slider / colour in place, A opens long choices in a full-screen sheet, X resets,
  a kind it does not draw keeps the desk's row; `tvSettingRow.ts` decides, pure); the quick menu and the other
  sections keep the desk's layout scaled up (`zoom-tv`).
- **`cn()`** (`app/src/design-system/lib/cn.ts`) merges classes and learns the token names from the theme CSS (`design-system/lib/themeKeys.ts`).
- **Controller**: one generic rule in `nav.ts`. An element that owns its keys (by the attributes Reka renders, never
  by component name) takes the arrows along its axis and Enter; a controller step is sent to it as that key; at its
  ends nav.ts moves spatially. Select, Dialog, Popover and DropdownMenu take B while open; a menu item owns Enter
  (its arrows stay nav.ts's, inside the menu's scope); entering a radio group lands on its chosen radio. Tested in Vitest
  (`design-system/components/nav.test.ts`) and in a real browser (`scripts/headless/kit-nav.mjs`).
- **Kit page** (`app/src/design-system/kit/`, `#kit`, development only, not in the build): every primitive and the main feature
  components (`kit/features/`) in every state, the three themes, the five accents. A new primitive or variant is
  added there first. It is also the review board: under each section `KitIssue`s (problem, «Ora» /
  «Proposta» rendered with the real components, solution; status open / proposed / decided) and an index with the
  open count; a decided issue is deleted once the component carries it. It opens in a plain browser (`pnpm dev`,
  then `#kit`): `emu.ts` stands in for Tauri there (development and `#kit` only).
- **The look**: compact and near-square, no empty band, sober motion. Radius roles 2 / 2 / 3 / 4
  (`badge` / `control` / `card` / `panel`), buttons a little rounder (`radius-button` 4); controls 24 / 28 / 30 / 32;
  role paddings tight (`card-pad` 10, `dialog-pad` 16, `section-x` 16); appearing things fade with 4 px of rise or a
  3% pop, a pressed button gives 2% (`tokens/motion.css`, keyed by the primitives' `data-slot`); the shell's sections
  enter the same way. The phone's touch target (44) and the TV sizes stay. A new component folder must be listed as
  an `@source` in `design-system/index.css`, or the classes only it uses are never generated (the closing strip once lost
  its padding that way).
- **Where a file goes** (`app/src/`), by what owns it:
  - `design-system/`: the design system, all of it (tokens, named looks, primitives in `components/<name>/`, their
    helpers, the mark, the Kit); `design-system/README.md` is its map. It imports nothing of the app but `nav.ts`
    and `generated/` (ESLint, `app/eslint.config.js`; `kit/` excepted: it shows the feature components too).
  - `components/<feature>/`: everything one feature is: its screen root (`LibraryView.vue`, `QuickMenu.vue`…), its
    parts, its composables (`useX.ts`), its pure modules (`x.ts`) and their tests. A component another feature reuses
    stays with the feature it belongs to and is imported from there (`settings/SettingRow.vue`, `cover/GameCover.vue`).
  - `composables/` (`useX`) and `lib/` (plain modules and helpers) only for what 2+ features, or the root for the
    whole app, use. A module only one feature uses moves into that feature's folder.
  - The root holds the app's wiring only: `main.ts`, `App.vue`, `emu.ts`, `nav.ts`, `styles.css`, `i18n/`,
    `design-system/`.
- **Imports**: `./name` for a file in the same folder, `@/path` for anything else (never `../`, never `./sub/x`),
  so a path reads the same in every file and a move changes only the moved file's importers. `vi.mock` names the
  module as it is imported. Checked by ESLint (`no-restricted-imports` and `no-restricted-syntax`,
  `app/eslint.config.js`); blind spot: a path that is not a plain string literal.
- **Clean code**: small components with one job (a screen composes panels, a panel composes primitives); logic in
  composables (`useX`) and pure functions with tests, the template only composes; nothing duplicated across screens
  (one slot tile, one button tester, one combo table, one confirm dialog); names that say what a thing is for.
- **Checked** by `scripts/scan-conventions.mjs` (in `scripts/check`, each rule self-tested on its fixture, blocking in
  every file): the rules listed in «What is checked». The conventions as a whole, and what each scanner rule cannot
  see: `docs/frontend-conventions.md`.

## References

- Analysis and integrations (cores, platforms, settings catalogue, risks, roadmap):
  https://claude.ai/artifact/A8hWdUgrW6qieC7XxQm6ou
- Interface prototype (draft, paused): https://claude.ai/artifact/HeL9MWZ9Pd41dGLGCZWzaj
- `docs/decisions.md`: the project's tooling and product decisions with their reasons (GB/GBC support, Google Drive
  sync, game identity from the content, no cover scraping, positional face layout, NDS/3DS screens composed by
  EmuDome, no core of our own). Read it before designing anything they touch.
- `docs/customization.md` (what can be customized, how, at what cost), `docs/feasibility.md` (desk checks of
  those decisions), `docs/combo-tasti.md` (controller combos), `docs/fase0-esito.md` (Phase 0 test log),
  `docs/release.md` (the version, the packages as they are, the release flow: tag on `main`, built from a clean clone).
- Roadmap:
  - Phase 0: video, GBA, Windows and Android
  - Phase 1: desktop shell (library, settings engine, complete saves, shaders with librashader)
  - Phase 2: NDS
  - Phase 3: 3DS
  - Phase 4: full Android
  - Phase 5: extras

## Known gaps of the prototype (intentional, for later)

The list lives in `docs/known-gaps.md` (core options, touch controls, Android controllers and folders, regional
versions, window / video / audio settings, DS / 3DS screens, TV mode, the phone, «Gestisci lo spazio», search and
notifications, cores and releases, the game key migration, rewind and covers, the core host, Google Drive sync and
the Cloud interface). Read it before working on one of those areas; a change that closes or opens a gap updates it.

## Design and interface work

There is no separate Cowork session any more. The design system (tokens, primitives on Reka, the
Kit page) and the boards already drawn on the canvas are the reference.

**Who does what:**
- **Interface and design: with the owner, in dedicated sessions.** The UI step cards (screens, layouts, flows) are not
  started autonomously. Claude Code does not invent or restyle UI on its own: a visual problem it notices goes on Trello
  for the owner, not into the code. New user-visible texts follow the existing tone and are listed on the card for the
  owner to review.
- **Claude Code: backend structure and feasibility.** Rust crates, cores, platforms, video/audio/input, IPC, settings
  engine, build and checks, prototypes that answer "can it be done", and the interface wiring of what is already
  designed; until a design exists, the UI stays a functional placeholder (correct, i18n, controller-usable,
  design-system tokens), not a design proposal.

**Design principles** (the user-level skill `emudome-design-taste` has the reasoning; load it before any mockup or
interface change): content first and large (game screens edge to edge on the phone); few things on a level, the
frequent ones one tap away; density over empty bands; nothing that pretends (a handle that does not drag); buttons with
icons and verbs as labels; squared and tight; amber only on the main action. A visual question is asked with a picture
(an artifact page with phone frames at the real size and the app's tokens), numbered options and one recommendation.

After every Phase 0 test, update **`docs/fase0-esito.md`** (create it if missing) in this format, without deleting the earlier entries:

```
## <date and time> · <what was tested>
| Item | Result | Notes |
| --- | --- | --- |
| 1. Native video under the menu | ok / ko / not tested | what is visible, screenshots if any |
| 2. FPS at 1× | measured value | headless or in app |
| 3. Audio | ok / ko / not tested | crackles, delay |
| 4. Controller | ok / ko / not tested | model |
| 5. Native vs webview | differences noticed | smoothness, latency |

Errors and fixes: files touched, error, cause, how it was solved.
Proposed next step: one line.
```

Rules: write only verified facts, mark "not tested" where the test is missing, do not change the sections of this file without telling the user.
