import { describe, expect, it } from 'vitest'
import { commit, historyOf, undo } from './editorHistory'
import {
  centreGuides,
  elementIds,
  moveScreen,
  nextElement,
  ratioOf,
  resizeScreen,
  screensShown,
  startArea,
  withScreen,
} from './editorScreens'
import { NO_INSET } from '@/components/game/inset'

const area = { width: 360, height: 780 }
const top = { id: 'top', x: 0, y: 0, w: 1, h: 0.35 } as const
const bottom = { id: 'bottom', x: 0, y: 0.45, w: 1, h: 0.35 } as const
const menu = { id: 'menu', x: 0.5, y: 0.9, size: 100, opacity: 100, behaviour: 'tap' } as const
const a = { ...menu, id: 'a' } as const

describe("the editor's screens", () => {
  it('picks the screens first, then the controls, wrapping', () => {
    const ids = elementIds({ controls: [menu, a], screens: [] }, [top, bottom])
    expect(ids).toEqual(['top', 'bottom', 'menu', 'a'])
    expect(nextElement(ids, 'a', 1)).toBe('top')
    expect(nextElement(ids, 'top', -1)).toBe('a')
    expect(nextElement([], null, 1)).toBe(null)
  })

  it('moves one screen and keeps them all, or none', () => {
    const moved = { ...top, y: 0.1 }
    expect(withScreen({ controls: [menu], screens: [] }, [top, bottom], moved).screens).toEqual([
      moved,
      bottom,
    ])
    expect(screensShown({ controls: [menu], screens: [] }, [top, bottom])).toEqual([top, bottom])
    const h = commit(
      historyOf({ controls: [menu], screens: [] }),
      withScreen({ controls: [menu], screens: [] }, [top, bottom], moved),
    )
    expect(undo(h).present.screens).toEqual([])
  })

  it('snaps a dragged screen to the 8 px grid and keeps it inside', () => {
    const main = { id: 'main', x: 0, y: 0, w: 0.5, h: 0.25 } as const
    expect(moveScreen(main, 101, 203, area, true)).toEqual({ ...main, x: 0.2889, y: 0.2564 })
    expect(moveScreen(main, 400, 0, area, true).x).toBe(0.5)
  })

  it('resizes keeping the screen’s proportions, never under the minimum', () => {
    const main = { id: 'main', x: 0, y: 0, w: 0.5, h: 0.25 } as const
    expect(resizeScreen(main, 360, 0, true, 1.5, area)).toEqual({ ...main, w: 1, h: 0.3077 })
    expect(resizeScreen(main, 180, 390, false, 1.5, area)).toEqual({ ...main, w: 0.5, h: 0.5 })
    expect(resizeScreen(main, 10, 10, false, 1.5, area).w).toBe(0.08)
  })

  it('takes the ratio of the picture the frame shows', () => {
    const shapes = [
      { id: 'top', width: 400, height: 240 },
      { id: 'bottom', width: 320, height: 240 },
    ] as const
    expect(ratioOf(shapes, 'top', false)).toBeCloseTo(400 / 240)
    expect(ratioOf(shapes, 'top', true)).toBeCloseTo(320 / 240)
  })

  it('starts the screens inside the edges the controls keep clear, as the game is placed', () => {
    const inset = { ...NO_INSET, bottom: 234, align: 'bottom' as const }
    expect(startArea(area, inset)).toEqual({
      width: 360,
      height: 780,
      inset: { left: 0, top: 0, right: 0, bottom: 0.3, align: 'bottom', between: 0 },
    })
  })

  it('shows the window-centre guides when a screen is centred', () => {
    expect(centreGuides({ id: 'main', x: 0.25, y: 0, w: 0.5, h: 0.2 }, area)).toEqual({
      across: true,
      down: false,
    })
  })
})
