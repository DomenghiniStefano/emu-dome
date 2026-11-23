import { describe, expect, it } from 'vitest'
import type { ScreenStatus } from '@/emu'
import { MoreId, StripId, canMakeQuick, moreRows, stripButtons } from './phoneMenu'
import { QuickMenuView, phoneParent } from './view'

describe('stripButtons', () => {
  const off = { rewind: false, fastForward: false, muted: false }

  it('shows three buttons when rewind is off for the game', () => {
    expect(stripButtons(off).map(b => b.id)).toEqual([
      StripId.FastForward,
      StripId.Mute,
      StripId.More,
    ])
  })

  it('puts «Riavvolgi» first when rewind is on', () => {
    expect(stripButtons({ ...off, rewind: true }).map(b => b.id)).toEqual([
      StripId.Rewind,
      StripId.FastForward,
      StripId.Mute,
      StripId.More,
    ])
  })

  it('names each button with a verb, «Riattiva» while muted, and lights what is on', () => {
    const plain = stripButtons(off)
    expect(plain.map(b => b.label)).toEqual(['qm.verb.ff', 'qm.verb.mute', 'qm.moreLabel'])
    expect(plain.map(b => b.pressed)).toEqual([false, false, undefined])
    const on = stripButtons({ rewind: true, fastForward: true, muted: true })
    expect(on.map(b => b.label)).toEqual([
      'qm.verb.rewind',
      'qm.verb.ff',
      'qm.verb.unmute',
      'qm.moreLabel',
    ])
    expect(on.map(b => b.pressed)).toEqual([undefined, true, true, undefined])
  })
})

describe('moreRows', () => {
  const status = (s: Partial<ScreenStatus>): ScreenStatus => s as ScreenStatus

  it('lists the pages, the screenshot, restart and «Chiudi il gioco» last on a one-screen game', () => {
    const rows = moreRows({ dualScreen: false, screens: null })
    expect(rows.map(r => r.id)).toEqual([
      MoreId.Saves,
      MoreId.Speed,
      MoreId.Screens,
      MoreId.Controls,
      MoreId.Layout,
      MoreId.Screenshot,
      MoreId.Restart,
      MoreId.Close,
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
    expect(rows.at(-1)?.danger).toBe(true)
    expect(rows.filter(r => r.danger)).toHaveLength(1)
  })

  it('adds swap and the lid on a DS, pressed as the screens are', () => {
    const rows = moreRows({
      dualScreen: true,
      screens: status({ swapped: true, lidSupported: true, lidClosed: false }),
    })
    expect(rows.map(r => r.id)).toContain(MoreId.CloseLid)
    expect(rows.find(r => r.id === MoreId.SwapScreens)?.pressed).toBe(true)
    expect(rows.find(r => r.id === MoreId.CloseLid)?.pressed).toBe(false)
    expect(rows.at(-1)?.id).toBe(MoreId.Close)
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
  it('can be made quick only on a numbered slot the shortcuts reach, not the quick one', () => {
    expect(canMakeQuick(0, 1)).toBe(false)
    expect(canMakeQuick(1, 1)).toBe(false)
    expect(canMakeQuick(2, 1)).toBe(true)
    expect(canMakeQuick(9, 1)).toBe(true)
    expect(canMakeQuick(10, 1)).toBe(false)
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
