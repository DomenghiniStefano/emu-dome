import { describe, expect, it, vi } from 'vitest'
import { choiceNext, coverChoices } from './coverChoice'
import { screenshotName } from './useCoverRequest'

// libraryView reads AUTO_SLOT from the bridge, which needs Tauri.
vi.mock('@/emu', () => ({ AUTO_SLOT: 0 }))

describe('coverChoice', () => {
  it('offers the screenshot before the user image, the icon only when the ROM has one', () => {
    expect(coverChoices({ hasIcon: true })).toEqual(['generated', 'icon', 'screenshot', 'user'])
    expect(coverChoices({ hasIcon: false })).toEqual(['generated', 'screenshot', 'user'])
  })

  it('applies generated and icon at once, frames a screenshot or an image first', () => {
    expect(choiceNext('generated')).toBe('apply')
    expect(choiceNext('icon')).toBe('apply')
    expect(choiceNext('screenshot')).toBe('screenshots')
    expect(choiceNext('user')).toBe('pick')
  })

  it('reads the screenshot name from the saved path on every platform', () => {
    expect(screenshotName('C:\\data\\saves\\screenshots\\gba\\1a\\2026-11-22_10-11-12.png')).toBe(
      '2026-11-22_10-11-12.png',
    )
    expect(screenshotName('/data/saves/screenshots/nds/2b/2026-11-22_10-11-12-2.png')).toBe(
      '2026-11-22_10-11-12-2.png',
    )
  })
})
