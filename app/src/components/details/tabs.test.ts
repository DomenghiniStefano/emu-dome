import { describe, expect, it } from 'vitest'
import { DETAILS_TABS, isDetailsTab, stepTab, tabLabelKey, type DetailsTab } from './tabs'

describe('stepTab', () => {
  it('moves to the next and the previous tab', () => {
    expect(stepTab('settings', 1)).toBe('controls')
    expect(stepTab('settings', -1)).toBe('saves')
  })

  it('wraps around at both ends', () => {
    expect(stepTab('files', 1)).toBe('saves')
    expect(stepTab('saves', -1)).toBe('files')
  })

  it('visits every tab once in a full turn', () => {
    const seen = new Set<string>()
    let tab: DetailsTab = DETAILS_TABS[0]
    for (let i = 0; i < DETAILS_TABS.length; i++) {
      seen.add(tab)
      tab = stepTab(tab, 1)
    }
    expect(seen.size).toBe(DETAILS_TABS.length)
    expect(tab).toBe(DETAILS_TABS[0])
  })
})

describe('tabLabelKey', () => {
  it('names each tab after itself', () => {
    expect(tabLabelKey('files', false)).toBe('details.tabs.files')
    expect(tabLabelKey('settings', false)).toBe('details.tabs.settings')
  })

  it('shortens only «Impostazioni» on the phone', () => {
    expect(tabLabelKey('settings', true)).toBe('details.tabs.settingsShort')
    expect(tabLabelKey('saves', true)).toBe('details.tabs.saves')
  })
})

describe('isDetailsTab', () => {
  it('accepts the tabs and nothing else', () => {
    expect(isDetailsTab('files')).toBe(true)
    expect(isDetailsTab('overview')).toBe(false)
    expect(isDetailsTab(2)).toBe(false)
  })
})
