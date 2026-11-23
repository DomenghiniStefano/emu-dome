// Mock data for the in-game, shell and controls sections of the Kit page: the shapes the bridge
// sends (generated types), with fixed dates so the page reads the same every time. Pictures are
// inline SVG, so no request leaves the page.
import { AUTO_SLOT, type SlotInfo, type Speed } from '@/emu'
import type { ScreenCard } from '@/components/quick-menu/screenCards'
import { speedChoices } from '@/components/quick-menu/speedChoices'
import { quickTools, type ToolContext } from '@/components/quick-menu/tools'

/** 21 November 2026, 18:30 UTC: every date on these sections counts from it. */
export const FIXED = Date.UTC(2026, 10, 21, 18, 30)
const HOUR = 3_600_000

/** A tiny game frame: a sky and a ground, crisp pixels. */
export const thumb = (sky: string, ground: string) =>
  'data:image/svg+xml,' +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="16" shape-rendering="crispEdges"><rect width="24" height="16" fill="${sky}"/><rect y="11" width="24" height="5" fill="${ground}"/><rect x="10" y="6" width="4" height="5" fill="gold"/></svg>`,
  )

export const SLOTS: SlotInfo[] = [
  { slot: AUTO_SLOT, savedAt: FIXED - HOUR / 6, thumbUrl: thumb('steelblue', 'seagreen') },
  { slot: 1, savedAt: FIXED - 26 * HOUR, thumbUrl: thumb('midnightblue', 'slategray') },
  { slot: 2, savedAt: FIXED - 3 * HOUR, thumbUrl: thumb('teal', 'darkolivegreen') },
  { slot: 3, savedAt: null, thumbUrl: null },
  { slot: 4, savedAt: null, thumbUrl: null },
]

export const FF_4: Speed = { kind: 'factor', value: 4 }
export const SLOW_HALF: Speed = { kind: 'factor', value: 0.5 }
export const NORMAL: Speed = { kind: 'factor', value: 1 }
export const MAX: Speed = { kind: 'max' }
export const SPEED_CHOICES = speedChoices(FF_4, SLOW_HALF)

const CONTEXT: ToolContext = {
  phone: false,
  dualScreen: false,
  muted: false,
  fullscreen: false,
  screens: null,
}
/** GBA: four tools, Muto on. */
export const TOOLS_GBA = quickTools({ ...CONTEXT, muted: true })
/** NDS on Azahar-like terms: six tools, the lid unsupported (disabled with the reason). */
export const TOOLS_DUAL = quickTools({
  ...CONTEXT,
  dualScreen: true,
  fullscreen: true,
  screens: {
    dualScreen: true,
    layout: 'vertical',
    cycle: ['vertical', 'side'],
    swapped: true,
    swapHold: false,
    hasCustom: false,
    lidSupported: false,
    lidClosed: false,
  },
})

const noop = () => {}
const card = (over: Partial<ScreenCard>): ScreenCard => ({
  key: 'video.scale',
  name: { key: 'settings.video.scale.name' },
  value: { key: 'settings.video.scale.option.integer' },
  sub: { key: 'settingsUi.sameAs', level: { key: 'settingsUi.scope.global' } },
  trial: false,
  prev: noop,
  next: noop,
  ...over,
})

export const SCREEN_CARDS = {
  plain: card({}),
  trial: card({
    key: 'video.filter',
    name: { key: 'settings.video.filter.name' },
    value: { key: 'settings.video.filter.option.soft' },
    sub: { key: 'qm.source.session' },
    trial: true,
    next: null,
  }),
  locked: card({
    key: 'fx.fxpick',
    name: { key: 'settings.fx.fxpick.name' },
    value: { key: 'settings.fx.fxpick.option.crt_curved' },
    sub: { key: 'qm.noNative' },
    prev: null,
    next: null,
  }),
}

/** The 12 cells of «Prova i pulsanti», by position, as a generic controller names them. */
export const TESTER_CELLS = ['L1', 'L2', 'R2', 'R1', '↑', '←', '→', '↓', 'A', 'B', 'X', 'Y'].map(
  label => ({ id: label, label }),
)
