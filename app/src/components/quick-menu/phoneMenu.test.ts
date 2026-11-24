import { describe, expect, it } from 'vitest'
import type { ScreenStatus } from '@/emu'
import {
  MoreId,
  SIDE_STRIP_ONE_ROW,
  StripId,
  canMakeQuick,
  moreRows,
  stripButtons,
  stripTwoRows,
} from './phoneMenu'
import { QuickMenuView, phoneParent } from './view'

describe('stripButtons', () => {
  const off = { rewind: false, fastForward: false, muted: false }

  it('shows four buttons when rewind is off for the game', () => {
    expect(stripButtons(off).map(b => b.id)).toEqual([
      StripId.FastForward,
      StripId.Mute,
      StripId.Close,
      StripId.More,
    ])
  })

  it('puts «Riavvolgi» first when rewind is on', () => {
    expect(stripButtons({ ...off, rewind: true }).map(b => b.id)).toEqual([
      StripId.Rewind,
      StripId.FastForward,
      StripId.Mute,
      StripId.Close,
      StripId.More,
    ])
  })

  it('names each button with a verb, «Riattiva» while muted, and lights what is on', () => {
    const plain = stripButtons(off)
    expect(plain.map(b => b.label)).toEqual([
      'qm.verb.ff',
      'qm.verb.mute',
      'qm.verb.close',
      'qm.moreLabel',
    ])
    expect(plain.map(b => b.pressed)).toEqual([false, false, undefined, undefined])
    const on = stripButtons({ rewind: true, fastForward: true, muted: true })
    expect(on.map(b => b.label)).toEqual([
      'qm.verb.rewind',
      'qm.verb.ff',
      'qm.verb.unmute',
      'qm.verb.close',
      'qm.moreLabel',
    ])
    expect(on.map(b => b.pressed)).toEqual([undefined, true, true, undefined, undefined])
  })

  it('has «Chiudi il gioco» one tap away (#121): in danger ink, named in full, «Altro» last', () => {
    for (const rewind of [false, true]) {
      const list = stripButtons({ ...off, rewind })
      const close = list.filter(b => b.danger)
      expect(close.map(b => b.id)).toEqual([StripId.Close])
      expect(close[0]?.name).toBe('qm.close')
      expect(list.at(-1)?.id).toBe(StripId.More)
      expect(list.at(-2)?.id).toBe(StripId.Close)
    }
  })
})

describe('stripTwoRows', () => {
  it('splits the strip only on its side with five buttons', () => {
    expect(stripTwoRows(true, SIDE_STRIP_ONE_ROW)).toBe(false)
    expect(stripTwoRows(true, SIDE_STRIP_ONE_ROW + 1)).toBe(true)
    expect(stripTwoRows(false, SIDE_STRIP_ONE_ROW + 1)).toBe(false)
  })
})

describe('moreRows', () => {
  const status = (s: Partial<ScreenStatus>): ScreenStatus => s as ScreenStatus

  it('lists the pages, the screenshot and restart last on a one-screen game', () => {
    const rows = moreRows({ dualScreen: false, screens: null })
    expect(rows.map(r => r.id)).toEqual([
      MoreId.Saves,
      MoreId.Speed,
      MoreId.Screens,
      MoreId.Controls,
      MoreId.Layout,
      MoreId.Screenshot,
      MoreId.Restart,
    ])
    expect(rows.filter(r => r.page).map(r => r.page)).toEqual([
      QuickMenuView.Slots,
      QuickMenuView.Speed,
      QuickMenuView.Screens,
      QuickMenuView.Controls,
      QuickMenuView.TouchEditor,
    ])
    // «Modifica la disposizione» (Trello #111): a verb, the touch editor's icon.
    expect(rows.find(r => r.id === MoreId.Layout)?.label).toBe('qm.editLayout')
  })

  it('adds swap and the lid on a DS, pressed as the screens are', () => {
    const rows = moreRows({
      dualScreen: true,
      screens: status({ swapped: true, lidSupported: true, lidClosed: false }),
    })
    expect(rows.map(r => r.id)).toContain(MoreId.CloseLid)
    expect(rows.find(r => r.id === MoreId.SwapScreens)?.pressed).toBe(true)
    expect(rows.find(r => r.id === MoreId.CloseLid)?.pressed).toBe(false)
    expect(rows.at(-1)?.id).toBe(MoreId.Restart)
  })

  it('leaves the lid out where the core has none (3DS)', () => {
    const rows = moreRows({
      dualScreen: true,
      screens: status({ swapped: false, lidSupported: false, lidClosed: false }),
    })
    expect(rows.map(r => r.id)).toContain(MoreId.SwapScreens)
    expect(rows.map(r => r.id)).not.toContain(MoreId.CloseLid)
  })
})

describe('the quick slot', () => {
  it('can be made quick on any numbered slot, not the automatic one nor the quick one', () => {
    expect(canMakeQuick(0, 1)).toBe(false)
    expect(canMakeQuick(1, 1)).toBe(false)
    expect(canMakeQuick(2, 1)).toBe(true)
    expect(canMakeQuick(9, 1)).toBe(true)
    // The tenth slot too (#149): the shortcuts cycle through 1–9, «Rendi rapido» reaches it.
    expect(canMakeQuick(10, 1)).toBe(true)
    expect(canMakeQuick(10, 10)).toBe(false)
  })
})

describe('phoneParent', () => {
  it('sends «Altro» to the main screen and its pages to «Altro»', () => {
    expect(phoneParent(QuickMenuView.More)).toBe(QuickMenuView.Home)
    expect(phoneParent(QuickMenuView.Slots)).toBe(QuickMenuView.More)
    expect(phoneParent(QuickMenuView.Speed)).toBe(QuickMenuView.More)
    expect(phoneParent(QuickMenuView.Restart)).toBe(QuickMenuView.More)
  })
})
