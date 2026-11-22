// Mock games for the library group of the Kit page: one per console, the awkward cases the real
// library meets (a long title, no cover, the ROM icon, the user's image, a missing file, never
// played, a 3DS title ID). Never sent to the backend: the sections only render them.
import type { GameScreenshot, LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import type { OverrideRow } from '@/components/details/gameOverrides'

const NOW = Date.now()
const HOUR = 3600
const DAY = 86_400_000

/** A stand-in picture as an SVG data URL: a gradient sky, a sun and a hill. */
function picture(w: number, h: number, from: string, to: string): string {
  return (
    'data:image/svg+xml,' +
    encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">` +
        `<defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs>` +
        `<rect width="${w}" height="${h}" fill="url(#g)"/>` +
        `<circle cx="${w * 0.72}" cy="${h * 0.28}" r="${Math.min(w, h) * 0.12}" fill="#ffe9a8"/>` +
        `<rect x="0" y="${h * 0.7}" width="${w}" height="${h * 0.3}" fill="#3a7a44"/>` +
        `</svg>`,
    )
  )
}

/** A 32×32 pixel-art icon, as an NDS ROM carries. */
const ICON =
  'data:image/svg+xml,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" shape-rendering="crispEdges">' +
      '<rect width="32" height="32" fill="#1d3557"/><rect x="8" y="6" width="16" height="20" fill="#f1c40f"/>' +
      '<rect x="12" y="10" width="3" height="3" fill="#000"/><rect x="18" y="10" width="3" height="3" fill="#000"/>' +
      '<rect x="12" y="18" width="9" height="2" fill="#c0392b"/></svg>',
  )

export const USER_COVER = picture(600, 900, '#24345c', '#8a4fa3')
export const SHOT_GBA = picture(240, 160, '#2f6fb0', '#9fd3f5')
export const SHOT_NDS = picture(256, 384, '#5b2a86', '#e79bd1')
export const SHOT_3DS = picture(400, 480, '#7a2e1f', '#f0b27a')

function game(over: Partial<LibraryGame> & Pick<LibraryGame, 'id' | 'console' | 'title'>) {
  const base: LibraryGame = {
    path: `D:/Giochi/${over.console}/${over.title}.${over.console}`,
    folder: `D:/Giochi/${over.console}`,
    size: 8 * 1024 * 1024,
    available: true,
    playTimeSecs: 2 * HOUR,
    lastPlayed: NOW - DAY,
    addedAt: NOW - 30 * DAY,
    cover: 'generated',
    coverUrl: null,
    hasIcon: false,
    colour: '#3d4a6b',
    favourite: false,
    hidden: false,
    lastStateThumbUrl: null,
    renamed: false,
    romTitle: over.title,
    code: 'ABCD',
    version: 0,
    region: 'europe',
    core: 'mgba',
    ...over,
  }
  return base
}

export const GBA = game({
  id: 'gba:3f0a91c2d4e5b6a7980112233445566778899aab',
  console: 'gba',
  title: 'Pocket Quest Advance',
  colour: '#3d4a6b',
  playTimeSecs: 14 * HOUR + 1200,
  lastPlayed: NOW - 2 * HOUR,
  code: 'BPQE',
})

export const NDS_ICON = game({
  id: 'nds:9c1b2a3d4e5f60718293a4b5c6d7e8f901234567',
  console: 'nds',
  title: 'Dual Screen Tales',
  colour: '#5b3d6b',
  cover: 'icon',
  coverUrl: ICON,
  hasIcon: true,
  core: 'melondsds',
  code: 'ADST',
  playTimeSecs: 40 * HOUR,
  lastPlayed: NOW - 3 * DAY,
})

export const N3DS_USER = game({
  id: '3ds:0004000000155a00000000000000000000000000',
  console: '3ds',
  title: 'Mountain Ranger 3D',
  colour: '#6b4a3d',
  cover: 'user',
  coverUrl: USER_COVER,
  hasIcon: true,
  core: 'azahar',
  code: 'CTR-P-AMRP',
  version: 1040,
  region: 'world',
  size: 2_147_483_648,
  playTimeSecs: 3 * HOUR,
  lastPlayed: NOW - 10 * DAY,
})

/** A long title without spaces where it breaks, as many translated ROMs have. */
export const GB_LONG = game({
  id: 'gb:00112233445566778899aabbccddeeff00112233',
  console: 'gb',
  title:
    'The Extraordinarily Long Adventures of Captain Pixel and the Kingdom of Forgotten Cartridges',
  colour: '#3d6b52',
  lastPlayed: null,
  playTimeSecs: 0,
  code: null,
  version: null,
  region: null,
})

/** Its folder is on a disk that is not connected (E01). */
export const GBC_MISSING = game({
  id: 'gbc:aabbccddeeff00112233445566778899aabbccdd',
  console: 'gbc',
  title: 'Colour Runner DX',
  colour: '#6b3d3d',
  available: false,
  path: 'E:/Chiavetta/Retro/Game Boy Color/Colour Runner DX (Europe) (En,Fr,De,Es,It).gbc',
  folder: 'E:/Chiavetta/Retro/Game Boy Color',
  playTimeSecs: 25 * 60,
  lastPlayed: NOW - 40 * DAY,
})

/** A 3DS game nobody played yet, its generated cover. */
export const N3DS_NEW = game({
  id: '3ds:00040000001b5100000000000000000000000000',
  console: '3ds',
  title: 'Snow Garden',
  colour: '#3d6b6b',
  core: 'azahar',
  code: 'CTR-P-BSGP',
  version: 0,
  region: 'europe',
  playTimeSecs: 0,
  lastPlayed: null,
})

export const ALL_GAMES = [GBA, NDS_ICON, N3DS_USER, GB_LONG, GBC_MISSING, N3DS_NEW]

export const RESUME_AUTO: ResumePoint = { slot: 0, thumbUrl: SHOT_GBA, none: false }
export const RESUME_SLOT: ResumePoint = { slot: 3, thumbUrl: SHOT_NDS, none: false }
export const RESUME_IN_GAME: ResumePoint = { slot: null, thumbUrl: null, none: false }
export const RESUME_NONE: ResumePoint = { slot: null, thumbUrl: null, none: true }

export const SHOTS: GameScreenshot[] = [
  { name: '2026-11-20_21-14-03.png', url: SHOT_GBA, takenAt: NOW - DAY, width: 240, height: 160 },
  {
    name: '2026-11-18_18-02-44.png',
    url: SHOT_NDS,
    takenAt: NOW - 3 * DAY,
    width: 256,
    height: 384,
  },
]

export const OVERRIDES: OverrideRow[] = [
  {
    key: 'video.aspect',
    name: 'Proporzioni',
    where: 'Video',
    value: '16:9 allargato',
    base: '4:3',
  },
  {
    key: 'speed.rew',
    name: 'Riavvolgimento',
    where: 'Velocità',
    value: 'Disattivo',
    base: null,
  },
  {
    key: 'fx.fxpick',
    name: 'Effetto schermo con un nome molto lungo che non entra nella riga',
    where: 'Effetti schermo',
    value: 'CRT curvo con bagliore e maschera d’ombra',
    base: 'Nessuno',
  },
]
