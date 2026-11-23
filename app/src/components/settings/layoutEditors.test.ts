import { describe, expect, it } from 'vitest'
import { chosenTarget, directTarget, editorOf, EDITOR_CONSOLES } from './layoutEditors'

describe('layoutEditors', () => {
  it('knows the editor by its row', () => {
    expect(editorOf('touch.tedit')).toBe('touch')
    expect(editorOf('screens.custom')).toBe('screens')
  })

  it('offers two-screen consoles to C09 and every console to C06', () => {
    expect(EDITOR_CONSOLES.screens).toEqual(['nds', '3ds'])
    expect(EDITOR_CONSOLES.touch).toEqual(['gb', 'gbc', 'gba', 'nds', '3ds'])
  })

  it('asks for the console at «Tutti i giochi»', () => {
    expect(directTarget('screens', { level: 'global' })).toBeNull()
    expect(directTarget('touch', { level: 'global' })).toBeNull()
  })

  it('opens at once at a console it has layouts for, else asks', () => {
    expect(directTarget('screens', { level: 'console', console: 'nds' })).toEqual({
      editor: 'screens',
      console: 'nds',
      scope: { level: 'console', console: 'nds' },
    })
    expect(directTarget('screens', { level: 'console', console: 'gba' })).toBeNull()
    expect(directTarget('touch', { level: 'console', console: 'gba' })?.console).toBe('gba')
  })

  it('saves C09 for the game at a game scope, C06 for its console', () => {
    const game = { level: 'game', console: '3ds', game: '3ds:ab' } as const
    expect(directTarget('screens', game)?.scope).toEqual(game)
    expect(directTarget('touch', game)?.scope).toEqual({ level: 'console', console: '3ds' })
  })

  it('saves a chosen console at its own level', () => {
    expect(chosenTarget('touch', 'gb')).toEqual({
      editor: 'touch',
      console: 'gb',
      scope: { level: 'console', console: 'gb' },
    })
  })
})
