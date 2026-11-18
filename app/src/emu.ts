// Typed bridge to the Rust commands. The interface goes through here only.
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWebview } from '@tauri-apps/api/webview'

export type Speed = { kind: 'factor'; value: number } | { kind: 'max' }
export type Button =
  | 'a'
  | 'b'
  | 'x'
  | 'y'
  | 'l'
  | 'r'
  | 'l2'
  | 'r2'
  | 'l3'
  | 'r3'
  | 'start'
  | 'select'
  | 'up'
  | 'down'
  | 'left'
  | 'right'

export interface GameInfo {
  title: string
  console: string
  consoleId: ConsoleId
  core: string
  /** `saves.resume` is «ask» and an auto quick save exists: ask whether to resume. */
  resumable: boolean
  /** `saves.autoexit`: closing writes the auto quick save. */
  autosaveOnExit: boolean
}
export interface Stats {
  running: boolean
  emuFps: number
  targetFps: number
  speed: Speed | null
  paused: boolean
  /** Frames emulated since the start (hang watchdog, E11). */
  frames: number
  /** Unix ms of the last auto quick save in this session. */
  autoSavedAt: number | null
}

/** A quick save slot of the open game (G07). Mirror of `SlotDto` in src-tauri/src/lib.rs. */
export interface SlotInfo {
  slot: number
  /** Unix ms of the last save; null = empty. */
  savedAt: number | null
  thumbUrl: string | null
}

/** Slot 0: the auto quick save («Salvataggio rapido automatico»). */
export const AUTO_SLOT = 0

/**
 * Mirror of `emudome_core::ErrorCode` (crates/emudome-core/src/error.rs): what every
 * command rejects with and what `error` events carry. Turned into text by `errorText`
 * (src/i18n/labels.ts) through `errors.<code>`. `detail` is technical text, not a sentence.
 */
export type ErrorCode =
  | { code: 'unsupportedFormat'; params: { ext: string } }
  | { code: 'consoleNotYet'; params: { console: string; phase: number } }
  | { code: 'coreNotFound'; params: { file: string } }
  | { code: 'openFailed'; params: { detail: string } }
  | { code: 'noGame' }
  | { code: 'nativeVideoUnavailable' }
  | { code: 'webviewBackgroundFailed'; params: { detail: string } }
  | { code: 'slotEmpty'; params: { slot: number } }
  | { code: 'stateSaveFailed'; params: { slot: number; detail: string } }
  | { code: 'stateLoadFailed'; params: { slot: number; detail: string } }
  | { code: 'stateInvalid'; params: { slot: number } }
  | { code: 'settingUnknown'; params: { key: string } }
  | {
      code: 'settingLevelRefused'
      /** `levels`: where the setting can be stored, weakest first. */
      params: { key: string; level: SettingLevel; levels: SettingLevel[] }
    }
  | { code: 'settingInvalidValue'; params: { key: string } }
  | { code: 'settingsWriteFailed'; params: { detail: string } }
  | { code: 'libraryFolderUnreachable'; params: { path: string } }
  | { code: 'libraryGameUnknown'; params: { id: string } }
  | { code: 'libraryGameUnavailable'; params: { id: string } }
  | { code: 'libraryWriteFailed'; params: { detail: string } }
  | { code: 'coverInvalid'; params: { detail: string } }
  | { code: 'nothingToUndo' }
  | { code: 'undoFailed'; params: { detail: string } }
  | { code: 'saveUnreadable'; params: { aside: string; backupAt: number | null } }
  | { code: 'saveGameOpen' }
  | { code: 'saveImportInvalid'; params: { detail: string } }
  | { code: 'saveFileFailed'; params: { detail: string } }
  | { code: 'screenshotFailed'; params: { detail: string } }
  | { code: 'effectNotLoaded'; params: { name: string; detail: string } }

export type EmuEvent =
  | { type: 'started'; core: string; game: string; fps: number; width: number; height: number }
  | { type: 'stateSaved'; slot: number }
  | { type: 'stateLoaded'; slot: number }
  | { type: 'loadUndone' }
  /** «Salvato nello slot N · Annulla»: `slot` holds what it held before the last save. */
  | { type: 'saveUndone'; slot: number }
  /** G13 «Schermata salvata»: `path` is the PNG written next to the saves. */
  | { type: 'screenshotSaved'; path: string }
  | { type: 'error'; error: ErrorCode }
  | { type: 'stopped' }

// ---- Controllers (mirrors of emudome_core::bindings and src-tauri/src/pads.rs) ----

/**
 * A place on the controller, by position (SDL's names), the same on every controller whatever
 * is printed on it. Mirror of `Pos`. Names per controller type: `controls.pos.<family>.<pos>`.
 */
export type Pos =
  | 'south'
  | 'east'
  | 'west'
  | 'north'
  | 'l1'
  | 'r1'
  | 'l2'
  | 'r2'
  | 'l3'
  | 'r3'
  | 'back'
  | 'start'
  | 'up'
  | 'down'
  | 'left'
  | 'right'

/** Controller type: one remap table each, plus the keyboard. Mirror of `Family`. */
export type PadFamily = 'xbox' | 'playstation' | 'switch' | 'generic' | 'keyboard'

/**
 * A connected controller: payload of `emu://gamepad-connected` / `emu://gamepad-disconnected`
 * and the rows of `gamepad_list`. `name` is null when SDL knows none.
 */
export interface PadInfo {
  id: number
  name: string | null
  family: Exclude<PadFamily, 'keyboard'>
  vendor: number
  product: number
  /** Battery charge left, 0..100; null when wired or unknown (`emu://gamepad-changed` updates it). */
  battery: number | null
}
/** Kept for the toasts, which only need the name. */
export type GamepadEvent = Pick<PadInfo, 'name'>

/** One pad's live state (`emu://pad-state`, while watched). */
export interface PadState {
  id: number
  /** Positions held: buttons, and triggers past `pad.trig`. */
  held: Pos[]
  /** Left x, left y, right x, right y (-32768..32767, y down), left trigger, right trigger. */
  axes: [number, number, number, number, number, number]
}

/** One game button of the remap table. `source`: a `Pos` (controllers) or a key code (keyboard). */
export interface RemapRow {
  button: Button
  source: string | null
  default: string | null
  changed: boolean
}

/** One controller shortcut: `pos` pressed with the shortcut button (null = none). */
export interface ComboRow {
  hotkey: HotkeyId
  pos: Pos | null
  default: Pos | null
  /** Whether it works yet: reserved rows are listed and do nothing. */
  status: HotkeyStatus
  /** Listed among «Scorciatoie da ricordare» (C01). */
  toRemember: boolean
}

/** Mirror of `Hotkey::id` (crates/emudome-core/src/hotkeys.rs), in `Hotkey::ALL` order. */
export type HotkeyId =
  | 'menu'
  | 'save'
  | 'load'
  | 'next_slot'
  | 'prev_slot'
  | 'ff_hold'
  | 'ff_toggle'
  | 'slow'
  | 'rewind'
  | 'pause'
  | 'screenshot'
  | 'fps'
  | 'swap'
  | 'layout'
  | 'lid'

/** Mirror of `hotkeys::Status`: works, comes later, comes with the Nintendo DS and 3DS. */
export type HotkeyStatus = 'active' | 'later' | 'dualScreen'

/** The bindings of a family seen from a scope, with the defaults already applied. */
export interface ControlsBindings {
  remap: RemapRow[]
  combos: ComboRow[]
  /** The shortcut button (`pad.mod`). */
  modifier: Pos
}

export type VideoMode = 'native' | 'webview'

/** Interface theme (mirror of `Theme` in src-tauri/src/theme.rs): Scuro, Nero, Chiaro. */
export type Theme = 'dark' | 'black' | 'light'

/** One focus step (`emu://nav`, mirror of `emudome_core::hotkeys::Nav`), also produced by the keyboard. */
/** `prev` / `next` are L1 / R1: tabs or levels of the screen. */
/** `more` is X: the focused item's other actions (as a right click). */
export type NavAction =
  'up' | 'down' | 'left' | 'right' | 'confirm' | 'back' | 'prev' | 'next' | 'more'

/**
 * Payload of `emu://hotkey` (mirror of `HotkeyEvent` in src-tauri/src/gamepad.rs): what a
 * controller combo changed that the interface shows. The combos themselves run in Rust.
 */
export type HotkeyEvent =
  | { kind: 'fps' }
  | { kind: 'slot'; slot: number }
  | { kind: 'speed'; speed: Speed }
  | { kind: 'turbo'; held: boolean }
  | { kind: 'paused'; paused: boolean }

// ---- Settings (mirrors of crates/emudome-core/src/settings, schema.rs and mod.rs) ----

/** Where a value comes from, weakest first: default, «Tutti i giochi», «Giochi <console>», «Solo questo gioco», «in prova». */
export type SettingLevel = 'default' | 'global' | 'console' | 'game' | 'session'
export type ConsoleId = 'gb' | 'gbc' | 'gba' | 'nds' | '3ds'
export type SettingDevice = 'desktop' | 'phone'
/** `touch`: only where there is a touch screen (for now the phone). */
export type SettingPlatform = 'all' | SettingDevice | 'touch'
/** bool (toggle), number (slider), string (choice, file, colour, text), string[] (multi, folders). */
export type SettingValue = boolean | number | string | string[]

/** Kind of an entry. Texts: `settings.<key>.name|desc`, options `settings.<key>.option.<id>`. */
export type SettingKind =
  | { type: 'toggle' }
  | {
      type: 'choice'
      style: 'segments' | 'list' | 'cards'
      options: string[]
      /** The device adds options after these (audio outputs, saved touch layouts). */
      moreAtRuntime: boolean
    }
  | { type: 'multi'; options: string[] }
  | {
      type: 'slider'
      min: number
      max: number
      step: number
      unit: 'percent' | 'px' | 'seconds' | 'copies'
    }
  | { type: 'file' }
  | { type: 'folders' }
  | { type: 'colour'; swatches: string[] }
  | { type: 'text' }
  | { type: 'action'; style: 'button' | 'link' | 'info' | 'storage' }
  /** No value: cards that each apply a bundle of settings after a confirmation (`presets.preset`). */
  | { type: 'apply'; options: string[] }
  /** A binding list edited in the Controls section (`pad.remap`, `pad.combos`): a link here. */
  | { type: 'bindings'; table: 'remap' | 'combos' }

/** When a row is shown (mirror of `ShowIf`, schema.rs): entry `key` of the same page, resolved at
 * the level being edited, is (with `not`: is not) one of `values`. */
export interface SettingShowIf {
  key: string
  values: string[]
  not: boolean
}

export interface SettingEntry {
  /** `<prefix>.<key>`, stable. */
  key: string
  /** The page it is shown on: usually the key's prefix, not always («Avanzate»). */
  page: string
  kind: SettingKind
  /** null for actions. */
  default: SettingValue | null
  /** Accepted levels, weakest first. */
  levels: SettingLevel[]
  perDevice: boolean
  platform: SettingPlatform
  phase: number
  /** Options of a choice this device does not offer: not shown. */
  hiddenOptions: string[]
  /** Extra words the search matches, in any language. */
  synonyms: string[]
  /** Listed in «Usate più spesso» (T01), at the Global level. */
  mostUsed: boolean
  /** Shown only while another row has (or has not) some values; absent: always. */
  showIf?: SettingShowIf
  /** Works only on the Accelerated (native) video method: disabled with the reason otherwise. */
  needsNativeVideo: boolean
}

/** A heading of a settings page (`settings.sections.<page>.<id>`) and its entries in order. */
export interface SettingSection {
  page: string
  id: string
  keys: string[]
}

export interface SettingsSchema {
  device: SettingDevice
  /** Entries with a later `phase`, or whose `platform` excludes `device`, are not shown. */
  currentPhase: number
  pages: string[]
  /** The category list's groups (`settings.groups.<id>`), in order, with their pages. */
  groups: { id: string; pages: string[] }[]
  /** Every page's headings in order, each with its entries. */
  sections: SettingSection[]
  entries: SettingEntry[]
}

/** What a page shows or a change targets. `session` = the open game with its changes «in prova». */
export type SettingsScope =
  | { level: 'global' }
  | { level: 'console'; console: ConsoleId }
  | { level: 'game'; console: ConsoleId; game: string }
  | { level: 'session' }

export interface SettingRow {
  key: string
  value: SettingValue | null
  /** Weaker than the page's scope → «Come in "…"»; equal → the row can be reset. */
  source: SettingLevel
  /** Differs from the default. */
  changed: boolean
}

/** A value changed recently at one level («Modificate di recente», T01). Mirror: `RecentChange`. */
export interface SettingRecent {
  key: string
  level: 'global' | 'console' | 'game'
  /** The console of a `console` level. */
  console: ConsoleId | null
  /** The game of a `game` level (settings identity, not the library's). */
  game: string | null
  /** Unix ms. */
  at: number
}

/** A value stored for one game, with what its console's games get (L06). Mirror: `GameOverride`. */
export interface GameOverride {
  key: string
  /** The settings page it is shown on. */
  page: string
  value: SettingValue
  /** «Per tutti i giochi <console>: …». */
  base: SettingValue | null
}

/** Payload of `settings://changed`. */
export interface SettingsChanged {
  key: string
  level: SettingLevel
}

// ---- Library (mirrors of crates/emudome-core/src/library and src-tauri/src/library.rs) ----

/**
 * Identity of a game, from its content (docs/feasibility.md "Game identity"), never the file
 * name: `<console>:<hex>` (SHA-1 of the file for GB/GBC/GBA, of the used size for NDS; title
 * id + version for 3DS). Opaque to the interface.
 */
export type GameId = string

/** Which picture a cover shows (L12): the user's image, the icon inside the ROM, or a generated one. */
export type CoverKind = 'user' | 'icon' | 'generated'

/** Where a game was sold, from its game code (`details.region.<id>`). Mirror: `Region`. */
export type GameRegion =
  | 'japan'
  | 'usa'
  | 'europe'
  | 'germany'
  | 'france'
  | 'italy'
  | 'spain'
  | 'netherlands'
  | 'australia'
  | 'korea'
  | 'china'
  | 'world'

export interface LibraryGame {
  id: GameId
  console: ConsoleId
  /** From the header / NDS banner / 3DS SMDH, or the user's override. Never the id. */
  title: string
  path: string
  /** The library folder it was found in; null when added as a single file (drag and drop). */
  folder: string | null
  size: number
  /** false when the file or its folder is not reachable now (E01): shown faded, not removed. */
  available: boolean
  playTimeSecs: number
  /** Unix ms, null if never played. */
  lastPlayed: number | null
  /** Unix ms. */
  addedAt: number
  cover: CoverKind
  /** What `cover` shows, loadable by the webview (`emu://cover/...`); null for `generated`. */
  coverUrl: string | null
  /** True when the ROM has an icon (NDS banner, 3DS SMDH): L12 offers it. */
  hasIcon: boolean
  /** Background colour of the generated cover, `#rrggbb`, stable per game. */
  colour: string
  favourite: boolean
  hidden: boolean
  /** Thumbnail of the newest save state for the «Riprendi» hero; null while states have none. */
  lastStateThumbUrl: string | null
  /** `title` is the user's «Nome mostrato» (L10). */
  renamed: boolean
  /** The ROM's own title, what «Nome mostrato» goes back to. */
  romTitle: string
  /** Game code from the header (L10). */
  code: string | null
  /** ROM version from the header. */
  version: number | null
  region: GameRegion | null
  /** The core that runs it: `mgba`, `melondsds`, `azahar` (`details.files.core.<id>`). */
  core: string
  /** Its «Solo questo gioco» settings identity; null when the id cannot name a file. */
  settingsGame: string | null
}

export interface LibraryFolder {
  path: string
  reachable: boolean
}

export interface Library {
  games: LibraryGame[]
  /** `library.folders` of this device, with whether each is reachable now. */
  folders: LibraryFolder[]
  scanning: boolean
}

/** Payload of `library://scan`. `added` at `done` drives L11 «N giochi aggiunti». */
export type LibraryScanEvent =
  | { phase: 'started' }
  | { phase: 'progress'; checked: number; found: number }
  | { phase: 'done'; found: number; added: number; unreachable: string[] }

// ---- Saves (mirrors of src-tauri/src/saves.rs; games named by their ROM path) ----

/** V01: what a game has saved. Mirror: `SummaryDto`. */
export interface SavesSummary {
  path: string
  inGame: boolean
  quickCount: number
  /** In-game save, quick saves and thumbnails. */
  bytes: number
  /** Unix ms of the newest change. */
  lastChange: number | null
  /** An unreadable in-game save was set aside (E04). */
  broken: boolean
}

export interface SaveFile {
  size: number
  modified: number | null
}

/** A quick save that exists; slot 0 is the auto quick save. */
export interface QuickSave {
  slot: number
  size: number
  savedAt: number | null
  thumbUrl: string | null
  /** Made by an older version of its core: «Potrebbe non caricarsi». */
  outdated: boolean
}

export type BackupReason = 'launch' | 'import' | 'restore'

export interface SaveBackup {
  /** What `restore` takes. */
  name: string
  /** Unix ms. */
  at: number
  reason: BackupReason
  size: number
}

/** V02. Mirror: `GameSavesDto`. */
export interface GameSaves {
  inGame: SaveFile | null
  quick: QuickSave[]
  /** Newest first. */
  backups: SaveBackup[]
  /** In-game saves set aside as unreadable (E04), newest first. */
  broken: string[]
}

export type ImportFormat = 'raw' | 'desmume' | 'actionReplay'

/** V05: the chosen file, read and recognised. */
export interface ImportCandidate {
  path: string
  fileName: string
  format: ImportFormat
  /** Bytes of the save inside the file. */
  size: number
}

/** First run S06: which files a folder import reads. Mirror: `FolderImport` (emudome_core::saves). */
export type FolderImport = 'desmume' | 'any'

/** A found save paired with its game (ROM paths). Mirror: `MatchDto` in src-tauri/src/saves.rs. */
export interface SaveMatch {
  file: string
  game: string
}

/** What a chosen folder offers. Mirror: `FolderSavesDto`. */
export interface FolderSaves {
  folder: string
  /** Saves found that can be imported. */
  found: number
  /** The ones that belong to a library game, one per game. */
  matches: SaveMatch[]
}

/** A step of the first run (mirror of `emudome_core::first_run::Step`): S01, S02, S04, S06. */
export type FirstRunStep = 'welcome' | 'games' | 'controller' | 'ready'

export type SaveExport = { kind: 'inGame' } | { kind: 'quick'; slot: number }

/**
 * One round of the canvas loop in bench mode (mirror of `WebviewFrame` in
 * src-tauri/src/bench.rs): `t` in ms since the Unix epoch, `seq` the game frame on the canvas.
 */
export interface BenchFrame {
  t: number
  seq: number
  drawn: boolean
}

/** URL of the `emu://` protocol (on Windows it becomes http://emu.localhost/...). */
export const frameUrl = convertFileSrc('frame', 'emu')
/** URL of an image the user chose as background (`video.surroundImg` holds its file name). */
export const backgroundUrl = (name: string) =>
  `${convertFileSrc('background', 'emu')}/${encodeURIComponent(name)}`

export const emu = {
  startupGame: () => invoke<string | null>('startup_game'),
  videoMode: () => invoke<[VideoMode, boolean]>('video_mode'),
  setVideoMode: (mode: VideoMode) => invoke<void>('set_video_mode', { mode }),
  openGame: (path: string) => invoke<GameInfo>('open_game', { path }),
  closeGame: () => invoke<void>('close_game'),
  /** E11: let go of a core that stopped answering (does not wait for it). */
  abandonGame: () => invoke<void>('abandon_game'),
  /** The auto quick save (slot 0) and slots 1..count of the open game. */
  listSlots: (count: number) => invoke<SlotInfo[]>('list_slots', { count }),
  setSpeed: (speed: Speed) => invoke<void>('set_speed', { speed }),
  setPaused: (paused: boolean) => invoke<void>('set_paused', { paused }),
  /** While the menu is open the controllers navigate it (`emu://nav`) instead of playing. */
  setMenuOpen: (open: boolean) => invoke<void>('set_menu_open', { open }),
  /** F11 and «A schermo intero»: fullscreen on / off; resolves whether it is now on. */
  toggleFullscreen: () => invoke<boolean>('toggle_fullscreen'),
  isFullscreen: () => invoke<boolean>('is_fullscreen'),
  /** G09: the bottom `fraction` of the window is the menu's bar; the native game shrinks above it. */
  setVideoInset: (fraction: number) => invoke<void>('set_video_inset', { fraction }),
  /** «Muto» in the quick menu: silence for the open game only. */
  setMuted: (muted: boolean) => invoke<void>('set_muted', { muted }),
  saveState: (slot: number) => invoke<void>('save_state', { slot }),
  loadState: (slot: number) => invoke<void>('load_state', { slot }),
  reset: () => invoke<void>('reset_game'),
  /** Backspace / L3 + L2: rewind on (held, or toggled with `a11y.toggle`) or off. */
  rewind: (on: boolean) => invoke<void>('rewind', { on }),
  /** F12, L3 + B, quick menu › Strumenti: PNG next to the saves (`screenshotSaved`). */
  screenshot: () => invoke<void>('screenshot'),
  /** «Annulla il caricamento» (`saves.undo`): back to before the last load (`loadUndone`). */
  undoLoad: () => invoke<void>('undo_load'),
  /** «Salvato nello slot N · Annulla»: the slot gets back what it held (`saveUndone`). */
  undoSave: () => invoke<void>('undo_save'),
  input: (button: Button, pressed: boolean) => invoke<void>('input', { button, pressed }),
  stats: () => invoke<Stats>('stats'),
  /** Bench mode (`EMUDOME_BENCH`): the canvas reports its rounds with `benchFrames`. */
  benchActive: () => invoke<boolean>('bench_active'),
  benchFrames: (frames: BenchFrame[]) => invoke<void>('bench_frames', { frames }),
  /** The theme to show: `EMUDOME_THEME` if set, else `general.theme`, else the system's. */
  theme: (systemDark: boolean) => invoke<Theme>('theme', { systemDark }),
  settings: {
    schema: () => invoke<SettingsSchema>('settings_schema'),
    page: (page: string, scope: SettingsScope) =>
      invoke<SettingRow[]>('settings_page', { page, scope }),
    /** The `limit` values changed most recently, at any level, newest first. */
    recent: (limit: number) => invoke<SettingRecent[]>('settings_recent', { limit }),
    /** L06: the values stored for one game (a `game` scope), with the console's. */
    gameOverrides: (scope: SettingsScope) =>
      invoke<GameOverride[]>('settings_game_overrides', { scope }),
    /** The open game's scope (null without one). */
    gameScope: () => invoke<SettingsScope | null>('settings_game_scope'),
    set: (key: string, scope: SettingsScope, value: SettingValue) =>
      invoke<void>('settings_set', { key, scope, value }),
    reset: (key: string, scope: SettingsScope) => invoke<void>('settings_reset', { key, scope }),
    /** Saves the changes «in prova» for this game or its console (all or nothing). */
    commitSession: (target: 'game' | 'console') =>
      invoke<void>('settings_session_commit', { target }),
    discardSession: () => invoke<void>('settings_session_discard'),
    /** «Scegli un’immagine…» of `video.surroundImg`: an image picker, the image copied into the
     * app's backgrounds and stored at `scope`. Resolves false if the picker was cancelled. */
    chooseBackground: (scope: SettingsScope) =>
      invoke<boolean>('settings_choose_background', { scope }),
    onChanged: (cb: (e: SettingsChanged) => void): Promise<UnlistenFn> =>
      listen<SettingsChanged>('settings://changed', e => cb(e.payload)),
  },
  /** EmuDome's version (tauri.conf.json). */
  appVersion: () => getVersion(),
  library: {
    get: () => invoke<Library>('library_get'),
    /** Native folder picker; the folder joins `library.folders` and is scanned. null = cancelled. */
    chooseFolder: () => invoke<string | null>('library_choose_folder'),
    removeFolder: (path: string) => invoke<void>('library_remove_folder', { path }),
    /** Drag and drop: folders join `library.folders`, files are added one by one. Then a scan. */
    addPaths: (paths: string[]) => invoke<void>('library_add_paths', { paths }),
    /** Rescans every folder in the background; progress on `library://scan`. */
    scan: () => invoke<void>('library_scan'),
    /** Opens the game (like `openGame`) and counts its play time until it closes. */
    open: (id: GameId) => invoke<GameInfo>('library_open', { id }),
    /** L12. `user` opens an image picker first; resolves false if it was cancelled. */
    setCover: (id: GameId, kind: CoverKind) => invoke<boolean>('library_set_cover', { id, kind }),
    setFlags: (id: GameId, flags: { favourite?: boolean; hidden?: boolean }) =>
      invoke<void>('library_set_flags', { id, ...flags }),
    /** «Nome mostrato» (L10); blank goes back to the ROM's title. */
    setName: (id: GameId, name: string) => invoke<void>('library_set_name', { id, name }),
    onScan: (cb: (e: LibraryScanEvent) => void): Promise<UnlistenFn> =>
      listen<LibraryScanEvent>('library://scan', e => cb(e.payload)),
    /** Something in the library changed (scan, cover, flags, play time): call `get` again. */
    onChanged: (cb: () => void): Promise<UnlistenFn> => listen('library://changed', () => cb()),
    /** Files or folders dragged over the window: `over` while hovering, `drop` with the paths, `leave`. */
    onDragDrop: (
      cb: (e: { type: 'over' } | { type: 'drop'; paths: string[] } | { type: 'leave' }) => void,
    ): Promise<UnlistenFn> =>
      getCurrentWebview().onDragDropEvent(e => {
        const p = e.payload
        if (p.type === 'drop') cb({ type: 'drop', paths: p.paths })
        else if (p.type === 'leave') cb({ type: 'leave' })
        else cb({ type: 'over' })
      }),
  },
  saves: {
    summary: (paths: string[], slots: number) =>
      invoke<SavesSummary[]>('saves_summary', { paths, slots }),
    game: (path: string, slots: number) => invoke<GameSaves>('saves_game', { path, slots }),
    /** The current in-game save becomes a backup first. */
    restore: (path: string, name: string) => invoke<void>('saves_restore', { path, name }),
    /** E04: puts the newest backup back. */
    restoreLatest: (path: string) => invoke<void>('saves_restore_latest', { path }),
    deleteQuick: (path: string, slot: number) => invoke<void>('saves_delete_quick', { path, slot }),
    /** Opens a save dialog; resolves false if it was cancelled. */
    export: (path: string, what: SaveExport) => invoke<boolean>('saves_export', { path, what }),
    /** Opens a file picker and recognises the file; null = cancelled. */
    importPick: () => invoke<ImportCandidate | null>('saves_import_pick'),
    /** `file` becomes the in-game save of `game`; the current one becomes a backup first. */
    import: (file: string, game: string) => invoke<void>('saves_import', { file, game }),
    /** Picks a folder and pairs its saves with `games` (ROM paths) by file name; null = cancelled. */
    importFolder: (what: FolderImport, games: string[]) =>
      invoke<FolderSaves | null>('saves_import_folder', { what, games }),
    /** Imports every pair; resolves how many were imported. */
    importMany: (matches: SaveMatch[]) => invoke<number>('saves_import_many', { matches }),
  },
  firstRun: {
    /** Whether to start with the first run (not done on this device, library not set up). */
    show: () => invoke<boolean>('first_run_show'),
    /** The steps, given whether a controller is connected now. */
    steps: (controller: boolean) => invoke<FirstRunStep[]>('first_run_steps', { controller }),
    /** «Vai alla libreria»: never again on this device. */
    done: () => invoke<void>('first_run_done'),
  },
  onEvent: (cb: (e: EmuEvent) => void): Promise<UnlistenFn> =>
    listen<EmuEvent>('emu://event', e => cb(e.payload)),
  onMenu: (cb: () => void): Promise<UnlistenFn> => listen('emu://menu', () => cb()),
  onNav: (cb: (action: NavAction) => void): Promise<UnlistenFn> =>
    listen<{ action: NavAction }>('emu://nav', e => cb(e.payload.action)),
  onHotkey: (cb: (e: HotkeyEvent) => void): Promise<UnlistenFn> =>
    listen<HotkeyEvent>('emu://hotkey', e => cb(e.payload)),
  /** A screen effect could not be used (`effectNotLoaded`): the game shows without it. */
  onEffectError: (cb: (e: ErrorCode) => void): Promise<UnlistenFn> =>
    listen<ErrorCode>('emu://effect-error', e => cb(e.payload)),
  effects: {
    /** «Aggiungi un effetto da file»: picks a `.slangp` into `fx.fxfile`; null = cancelled. */
    chooseFile: () => invoke<string | null>('effects_choose_file'),
  },
  onGamepadConnected: (cb: (e: PadInfo) => void): Promise<UnlistenFn> =>
    listen<PadInfo>('emu://gamepad-connected', e => cb(e.payload)),
  onGamepadDisconnected: (cb: (e: PadInfo) => void): Promise<UnlistenFn> =>
    listen<PadInfo>('emu://gamepad-disconnected', e => cb(e.payload)),
  /** A connected controller's battery charge changed. */
  onGamepadChanged: (cb: (e: PadInfo) => void): Promise<UnlistenFn> =>
    listen<PadInfo>('emu://gamepad-changed', e => cb(e.payload)),
  controls: {
    /** The connected controllers. */
    pads: () => invoke<PadInfo[]>('gamepad_list'),
    /** While on, `onPadState` receives every change (tester, capture). */
    watch: (on: boolean) => invoke<void>('gamepad_watch', { on }),
    /** While on, the controllers neither play nor navigate (a button is being captured). */
    capture: (on: boolean) => invoke<void>('gamepad_capture', { on }),
    /** The current quick save slot (L3 + ← →, F2 / F3 move it; F1 / F4 use it). */
    currentSlot: () => invoke<number>('current_slot'),
    /** One step of the current slot (`forward`: next); answers the new slot. */
    stepSlot: (forward: boolean) => invoke<number>('step_slot', { forward }),
    /** Remap table and shortcuts of a family at a scope, defaults applied. */
    bindings: (family: PadFamily, scope: SettingsScope) =>
      invoke<ControlsBindings>('controls_bindings', { family, scope }),
    onPadState: (cb: (pads: PadState[]) => void): Promise<UnlistenFn> =>
      listen<{ pads: PadState[] }>('emu://pad-state', e => cb(e.payload.pads)),
  },
}
