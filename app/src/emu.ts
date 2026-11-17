// Typed bridge to the Rust commands. The interface goes through here only.
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

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
  /** The theme to show: `EMUDOME_THEME` if set, else the system's (dark → Scuro, light → Chiaro). */
  theme: (systemDark: boolean) => invoke<Theme>('theme', { systemDark }),
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
