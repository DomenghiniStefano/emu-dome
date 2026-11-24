import { describe, expect, it } from 'vitest'
import type { TouchLayout } from '@/emu'
import { commit, historyOf, undo } from './editorHistory'
import { draftOf, layoutsToSave, turnHint, withDraft, type Drafts } from './orientationDrafts'

const layout = (x: number): TouchLayout => ({
  controls: [{ id: 'menu', x, y: 0.5, size: 100, opacity: 100, behaviour: 'tap' }],
  screens: [],
})
const untouched = (x: number) => ({ history: historyOf(layout(x)), selected: null })
const changed = (from: number, to: number) => ({
  history: commit(historyOf(layout(from)), layout(to)),
  selected: 'menu' as const,
})

describe('one draft per orientation', () => {
  it('keeps each orientation as it was left, history and pick included', () => {
    const drafts = withDraft({}, 'portrait', changed(0.1, 0.2))
    const both = withDraft(drafts, 'landscape', untouched(0.3))
    expect(draftOf(both, 'portrait')).toEqual(changed(0.1, 0.2))
    expect(draftOf(both, 'landscape')).toEqual(untouched(0.3))
    expect(draftOf({}, 'landscape')).toBeUndefined()
  })
})

describe('what «Salva» writes', () => {
  it('every orientation changed in this session', () => {
    const drafts: Drafts = { portrait: changed(0.1, 0.2), landscape: changed(0.3, 0.4) }
    expect(layoutsToSave(drafts, 'portrait')).toEqual({
      portrait: layout(0.2),
      landscape: layout(0.4),
    })
  })

  it('not an orientation only looked at', () => {
    const drafts: Drafts = { portrait: untouched(0.1), landscape: changed(0.3, 0.4) }
    expect(layoutsToSave(drafts, 'portrait')).toEqual({ portrait: null, landscape: layout(0.4) })
  })

  it('a change undone back is still a change made in this session', () => {
    const back = { history: undo(changed(0.1, 0.2).history), selected: null }
    expect(layoutsToSave({ portrait: back }, 'portrait')).toEqual({
      portrait: layout(0.1),
      landscape: null,
    })
  })

  it('the orientation on screen when nothing was changed', () => {
    const drafts: Drafts = { portrait: untouched(0.1), landscape: untouched(0.3) }
    expect(layoutsToSave(drafts, 'landscape')).toEqual({ portrait: null, landscape: layout(0.3) })
  })
})

describe('the hint to turn the phone', () => {
  it('names the orientation not shown yet, and goes once both were', () => {
    expect(turnHint(['portrait'], 'portrait')).toBe('landscape')
    expect(turnHint(['landscape'], 'landscape')).toBe('portrait')
    expect(turnHint(['portrait', 'landscape'], 'landscape')).toBeNull()
  })
})
