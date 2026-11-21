// Typed bridge to the Rust commands. The interface goes through here only.
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { BiosStatus } from '@/generated/BiosStatus'
import type { Button } from '@/generated/Button'
import type { ClosingProgress } from '@/generated/ClosingProgress'
import type { CoverKind } from '@/generated/CoverKind'
import type { EmuEvent } from '@/generated/EmuEvent'
import type { ErrorCode } from '@/generated/ErrorCode'
import type { FirstRunStep } from '@/generated/FirstRunStep'
import type { FolderImport } from '@/generated/FolderImport'
import type { GameId } from '@/generated/GameId'
import type { GameOverride } from '@/generated/GameOverride'
import type { NavAction } from '@/generated/NavAction'
import type { PadFamily } from '@/generated/PadFamily'
import type { SettingKey } from '@/generated/SettingKey'
import type { SettingRecent } from '@/generated/SettingRecent'
import type { SettingRow } from '@/generated/SettingRow'
import type { SettingValue } from '@/generated/SettingValue'
import type { SettingsSchema } from '@/generated/SettingsSchema'
import type { SettingsScope } from '@/generated/SettingsScope'
import type { Speed } from '@/generated/Speed'
import type { BiosRow } from '@/generated/app/BiosRow'
import type { ControlsBindings } from '@/generated/app/ControlsBindings'
import type { CoreRestart } from '@/generated/app/CoreRestart'
import type { FolderSaves } from '@/generated/app/FolderSaves'
import type { GameInfo } from '@/generated/app/GameInfo'
import type { GameSaves } from '@/generated/app/GameSaves'
import type { HotkeyEvent } from '@/generated/app/HotkeyEvent'
import type { ImportCandidate } from '@/generated/app/ImportCandidate'
import type { Library } from '@/generated/app/Library'
import type { LibraryChanged } from '@/generated/app/LibraryChanged'
import type { LibraryScanEvent } from '@/generated/app/LibraryScanEvent'
import type { PadInfo } from '@/generated/app/PadInfo'
import type { PadState } from '@/generated/app/PadState'
import type { PointerArea } from '@/generated/app/PointerArea'
import type { SaveExport } from '@/generated/app/SaveExport'
import type { SaveMatch } from '@/generated/app/SaveMatch'
import type { SavesMigration } from '@/generated/app/SavesMigration'
import type { SavesSummary } from '@/generated/app/SavesSummary'
import type { SettingsChanged } from '@/generated/app/SettingsChanged'
import type { SlotInfo } from '@/generated/app/SlotInfo'
import type { Stats } from '@/generated/app/Stats'
import type { Theme } from '@/generated/app/Theme'
import type { VideoMode } from '@/generated/app/VideoMode'

// The bridge's types are generated from the Rust types (ts-rs) and the settings catalogue into
// src/generated/ (crates/emudome-core/src/contract.rs, app/src-tauri/src/contract.rs): a `cargo
// test` fails while they are stale. Never write one here by hand; the one below has no Rust
// type the contract can reach yet (see its comment).
export type { ActionStyle } from '@/generated/ActionStyle'
export type { BackupReason } from '@/generated/BackupReason'
export type { BindingTable } from '@/generated/BindingTable'
export type { BiosSlot } from '@/generated/BiosSlot'
export type { BiosStatus } from '@/generated/BiosStatus'
export type { Button } from '@/generated/Button'
export type { ChoiceStyle } from '@/generated/ChoiceStyle'
export type { ClosingPhase } from '@/generated/ClosingPhase'
export type { ClosingProgress } from '@/generated/ClosingProgress'
export type { ClosingPurpose } from '@/generated/ClosingPurpose'
export type { CommitTarget } from '@/generated/CommitTarget'
export type { ConsoleId } from '@/generated/ConsoleId'
export type { CoverKind } from '@/generated/CoverKind'
export type { EmuEvent } from '@/generated/EmuEvent'
export type { ErrorCode } from '@/generated/ErrorCode'
export type { FirstRunStep } from '@/generated/FirstRunStep'
export type { FolderImport } from '@/generated/FolderImport'
export type { GameId } from '@/generated/GameId'
export type { GameOverride } from '@/generated/GameOverride'
export type { GameRegion } from '@/generated/GameRegion'
export type { HotkeyId } from '@/generated/HotkeyId'
export type { HotkeyStatus } from '@/generated/HotkeyStatus'
export type { ImportFormat } from '@/generated/ImportFormat'
export type { NavAction } from '@/generated/NavAction'
export type { PadFamily } from '@/generated/PadFamily'
export type { Pos } from '@/generated/Pos'
export type { SettingDevice } from '@/generated/SettingDevice'
export type { SettingEntry } from '@/generated/SettingEntry'
export type { SettingKey } from '@/generated/SettingKey'
export type { SettingKind } from '@/generated/SettingKind'
export type { SettingLevel } from '@/generated/SettingLevel'
export type { SettingPageGroup } from '@/generated/SettingPageGroup'
export type { SettingPlatform } from '@/generated/SettingPlatform'
export type { SettingRecent } from '@/generated/SettingRecent'
export type { SettingRow } from '@/generated/SettingRow'
export type { SettingSection } from '@/generated/SettingSection'
export type { SettingShowIf } from '@/generated/SettingShowIf'
export type { SettingsSchema } from '@/generated/SettingsSchema'
export type { SettingsScope } from '@/generated/SettingsScope'
export type { SettingValue } from '@/generated/SettingValue'
export type { SliderUnit } from '@/generated/SliderUnit'
export type { Speed } from '@/generated/Speed'
export type { StateLoadBlock } from '@/generated/StateLoadBlock'
export type { BiosRow } from '@/generated/app/BiosRow'
export type { ComboRow } from '@/generated/app/ComboRow'
export type { ControlsBindings } from '@/generated/app/ControlsBindings'
export type { CoreRestart } from '@/generated/app/CoreRestart'
export type { FolderSaves } from '@/generated/app/FolderSaves'
export type { GameInfo } from '@/generated/app/GameInfo'
export type { GameSaves } from '@/generated/app/GameSaves'
export type { HotkeyEvent } from '@/generated/app/HotkeyEvent'
export type { ImportCandidate } from '@/generated/app/ImportCandidate'
export type { Library } from '@/generated/app/Library'
export type { LibraryFolder } from '@/generated/app/LibraryFolder'
export type { LibraryGame } from '@/generated/app/LibraryGame'
export type { LibraryChanged } from '@/generated/app/LibraryChanged'
export type { LibraryScanEvent } from '@/generated/app/LibraryScanEvent'
export type { PadInfo } from '@/generated/app/PadInfo'
export type { PadState } from '@/generated/app/PadState'
export type { PointerArea } from '@/generated/app/PointerArea'
export type { QuickSave } from '@/generated/app/QuickSave'
export type { RemapRow } from '@/generated/app/RemapRow'
export type { SaveBackup } from '@/generated/app/SaveBackup'
export type { SaveExport } from '@/generated/app/SaveExport'
export type { SaveFile } from '@/generated/app/SaveFile'
export type { SaveMatch } from '@/generated/app/SaveMatch'
export type { SavesMigration } from '@/generated/app/SavesMigration'
export type { SavesSummary } from '@/generated/app/SavesSummary'
export type { SettingsChanged } from '@/generated/app/SettingsChanged'
export type { SlotInfo } from '@/generated/app/SlotInfo'
export type { Stats } from '@/generated/app/Stats'
export type { Theme } from '@/generated/app/Theme'
export type { VideoMode } from '@/generated/app/VideoMode'

/** Kept for the toasts, which only need the name. */
export type GamepadEvent = Pick<PadInfo, 'name'>

/** Slot 0: the auto quick save («Salvataggio rapido automatico»). */
export const AUTO_SLOT = 0

/**
 * One round of the canvas loop in bench mode (`WebviewFrame` in src-tauri/src/bench.rs, behind
 * the off-by-default `bench` feature): `t` in ms since the Unix epoch, `seq` the game frame.
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
  /** The file extensions «Apri un file» offers (every console whose games open today). */
  openExtensions: () => invoke<string[]>('open_extensions'),
  videoMode: () => invoke<[VideoMode, boolean]>('video_mode'),
  setVideoMode: (mode: VideoMode) => invoke<void>('set_video_mode', { mode }),
  openGame: (path: string) => invoke<GameInfo>('open_game', { path }),
  closeGame: () => invoke<void>('close_game'),
  /** Closing a game took too long: keep waiting for its saves (`emu://closing`). */
  closeWait: () => invoke<void>('close_wait'),
  /** Closing a game took too long or a save failed: close without it (progress may be lost). */
  closeAnyway: () => invoke<void>('close_anyway'),
  /** Every step of closing a game, the window or the app (saving, saved, timeout, failed). */
  onClosing: (cb: (e: ClosingProgress) => void): Promise<UnlistenFn> =>
    listen<ClosingProgress>('emu://closing', e => cb(e.payload)),
  /** E11: let go of a core that stopped answering (does not wait for it). */
  abandonGame: () => invoke<void>('abandon_game'),
  /** The auto quick save (slot 0) and slots 1..count of the open game. */
  listSlots: (count: number) => invoke<SlotInfo[]>('list_slots', { count }),
  setSpeed: (speed: Speed) => invoke<void>('set_speed', { speed }),
  setPaused: (paused: boolean) => invoke<void>('set_paused', { paused }),
  /** While the menu is open the controllers navigate it (`emu://nav`) instead of playing.
   * Opening it lets go of the keyboard's buttons. */
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
  /** Rewind on or off (Backspace and L3 + L2 reach it in Rust, `key` and the controllers). */
  rewind: (on: boolean) => invoke<void>('rewind', { on }),
  /** F12, L3 + B, quick menu › Strumenti: PNG next to the saves (`screenshotSaved`). */
  screenshot: () => invoke<void>('screenshot'),
  /** «Annulla il caricamento» (`saves.undo`): back to before the last load (`loadUndone`). */
  undoLoad: () => invoke<void>('undo_load'),
  /** «Salvato nello slot N · Annulla»: the slot gets back what it held (`saveUndone`). */
  undoSave: () => invoke<void>('undo_save'),
  /** A game button from the touch controls. */
  input: (button: Button, pressed: boolean) => invoke<void>('input', { button, pressed }),
  /** A key (`KeyboardEvent.code`) for Rust to route in game: a game button or a shortcut. */
  key: (code: string, pressed: boolean, repeat: boolean) =>
    invoke<void>('key', { code, pressed, repeat }),
  /** The page lost the focus: Rust lets go of every key (game buttons, fast forward, rewind). */
  releaseKeys: () => invoke<void>('release_keys'),
  /** The DS touch screen: the pointer at (x, y) in `area`; Rust maps it to the bottom screen. */
  pointer: (x: number, y: number, pressed: boolean, area: PointerArea) =>
    invoke<void>('pointer', { x, y, pressed, area }),
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
    set: (key: SettingKey, scope: SettingsScope, value: SettingValue) =>
      invoke<void>('settings_set', { key, scope, value }),
    reset: (key: SettingKey, scope: SettingsScope) =>
      invoke<void>('settings_reset', { key, scope }),
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
    /** Settings that wait for the game to be opened again (`emu://core-restart`). */
    onCoreRestart: (cb: (e: CoreRestart) => void): Promise<UnlistenFn> =>
      listen<CoreRestart>('emu://core-restart', e => cb(e.payload)),
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
    /** What changed in the library: some games' rows (cover, flags, name, play time), or all of
     * it (a scan, a folder removed). */
    onChanged: (cb: (e: LibraryChanged) => void): Promise<UnlistenFn> =>
      listen<LibraryChanged>('library://changed', e => cb(e.payload)),
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
    summary: (games: GameId[], slots: number) =>
      invoke<SavesSummary[]>('saves_summary', { games, slots }),
    game: (game: GameId, slots: number) => invoke<GameSaves>('saves_game', { game, slots }),
    /** The current in-game save becomes a backup first. */
    restore: (game: GameId, name: string) => invoke<void>('saves_restore', { game, name }),
    /** E04: puts the newest backup back. */
    restoreLatest: (game: GameId) => invoke<void>('saves_restore_latest', { game }),
    deleteQuick: (game: GameId, slot: number) => invoke<void>('saves_delete_quick', { game, slot }),
    /** Opens a save dialog; resolves false if it was cancelled. */
    export: (game: GameId, what: SaveExport) => invoke<boolean>('saves_export', { game, what }),
    /** Opens a file picker and recognises the file; null = cancelled. */
    importPick: () => invoke<ImportCandidate | null>('saves_import_pick'),
    /** `file` becomes the in-game save of `game`; the current one becomes a backup first. */
    import: (file: string, game: GameId) => invoke<void>('saves_import', { file, game }),
    /** Picks a folder and pairs its saves with the library `games` by their ROM's file name; null = cancelled. */
    importFolder: (what: FolderImport, games: GameId[]) =>
      invoke<FolderSaves | null>('saves_import_folder', { what, games }),
    /** Imports every pair; resolves how many were imported. */
    importMany: (matches: SaveMatch[]) => invoke<number>('saves_import_many', { matches }),
    /** Where the one-time move of old save names stands (asked once at start). */
    migration: () => invoke<SavesMigration>('saves_migration_status'),
    /** The one-time move ended (`saves://migration`). */
    onMigration: (cb: (e: SavesMigration) => void): Promise<UnlistenFn> =>
      listen<SavesMigration>('saves://migration', e => cb(e.payload)),
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
    listen<EmuEvent>('emu://session', e => cb(e.payload)),
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
  bios: {
    /** A `bios.*` row's picker: checks the file and keeps it; the row's new status, null = cancelled. */
    choose: (key: SettingKey) => invoke<BiosStatus | null>('bios_choose', { key }),
    /** What every `bios.*` row EmuDome handles shows. */
    status: () => invoke<BiosRow[]>('bios_status'),
  },
  /** A game opened without the system files a setting asked for, and fell back (`bios*Missing`). */
  onBiosFallback: (cb: (e: ErrorCode) => void): Promise<UnlistenFn> =>
    listen<ErrorCode>('emu://bios-fallback', e => cb(e.payload)),
  /** What is not working now (native video, surface, audio output, controllers, a stopped thread). */
  diagnostics: () => invoke<ErrorCode[]>('diagnostics'),
  /** The list of what is not working changed (`emu://health`): the whole list, empty when all is well. */
  onHealth: (cb: (issues: ErrorCode[]) => void): Promise<UnlistenFn> =>
    listen<ErrorCode[]>('emu://health', e => cb(e.payload)),
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
      listen<{ pads: PadState[] }>('emu://gamepad-state', e => cb(e.payload.pads)),
  },
}
