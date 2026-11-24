import { describe, expect, it } from 'vitest'
import { controlsPageInput } from './controlsPage'

describe('controlsPageInput (#150)', () => {
  it('on a touch screen with no controller names the touch controls and hides the shortcuts', () => {
    expect(controlsPageInput(true, false)).toEqual({
      noController: 'qm.noControllerTouch',
      shortcuts: false,
    })
  })
  it('shows the shortcuts once a controller is connected', () => {
    expect(controlsPageInput(true, true).shortcuts).toBe(true)
  })
  it('keeps the keyboard line and the shortcuts on a computer', () => {
    expect(controlsPageInput(false, false)).toEqual({
      noController: 'qm.noController',
      shortcuts: true,
    })
    expect(controlsPageInput(false, true).shortcuts).toBe(true)
  })
})
