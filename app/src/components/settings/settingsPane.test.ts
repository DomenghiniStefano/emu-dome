import { describe, expect, it } from 'vitest'
import { listShown, pageShown, settingsPane, SettingsPane } from './settingsPane'

describe('settingsPane', () => {
  it('shows both columns on the desk and on the phone on its side, whatever else', () => {
    for (const listOpen of [true, false])
      for (const searching of [true, false])
        expect(settingsPane(false, listOpen, searching)).toBe(SettingsPane.Both)
  })

  it('upright: the list first, then the page picked from it', () => {
    expect(settingsPane(true, true, false)).toBe(SettingsPane.List)
    expect(settingsPane(true, false, false)).toBe(SettingsPane.Page)
  })

  it('upright with a search typed: the field and its results, list open or not', () => {
    expect(settingsPane(true, true, true)).toBe(SettingsPane.Search)
    expect(settingsPane(true, false, true)).toBe(SettingsPane.Search)
  })

  it('says which panes are on screen', () => {
    expect([listShown(SettingsPane.Both), pageShown(SettingsPane.Both)]).toEqual([true, true])
    expect([listShown(SettingsPane.List), pageShown(SettingsPane.List)]).toEqual([true, false])
    expect([listShown(SettingsPane.Page), pageShown(SettingsPane.Page)]).toEqual([false, true])
    expect([listShown(SettingsPane.Search), pageShown(SettingsPane.Search)]).toEqual([true, true])
  })
})
