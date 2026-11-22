import { describe, expect, it } from 'vitest'
import { canRedo, canUndo, commit, historyOf, redo, undo } from './editorHistory'
import { fitToolbar, TOOLS, type ToolbarWidths } from './editorToolbar'

describe("the editor's history", () => {
  it('undoes and redoes, and a new change forgets the steps forward', () => {
    let h = historyOf(1)
    expect(canUndo(h)).toBe(false)
    h = commit(h, 2)
    h = commit(h, 3)
    h = undo(h)
    expect(h.present).toBe(2)
    expect(canRedo(h)).toBe(true)
    h = redo(h)
    expect(h.present).toBe(3)
    h = undo(undo(h))
    h = commit(h, 9)
    expect(h.present).toBe(9)
    expect(canRedo(h)).toBe(false)
    expect(undo(h).present).toBe(1)
  })

  it('takes the same layout again as no change', () => {
    const h = commit(historyOf({ a: 1 }), { a: 1 })
    expect(canUndo(h)).toBe(false)
    expect(undo(historyOf(1))).toEqual(historyOf(1))
    expect(redo(historyOf(1))).toEqual(historyOf(1))
  })
})

describe("the editor's toolbar on a phone", () => {
  // The Italian labels, the longest (measured in a headless Chromium at the touch density):
  // icon buttons 44, «Aggiungi» 96, «Dimensione» 108, «Trasparenza» 116, «Comportamento» 140,
  // «Togli» 72, «Torna alla disposizione predefinita» 268, «Esci senza salvare» 164,
  // «Riduci» 80, «Salva» 76; 4 between two buttons.
  const widths: ToolbarWidths = {
    tools: {
      undo: 44,
      redo: 44,
      add: 96,
      size: 108,
      opacity: 116,
      behaviour: 140,
      remove: 72,
      reset: 268,
      discard: 164,
    },
    more: 44,
    collapse: 80,
    save: 76,
    gap: 4,
  }
  /** The toolbar's inside width on a phone `w` px wide: 16 px from each edge, 6 px of padding
   * and its 1 px edge each side. */
  const inside = (w: number) => w - 2 * 16 - 2 * 6 - 2

  const used = (shown: readonly string[], more: boolean) => {
    const parts = [
      ...shown.map(t => widths.tools[t as keyof ToolbarWidths['tools']]),
      ...(more ? [widths.more] : []),
      widths.collapse,
      widths.save,
    ]
    return parts.reduce((a, b) => a + b, 0) + widths.gap * (parts.length - 1)
  }

  for (const phone of [320, 360, 390, 430, 740, 844]) {
    it(`never overflows at ${phone} px, and loses nothing`, () => {
      const fit = fitToolbar(inside(phone), widths)
      expect(used(fit.shown, fit.more.length > 0)).toBeLessThanOrEqual(inside(phone))
      expect([...fit.shown, ...fit.more].sort()).toEqual([...TOOLS].sort())
    })
  }

  it('keeps undo and redo longest, and moves the rare actions first', () => {
    expect(fitToolbar(inside(320), widths)).toEqual({
      shown: ['undo'],
      more: ['redo', 'add', 'size', 'opacity', 'behaviour', 'remove', 'reset', 'discard'],
    })
    expect(fitToolbar(inside(360), widths).shown).toEqual(['undo', 'redo'])
    expect(fitToolbar(inside(430), widths).shown).toEqual(['undo', 'redo'])
    expect(fitToolbar(inside(844), widths)).toEqual({
      shown: ['undo', 'redo', 'add', 'size', 'opacity', 'behaviour'],
      more: ['remove', 'reset', 'discard'],
    })
    expect(fitToolbar(5000, widths)).toEqual({ shown: [...TOOLS], more: [] })
  })
})
