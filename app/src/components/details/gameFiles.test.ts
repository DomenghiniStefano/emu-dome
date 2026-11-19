import { describe, expect, it } from 'vitest'
import type { LibraryGame } from '@/emu'
import { coverDescription, deviceLine, gameFacts } from './gameFiles'

const t = (key: string) => key

const game = (over: Partial<LibraryGame>): LibraryGame =>
  ({
    id: 'gba:0123abcd',
    console: 'gba',
    cover: 'generated',
    region: 'europe',
    version: 1,
    code: 'ABCD',
    core: 'mgba',
    ...over,
  }) as LibraryGame

describe('gameFacts', () => {
  it('lists region, version, code, hash and engine', () => {
    expect(gameFacts(game({}), t)).toEqual([
      { label: 'details.files.region', value: 'details.region.europe' },
      { label: 'details.files.version', value: '1.1' },
      { label: 'details.files.code', value: 'ABCD' },
      { label: 'details.files.hash', value: '0123abcd', mono: true },
      { label: 'details.files.engine', value: 'details.files.core.mgba' },
    ])
  })

  it('leaves out what the header does not have', () => {
    const labels = gameFacts(game({ region: null, version: null, code: null }), t).map(f => f.label)
    expect(labels).toEqual(['details.files.hash', 'details.files.engine'])
  })

  it('shows a 3DS title ID in upper case and its version as a number', () => {
    const facts = gameFacts(
      game({ id: '3ds:0004000000abcdef0002', console: '3ds', version: 2, core: 'azahar' }),
      t,
    )
    expect(facts).toContainEqual({ label: 'details.files.version', value: '2' })
    expect(facts).toContainEqual({
      label: 'details.files.titleId',
      value: '0004000000ABCDEF',
      mono: true,
    })
  })
})

describe('coverDescription', () => {
  it('says what each kind of cover is made of', () => {
    expect(coverDescription(game({ cover: 'generated' }), t)).toBe('library.cover.generatedDesc')
    expect(coverDescription(game({ cover: 'icon' }), t)).toBe('library.cover.icon')
    expect(coverDescription(game({ cover: 'user' }), t)).toBe('library.cover.user')
  })
})

describe('deviceLine', () => {
  it('names the phone, else the computer', () => {
    expect(deviceLine('phone', t)).toBe('details.files.onPhone')
    expect(deviceLine('desktop', t)).toBe('details.files.onComputer')
    expect(deviceLine(null, t)).toBe('details.files.onComputer')
  })
})
