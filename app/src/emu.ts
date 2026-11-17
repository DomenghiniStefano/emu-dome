// Typed bridge to the Rust commands. The interface goes through here only.
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
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
  core: string
}
export interface Stats {
  running: boolean
  emuFps: number
  targetFps: number
  speed: Speed | null
  paused: boolean
}

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
  | { code: 'settingLevelRefused'; params: { key: string; level: SettingLevel } }
  | { code: 'settingInvalidValue'; params: { key: string } }
  | { code: 'settingsWriteFailed'; params: { detail: string } }
  | { code: 'libraryFolderUnreachable'; params: { path: string } }
  | { code: 'libraryGameUnknown'; params: { id: string } }
  | { code: 'libraryGameUnavailable'; params: { id: string } }
  | { code: 'libraryWriteFailed'; params: { detail: string } }
  | { code: 'coverInvalid'; params: { detail: string } }

export type EmuEvent =
  | { type: 'started'; core: string; game: string; fps: number; width: number; height: number }
  | { type: 'stateSaved'; slot: number }
  | { type: 'stateLoaded'; slot: number }
  | { type: 'error'; error: ErrorCode }
  | { type: 'stopped' }

/** Payload of `emu://gamepad-connected` / `emu://gamepad-disconnected`; `name` is null when SDL knows none. */
export interface GamepadEvent {
  name: string | null
}

export type VideoMode = 'native' | 'webview'

/** Interface theme (mirror of `Theme` in src-tauri/src/theme.rs): Scuro, Nero, Chiaro. */
export type Theme = 'dark' | 'black' | 'light'

/** One focus step (`emu://nav`, mirror of `emudome_core::hotkeys::Nav`), also produced by the keyboard. */
export type NavAction = 'up' | 'down' | 'left' | 'right' | 'confirm' | 'back'

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
export type SettingPlatform = 'all' | SettingDevice
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

export interface SettingEntry {
  /** `<page>.<key>`, stable. */
  key: string
  page: string
  kind: SettingKind
  /** null for actions. */
  default: SettingValue | null
  /** Accepted levels, weakest first. */
  levels: SettingLevel[]
  perDevice: boolean
  platform: SettingPlatform
  phase: number
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

/** URL of the `emu://` protocol (on Windows it becomes http://emu.localhost/...). */
export const frameUrl = convertFileSrc('frame', 'emu')

export const emu = {
  startupGame: () => invoke<string | null>('startup_game'),
  videoMode: () => invoke<[VideoMode, boolean]>('video_mode'),
  setVideoMode: (mode: VideoMode) => invoke<void>('set_video_mode', { mode }),
  openGame: (path: string) => invoke<GameInfo>('open_game', { path }),
  closeGame: () => invoke<void>('close_game'),
  setSpeed: (speed: Speed) => invoke<void>('set_speed', { speed }),
  setPaused: (paused: boolean) => invoke<void>('set_paused', { paused }),
  /** While the menu is open the controllers navigate it (`emu://nav`) instead of playing. */
  setMenuOpen: (open: boolean) => invoke<void>('set_menu_open', { open }),
  saveState: (slot: number) => invoke<void>('save_state', { slot }),
  loadState: (slot: number) => invoke<void>('load_state', { slot }),
  reset: () => invoke<void>('reset_game'),
  input: (button: Button, pressed: boolean) => invoke<void>('input', { button, pressed }),
  stats: () => invoke<Stats>('stats'),
  /** The theme to show: `EMUDOME_THEME` if set, else `general.theme`, else the system's. */
  theme: (systemDark: boolean) => invoke<Theme>('theme', { systemDark }),
  settings: {
    schema: () => invoke<SettingsSchema>('settings_schema'),
    page: (page: string, scope: SettingsScope) =>
      invoke<SettingRow[]>('settings_page', { page, scope }),
    /** The open game's scope (null without one). */
    gameScope: () => invoke<SettingsScope | null>('settings_game_scope'),
    set: (key: string, scope: SettingsScope, value: SettingValue) =>
      invoke<void>('settings_set', { key, scope, value }),
    reset: (key: string, scope: SettingsScope) => invoke<void>('settings_reset', { key, scope }),
    /** Saves the changes «in prova» for this game or its console (all or nothing). */
    commitSession: (target: 'game' | 'console') =>
      invoke<void>('settings_session_commit', { target }),
    discardSession: () => invoke<void>('settings_session_discard'),
    onChanged: (cb: (e: SettingsChanged) => void): Promise<UnlistenFn> =>
      listen<SettingsChanged>('settings://changed', e => cb(e.payload)),
  },
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
  onEvent: (cb: (e: EmuEvent) => void): Promise<UnlistenFn> =>
    listen<EmuEvent>('emu://event', e => cb(e.payload)),
  onMenu: (cb: () => void): Promise<UnlistenFn> => listen('emu://menu', () => cb()),
  onNav: (cb: (action: NavAction) => void): Promise<UnlistenFn> =>
    listen<{ action: NavAction }>('emu://nav', e => cb(e.payload.action)),
  onHotkey: (cb: (e: HotkeyEvent) => void): Promise<UnlistenFn> =>
    listen<HotkeyEvent>('emu://hotkey', e => cb(e.payload)),
  onGamepadConnected: (cb: (e: GamepadEvent) => void): Promise<UnlistenFn> =>
    listen<GamepadEvent>('emu://gamepad-connected', e => cb(e.payload)),
  onGamepadDisconnected: (cb: (e: GamepadEvent) => void): Promise<UnlistenFn> =>
    listen<GamepadEvent>('emu://gamepad-disconnected', e => cb(e.payload)),
}

/** Keyboard → RetroPad. Becomes configurable in the settings (Phase 1). */
export const KEYMAP: Record<string, Button> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  KeyX: 'a',
  KeyZ: 'b',
  KeyA: 'l',
  KeyS: 'r',
  KeyW: 'x',
  KeyQ: 'y',
  Enter: 'start',
  ShiftRight: 'select',
  Backspace: 'select',
}
