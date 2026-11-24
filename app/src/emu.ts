// Typed bridge to the Rust commands. The interface goes through here only.
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { BiosStatus } from '@/generated/BiosStatus'
import type { Button } from '@/generated/Button'
import type { ClosingProgress } from '@/generated/ClosingProgress'
import type { ConsoleId } from '@/generated/ConsoleId'
import type { CoverKind } from '@/generated/CoverKind'
import type { CoverView } from '@/generated/CoverView'
import type { EmuEvent } from '@/generated/EmuEvent'
import type { ErrorCode } from '@/generated/ErrorCode'
import type { FirstRunStep } from '@/generated/FirstRunStep'
import type { FolderImport } from '@/generated/FolderImport'
import type { GameId } from '@/generated/GameId'
import type { GameOverride } from '@/generated/GameOverride'
import type { NavAction } from '@/generated/NavAction'
import type { Notification } from '@/generated/Notification'
import type { UpdateStatus } from '@/generated/UpdateStatus'
import type { PadFamily } from '@/generated/PadFamily'
import type { SettingKey } from '@/generated/SettingKey'
import type { SettingRecent } from '@/generated/SettingRecent'
import type { SettingRow } from '@/generated/SettingRow'
import type { SettingValue } from '@/generated/SettingValue'
import type { SettingsSchema } from '@/generated/SettingsSchema'
import type { SettingsScope } from '@/generated/SettingsScope'
import type { Speed } from '@/generated/Speed'
import type { BiosRow } from '@/generated/app/BiosRow'
import type { CloudActivity } from '@/generated/app/CloudActivity'
import type { CloudChoice } from '@/generated/app/CloudChoice'
import type { CloudDevice } from '@/generated/app/CloudDevice'
import type { CloudGame } from '@/generated/app/CloudGame'
import type { CloudGameConflict } from '@/generated/app/CloudGameConflict'
import type { CloudInUse } from '@/generated/app/CloudInUse'
import type { CloudInUseEvent } from '@/generated/app/CloudInUseEvent'
import type { CloudProgress } from '@/generated/app/CloudProgress'
import type { CloudStatus } from '@/generated/app/CloudStatus'
import type { ControlsBindings } from '@/generated/app/ControlsBindings'
import type { CoreRestart } from '@/generated/app/CoreRestart'
import type { CoverSource } from '@/generated/app/CoverSource'
import type { Licence } from '@/generated/app/Licence'
import type { StorageStatus } from '@/generated/app/StorageStatus'
import type { StorageBreakdown } from '@/generated/app/StorageBreakdown'
import type { StorageQuickGame } from '@/generated/app/StorageQuickGame'
import type { StorageQuickPick } from '@/generated/app/StorageQuickPick'
import type { GameScreenshot } from '@/generated/app/GameScreenshot'
import type { PickedImage } from '@/generated/app/PickedImage'
import type { FolderSaves } from '@/generated/app/FolderSaves'
import type { GameInfo } from '@/generated/app/GameInfo'
import type { GameSaves } from '@/generated/app/GameSaves'
import type { HotkeyEvent } from '@/generated/app/HotkeyEvent'
import type { ImportCandidate } from '@/generated/app/ImportCandidate'
import type { Library } from '@/generated/app/Library'
import type { LibraryChanged } from '@/generated/app/LibraryChanged'
import type { LibraryGame } from '@/generated/app/LibraryGame'
import type { LibraryScanEvent } from '@/generated/app/LibraryScanEvent'
import type { PadInfo } from '@/generated/app/PadInfo'
import type { PadState } from '@/generated/app/PadState'
import type { PointerArea } from '@/generated/app/PointerArea'
import type { ScreenPlacements } from '@/generated/app/ScreenPlacements'
import type { CustomLayout } from '@/generated/CustomLayout'
import type { CustomScreen } from '@/generated/CustomScreen'
import type { ScreenLayoutId } from '@/generated/ScreenLayoutId'
import type { VideoInset } from '@/generated/VideoInset'
import type { ScreenStatus } from '@/generated/ScreenStatus'
import type { TouchLayout } from '@/generated/TouchLayout'
import type { TouchSetup } from '@/generated/TouchSetup'
import type { SaveExport } from '@/generated/app/SaveExport'
import type { SaveMatch } from '@/generated/app/SaveMatch'
import type { SavesMigration } from '@/generated/app/SavesMigration'
import type { SavesSummary } from '@/generated/app/SavesSummary'
import type { SettingsChanged } from '@/generated/app/SettingsChanged'
import type { SlotInfo } from '@/generated/app/SlotInfo'
import type { Stats } from '@/generated/app/Stats'
import type { Theme } from '@/generated/app/Theme'
import type { VideoMode } from '@/generated/app/VideoMode'
import { DevRoute } from '@/lib/devRoutes'

// The Kit page in a plain browser (`pnpm dev`, then `#kit`): there is no Tauri there, and this
// module reads Tauri's internals as it loads (`convertFileSrc` below), so the page would stop before
// mounting. Development and the Kit page only, it stands in for the bridge, answering every command
// with nothing, as scripts/headless/kit-nav.mjs does. It lives here, the one module that talks to
// Tauri, and runs before anything below reads the bridge.
if (
  import.meta.env.DEV &&
  window.location.hash === DevRoute.Kit &&
  !('__TAURI_INTERNALS__' in window)
) {
  Object.assign(window, {
    __TAURI_INTERNALS__: {
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { windowLabel: 'main', label: 'main' },
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
      convertFileSrc: (path: string, protocol = 'asset') =>
        `http://${protocol}.localhost/${encodeURIComponent(path)}`,
      invoke: async (cmd: string) => (cmd === 'plugin:event|listen' ? 1 : null),
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => undefined },
  })
}

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
export type { CoverView } from '@/generated/CoverView'
export type { EmuEvent } from '@/generated/EmuEvent'
export type { ErrorCode } from '@/generated/ErrorCode'
export type { FirstRunStep } from '@/generated/FirstRunStep'
export type { FolderImport } from '@/generated/FolderImport'
export type { GameId } from '@/generated/GameId'
export type { GameFamily } from '@/generated/GameFamily'
export type { GameOverride } from '@/generated/GameOverride'
export type { GameRegion } from '@/generated/GameRegion'
export type { HotkeyId } from '@/generated/HotkeyId'
export type { HotkeyStatus } from '@/generated/HotkeyStatus'
export type { ImportFormat } from '@/generated/ImportFormat'
export type { NavAction } from '@/generated/NavAction'
export type { Notice } from '@/generated/Notice'
export type { Notification } from '@/generated/Notification'
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
export type { IoCause } from '@/generated/IoCause'
export type { ImageProblem } from '@/generated/ImageProblem'
export type { SignInProblem } from '@/generated/SignInProblem'
export type { SettingsSchema } from '@/generated/SettingsSchema'
export type { SettingsScope } from '@/generated/SettingsScope'
export type { SettingValue } from '@/generated/SettingValue'
export type { SliderUnit } from '@/generated/SliderUnit'
export type { Speed } from '@/generated/Speed'
export type { StateLoadBlock } from '@/generated/StateLoadBlock'
export type { UpdateState } from '@/generated/UpdateState'
export type { UpdateStatus } from '@/generated/UpdateStatus'
export type { CustomLayout } from '@/generated/CustomLayout'
export type { CustomScreen } from '@/generated/CustomScreen'
export type { PxRect } from '@/generated/PxRect'
export type { ScreenId } from '@/generated/ScreenId'
export type { ScreenLayoutId } from '@/generated/ScreenLayoutId'
export type { ScreenPlacement } from '@/generated/ScreenPlacement'
export type { VideoAlign } from '@/generated/VideoAlign'
export type { VideoInset } from '@/generated/VideoInset'
export type { ScreenRect } from '@/generated/ScreenRect'
export type { ScreenRotation } from '@/generated/ScreenRotation'
export type { ScreenStatus } from '@/generated/ScreenStatus'
export type { TouchBehaviour } from '@/generated/TouchBehaviour'
export type { TouchControl } from '@/generated/TouchControl'
export type { TouchControlId } from '@/generated/TouchControlId'
export type { TouchLayout } from '@/generated/TouchLayout'
export type { TouchScreenShape } from '@/generated/TouchScreenShape'
export type { TouchOrientation } from '@/generated/TouchOrientation'
export type { TouchSetup } from '@/generated/TouchSetup'
export type { ScreenPlacements } from '@/generated/app/ScreenPlacements'
export type { BiosRow } from '@/generated/app/BiosRow'
export type { CloudAccount } from '@/generated/app/CloudAccount'
export type { CloudActivity } from '@/generated/app/CloudActivity'
export type { CloudActivityKind } from '@/generated/app/CloudActivityKind'
export type { CloudChoice } from '@/generated/app/CloudChoice'
export type { CloudConflict } from '@/generated/app/CloudConflict'
export type { CloudDevice } from '@/generated/app/CloudDevice'
export type { CloudGame } from '@/generated/app/CloudGame'
export type { CloudGameConflict } from '@/generated/app/CloudGameConflict'
export type { CloudGameState } from '@/generated/app/CloudGameState'
export type { CloudInUse } from '@/generated/app/CloudInUse'
export type { CloudInUseEvent } from '@/generated/app/CloudInUseEvent'
export type { CloudPhase } from '@/generated/app/CloudPhase'
export type { CloudProgress } from '@/generated/app/CloudProgress'
export type { CloudStatus } from '@/generated/app/CloudStatus'
export type { CloudVersion } from '@/generated/app/CloudVersion'
export type { DriveQuota } from '@/generated/app/DriveQuota'
export type { ComboRow } from '@/generated/app/ComboRow'
export type { ControlsBindings } from '@/generated/app/ControlsBindings'
export type { CoreRestart } from '@/generated/app/CoreRestart'
export type { CoverSource } from '@/generated/app/CoverSource'
export type { Licence } from '@/generated/app/Licence'
export type { StorageStatus } from '@/generated/app/StorageStatus'
export type { StorageBreakdown } from '@/generated/app/StorageBreakdown'
export type { StorageGame } from '@/generated/app/StorageGame'
export type { StorageGameLocation } from '@/generated/app/StorageGameLocation'
export type { StorageQuickGame } from '@/generated/app/StorageQuickGame'
export type { StorageQuickSlot } from '@/generated/app/StorageQuickSlot'
export type { StorageQuickPick } from '@/generated/app/StorageQuickPick'
export type { StorageScreenshotCount } from '@/generated/app/StorageScreenshotCount'
export type { LicenceKind } from '@/generated/app/LicenceKind'
export type { GameScreenshot } from '@/generated/app/GameScreenshot'
export type { PickedImage } from '@/generated/app/PickedImage'
export type { FolderSaves } from '@/generated/app/FolderSaves'
export type { GameInfo } from '@/generated/app/GameInfo'
export type { GameSaves } from '@/generated/app/GameSaves'
export type { HotkeyEvent } from '@/generated/app/HotkeyEvent'
export type { ImportCandidate } from '@/generated/app/ImportCandidate'
export type { Library } from '@/generated/app/Library'
export type { FolderAccess } from '@/generated/app/FolderAccess'
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
  /** `general.start` = «last game»: the library game to open at startup (none: the library). */
  startupLastGame: () => invoke<LibraryGame | null>('startup_last_game'),
  /** The file extensions «Apri un file» offers (every console whose games open today). */
  openExtensions: () => invoke<string[]>('open_extensions'),
  /** «Apri un file»: the game file picked (a path, or a `content://` document on Android, its
   * extension checked by Rust); `filter` names the extensions in the desktop's dialog. null =
   * cancelled. */
  pickGameFile: (filter: string) => invoke<string | null>('pick_game_file', { filter }),
  /** The title of a game file outside the library: its file name without the extension (a
   * document's display name, never its URI); null when a document's name cannot be read. */
  gameFileTitle: (path: string) => invoke<string | null>('game_file_title', { path }),
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
  /** The edges kept clear of the native game (shares of the window: the quick menu, the touch
   * controls); it is fitted in the rest. */
  setVideoInset: (inset: VideoInset) => invoke<void>('set_video_inset', { inset }),
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
  /** The touch controls (Trello #29): the layouts of one console, C06's «Salva», the Circle Pad
   * and the vibration of a press. */
  touch: {
    /** The layouts saved on this device for `console`, its controls and the size / opacity
     * limits. */
    layouts: (console: ConsoleId) => invoke<TouchSetup>('touch_layouts', { console }),
    /** Saves each layout given for `console` (`null`: that orientation stays as stored) in one
     * write, and makes them the ones shown there. */
    setLayouts: (console: ConsoleId, portrait: TouchLayout | null, landscape: TouchLayout | null) =>
      invoke<void>('touch_set_layouts', { console, portrait, landscape }),
    /** The Circle Pad at (x, y), each -1..1, y downwards; (0, 0) let go. */
    stick: (x: number, y: number) => invoke<void>('touch_stick', { x, y }),
    /** A press's vibration, `strength` 1–100 (`touch.thapt`); nothing on the desktop. */
    haptic: (strength: number) => invoke<void>('haptic', { strength }),
  },
  /** A key (`KeyboardEvent.code`) for Rust to route in game: a game button or a shortcut. */
  key: (code: string, pressed: boolean, repeat: boolean) =>
    invoke<void>('key', { code, pressed, repeat }),
  /** The page lost the focus (Rust lets go of every key: game buttons, fast forward, rewind) or
   * got it back; with `general.bgpause` the game pauses meanwhile (`emu://hotkey` `paused`). */
  windowFocus: (focused: boolean) => invoke<void>('window_focus', { focused }),
  /** The DS touch screen: the pointer at (x, y) in `area`; Rust maps it to the bottom screen. */
  pointer: (x: number, y: number, pressed: boolean, area: PointerArea) =>
    invoke<void>('pointer', { x, y, pressed, area }),
  stats: () => invoke<Stats>('stats'),
  /**
   * NDS / 3DS screens (Trello #28): the open game's layout, swap and lid. Every change (from
   * these, L3 + Y / R3 / Select, or a `screens.*` setting) arrives on `onChanged`.
   */
  screens: {
    /** Now (`dualScreen` false without a two-screen game). */
    status: () => invoke<ScreenStatus>('screens_status'),
    /** The next layout of `screens.cycle` (like L3 + R3). */
    nextLayout: () => invoke<ScreenStatus>('screens_next_layout'),
    /** Shows `layout`, in the cycle or not (`custom` only once one is saved). */
    setLayout: (layout: ScreenLayoutId) => invoke<ScreenStatus>('screens_set_layout', { layout }),
    /** «Scambia schermi»: a toggle (L3 + Y follows `screens.swap`). */
    swap: () => invoke<ScreenStatus>('screens_swap'),
    /** «Chiudi il coperchio»: closes or opens it (`lidUnsupported` where the core cannot). */
    toggleLid: () => invoke<ScreenStatus>('screens_toggle_lid'),
    /** The webview canvas: where each screen of the current frame goes in `area` (CSS px). */
    placements: (area: PointerArea) => invoke<ScreenPlacements>('screens_placements', { area }),
    /** The custom layout as `scope` resolves it (empty: none). */
    custom: (scope: SettingsScope) => invoke<CustomLayout>('screens_custom', { scope }),
    /** Saves it at a console / game scope, or previews it «in prova» at the session scope;
     * removing it is `settings.reset('screens.custom', scope)`. */
    setCustom: (scope: SettingsScope, layout: CustomLayout) =>
      invoke<void>('screens_set_custom', { scope, layout }),
    /** The editor's «Parti da»: where `layout` puts a `console` game's screens in a window of
     * `width` × `height`, composed by Rust (gap and ratio as `scope` resolves them). */
    presetCustom: (
      scope: SettingsScope,
      console: ConsoleId,
      layout: ScreenLayoutId,
      width: number,
      height: number,
    ) => invoke<CustomLayout>('screens_preset_custom', { scope, console, layout, width, height }),
    /** Where the touch editor starts a `console` game's screens in `area` (fractions of it),
     * composed by Rust: at the session scope over the open game, where it shows them now (the
     * layout cycled to, swapped: each keyed by its frame); otherwise with the settings of `scope`. */
    phoneStart: (scope: SettingsScope, console: ConsoleId, area: PointerArea) =>
      invoke<CustomScreen[]>('screens_phone_start', { scope, console, area }),
    /** The phone's own placement of the open game's screens (empty: withdrawn) and the edges kept
     * clear of the game it is fitted in (as `setVideoInset`), applied together. */
    setPhone: (screens: CustomScreen[], inset: VideoInset) =>
      invoke<ScreenStatus>('screens_set_phone', { screens, inset }),
    onChanged: (cb: (e: ScreenStatus) => void): Promise<UnlistenFn> =>
      listen<ScreenStatus>('emu://screens', e => cb(e.payload)),
  },
  /** Bench mode (`EMUDOME_BENCH`): the canvas reports its rounds with `benchFrames`. */
  benchActive: () => invoke<boolean>('bench_active'),
  benchFrames: (frames: BenchFrame[]) => invoke<void>('bench_frames', { frames }),
  /** The theme to show: `EMUDOME_THEME` if set, else `general.theme`, else the system's. */
  theme: (systemDark: boolean) => invoke<Theme>('theme', { systemDark }),
  settings: {
    schema: () => invoke<SettingsSchema>('settings_schema'),
    /** A choice's options found at run time, after the schema's (`moreAtRuntime`: the audio
     * outputs of `audio.out`); each id is its own text (a device's name). */
    runtimeOptions: (key: SettingKey) => invoke<string[]>('settings_runtime_options', { key }),
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
    /** Native folder picker; the folder joins `library.folders` and is scanned. null = cancelled.
     * `start`: a picked Android folder to choose again, where the picker opens (#141). */
    chooseFolder: (start?: string) =>
      invoke<string | null>('library_choose_folder', { start: start ?? null }),
    removeFolder: (path: string) => invoke<void>('library_remove_folder', { path }),
    /** Drag and drop: folders join `library.folders`, files are added one by one. Then a scan. */
    addPaths: (paths: string[]) => invoke<void>('library_add_paths', { paths }),
    /** Rescans every folder in the background; progress on `library://scan`. */
    scan: () => invoke<void>('library_scan'),
    /** Opens the game (like `openGame`) and counts its play time until it closes. */
    open: (id: GameId) => invoke<GameInfo>('library_open', { id }),
    /** L12: the generated cover, the ROM's icon, or the user cover the game already has. */
    setCover: (id: GameId, kind: CoverKind) => invoke<void>('library_set_cover', { id, kind }),
    /** L12 «Immagine tua»: the native image picker; the image is kept for «Adatta la copertina».
     * null = cancelled. */
    pickCoverImage: () => invoke<PickedImage | null>('library_pick_cover_image'),
    /** «Usa questa copertina»: the cover made from `source` showing `view` (600 × 900, in Rust). */
    setCoverImage: (id: GameId, source: CoverSource, view: CoverView) =>
      invoke<void>('library_set_cover_image', { id, source, view }),
    /** The game's screenshots, newest first (L12 «Da uno screenshot»). */
    screenshots: (id: GameId) => invoke<GameScreenshot[]>('library_screenshots', { id }),
    setFlags: (id: GameId, flags: { favourite?: boolean; hidden?: boolean }) =>
      invoke<void>('library_set_flags', { id, ...flags }),
    /** «Nome mostrato» (L10); blank goes back to the ROM's title. */
    setName: (id: GameId, name: string) => invoke<void>('library_set_name', { id, name }),
    /** «Mostra questa in libreria» (#102): the library shows `id` for its family; `pin` false
     * gives the family back to the version launched last. Arrives as `library://changed`
     * `families`. */
    pinVersion: (id: GameId, pin: boolean) => invoke<void>('library_pin_version', { id, pin }),
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
  /** `system`: Android's own back button, not a controller (`hotkeys::MenuEvent`, #131). */
  onMenu: (cb: (system: boolean) => void): Promise<UnlistenFn> =>
    listen<{ system: boolean } | null>('emu://menu', e => cb(e.payload?.system === true)),
  /** `system`: Android's own back button, not a controller (`hotkeys::NavEvent`, #131). */
  onNav: (cb: (action: NavAction, system: boolean) => void): Promise<UnlistenFn> =>
    listen<{ action: NavAction; system?: boolean }>('emu://nav', e =>
      cb(e.payload.action, e.payload.system === true),
    ),
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
  /** X03, the notification centre (per device, kept until read). */
  /** EmuDome updating itself (Trello #44, `updates.rs`). */
  updates: {
    status: () => invoke<UpdateStatus>('updates_status'),
    /** «Controlla ora» (`about.upd`): one check, whatever `general.updates` says. */
    check: () => invoke<UpdateStatus>('updates_check'),
    /** X06 «Più tardi». */
    later: () => invoke<UpdateStatus>('updates_later'),
    /** X06 «Riavvia e aggiorna»: refused while a game is open. */
    install: () => invoke<UpdateStatus>('updates_install'),
    /** The status changed (`updates://changed`). */
    onChanged: (cb: (s: UpdateStatus) => void): Promise<UnlistenFn> =>
      listen<UpdateStatus>('updates://changed', e => cb(e.payload)),
  },
  /** E13 (#29): the room left where the saves are written; null when the system does not say. */
  storage: {
    status: () => invoke<StorageStatus | null>('storage_status'),
    /** «Gestisci lo spazio»: the disk's room and what EmuDome's files weigh (no Drive check). */
    breakdown: () => invoke<StorageBreakdown>('storage_breakdown'),
    /** «Salvataggi rapidi › Scegli»: every game's quick saves. */
    quickSaves: () => invoke<StorageQuickGame[]>('storage_quick_saves'),
    /** Deletes the quick saves picked (never an in-game save); how many went. */
    deleteQuick: (picked: StorageQuickPick[]) => invoke<number>('storage_delete_quick', { picked }),
    /** Deletes screenshots of a game by name; how many went. */
    deleteScreenshots: (game: GameId, names: string[]) =>
      invoke<number>('storage_delete_screenshots', { game, names }),
    /** «File temporanei › Svuota»: the bytes freed. */
    clearTemp: () => invoke<number>('storage_clear_temp'),
    /** «Togli dal telefono»: a game file in EmuDome's own folder that Drive has; bytes freed. */
    removeGame: (id: GameId) => invoke<number>('storage_remove_game', { id }),
    /** «Apri la cartella»: the game's folder in the system's file manager (the desktop). */
    openFolder: (id: GameId) => invoke<void>('storage_open_folder', { id }),
  },
  /** Android's back at the shell's root (#29): the app to the background; false on the desktop. */
  leaveToBackground: () => invoke<boolean>('leave_to_background'),
  /** Android's system bars hidden / shown for a layout editor outside a game (#131); nothing on
   * the desktop. A game holds them hidden by itself. */
  setImmersive: (on: boolean) => invoke<void>('set_immersive', { on }),
  /** The «Informazioni» page's actions (`about.rs`). */
  about: {
    /** `about.lic`: EmuDome, the shipped cores and the other components, with their licences. */
    licences: () => invoke<Licence[]>('about_licences'),
    licenceText: (id: string, file: string) => invoke<string>('about_licence_text', { id, file }),
    /** `about.log`: the log through the save dialog; the file's name, or null when cancelled. */
    exportLog: () => invoke<string | null>('about_export_log'),
  },
  notifications: {
    /** Every notification, newest first. */
    list: () => invoke<Notification[]>('notifications_list'),
    markRead: (id: number) => invoke<void>('notifications_mark_read', { id }),
    /** «Segna tutte come lette». */
    markAllRead: () => invoke<void>('notifications_mark_all_read'),
    /** The list changed (`notifications://changed`): the whole list, newest first. */
    onChanged: (cb: (list: Notification[]) => void): Promise<UnlistenFn> =>
      listen<Notification[]>('notifications://changed', e => cb(e.payload)),
  },
  /** Google Drive (#31): the account, the sync, the games on Drive, the game flow's questions. */
  cloud: {
    /** The whole status (the account and its space follow on `onStatus` when not known yet). */
    status: () => invoke<CloudStatus>('cloud_status'),
    /** Opens the system browser on Google's page; `done` / `failed` are shown back in the browser. */
    signIn: (done: string, failed: string) =>
      invoke<CloudStatus>('cloud_sign_in', { done, failed }),
    signOut: () => invoke<CloudStatus>('cloud_sign_out'),
    /** «Sincronizza ora»: settings, every game's saves (not the open game's), the library list. */
    syncNow: () => invoke<CloudStatus>('cloud_sync_now'),
    /** «Giochi su Google Drive»: every library game and every game file on Drive. */
    games: () => invoke<CloudGame[]>('cloud_games'),
    /** «Carica»: the game file goes to Drive (`cloudQuotaFull` when it does not fit). */
    upload: (id: GameId) => invoke<CloudGame>('cloud_game_upload', { id }),
    /** «Scarica»: the game file comes here and joins the library (`library://changed`). */
    download: (id: GameId) => invoke<CloudGame>('cloud_game_download', { id }),
    /** «Rimuovi da Drive»: the game file goes to Drive's trash; saves and local files stay. */
    remove: (id: GameId) => invoke<CloudGame>('cloud_game_remove', { id }),
    /** «I tuoi dispositivi»: this device first. */
    devices: () => invoke<CloudDevice[]>('cloud_devices'),
    /** «Attività recenti», newest first (this run's). */
    activity: () => invoke<CloudActivity[]>('cloud_activity'),
    /** G04: the two versions of a game's save; null when it has no conflict. */
    conflict: (id: GameId) => invoke<CloudGameConflict | null>('cloud_conflict', { id }),
    /** G04's answer (`saveGameOpen` while the game is open). */
    resolve: (id: GameId, choice: CloudChoice) =>
      invoke<CloudStatus>('cloud_resolve', { id, choice }),
    /** G05, before opening a library game: `elsewhere` when another device has it open. */
    launchCheck: (id: GameId) => invoke<CloudInUse>('cloud_launch_check', { id }),
    /** G03's «Gioca senza aspettare»: false when no pull of that game is running. */
    skipPull: (id: GameId) => invoke<boolean>('cloud_skip_pull', { id }),
    /** The status changed (`cloud://status`): the whole status. */
    onStatus: (cb: (s: CloudStatus) => void): Promise<UnlistenFn> =>
      listen<CloudStatus>('cloud://status', e => cb(e.payload)),
    /** A pull (G03), push (G14), sync or game file moving (`cloud://progress`). */
    onProgress: (cb: (p: CloudProgress) => void): Promise<UnlistenFn> =>
      listen<CloudProgress>('cloud://progress', e => cb(e.payload)),
    /** A game's save changed on both sides (`cloud://conflict`): G04. */
    onConflict: (cb: (c: CloudGameConflict) => void): Promise<UnlistenFn> =>
      listen<CloudGameConflict>('cloud://conflict', e => cb(e.payload)),
    /** A game opened while another device has it open (`cloud://in-use`), without G05 first. */
    onInUse: (cb: (e: CloudInUseEvent) => void): Promise<UnlistenFn> =>
      listen<CloudInUseEvent>('cloud://in-use', e => cb(e.payload)),
  },
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
    /** «Rendi rapido»: `slot` becomes the open game's quick slot, recorded for that game; answers
     * the slot now current (clamped to 1–9). */
    setSlot: (slot: number) => invoke<number>('set_slot', { slot }),
    /** Remap table and shortcuts of a family at a scope, defaults applied. */
    bindings: (family: PadFamily, scope: SettingsScope) =>
      invoke<ControlsBindings>('controls_bindings', { family, scope }),
    onPadState: (cb: (pads: PadState[]) => void): Promise<UnlistenFn> =>
      listen<{ pads: PadState[] }>('emu://gamepad-state', e => cb(e.payload.pads)),
  },
}
