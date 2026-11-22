// Mock data for the settings and saves sections of the Kit page: catalogue entries as Rust sends
// them (`settings_schema`), rows at a level, games and saves. Nothing here reaches the backend.
import type {
  LibraryGame,
  SettingEntry,
  SettingKey,
  SettingLevel,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import type { ConflictVersion } from '@/components/saves/conflict'

export const ALL_LEVELS: SettingLevel[] = ['default', 'global', 'console', 'game', 'session']
export const GLOBAL_ONLY: SettingLevel[] = ['default', 'global']

/** A catalogue entry, as `settings_schema` gives it (a toggle at every level unless told). */
export const entry = (key: SettingKey, over: Partial<SettingEntry> = {}): SettingEntry => ({
  key,
  page: key.split('.')[0] ?? '',
  kind: { type: 'toggle' },
  default: false,
  levels: ALL_LEVELS,
  perDevice: false,
  platform: 'all',
  phase: 0,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
  wired: true,
  unsupportedConsoles: [],
  ...over,
})

const seg = (options: string[]) =>
  ({ type: 'choice', style: 'segments', options, moreAtRuntime: false }) as const
const list = (options: string[]) =>
  ({ type: 'choice', style: 'list', options, moreAtRuntime: false }) as const
const cards = (options: string[]) =>
  ({ type: 'choice', style: 'cards', options, moreAtRuntime: false }) as const

// The entries the sections show, with the catalogue's kinds, defaults and levels (catalog.rs).
export const E = {
  toggle: entry('library.versions', { default: true, levels: GLOBAL_ONLY }),
  rewind: entry('speed.rew', { default: true }),
  fps: entry('video.fps', { levels: GLOBAL_ONLY, wired: false }),
  scale: entry('video.scale', { kind: seg(['fit', 'integer', 'fill']), default: 'fit' }),
  theme: entry('general.theme', {
    kind: seg(['system', 'dark', 'black', 'light']),
    default: 'system',
    levels: GLOBAL_ONLY,
  }),
  autoq: entry('saves.autoq', { kind: seg(['1min', '5min', 'never']), default: '5min' }),
  latency: entry('audio.lat', {
    kind: seg(['low', 'balanced', 'safe']),
    default: 'balanced',
    levels: GLOBAL_ONLY,
    perDevice: true,
  }),
  lang: entry('general.lang', {
    kind: list(['system', 'it', 'en']),
    default: 'system',
    levels: GLOBAL_ONLY,
  }),
  gbpal: entry('consoles.gbpal', {
    kind: list(['green', 'pocket', 'light', 'gbc']),
    default: 'green',
    levels: ['default', 'console', 'game', 'session'],
  }),
  frameskip: entry('perf.frameskip', {
    kind: list(['auto', 'never', '1', '2', '3', '4']),
    default: 'auto',
    perDevice: true,
  }),
  volume: entry('audio.vol', {
    kind: { type: 'slider', min: 0, max: 100, step: 1, unit: 'percent' },
    default: 80,
    levels: ['default', 'global', 'game', 'session'],
  }),
  backups: entry('saves.backups', {
    kind: { type: 'slider', min: 0, max: 10, step: 1, unit: 'copies' },
    default: 5,
    levels: GLOBAL_ONLY,
  }),
  cpuclock: entry('perf.cpuclock', {
    kind: { type: 'slider', min: 50, max: 400, step: 25, unit: 'percent' },
    default: 100,
    unsupportedConsoles: ['gb', 'gbc', 'gba', 'nds'],
  }),
  dsname: entry('consoles.dsname', {
    kind: { type: 'text' },
    default: '',
    levels: ['default', 'console'],
  }),
  rtcdate: entry('saves.rtcdate', {
    kind: { type: 'dateTime', minYear: 2000, maxYear: 2099 },
    default: '2004-01-01T00:00',
  }),
  surroundImg: entry('video.surroundImg', { kind: { type: 'file' }, default: '' }),
  fxfile: entry('fx.fxfile', { kind: { type: 'file' }, default: '', levels: GLOBAL_ONLY }),
  remap: entry('pad.remap', { kind: { type: 'bindings', table: 'remap' }, default: [] }),
  layout: entry('screens.custom', {
    kind: { type: 'screenLayout' },
    default: [],
    levels: ['default', 'console', 'game', 'session'],
  }),
  importSave: entry('saves.import', {
    kind: { type: 'action', style: 'button' },
    default: null,
    wired: false,
  }),
  turbo: entry('pad.turbo', {
    kind: { type: 'action', style: 'link' },
    default: null,
    wired: false,
  }),
  licences: entry('about.lic', {
    kind: { type: 'action', style: 'link' },
    default: null,
    levels: GLOBAL_ONLY,
  }),
  cycle: entry('screens.cycle', {
    kind: { type: 'multi', options: ['vertical', 'side', 'big_small', 'top_only', 'custom'] },
    default: ['vertical', 'side', 'big_small', 'top_only'],
    levels: ['default', 'console', 'game', 'session'],
  }),
  accent: entry('general.accent', {
    kind: { type: 'colour', swatches: ['#faa40f', '#7152f7', '#2f7ae5', '#1f9d8a', '#c2417a'] },
    default: '#faa40f',
    levels: GLOBAL_ONLY,
  }),
  surround: entry('video.surround', {
    kind: cards(['black', 'cover', 'blur', 'builtin', 'image']),
    default: 'black',
  }),
  preset: entry('presets.preset', {
    kind: { type: 'apply', options: ['faithful', 'performance', 'quality', 'battery'] },
    default: null,
    levels: GLOBAL_ONLY,
    wired: false,
  }),
  folders: entry('library.folders', {
    kind: { type: 'folders' },
    default: [],
    levels: GLOBAL_ONLY,
    perDevice: true,
  }),
} satisfies Record<string, SettingEntry>

export const schema: SettingsSchema = {
  device: 'desktop',
  currentPhase: 1,
  pages: [],
  groups: [],
  sections: [],
  entries: Object.values(E),
}

/** A row's value and where it comes from (default: the schema's default, from Default). */
export const row = (
  e: SettingEntry,
  value: SettingValue | null = e.default,
  source: SettingLevel = 'default',
): SettingRow => ({ key: e.key, value, source, changed: source !== 'default' })

/** A choice's options as the controls take them (RowOption), from [id, text] pairs. */
export const options = (...pairs: [string, string, string?][]) =>
  pairs.map(([id, text, desc = '']) => ({ id, text, desc, notYet: false }))

/** The scope bar's words for a level, as useSettingsScope says them (in Italian, the board's). */
export const levelName = (l: SettingLevel): string =>
  l === 'console'
    ? 'Giochi Game Boy Advance'
    : l === 'game'
      ? 'Solo questo gioco'
      : 'Tutti i giochi'

// A pixel-art stand-in for a game's last screen: a few blocks on a sky, as an SVG data URL (a
// thumbnail has to be an image URL; the colours are the picture's, not the interface's).
const svg = (body: string, w = 240, h = 160) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" shape-rendering="crispEdges">${body}</svg>`,
  )}`
export const THUMB_GBA = svg(
  '<rect width="240" height="160" fill="#6ab0e8"/><rect y="120" width="240" height="40" fill="#4f9a3a"/>' +
    '<rect y="128" width="240" height="32" fill="#8a5a2b"/><rect x="40" y="96" width="16" height="24" fill="#d8402f"/>' +
    '<rect x="150" y="80" width="48" height="16" fill="#e8b23a"/><rect x="20" y="20" width="40" height="12" fill="#fff"/>',
)
export const THUMB_NDS = svg(
  '<rect width="256" height="192" fill="#20243a"/><rect y="150" width="256" height="42" fill="#3a6e4f"/>' +
    '<rect x="100" y="110" width="20" height="40" fill="#f0d070"/><rect x="180" y="30" width="12" height="12" fill="#fff"/>',
  256,
  192,
)
/** A portrait screen (a 3DS game turned, a phone screenshot): letterboxed in a 4:3 tile. */
export const THUMB_TALL = svg(
  '<rect width="120" height="200" fill="#2a1a3a"/><rect x="30" y="140" width="60" height="60" fill="#c2417a"/>',
  120,
  200,
)

const game = (over: Partial<LibraryGame>): LibraryGame => ({
  id: 'gba:0123abcd',
  console: 'gba',
  title: 'Pocket Quest',
  path: 'D:/Giochi/pocket-quest.gba',
  folder: 'D:/Giochi',
  size: 8_388_608,
  available: true,
  playTimeSecs: 3600 * 12,
  lastPlayed: null,
  addedAt: 0,
  cover: 'generated',
  coverUrl: null,
  hasIcon: false,
  colour: '#2f5d8a',
  favourite: false,
  hidden: false,
  lastStateThumbUrl: null,
  renamed: false,
  romTitle: 'POCKETQUEST',
  code: 'APQE',
  version: 0,
  region: 'europe',
  core: 'mgba',
  ...over,
})

export const GAMES = {
  short: game({}),
  long: game({
    id: 'nds:89ab0011',
    console: 'nds',
    title: 'La leggenda delle sette lune: edizione speciale del decimo anniversario',
    colour: '#5b3f8c',
    core: 'melondsds',
  }),
  gb: game({ id: 'gb:00ff00ff', console: 'gb', title: 'Tiny Tetromino', colour: '#4f6b2e' }),
}

const NOW = Date.UTC(2026, 10, 22, 18, 40)
export const CONFLICT: [ConflictVersion, ConflictVersion] = [
  {
    device: 'Questo computer',
    savedAt: NOW - 3 * 3600_000,
    playTimeSecs: 3600 * 12 + 600,
    size: 131_072,
    thumbUrl: THUMB_GBA,
  },
  {
    device: 'Galaxy A71',
    savedAt: NOW - 20 * 60_000,
    playTimeSecs: null,
    size: 131_072,
    thumbUrl: null,
  },
]
