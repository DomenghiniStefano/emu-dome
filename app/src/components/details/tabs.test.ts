import { describe, expect, it } from 'vitest'
import { cycle } from '../../lib/cycle'
import { DETAILS_TABS, isDetailsTab, tabLabelKey, type DetailsTab } from './tabs'

describe('the tabs with L1 / R1', () => {
  it('visits every tab once in a full turn', () => {
    const seen = new Set<string>()
    let tab: DetailsTab = DETAILS_TABS[0]
    for (let i = 0; i < DETAILS_TABS.length; i++) {
      seen.add(tab)
      tab = cycle(DETAILS_TABS, tab, 1)
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
