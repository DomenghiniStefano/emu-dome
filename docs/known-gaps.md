# Known gaps of the prototype

Intentional, for later. Moved out of `CLAUDE.md` to keep the instruction files under their size limit; read this
before working on any area listed here, and update it in the same commit as a change that closes or opens a gap.

- Core options: every `consoles.*` / `cores.*` entry with a real mGBA option is mapped (`settings/core_values/mgba.rs`);
  melonDS DS has a table (`core_values/melondsds.rs`); Azahar has one (`core_values/azahar.rs`: 3DS model, region,
  internal resolution, texture filter, CPU clock, audio engine, JIT, disk shader cache, all applied when the game
  opens again; not mapped: `screens.stereo`, `pad.gyro`, `bios.b3ds`, reasons in that file). `perf.jit` drives
  melonDS DS and Azahar (Trello #99); `perf.shcache` Azahar only, forced on where turning it off crashes (Vulkan,
  title ID 0, #93: `StateLoadBlock::forced_options`, the row still shows the user's value). `perf.frameskip`
  drives mGBA only, `video.res3d` / `video.texf` / `perf.cpuclock` Azahar only; elsewhere their rows are disabled
  with the reason (`CoreProfile::unsupported_settings`). The game's clock (`saves.rtc`, Trello #102) drives melonDS
  DS only (`melonds_start_time_mode` real / relative / absolute, applied when the game opens again): «Shifted» reads
  `saves.rtcdays` / `rtchours` / `rtcminutes` (days past 14 round to the core's 30, 60… 364), «Fixed» reads
  `saves.rtcdate` (`Kind::DateTime`, local `YYYY-MM-DDTHH:MM`, 2000–2099); mGBA and Azahar declare no clock
  option, so the rows are disabled on GB / GBC / GBA / 3DS with the reason. The 3DS rows are phase 3, so the interface still hides them
  (`CURRENT_PHASE`). The interface does not show `emu://core-restart` yet. The «All … options»
  rows (`cores.*all`, still links to nothing) show the count of options the pinned build declares
  (`CoreProfile::declared_options`, measured with `headless --list-options`, held to `cores.lock`'s version by a
  test); Azahar's is not measured yet.
- 18 catalogue entries have no reader yet (`wired: false`, shown disabled with «arrives in a future update»; the
  list is `the_entries_not_wired_yet_are_these` in settings/tests.rs). The action rows carried out are `cloud.acct`,
  `cloud.space`, `about.ver`, `about.upd`, `about.lic` and `about.log`, and the rows that lead to where their
  feature lives (`settings/rowJumps.ts`, through the shell's `Jump` / `jumpTo` in `shell/route.ts`, provided by
  AppShell with `shell/useShellJump.ts`): `cloud.sgames` the Cloud section (its first tab is the games),
  `library.hidden` the library with L03's «Nascosti» on (in TV the desk's grid: the shelves leave hidden games out),
  `saves.import` the Saves import with no game chosen, `pad.dev1` / `pad.dev2` C02 at «Tutti i giochi» on the
  first controller / the keyboard (the same test); every other action row is disabled with that reason, and no
  preset card is carried out. C01's «Controlli touch» on a phone opens Settings › Controlli touch with the C06
  editor (its console chosen first, the `touch.tedit` row's own path); elsewhere it is a plain row with no chevron.
- Touch controls (Trello #29): layouts per console and orientation (defaults for GB / GBC, GBA, DS and 3DS, the
  user's own from C06 kept per device), tap / hold / turbo (the repeat is the interface's, about 12 a second: the
  core has no turbo of its own), vibration on Android, `touch.thide` / `touch.tpad`, two built-in styles from the
  tokens («Fumé», dark see-through glass, the default, and «Contorno», the outline alone; all
  colours opaque, each whole control faded by `opacity-pad` 60 % times `touch.topac` (default 100) times its own
  opacity; the old `glass` / `outlines` / `contrast` values read as `smoke` / `outline` / `smoke`, settings layout
  2), compact shapes (the d-pad one cross with arrows, A B X Y round in a 128 diamond, L / R shoulder-shaped, Select
  / Start small pills, the menu a small square), E14. A button's box is what takes the touch; L / R (84 × 36),
  Select / Start (60 × 32) and the menu (36) are under the 44 touch minimum by design and never shrink below their
  own size in C06 (`minSize`); the face buttons (48) and the d-pad (116, the whole box) are not. Not yet: imported skins (our pack format, `.deltaskin`, RetroArch overlays: Phase 5,
  `docs/decisions.md`), more than one control of the same kind (a second, turbo A), a layout per game (#135; the
  touch layout and the screens' placement are per console), the right stick of the 3DS (C-stick) and ZL / ZR. The `touch.*` rows are phase 4 except the style
  (`touch.tskin`), the vibration (`touch.tvib`, `touch.thapt`) and the editor (`touch.tedit`, #111), phase 1, so
  Settings › Controlli touch shows those rows while `CURRENT_PHASE` is 1 (on the phone; on the desktop disabled
  with «Solo su questo telefono.»), and the other rows keep their defaults (E14 «Pausa e domanda», 100 %). C06
  opens from the quick menu's «Controlli» and, on the phone, «Altro › Modifica la disposizione» (the open game's
  console), and from the `touch.tedit` row (at «Tutti i giochi» a dialog asks for the console first: GB, GBC,
  GBA, DS, 3DS). The game screens are elements of C06 like the controls (#134): editable frames (C09's
  `ScreenFrame`) moved, resized (size slider, «Mantieni le proporzioni» on by default) and undone like a control,
  never removed, opacity and behaviour disabled. From Settings there is no game: the elements are laid on the
  theme's flat ground (no dot grid, no screenshot), the screens as named frames starting where Rust would place
  them for the layout the game opens with (`screens_phone_start`: the cycle's first layout, not swapped, at 1×);
  with a game open the frames start where the game shows its screens now (the layout cycled to, swapped, the
  frame at its real size, `video.scale` applied; a layout showing one screen of two starts both as «auto» places
  them) and the game itself moves under the frame. C06 keeps one draft per orientation (turning the phone
  and back keeps the edits and their undo; a line suggests turning it until the other orientation was shown).
  «Salva» stores the layout and the screens' placement (`touch.tscreens`, fractions of the whole window, saved
  with `touch.tedit` in one write) of every orientation changed in the editor (else the one the phone is in), for
  that console, on this device only (never synced, by design: a phone's placement does not
  belong on other devices). In game the placement is placed over the whole window (the controls may overlap it)
  while the touch controls are in use, not in what they leave; with the quick menu open its screens, taken
  together, are fitted into the room left (never past their size). Each stored rectangle is the console's screen
  it names; a placement saved by an earlier build while the game showed its screens swapped holds them
  exchanged until it is saved again. Not
  yet: guides only to the window's centre lines (not to other elements' edges); the safe edge (camera, bars) is a
  dashed line, a screen may still go past it; the placement ignores `video.aspect` / `video.scale`; the reset of
  the `touch.tedit` row leaves `touch.tscreens` stored (used only with saved controls) until the editor's
  «Torna alla disposizione predefinita» + «Salva» writes it empty. Under `screens.rot` (phase 2, its row hidden
  today) a phone placement is turned as a custom layout is, each rectangle turned inside the window, while the
  editor draws it unturned: the editor and the game then differ. To be decided before phase 2 shows the row.
- Android controllers (Trello #47): built from Android's input events (`pads/android.rs`, `GamepadInput.kt`) but
  not run on a device yet. To verify there: which physical button a Nintendo-layout pad (Switch Pro, an 8BitDo in
  Switch mode) reports as `KEYCODE_BUTTON_A` (the table reads it as the bottom one, as SDL does); whether Guide /
  Home reaches the app (`KEYCODE_BUTTON_MODE`) or the system takes it; the right stick and trigger guess on pads
  without a key layout file of their own. The battery is read once, at connection (no `emu://gamepad-changed` on
  Android yet); no rumble, no gamepad motion sensors; a keyboard or a TV remote that has arrow keys only
  (`SOURCE_DPAD`) is not a controller and stays the system's.
  Not run on a device yet (vibration, the Circle Pad on Azahar). The game is fitted in what the controls leave
  (`reservedInset`, the video inset of `screens::Inset`): upright against the band at the bottom, the whole width
  by default (`video.scale` «fit» on a phone, #105: a GBA 1.5× on a 360 px phone; what the window has left over
  is one band at its top; «Pixel interi» chosen by the user floors the scale again), on the side between the two
  columns, Select / Start / menu in the columns; the quick menu's sheet and side panel keep their own edges the
  same way, measured again when the phone turns. DS and 3DS keep their screens large and apart
  instead (`isDualScreen`, `reservedInset`'s two-screen rule): upright the top screen under the camera (the safe top,
  read from the `safe-*` tokens), L / menu / R in a 36 px strip between the screens (anchored halfway between the
  safe top and the band, `defaultBand`), the touch screen on the band of the d-pad, A B X Y and Select / Start (in a
  column between the clusters, on every width); on its side the screens side by side at the top, 16 px from each
  edge or the camera's notch, 8 px apart, the controls over the strip below and the screens' lower outer corners,
  L / R small in the lower corners. On the S23 as it reports itself (360 × 780, camera 38): the DS band 166, the
  two screens 360 × 270 edge to edge (38–308, strip 308–344, 344–614), 359 × 269.25 on its side; the 3DS band 272,
  its top screen full width; on a 412 × 892 phone 412 × 309 each upright, 418 × 313.5 on its side. With the
  quick menu open the screens follow the menu's inset, which keeps no strip: they join, still filling their room.
  C06 opened over the game sends the inset of the layout being edited instead (`TouchEditor`'s `inset`, the menu's
  `MenuInset.Own`), so the game sits where it plays, again after each step of the edit.
  Native video's «Sfondo ai lati del gioco» cuts one hole per screen where Rust places it (`GameSurround.vue`
  asks `screens_placements` for the window's surface in physical pixels, `lib/surround.ts` `holesClipPath`; #105),
  so the room between two screens apart is painted too; screens that overlap (a custom layout) share one hole,
  their bounding box. It asks again on a resize, an inset, a new frame size or `emu://screens`; before the
  core's first frame it retries every 200 ms and draws nothing meanwhile.
- Regional versions (Trello #102): with `library.versions` on (default), Rust groups them
  (`library::group`, `Library::families`) and sends them with the library and as `library://changed` `families`
  (after a pin, a launch, a game hidden or shown, the switch); the pins are `versionPins` in `library.json`
  (`library_pin_version`). The interface indexes them once per change (`lib/families.ts`, `libraryFamilies`): the
  desk's grid and the TV shelves show one tile per family (a deck behind it, the square «N versioni» tag),
  «Continua a giocare» and the search the versions themselves, and the details' «Versioni» block lists them with
  «Gioca questa» / «Mostra questa in libreria». No region rule (`library.region` is gone). Game Boy / Game Boy
  Color are never grouped (the header's licensee and Japan / overseas flag are not read). On a TV shelf the
  focused cover's ring hides most of the deck behind it.
- Window, video and audio settings (Trello #100): `video.winsize` «free» is not remembered across starts (a start
  opens at 1280×800); `video.api` applies at the next start and nothing says a restart is waiting (no general
  restart notice; `emu://core-restart` is for core options), and its «Automatic ({api})» value is not shown;
  `general.bgpause` on the desktop follows the page's focus (a pause toggled with L3 + Start while EmuDome is in
  the background is not seen: coming back resumes); `audio.out` lists the devices when its row is shown, not live.
- DS / 3DS screens (Trello #28): layouts, swap, custom layout per console or game, rotation, lid, touch through
  the layout, both video paths, and their interface: the quick menu's swap and lid tiles, G09's layout card
  (stepping shows the layout at once and keeps it «in prova» as `screens.cycle` turned to start with it, the
  layout the game opens with), the C09 editor (`components/screen-editor`) and the G15 card. The `screens.*`
  rows are phase 2 except `screens.custom` (phase 1, #111), so Settings › Schermi di DS e 3DS shows that one row
  while `CURRENT_PHASE` is 1, on every device: its button opens C09 at the page's scope for DS / 3DS (a console
  or a game), and at «Tutti i giochi» or a one-screen console's scope a dialog asks for DS or 3DS first and saves
  for that console. With no game the screens are named placeholders over the dark ground (no screenshot behind
  them); nothing is «in prova». The editor's «Salva» stores the layout at its scope and
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
  The phone: `auto` («Automatica») is the first of the phone's cycle, so a phone's game opens
  with its screens apart and filling the width; `vertical` and `side` are left out of the phone's default cycle
  (`auto` is each of them in its orientation) and stay choosable in the cycle's setting. Every game on a phone,
  one screen or two, fills its room by default (`video.scale` «fit», #105); «Pixel interi» chosen by the user
  floors the scale there too. A cycle already saved by the user (a
  Console or Game value of `screens.cycle`) keeps its own first layout. The C09 editor's «Parti da» has no
  «Automatica» start (it starts from the four fixed layouts, composed at the device's `video.scale`).
- TV mode (#30): Saves, Controls, Cloud and a game's details keep the desk's layout scaled up in TV. Settings in TV
  has no «In primo piano» (T01), no search field (Y opens the search everywhere) and no «Mostra solo le impostazioni
  modificate»; a text and the game folders keep the desk's row scaled up; on a row that changes in place ◀ ▶ are
  the row's, so the pages are reached with B (or left from a row that does not take them). A section's own L1 / R1
  (the settings levels, the controls tabs) comes before the section switch; a settings page with one level lets
  it through. The
  library's banners (unreachable folders, the scan, «N giochi aggiunti») are not shown over the shelves.
- The phone (#29, the shell's half): the tab bar upright, the rail as a column on the camera's edge on its side
  (`screen.orientation`), Settings in one column upright (`settings/settingsPane.ts`; its scope card scrolls with
  the page instead of sticking, upright and on its side), Saves in one column upright and on its side (the
  games, then a game's panel or the import over them with «‹ Salvataggi», `saves/SavesView.vue`), a rail item or
  tab always opening its section at its root (the Library's grid even with details left open behind another
  section; the section shown, picked again, mounted again: `route.ts` `pickSection`, `AppShell.vue`), the quick menu
  (`quick-menu/PhoneHome.vue`, `PhoneMore.vue`, `PhoneSaves.vue`, `phoneMenu.ts`: a first
  level of «Salva rapido» / «Carica rapido» on Rust's current slot, «Riprendi» and a strip «Riavvolgi» (when
  `speed.rew` is on: held, the game unpaused meanwhile; A / Enter rewind for 1 s) / «Accelera» / «Silenzia» /
  «Chiudi» («Chiudi il gioco», danger ink, the close flow, #121) / «Altro» (on its side five go in two rows, three
  above and two below); «Altro» and «Altro › Salvataggi» growing the same sheet («Altro» also has «Modifica la disposizione»,
  C06 for the game's console, back onto that row when it closes, #111), no handle, a tap on the game resumes; a 300 px
  side panel on its side; «Rendi rapido» makes the chosen slot the game's quick slot with `set_slot`, the same
  slot the shortcuts move, kept per game in its saves folder: `emudome_core::quick_slot`), E13's warning (`storage_status`), Android's back at the shell's root. The library on the phone:
  no «Continua a giocare» and no heading, its own toolbar (`PhoneLibraryToolbar`:
  short console names on a sideways-scrolling row, the order's name, Lista / Griglia, a square «+»; one row on its
  side), the games as a list (the default; two columns of rows on its side, #105) or a compact grid as
  `library.view` says (per device: the list on a phone, the grid elsewhere; a desktop that picks the list in
  Settings gets it under «Continua a giocare», «Gioca» on every row), the game played last carrying «Riprendi» on
  its row or cover (`resumeTarget`). On the phone the shell has no cover-coloured haze (plain graphite; the
  desktop and TV keep it) and the top bar's search field on its side shows no Ctrl K (#105). The keyboard's keys
  (C01's «Tastiera» card, C03's fullscreen row, E05's button, G06's Esc, the search's F11) show on the phone only
  once a physical keyboard has been used in the run (#118, `lib/inputMode.ts` `keyboardSeen`; E05 then names the
  touch controls instead); G06 names Home there only with a controller connected, and with neither does not show
  (`game/quickMenuHint.ts`). The Settings descriptions that name keys (`video.fps`, `speed.ffmode`, `speed.rew`)
  are catalogue texts and still name them on the phone. Not yet: the «…»
  menu (favourite, hide) on a row or a compact tile (in the game's details only). The safe areas, the camera
  side and `moveTaskToBack` are checked headless and by review only, not on a device yet.
- «Gestisci lo spazio» (#29, S1–S3; `components/storage/StoragePage.vue`, `app/src-tauri/src/storage.rs`,
  `emudome_core::storage`): opened by E13's button (the shell's jump `manageSpace`) and by Settings › Salvataggi's
  `saves.space` row, in the page's place like the licences page, on every device. The space card (free / total, the
  bar of games / quick saves / EmuDome's other files / other apps), the games here from the heaviest, «Altri dati»
  (quick saves «Scegli», screenshots «Apri», temporary files «Svuota»). «Togli dal telefono» («… da questo
  dispositivo» off the phone) only for a game file in `<app local data>/games/` (a Drive download) that Drive has
  (asked of Drive before deleting; the button shows once `cloud_games` answered); the game stays in the library
  unavailable (faded) and Cloud lists it «Solo su Drive» for «Scarica». A game of the user's folders shows
  «In <cartella>» and, on the desktop, «Apri la cartella» (`emudome_platform::open_folder`; Android: no button).
  Every deletion asks first (whatever `general.confirmdel` says) and never touches `game.sav`, backups or a core's
  own files. Temporary files are the document links (`<cache>/content`, Android), a Drive download's leftover
  temporary file older than an hour, and Azahar's shader cache (`<game>/Azahar/shaders`, the folder name taken
  from Citra's layout, not seen on a device) of a 3DS game not open. Not yet: the screenshots gallery is per game
  with its own selection, not the details' look; sizes are walked on every open (no cache); the
  other apps' share counts games on another disk as ours.
- Audio off 1× follows `audio.ff` for fast forward and slow motion alike (no separate slow-motion setting).
- Search and notifications (#32): one palette opens from the shell and from the quick menu (Y, Ctrl + K; the
  hint in the menu's foot). In the shell the controller's shortcuts are disabled with «Apri un gioco per usarla»
  (only fullscreen runs) and a setting opens at «Tutti i giochi». In the quick menu the shortcuts run as the
  menu's own controls do (Salva / Carica and the slot steps on the menu's chosen slot, not Rust's current slot);
  the held ones (fast forward held, rewind) and the pause stay disabled with a reason; a setting opens only if a
  menu page shows it «in prova» (G09, G10, G16: seven rows), every other is disabled with «Aprila dalle
  Impostazioni della libreria»; another game closes this one as «Chiudi il gioco» does (G14, close flow) and
  then opens, with no question before (none designed). The page opened shows its first control, the row is not
  lit. The notifications recorded today are a controller connected after start and an update ready
  (`Notice::UpdateReady`, recorded by `updates.rs` once per version per run); sync done and cover not found have no
  producer and no `Notice` kind yet, and X06's install behaviour waits for the owner's answer and #44.
- Cores are pinned (`cores.lock`: commit, URL, sha256; Trello #71) and packages list only those files with their
  licences and `cores/SOURCES.md`. mGBA for Windows and Android is built by EmuDome from the pinned source archive
  of the commit the buildbot built (Trello #88, `scripts/build-mgba.ps1`): reproducible, but only with the toolchain
  `cores.lock` names (another one builds other bytes and is refused), so the built files are published in our GitHub
  release `cores-mgba-7a12d6d` (Trello #95, `built.release`) and every other machine downloads them from there
  (a new pin: `docs/release.md`). mGBA for Linux is still the libretro buildbot's `latest/` (`url_stable: false`):
  packaging refuses it unless `EMUDOME_ALLOW_UNSTABLE_CORES=1` (local test packages only) until it is built on Linux
  (Trello #92). melonDS DS for Android is built by EmuDome too (upstream's asset has 4 KB pages,
  refused by Google Play), from v1.4.0's source archive and the pinned archives of the libraries its CMake fetches
  (`build.dependencies`, configured with `FETCHCONTENT_FULLY_DISCONNECTED`), published as `cores-melondsds-f394adb`;
  its Windows and Linux files are still upstream's release assets. Azahar is fetchable but not packaged (Phase 3):
  a core this build does not ship for its platform (`cores.lock` `ship` off, or no build there;
  `emudome_platform::core_shipped`) and no folder holds is `CoreLookupError::NotShipped` in a release and on
  Android, and opening its game says `coreUnavailable` («Nintendo 3DS isn't available on this device yet»), not a
  reinstall; a desktop debug build without the file keeps `coreNotFound` (the fetch hint). The library still lists
  those games with «Gioca» enabled (showing it disabled with the reason needs a design). The Gradle side of the lock builds
  (`scripts/android-build.ps1`: melonDS DS packaged, sha256-checked) but was not run on a device yet, nor was the
  source-built Android mGBA. 16 KB pages (Google Play, `docs/release.md` «Android: 16 KB pages»): the APK stores
  its libraries 16 KB-aligned, `libemudome_app.so` links with 16 KB pages (`.cargo/config.toml`), mGBA and the
  NDK's `libc++_shared.so` are aligned, and melonDS DS for Android is now EmuDome's own build of the
  pinned v1.4.0 source (`cores-melondsds-f394adb`, 16 KB-aligned).
- Releases (`docs/release.md`, Trello #44 / #50): no `main` branch and no tag yet, no release built. Windows: a
  per-user NSIS installer, data kept on uninstall unless the uninstaller's box is ticked, unsigned (SmartScreen).
  Linux: AppImage and deb targets, never built (no Linux here; mGBA for Linux unpinned, #92). Android: `-Bundle`
  builds the AAB, run only as `-DryRun`. Updates: Tauri's updater from GitHub Releases' `latest.json`, signed with
  the owner's key (made outside the repo); a build without `EMUDOME_UPDATER_PUBKEY` cannot update itself. Its TLS
  is the system's (`native-tls`), not rustls, until the owner decides on `webpki-root-certs`
  (CDLA-Permissive-2.0). The licences page and X06 are functional placeholders for the owner's design; the
  licence texts are read from the package (a development build reads the repo's files; Android has none yet).
  `about.log` is desktop only (no save dialog on Android).
- The game key migration (Trello #70) leaves in place, and reports, what facts cannot give to one game (a file name
  shared by several library games, no library game, the new place already taken); the app no longer reads those.
  The core's save directory is now the game's own folder, so melonDS DS's DLDI / DSi SD card images are per game
  (`saves/nds/<hex>/melonDS DS/`); an existing shared `saves/melonDS DS/` is left untouched and not read any more.
- Rewind is silent and has no on-screen sign; screenshots are the core's frame only (not "as shown": on DS / 3DS
  both screens as the core stacks them, whatever layout is on screen), and the user frames that in «Adatta la
  copertina» when one becomes a cover (#85).
- Covers from a screenshot or an image (#85): the screenshot toast's «Usa come copertina» is a button for the mouse
  and touch; in game the controller has no way to press it (the cover is still reachable from L12 «Da uno
  screenshot»). A pressed one opens «Adatta la copertina» with the quick menu or once the game has closed. Zoom on
  the controller is L1 / R1 (LT / RT reach the interface as no step); no pinch zoom on touch (the slider). The
  picked image is kept in memory until used (one at a time). Covers are not synced
  (#31). The old screenshots the move could not give to one game stay in `saves/screenshots/` (listed in
  `migration-85/report.json`, not shown in the interface).
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
- Google Drive sync (Trello #20 backend, #31 wiring; `crates/emudome-sync`, `app/src-tauri/src/cloud.rs`): tested on
  `FakeDrive` and a local mock server, never against Google by the checks (`drive_probe` and the app's real sign-in
  are the owner's UAT). Wired: pull before launch (in the open, before anything reads the saves; «Gioca senza
  aspettare» stops it before its next file, a file already moving finishes), the «in use» marker and its heartbeat
  while the game runs, push after exit (background, after the exit writes; the close never waits; a game is pending
  in `sync/state.json` from its open until a push goes through, so a cut push, a quit or a crash is redone at the
  next start), at start the pending pushes, the settings and the library list, «Sincronizza ora» (every game's
  saves both ways, not the open game's), game files up / down / to Drive's trash on request only, devices, G04's
  answers. `cloud.ssaves`, `cloud.squick`, `cloud.ssettings`, `cloud.sauto` are read; `cloud.swifi` is not (nothing
  tells Wi-Fi from mobile data yet); `cloud.acct`, `cloud.space`, `cloud.sgames` are the account, space and games
  rows. Not yet: a periodic push while playing (only the marker beats);
  pulled settings reach the open screens only when the settings store reads the file again (no `settings://changed`
  for them); the activity list lives in memory (empty after a restart); a game file going up reports only its start
  and end (no per-chunk progress), and a resumable upload resumes within its call, not after a restart; the
  download has no free-disk check before it starts; G04's `thumb` is always null; `games()` lists every app file
  in one `list_all` and downloads each device's library list (one request per device). Deletions never travel (a
  file missing on one side is copied back). The side background images (`<app data>/backgrounds/`) are not synced,
  so `video.surroundImg` is per device; a name whose file is missing draws the built-in background (`GameSurround.vue`)
  and is logged. `changes.list` is implemented but no run uses it yet. Android: sign-in
  refused (`cloudSignInUnsupported`; plan in `emudome-platform/src/cloud.rs`), so nothing syncs there, and no TLS
  roots there (`emudome-sync/src/http.rs`). Linux: the Secret Service backend is not compiled or tried on this
  machine. The browser page after the sign-in shows the texts `cloud_sign_in` receives from the interface.
- Cloud interface (Trello #31, `components/cloud`): the games only on Drive are a block «Solo su Google Drive»
  under the library's grid (and under L02), not tiles of the windowed grid, and «Scarica e gioca» lives there,
  not in the game details (a Drive-only game has no `LibraryGame`; open question for the owner on the card); the
  TV shelves do not show them. After «Scarica e gioca» the game opens once `library://changed` lists it (20 s at
  most, else a notice). G04 shows outside the game only (its answer writes the saves the core holds); a conflict
  met while a game starts waits in the queue and in the status card. G04's versions have no picture (`thumb` is
  null). «Cosa sincronizzare» edits at «Tutti i giochi» only (`cloud.squick` per console stays in Settings ›
  Cloud). A failed `cloud_launch_check` never blocks an open. The first run's S06 «Accedi con Google» signs in the
  same way (`useCloudAccount`, `first-run/DriveRow.vue`), disabled with `cloudNotConfigured` /
  `cloudSignInUnsupported` where it cannot; the first run finishes without it.
- Android folders and files (Trello #47): not run on a device by the checks (the owner's UAT on the S23). A 3DS game
  (Azahar, `need_fullpath`) from a document whose descriptor cannot be opened again by path is refused
  (`documentNeedsPath`): the fix is serving it through libretro's VFS interface (the host has none yet), not a copy
  of GBs. `save_file` (exports) and the path pickers of screen effect presets, BIOS files and the saves import
  folder have no Android picker yet (`ACTION_CREATE_DOCUMENT` would write through a descriptor, not
  `write_atomic`'s rename); an image for a cover or the background has one (`emudome_platform::pick_image`). A tree game's details (L10) show its document URI as the path, and the saves import pairs
  saves with ROMs by file stem, which a URI does not have. Single files picked with «Apri un file» keep their
  persisted permission for good (Android caps an app's persisted grants). Providers that load their listing
  asynchronously (cloud folders, `EXTRA_LOADING`) are listed as they first answer.

