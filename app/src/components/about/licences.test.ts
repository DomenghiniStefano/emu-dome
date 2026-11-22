import { describe, expect, it } from 'vitest'
import type { Licence } from '@/emu'
import { licenceLine, licenceName, paragraphs } from './licences'

const row = (over: Partial<Licence>): Licence => ({
  id: 'emudome',
  kind: 'app',
  name: 'EmuDome',
  consoles: '',
  spdx: 'GPL-3.0-or-later',
  version: '0.0.1',
  source: 'https://github.com/DomenghiniStefano/emu-dome',
  files: ['LICENSE.txt'],
  ...over,
})

describe('licences page', () => {
  it('names the app and the cores as Rust says, the components from the message files', () => {
    expect(licenceName(row({}))).toBe('EmuDome')
    expect(licenceName(row({ kind: 'core', name: 'mGBA' }))).toBe('mGBA')
    expect(licenceName(row({ kind: 'components', name: '' }))).toEqual({
      key: 'about.licences.components',
    })
  })

  it('a row says its licence and version, a core its consoles too', () => {
    expect(licenceLine(row({}))).toEqual({
      key: 'about.licences.appLine',
      params: { spdx: 'GPL-3.0-or-later', version: '0.0.1' },
    })
    expect(
      licenceLine(
        row({ kind: 'core', consoles: 'NDS', spdx: 'GPL-3.0-or-later', version: '1.4.0' }),
      ),
    ).toEqual({
      key: 'about.licences.coreLine',
      params: { spdx: 'GPL-3.0-or-later', version: '1.4.0', consoles: 'NDS' },
    })
    expect(licenceLine(row({ kind: 'components' }))).toEqual({
      key: 'about.licences.componentsLine',
    })
  })

  it('cuts a text at its blank lines, none empty, Windows line ends too', () => {
    expect(paragraphs('one\nline\n\n  \ntwo\r\n\r\nthree\n')).toEqual(['one\nline', 'two', 'three'])
    expect(paragraphs('\n\n')).toEqual([])
  })
})
