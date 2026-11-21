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
  contract test and `app/src/i18n/parity.test.ts` fail until all three are there.
- Kept file names: `docs/fase0-esito.md` and `docs/combo-tasti.md` keep their Italian names (an external Cowork session
  refers to them); their content is English.

## Current state: Phase 1 desktop shell, Phase 2 core work started

Phase 0 (feasibility) is done. On `develop` now: shell and themes, settings engine and pages, library, game flow,
quick menu, saves, controls, first run, rewind and screenshots, screen effects (librashader), appearance, and the
melonDS DS core (#45), and 3DS games open on Azahar (#46, first milestone: Vulkan on the app's device, frames read
back to the CPU, encrypted games refused with E10). The audit of 2026-11-20 put its findings on Trello (#69–#83, P0 first): the P0 cards come
before any new feature.

Git: `develop` is the integration branch, checked out in the main worktree; work happens on branches in
`C:\Projects\emu-dome.wt\<name>` and is merged into `develop` from the main worktree. Pushing `develop` is allowed
(owner, 2026-11-16). No push to `main`. **Merge rule**: `./scripts/check.ps1` green on the merge commit itself
(not only on the branch) before `git push` of `develop`; `wip` commits are squashed before merging, never merged as
they are. **Checks on work branches** (owner, 2026-11-20 and 2026-11-21): only targeted ones while working (the
touched crates' tests and clippy, typecheck, lint, the touched Vitest files), `./scripts/check.ps1 --fast` once at the
end of a branch; the full check runs only on the merge commit into `develop`. No build only to measure or to confirm
something already verified.

Phase 0 answered the architecture question: the native video under a transparent webview holds up. The full test
log, entry by entry, is `docs/fase0-esito.md`; in short:

- **Linux** (virtual X11, mGBA 0.10.2): windowless host 1× ≈ 59.7 FPS, 2× ≈ 120, Max ≈ 900; app in webview mode
  works. Native mode does not (the GTK surface covers the webview).
- **Windows** (2026-11-16): native video under the quick menu ok (wgpu picks Vulkan), 59.7 FPS at 1×, audio heard ok,
  controllers ok through SDL3 (Steam Controller, 8BitDo). Last reported open: the window turns transparent in webview mode.
- **Android** (Galaxy A71, 2026-11-16): `SurfaceView` under the WebView ok, ≈ 59.8 FPS (Vulkan), touch controls,
  surface survives background/resume. Not tested yet: audio quality and delay (the test ROM is silent),
  Android controllers (SDL3 not built for Android), landscape layout.
- Since then, on `develop`: L3 controller combos, controller navigation in every menu, the L01 app shell and themes,
  the settings engine (2a).

## Commands

```powershell
./scripts/fetch-cores.ps1 [-Cores mgba,melondsds,azahar|all] [-Build]  # cores pinned by cores.lock into cores/, sha256-checked
                                              # (default: the shipped ones; sh scripts/fetch-cores.sh [--cores ...] [--build] on Linux);
                                              # mGBA comes from our release cores-mgba-<commit> (cores.lock built.release), built
                                              # from source (as build-mgba.ps1) only if that download fails or with -Build
./scripts/fetch-cores-android.ps1 [-Cores ...] [-Abi arm64-v8a] [-Build]  # same, into cores/android/<abi>/ as lib*_libretro_android.so
./scripts/build-mgba.ps1 [-Platform windows-x86_64|android-arm64-v8a] [-Jobs 2]  # mGBA from the source cores.lock
                                              # pins, installed only if its sha256 is the pinned one (scripts/core-build.mjs:
                                              # VS 2022 Build Tools + the NDK's clang-cl, NDK for Android, git; sh build-mgba.sh)
./scripts/android-build.ps1 [-Install] [-Release] [-DryRun]  # the Android APK (arm64-v8a) from a clean clone: cores,
                                              # frontend, Rust, Gradle; prints the APK; -Install = adb install -r
                                              # (sh scripts/android-build.sh [--install] ... on Linux / Git Bash)
node scripts/cores.mjs generate               # after editing cores.lock: cores/SOURCES.md, app/src-tauri/package/*.json
pnpm package:windows                          # installer (from app/): only the cores.lock files, verified (THIRD_PARTY.md)
                                              # releases (tag on main, clean clone, where published): docs/release.md
node scripts/third-party-tables.mjs --write   # after a dependency change (and pnpm install): THIRD_PARTY.md's tables
node scripts/versions.mjs                     # the version agrees in Cargo.toml, app/package.json, tauri.conf.json
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
cargo run -p emudome --example gamepad_probe  # windowless SDL3 controller probe (app/src-tauri/examples)
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
  - `RUST_LOG=debug` for the logs.
  - `EMUDOME_DATA_DIR=<folder>` puts every data file there (saves, settings, library, covers, webview profile) and
    skips the one-time copy of the old identifier's folders: test and bench runs never touch the real data.
- Data folders: Tauri's app data / app local data folders for the identifier `it.domehub.emudome` (`app/src-tauri/src/dirs.rs`).
  Until 2026-11-21 it was `com.domenghini.emudome` (#90): a desktop build's first start copies those folders once,
  verified, and leaves them in place (`emudome_core::data_move`); a copy that did not finish makes that run use the
  old folders and is tried again. Android data does not carry over (another package's storage).
- The executable accepts a game path as an argument and opens it at startup.
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
                         settings/ (settings engine: schema.rs types, catalog.rs the 160 entries,
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
                         `ScreenStatus` what the interface shows; pure, unit-tested)
                         save_path.rs (#94: does a core's deepest file fit under the game's folder within
                         the platform's `PathLimits`; checked before the core loads; pure, unit-tested)
                         contract.rs (the Rust ↔ TypeScript contract: ts-rs types and fixtures generated
                         into app/src/generated/, the staleness check; see the section of that name)
                         speed_audio.rs (audio off 1×: `audio.ff` mute / resample / WSOLA time-stretch; pure)
                         library/ (game library: identity.rs console, `<console>:<hex>` id from the content,
                         title and NDS/3DS icon; scan.rs folder walk with a path+size+mtime hash cache;
                         store.rs library.json, games never deleted; pure, unit-tested)
                         first_run.rs (first run: its steps, whether to show it; pure, unit-tested)
                         close_flow.rs (closing with a game open: the state machine; pure, unit-tested)
                         game_key.rs (`GameKey`, the one identity of a game: the library's content id;
                         saves in `saves/<console>/<hex>/` (`GameFiles`), settings `game/<console>-<hex>.json`)
                         migration.rs (the one-time move of saves named by ROM file stem and settings named
                         by path hash to the key: facts from the library only, copies in `migration-70/backup/`,
                         `migration-70/report.json`, marker `migration-70/done`; unit-tested)
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
                         picker (pick_file), logs, core paths, the external surface, the old identifier's
                         data folders (old_data_dirs: beside the new ones on desktop, none on Android),
                         paths.rs (how long a path this process can create: `path_limits`, Windows long
                         paths for the process and the `LongPathsEnabled` policy, read only);
                         android/ is the Gradle library (Kotlin NativeSurface: SurfaceView under the WebView,
                         jniLibs: only the cores.lock files from cores/android/<abi>/, sha256-checked)
app/src-tauri            Tauri shell: lib.rs (wiring only: modules, `AppState`, the per-game hooks, setup, the
                         command list, `blocking()` for heavy commands), game_session.rs (`GameSession`: the
                         open game and what lives as long as it does, open / take / close / abandon; exit-path
                         tests on `FakeCore`), game.rs (opening a game, `close_game`, `abandon_game`),
                         commands.rs (the one-line in-game commands, `key`), protocol.rs (the `emu://` scheme:
                         one route per resource, unknown paths 404), events.rs (every event name),
                         hotkeys.rs (`HotkeyRunner`: the shortcuts of the pads and of the keyboard carried out,
                         one hold shared by both), screens.rs (the open game's `ScreenSession` carried out:
                         the renderer's layout, `emu://screens`, the lid chord pressed on the core, the
                         `screens_*` commands; the webview canvas gets its placements from
                         `screens_placements`), tuning.rs (developer / bench knobs from the environment),
                         window.rs (the app's names for emudome_platform::create_window),
                         video.rs (wgpu; `GpuService`, one per app, owns instance / adapter / device for
                         the whole run: surfaces attach and detach on it, a 3DS game leases the device
                         (`VulkanLease`) and the device is never remade under it; every submit and present
                         holds its queue lock), gpu_interop.rs (the one module under wgpu: wgpu-hal / ash,
                         the device opened with what a 3DS core uses (`open_device_for_cores`), the core's
                         image as a texture, raw barriers), hw_render.rs (a Vulkan core's frames: GPU copy
                         of the core's image into a game texture, readback into `Shared::frame`;
                         `headless_setup` for `examples/headless_gpu.rs`),
                         audio.rs (cpal), gamepad.rs (SDL3), commands,
                         settings.rs (settings commands, file locations, settings://changed),
                         effects.rs (screen effects: built-in .slangp presets from effects/, fx.* → renderer,
                         emu://effect-error), video.rs runs them through librashader (native path only)
                         bios.rs (bios_choose / bios_status, per-core system folder
                         `<app local data>/system/<core>/` for cores with a BIOS table, emu://bios-fallback)
                         theme.rs (Scuro / Nero / Chiaro: EMUDOME_THEME, else general.theme, else the system)
                         closing.rs (the close flow carried out: worker, emu://closing, exit when saved)
                         health.rs (what is not working: ErrorCodes per part, emu://health, diagnostics)
                         dirs.rs (the data folders: Tauri's, EMUDOME_DATA_DIR, or the old identifier's for a
                         run whose copy did not finish), data_move.rs (that copy, first thing in setup)
                         library.rs (library commands, background scans, library://scan and library://changed,
                         play time, covers on emu://cover/<id>, library.watch with notify on desktop)
                         contract.rs (test only: the shell's DTOs generated into app/src/generated/app/)
app/src                  Vue interface, placed as «Interface structure» says: main.ts, App.vue (the root),
                         emu.ts (typed bridge, its types from generated/), generated/ (written by the Rust
                         contract tests, never by hand), nav.ts (controller/keyboard focus navigation),
                         components/<feature>/ one folder per feature: shell (AppShell: rail + sections,
                         outside the game), library, details, saves, settings, controls, quick-menu,
                         game (GameCanvas: webview mode, TouchControls: Android overlay, the game flow,
                         useScreenStatus: the open game's `ScreenStatus` from `emu://screens`,
                         LidClosedCard: G15 «Coperchio chiuso»),
                         screen-editor (C09, Trello #28: ScreenEditor over the game at its real size, from
                         G09 and from the `screens.custom` row; useScreenEditor the state, the live preview
                         at the Session level and the steps; screenEditor.ts the geometry, pure, tested),
                         first-run (FirstRun, 4 steps, once per device: marker `<app local data>/first-run/done`,
                         src-tauri/src/first_run.rs), cover, toasts, closing (the loader while a game closes);
                         components/ui/ (the primitives),
                         composables/ and lib/ (shared by 2+ features: lib/theme.ts data-theme on <html>,
                         lib/settingsModel.ts, lib/cn.ts, lib/design/…),
                         i18n/ (index.ts vue-i18n setup and locale choice, en.json, it.json, labels.ts codes → text),
                         assets/theme/ (design tokens, one file per family), kit/ (Kit page, development only)
```

- The interface never touches the core: it calls the Tauri commands, which send `Command` to the `Session`.
  The core runs on its own thread. The libretro callbacks find their context through the loaded core's slot
  (`callbacks.rs`: 8 static slots, one callback set per slot), so calls from a core's own threads reach it and
  a core let go keeps its slot, empty, forever. A `need_fullpath` core (Azahar) gets the game's path only: the
  host never reads its file (`GameContent`, `core.rs`).
- The open game is one `GameSession` (`app/src-tauri/src/game_session.rs`): the `Session`, `CurrentShared`, the
  key of the game whose files are in use (until its thread ended), and the per-game parts as `Hooks` started in
  order and stopped in reverse (renderer source, settings session, core options that follow the settings, play
  time; a new per-game part is one more hook in `lib.rs`).
- Ending a game: every way out (closing the game, opening another, closing the window, quitting) takes it out
  through `GameSession::take` (the close flow, `closing.rs`) or, for `RunEvent::Exit` only, `GameSession::close`,
  and waits up to `CLOSE_TIMEOUT` (10 s) for the thread's exit writes: the `.sav` first, then the auto quick save
  with `saves.autoexit`. Android going to the background
  (`onPause`) sends `Command::Flush` (the same writes, the game keeps running). A thread that does not stop in
  time, or is abandoned after E11, is let go: the session is dead (no file, no event, no `retro_unload_game` /
  `retro_deinit`) and its core is refused with `coreNeedsRestart` until the app restarts.
- Writes while the game runs (quick saves, the periodic auto quick save and `.sav` flush, «Annulla» of a save,
  screenshots) go to the session's I/O worker (`session/io.rs`, thread `emudome-io`): the emulation thread only
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
    options (present-driven / audio-driven): `docs/video-pacing.md`.
  - **Webview**: the RGBA frames reach a `<canvas>` through the `emu://frame?since=N` protocol.
  - **Screens** (Trello #28): both paths draw each screen of the frame where `emudome_core::screens::compose`
    puts it for the open game's layout (`screens.*`, the layout shown, swapped or not): the native renderer one
    draw per screen (`Renderer::set_layout`), the webview canvas from `screens_placements` (it decides no
    geometry: `GameCanvas.vue`, `screenDraw.ts`). Touch maps back through the same layout (`pointer`).
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
  callback, linear resampling to the device's rate, ±0.5% correction on the ring's fill level; the reader drops
  what is queued beyond ~150 ms, oldest first. Off 1× `audio.ff` decides (`speed_audio.rs`, on the emulation thread,
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
  (`WindowEvent::Focused(false)`, and the page's `blur` through `release_keys`) let go of every keyboard button and
  end the keyboard's fast forward and held rewind, in Rust. The touch controls still send buttons (`input`).
  Controllers go through an SDL3 thread
  (`gamepad.rs`) with RetroArch-style positional mapping; triggers and the left stick past half travel
  count as L2/R2 and d-pad. Both sticks also reach the core as `RETRO_DEVICE_ANALOG` while the game has the
  input (`Routed::game_has_input`). The core reads a frozen `input::Snapshot`, taken before each frame and at
  its `input_poll`, never the live state mid-frame.
- **Controller routing lives in Rust**, as a pure function: `emudome_core::hotkeys::Router::step`
  (previous state, buttons, time → game mask + actions), unit-tested without SDL. Three modes:
  - in game: buttons go to the core, except the modifier L3, which never does;
  - L3 held: combos from the **one table** `DEFAULT_COMBOS` (`hotkeys.rs`, remappable in Phase 1),
    carried out by `HotkeyRunner` in `hotkeys.rs`, shared with the keyboard (session `Command`s, or events for
    the UI; a hold from both, Tab and L3 + R2, ends with the last release);
    the full table and its TODOs are in `docs/combo-tasti.md` section 5;
  - the interface has the input (quick menu open, or no game): the game gets nothing, d-pad / A / B
    become `emu://nav` steps with key repeat (400 ms, then every 120 ms); L1 / R1 `prev` / `next`, X `more`,
    Select `select`, and the right stick past `pad.stickdpad` (`PadInput::stick`) `stickUp` / `stickDown` /
    `stickLeft` / `stickRight`, repeating like the d-pad (the C09 screens editor's own steps).
  The interface tells Rust the menu is open with the `set_menu_open` command.
- The library's cover grid is windowed (Trello #91, `components/library/useCoverWindow.ts`, sums in
  `coverWindow.ts`): only the rows near the screen plus 2 are in the DOM, the grid's padding stands for the rest
  (rows measured once rendered, the others estimated), and a re-render never moves what is on screen. Inside it
  the controller and the arrows move by row and column (`coverGridNav.ts`, registered with nav.ts's
  `useNavGrid`), and leave it at its edges by the spatial rule; a focused tile scrolled out of the DOM parks the
  focus on the grid (no ring) and gets it back when rendered again. Measured headless with 3005 / 10 005 mock
  games (`scripts/headless/screens/library-scale.mjs`): a d-pad step 64 / 146 ms before, under 1 ms after.
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
- **Settings engine** (`emudome_core::settings`): one static schema (`catalog.rs`, 160 entries on 18 pages: key
  `<prefix>.<key>`, page (its own field: «Avanzate» keeps the `perf.` / `cores.` / `adv.` keys), kind, default
  (with `default_rules`: another default on one device or for some consoles, first match wins, read by `resolve` at
  the Default level; e.g. `speed.rew` off on phones and for 3DS, Trello #89), accepted levels, per-device flag, platform, phase, `show_if` on another row's value, `wired`: whether anything
  reads it, the unwired ones shown disabled with «arrives in a future update»); the interface builds its
  pages from `settings_schema` and never lists settings by hand. A key the interface names is a `SettingKey`
  (generated union); one the Rust code reads is a typed constant of `settings/keys.rs`, its default the catalogue's. Levels Default → Global → Console → Game →
  Session; `resolve` returns the value and the level it comes from; a level the entry does not accept is refused
  (`settingLevelRefused`). Files: `<app data>/settings/{global,console/<id>,game/<console>-<hex>}.json` (synced later),
  per-device values in `<app local data>/local-settings/device.json` (never synced); atomic writes, unknown keys
  kept, invalid values ignored with a `settingInvalidValue` log; a file that is there but cannot be read is never
  written over (writes to it refused with `settingsReadFailed` until it reads again; the library does the same
  with `libraryReadFailed`). Every file says its layout (`"$schema": 1`, written on save; none = 0; older ones go
  through `store.rs`'s `MIGRATIONS` in memory) and is read again before use when its mtime or size changed. The
  sync rules as they are today are tests (`settings/tests.rs`, «Sync rules»). Session = the quick menu's changes «in prova»:
  in memory, bound to the open game, committed to Game or Console (all or nothing), dropped on close.
  A game is its `GameKey` everywhere (settings, saves, library, the interface's `GameId`); a file opened by its path
  is identified first (the library's hash cache, else hashed, with no lock held). Option ids, slider ranges and chips defaults are Cowork's final ones (Trello #41, #26); search synonyms
  live in `SYNONYMS` (catalog.rs).
- **The Rust ↔ TypeScript contract** (Trello #73): the bridge's types are generated from the Rust types by ts-rs
  (`#[derive(TS)]`, MIT) into `app/src/generated/` (core types; the shell's DTOs in `generated/app/`), with
  `SettingKey` (the catalogue's keys), `HotkeyId` and `Button` from Rust lists, and the fixtures
  `error-codes.json` / `settings-catalog.json` (its `colours`: a colour entry's swatches and default, read by
  `lib/accent.ts`) / `keyboard-shortcuts.json` (read by `lib/keyboard.ts`). `emu.ts` only re-exports them. The files are committed; the contract
  test of each crate (`cargo test -p emudome-core contract`, `cargo test -p emudome contract`) fails when they are
  stale, and `EMUDOME_BLESS=1` writes them again. `app/src/i18n/parity.test.ts` holds the fixtures against the
  message files. `session::Speed` / `Event` are typed through shadow types in `contract.rs` (session.rs untouched),
  held to the real serialization by a test. Still by hand: the command names and argument lists in `emu.ts` and
  `BenchFrame` (the `bench` feature). The generated folder is
  left out of ESLint and Prettier (compared byte for byte).
- Events, every name a constant of `app/src-tauri/src/events.rs` (`<domain>://<event>`; `emu://` the emulator,
  `library://`, `settings://`, `saves://` theirs): `emu://session` (the session's `Event`s), `emu://gamepad-connected`,
  `emu://gamepad-disconnected`, `emu://gamepad-changed` (a `PadInfo`), `emu://gamepad-state` (while watched), `emu://menu`,
  `emu://nav` (`{ action }`), `emu://hotkey` (`{ kind, ... }`: fps, slot, speed, turbo, paused),
  `emu://screens` (a `ScreenStatus`: the layout shown, the cycle, swapped, the lid supported / closed),
  `library://changed` (`LibraryChanged`: the rows that changed, or the whole library), `library://scan`,
  `saves://migration`, `emu://effect-error`,
  `settings://changed` (`{ key, level }`), `emu://core-restart` (`{ settings }`: keys waiting for the game to be
  opened again), `emu://bios-fallback` (an `ErrorCode`: a setting asked for system files that are not in place),
  `emu://closing` (`{ phase, purpose, error }`: a game closing, see «Closing with a game open» below),
  `emu://health` (the whole list of `ErrorCode`s of what is not working, see below).
- Health (Trello #79, `app/src-tauri/src/health.rs`): backend failures that change what the user gets are one
  `ErrorCode` per part (native video fallen back `nativeVideoFallback`, surface `videoSurfaceFailed`, audio output
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
- Branches developed in parallel break each other without a textual conflict (2026-11-20: #70 renamed
  `AppState::game_path` to `game` and changed `GameFiles::new` to take a `GameKey`; #86 still used both and its merge
  had no conflict there). After merging `develop` into a branch, build and test the touched crates before committing
  the merge.
- A cargo feature that is off by default needs clippy both ways, with and without it (2026-11-21: `bench` off left 8
  items unused). Items only that feature reads take `#[cfg_attr(not(feature = "…"), expect(dead_code, reason = …))]`;
  put it on the method, not on the field the method reads, or the field's expectation is unfulfilled.
- Do not start a build in a worktree while a merge or checkout is writing its files: on 2026-11-20 a build read the
  old `Cargo.toml` with the new code (`unresolved import md5`).
- Unit tests of `app/src-tauri` must not reach Tauri's runtime (an `AppHandle` method called from tested code): the
  test exe then imports Windows entry points it cannot load and dies with `STATUS_ENTRYPOINT_NOT_FOUND`
  (2026-11-21, `health.rs`). Keep the `AppHandle` behind a closure made where the app exists.

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

## Engineering rules (from the 2026-11-20 audit)

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
   `THIRD_PARTY.md` updates it in the same commit; «Current state» and «Known gaps» are checked at every merge.

Which of these a script will check, and when: Trello #72 (phase 1 done: Android clippy, the cfg gate with
`desktop` / `mobile` and target tables, exact pins; still open: phase 2 moves the gate's exceptions, Linux, the merge rule).
Rule 5 is checked by #73 (the generated contract below; what is still typed by hand is listed there). Rule 6's `void emu.x()` / bare `emu.x()` is refused by ESLint
(#74, `app/eslint.config.js`, its fixture in `app/src/lib/errors.lint.test.ts`): a call hands its promise to
`report` / `ignore` / `settle` (`app/src/lib/errors.ts`) or awaits it, and `catchEverything` (main.ts) reports what
escapes. Until then the others hold by review (listed below).

## What is checked, and what holds only because it is read

`./scripts/check.ps1` (`scripts/check`) is **the** list: the README, the pre-commit hook and this file point to it
instead of repeating commands. No CI, by choice. It runs everything and summarizes what failed, and what was
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
`components/ui`, no `<style>`, no `phone` / `touch` prop used only in `:class`, no hand-written widget role (`dialog`,
`radio`, `menu`…) outside `components/ui` except the shrinking list `HAND_ROLES_PENDING`; the rows and what each cannot see are in
`docs/frontend-conventions.md`), and the gates: no merge conflict marker,
no OS or form-factor code outside `crates/emudome-platform` (`cfg` naming `target_os` / `target_family` / `windows` /
`unix` anywhere; `desktop` / `mobile` and `[target.…]` tables in `Cargo.toml` too, except the temporary list in the gate:
`lib.rs`, `library.rs`, `saves.rs` and the SDL3 table of `app/src-tauri`, moved in #72 phase 2), tauri / wgpu /
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
page, section and group has its texts and no text is left for a key the catalogue lost; the keys the Rust code reads
exist with the kind it reads (at compile time, `settings/keys.rs`); the catalogue entries nothing reads are an exact
list in a test and shown disabled. Seen failing once on a new `ErrorCode` variant, a renamed key, a missing key and a
key of the wrong kind.

**Held only by review** (no script can see them):
- The Rust backend sends codes and values, never sentences (the contract checks that each code has its texts, not
  what a `detail` holds).
- Text built in `<script>` or TypeScript (toasts, computed labels) goes through `t()`: the visible-text gate reads
  templates only.
- Features live in the shell, not in the cores; a feature a core cannot support is shown disabled with the reason.
- Settings come from the single schema in Rust, never written by hand (keys are typed; which ones a screen picks,
  e.g. the quick menu's rows, is still written in the interface).
- A new reader of a setting marks it wired (`.not_wired()` removed in catalog.rs): the test lists the unwired ones,
  it cannot see a reader that is missing.
- Every menu is fully usable with the controller: a new screen registers `useNavScope` (`app/src/nav.ts`)
  and marks its first element; the routing itself is unit-tested (`hotkeys.rs`), the scope tree and the primitives'
  keys too (`nav.test.ts`, `components/ui/nav.test.ts`), the screens are not.
- Texts: address the user directly, verb first on buttons, no exclamation marks; errors say what to do.
- The design system's tokens and look (graphite, accent only on the main action, glass, Lucide icons).
- What the conventions scanner cannot see: a length in % / vh / vw, an icon size passed as a prop (`:size="14"`), a
  `style` binding, a look-only `phone` / `touch` prop read in `<script>` or passed down to a child.
- Clean code in the interface (see «Interface structure»): small components with one job, logic in composables and
  tested pure functions, nothing duplicated across screens, names that say what a thing is for.
- The nine «Engineering rules (from the 2026-11-20 audit)»; the OS gate cannot see a `cfg` split across lines, code
  chosen at runtime, or a target table written as dotted keys. Linux is compiled only by running the check on Linux,
  the whole workspace for Android only where an NDK is installed, and nothing links or runs on a device.

**A rule arrives with its check, in the same commit**: a rule added here either comes with its lint / test / gate
(seen failing once on a real violation or a fixture, its blind spots written next to it), or is added to the
review-only list above. A rule in neither place is the thing this section exists to stop.

## Interface and tone

- Design system: **EmuDome Design System** (a copy of GameDome's that evolves on its own),
  https://claude.ai/artifact/TEz4yQgd1w27HRQLt4beAy. Its tokens live in `app/src/assets/theme/` (see below).
- Modern console style, dark theme:
  - neutral graphite, the colour comes from the covers;
  - amber (`accent`, the app icon's #faa40f, dark text on it; `general.accent` offers four other
    swatches) only on the main action. For these colours the repo's tokens (`colors.css`,
    `lib/accent.ts`) are the reference, not the design system artifact (owner, 2026-11-21);
  - glass for panels and menus;
  - Lucide icons, no emoji.
- In game **no interface**: everything goes through the quick menu (Esc / Home button).
- **Every menu is fully usable with the controller** (d-pad/stick to move, A confirms, B goes back), as well as with mouse and keyboard.
  No screen may require mouse or keyboard.
- Texts (in every language): address the user directly ("tu" in Italian), short sentences, verb first on buttons,
  no exclamation marks. Errors say what is happening and what to do.

## Interface structure (design system)

Owner, 2026-11-18: cleanliness, components and tokens, so that changing anything later happens **in one place**,
written as clean code. The full rules and their reasons: `docs/specs/2026-11-18-design-system-foundations.md`.
Migration done: Trello #61 (foundations) → #62 (every screen moved) → #65 (one place for every look and rule:
role tokens, shared components, feature folders) → #63 (rules blocking everywhere). New screens and graphic work
are written only in this structure; `docs/frontend-conventions.md` is the full reference.

- **Tokens** in `app/src/assets/theme/`, one file per family: `colors.css`, `typography.css`, `spacing.css`,
  `radius.css`, `shadow.css`, `opacity.css`, `motion.css`, `layers.css`; imported by `assets/main.css`.
  - Tailwind v4 reads them in `@theme`, with its default scales **reset**: a colour, size, radius, shadow or
    duration that is not a token generates no class at all. Preflight is off.
  - Names come from the design system (`title`, `heading`, `body`, `space-*`, `control`, `radius-*`, `glow-*`…).
    One value per step: a new value is a design-system change (decided with the owner), never a literal in a screen.
  - Themes (Scuro / Nero / Chiaro) and the runtime accent (`lib/appearance.ts`, `lib/accent.ts`) are CSS variables on
    `:root` / `[data-theme]`; Tailwind points at them (`@theme inline`), so a theme or accent change needs no class.
  - Glass, glow and other compound looks are named utilities (`assets/utilities.css`), not long class strings.
- **Primitives** in `app/src/components/ui/<name>/`, all on **Reka UI** (one standard, no hand-made ones):
  `index.ts` re-exports, `variants.ts` holds the variants as `as const` constants (`ButtonVariant.Primary`, never
  `variant="primary"`), the parts are `.vue` files. Today: Button, Card, Chip, Tabs, Segmented, Slider, Select,
  Switch, Dialog, Popover, DropdownMenu, RadioGroup, Checkbox, Input, Heading, Badge, Kbd, Progress, EmptyState,
  LinkRow, StatTile, SectionPage, LabelledBlock. RadioGroup and Checkbox have no look of their own: their items are
  `as-child` on the screen's Card or Button. A screen never restyles a primitive; a missing look is a new variant.
- **Every visual value is a token**: colours, borders (width and colour, `theme/border.css`), margins, paddings, gaps,
  radius, shadows, opacity. Where a value has a role it has a role name (`radius-control`, `spacing-card-pad`,
  `spacing-section-x`…), so changing it everywhere is one edit. Values not decided yet are tokens under a
  «pending» comment (decided with the owner), never literals in a screen. Internal design variables only, not user settings.
- **Touch look**: one `data-density="touch"` on `<html>` (`composables/useDensity.ts`) and the `touch:` variant;
  components never take a `phone` / `touch` prop just to change their look.
- **`cn()`** (`app/src/lib/cn.ts`) merges classes and learns the token names from the theme CSS (`lib/design/`).
- **Controller**: one generic rule in `nav.ts`. An element that owns its keys (by the attributes Reka renders, never
  by component name) takes the arrows along its axis and Enter; a controller step is sent to it as that key; at its
  ends nav.ts moves spatially. Select, Dialog, Popover and DropdownMenu take B while open; a menu item owns Enter
  (its arrows stay nav.ts's, inside the menu's scope); entering a radio group lands on its chosen radio. Tested in Vitest
  (`components/ui/nav.test.ts`) and in a real browser (`scripts/headless/kit-nav.mjs`).
- **Kit page** (`app/src/kit/`, `#kit`, development only, not in the build): every primitive in every state, the
  three themes, the five accents. A new primitive or variant is added there first.
- **Where a file goes** (`app/src/`), by what owns it:
  - `components/ui/<name>/`: the primitives.
  - `components/<feature>/`: everything one feature is: its screen root (`LibraryView.vue`, `QuickMenu.vue`…), its
    parts, its composables (`useX.ts`), its pure modules (`x.ts`) and their tests. A component another feature reuses
    stays with the feature it belongs to and is imported from there (`settings/SettingRow.vue`, `cover/GameCover.vue`).
  - `composables/` (`useX`) and `lib/` (plain modules and helpers) only for what 2+ features, or the root for the
    whole app, use. A module only one feature uses moves into that feature's folder.
  - The root holds the app's wiring only: `main.ts`, `App.vue`, `emu.ts`, `nav.ts`, `styles.css`, `i18n/`,
    `assets/`, `kit/`.
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
- `docs/decisions.md`: the owner's tooling and product decisions with their reasons (GB/GBC support, Google Drive
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

- Core options: every `consoles.*` / `cores.*` entry with a real mGBA option is mapped (`settings/core_values/mgba.rs`);
  melonDS DS has a table (`core_values/melondsds.rs`); Azahar has one (`core_values/azahar.rs`: 3DS model, region,
  internal resolution, texture filter, CPU clock, audio engine, all applied when the game opens again; not mapped:
  `perf.shcache`, `screens.stereo`, `pad.gyro`, `bios.b3ds`, reasons in that file). `perf.frameskip` drives mGBA
  only, `video.res3d` / `video.texf` / `perf.cpuclock` Azahar only; elsewhere their rows are disabled with the reason
  (`CoreProfile::unsupported_settings`). The 3DS rows are phase 3, so the interface still hides them
  (`CURRENT_PHASE`). The interface does not show `emu://core-restart` yet. The «All … options»
  rows (`cores.*all`, still links to nothing) show the count of options the pinned build declares
  (`CoreProfile::declared_options`, measured with `headless --list-options`, held to `cores.lock`'s version by a
  test); Azahar's is not measured yet.
- 41 catalogue entries have no reader yet (`wired: false`, shown disabled with «arrives in a future update»; the
  list is `the_entries_not_wired_yet_are_these` in settings/tests.rs), and no action or preset card is carried out.
- DS / 3DS screens (Trello #28): layouts, swap, custom layout per console or game, rotation, lid, touch through
  the layout, both video paths, and their interface: the quick menu's swap and lid tiles, G09's layout card
  (stepping shows the layout at once and keeps it «in prova» as `screens.cycle` turned to start with it, the
  layout the game opens with), the C09 editor (`components/screen-editor`) and the G15 card. The `screens.*`
  rows are phase 2, so Settings › Schermi di DS e 3DS (and its `screens.custom` button) stays hidden while
  `CURRENT_PHASE` is 1: the editor opens from G09 only. The editor's «Salva» stores the layout at its scope and
  drops only its own preview; the other G09 changes «in prova» stay to be saved or undone. It draws the
  rectangles unturned: with `screens.rot` set, the game shows them turned as a whole. The G15 card's button is
  for the mouse and the keyboard: while the card shows the game keeps the controller, and the card names the
  lid's shortcut instead. The lid exists on melonDS DS only (its L2 + Y
  chord, pressed by the shell for 100 ms: the game sees Y in those frames, a paused game misses it, and the
  shell's «closed» is its own count, not read back from the core, so a state loaded across a lid change shows
  it the wrong way); Azahar has none (`pad.lid` unsupported on 3DS). The screen effects run once on the whole
  frame, so a curved CRT bends the frame as the core stacks it, not each screen. `screens.gap` is in the frame's
  pixels, so an upscaled frame (Azahar's internal resolution) gets a thinner gap. `pad.pointer`'s right-stick
  cursor has no backend (the mouse and touch go through `pointer`). No keyboard key for swap, layout or lid.
- Audio off 1× follows `audio.ff` for fast forward and slow motion alike (no separate slow-motion setting).
- Cores are pinned (`cores.lock`: commit, URL, sha256; Trello #71) and packages list only those files with their
  licences and `cores/SOURCES.md`. mGBA for Windows and Android is built by EmuDome from the pinned source archive
  of the commit the buildbot built (Trello #88, `scripts/build-mgba.ps1`): reproducible, but only with the toolchain
  `cores.lock` names (another one builds other bytes and is refused), so the built files are published in our GitHub
  release `cores-mgba-7a12d6d` (Trello #95, `built.release`) and every other machine downloads them from there
  (a new pin: `docs/release.md`). mGBA for Linux is still the libretro buildbot's `latest/` (`url_stable: false`):
  packaging refuses it unless `EMUDOME_ALLOW_UNSTABLE_CORES=1` (local test packages only) until it is built on Linux
  (Trello #92). Azahar is fetchable but not packaged (Phase 3). The Gradle side of the lock builds (2026-11-21,
  `scripts/android-build.ps1`: melonDS DS packaged, sha256-checked) but was not run on a device yet, nor was the
  source-built Android mGBA.
- Releases (`docs/release.md`, Trello #44 / #50): no `main` branch and no tag yet; `bundle.targets` is `["nsis"]`
  only, so Linux gets no AppImage / deb; the Android script builds APKs, not the App Bundle Google Play wants; no
  Windows code signing; the update channel is not decided.
- The game key migration (Trello #70) leaves in place, and reports, what facts cannot give to one game (a file name
  shared by several library games, no library game, the new place already taken); the app no longer reads those.
  The core's save directory is now the game's own folder, so melonDS DS's DLDI / DSi SD card images are per game
  (`saves/nds/<hex>/melonDS DS/`); an existing shared `saves/melonDS DS/` is left untouched and not read any more.
- Rewind is silent and has no on-screen sign; screenshots are the core's frame only (not "as shown").
- Core host (#78, #46): hardware rendering is Vulkan only, for Azahar (`Renderer::Vulkan` when the app's device
  is on Vulkan, else software with `hwRenderFallback` in `emu://health`); OpenGL (melonDS DS upscaling) is refused.
  Every Vulkan frame is read back into `Shared::frame` (the native renderer does not sample the GPU frame yet).
  Azahar 2126.2 crashes on a state load (and with its disk shader cache off) for content without a title ID
  (`.3dsx` homebrew: its pipeline cache is skipped for title 0). Guarded until Azahar fixes it (Trello #93):
  `core_profile::state_load_block` (Azahar + Vulkan + `n3ds::title_id` 0) makes the session refuse every load, the
  auto quick save's resume, «Annulla il caricamento» and rewind with `stateLoadUnsupported` (rewind records nothing),
  and `GameInfo.stateLoadBlock` has the quick menu's Carica and the resume question's «Riprendi» disabled with the
  reason; saving works. The issue text for Azahar: `docs/upstream/azahar-vulkan-state-load-title-id-0.md`. A core's log lines (`GET_LOG_INTERFACE`) are logged as their format
  string without the arguments (variadic C, no C shim); `GET_LANGUAGE` always answers English; `SET_MESSAGE` is
  logged, not shown; no sensors (plan in `docs/feasibility-hw-render.md`).
  Azahar's path limit (#94) is guarded for the NAND tail only: what it creates at load (`n3ds::SAVE_DEPTH`, measured
  on homebrew). The SD-card tail, the saves a retail game writes on the emulated SD card while it runs
  (`Azahar/sdmc/Nintendo 3DS/<32>/<32>/title/...`, names chosen by the game, deeper), is not covered: it cannot be
  measured without a retail game, and EmuDome never uses Nintendo material (#97). It matters only on Windows with the
  long-path policy off. To measure it in the owner's UAT, with the owner's own decrypted game in the installed app:
  play until the game has saved, close it, then in PowerShell, with `$g` the game's folder
  (`<data>\saves\3ds\<hex>`), `Get-ChildItem -LiteralPath $g -Recurse -Force | Sort-Object { $_.FullName.Length }
  -Descending | Select-Object -First 2 @{ n = 'tail'; e = { $_.FullName.Length - $g.Length } }, FullName`; the
  deepest folder's and file's tails become `SAVE_DEPTH`'s SD entries (with the game's names replaced by the longest
  the format allows). A game folder that cannot be created refuses the start (`saveFolderUnavailable`); a folder
  Azahar still cannot use makes it fall back to its own user folder (`%APPDATA%\Azahar` on Windows,
  `emudome_platform::azahar_user_dirs`), which the app sees only after the session, by comparing that folder's
  entries before and after (`n3ds::fallback`, `coreSavedOutsideGame` on `emu://health`, read only): a write there by
  another program (the standalone Azahar) during the session looks the same, and nothing is moved back.
  Cheats (L08) and patches (L09) have no backend yet. Screen effects run on the native video path only.

## Design and interface work

There is no separate Cowork session any more (owner, 2026-11-21). The design system (tokens, primitives on Reka, the
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
