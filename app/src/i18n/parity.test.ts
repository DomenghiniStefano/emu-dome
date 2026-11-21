// The texts the backend's codes and the settings catalogue need, held against both message files
// (Trello #73). The lists come from Rust, generated into src/generated/ (crates/emudome-core/src/
// contract.rs, which fails `cargo test` when they are stale): a new `ErrorCode` variant or a
// renamed setting key fails here until its texts exist in en.json and it.json.
// Blind spots: texts built from other data (controller names, speeds, regions) are not listed by
// Rust and are checked only by their own tests; a placeholder used by the code but missing in
// both files is not seen.
import { describe, expect, it, vi } from 'vitest'
import codes from '@/generated/error-codes.json'
import catalog from '@/generated/settings-catalog.json'
import { msg } from '@/lib/settingsModel'
import enMessages from './en.json'
import itMessages from './it.json'

// settingsModel reaches the bridge, which needs Tauri; only its pure `msg` is used here.
vi.mock('@/emu', () => ({ emu: {} }))

type Messages = { [key: string]: string | Messages }

/** Every message of a file by its dotted key. */
function flatten(messages: Messages, prefix = '', out = new Map<string, string>()) {
  for (const [key, value] of Object.entries(messages)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (typeof value === 'string') out.set(path, value)
    else flatten(value, path, out)
  }
  return out
}

const files = { en: flatten(enMessages as Messages), it: flatten(itMessages as Messages) }

/** `{name}` placeholders of a message, sorted, one each. */
const placeholders = (text: string) => [...new Set(text.match(/\{\w+\}/g) ?? [])].sort()
/** vue-i18n plural cases: `a | b | c`. */
const cases = (text: string) => text.split(' | ').length

/** Codes the interface makes itself (a failure that is no `ErrorCode`: `errorText`). */
const INTERFACE_ERRORS = ['unexpected']

/** Keys missing from either file, among `keys`. */
const missing = (keys: string[]) =>
  keys.flatMap(key =>
    (['en', 'it'] as const).filter(lang => !files[lang].has(key)).map(lang => `${lang}: ${key}`),
  )

describe('en.json and it.json', () => {
  it('have the same keys', () => {
    const onlyEn = [...files.en.keys()].filter(k => !files.it.has(k))
    const onlyIt = [...files.it.keys()].filter(k => !files.en.has(k))
    expect({ onlyEn, onlyIt }).toEqual({ onlyEn: [], onlyIt: [] })
  })

  it('use the same placeholders and plural cases in every message', () => {
    const differ: string[] = []
    for (const [key, text] of files.en) {
      const other = files.it.get(key)
      if (other === undefined) continue
      if (placeholders(text).join() !== placeholders(other).join()) differ.push(`${key} {…}`)
      if (cases(text) !== cases(other)) differ.push(`${key} |`)
    }
    expect(differ).toEqual([])
  })

  it('have no empty message', () => {
    const empty = (['en', 'it'] as const).flatMap(lang =>
      [...files[lang]].filter(([, text]) => text.trim() === '').map(([key]) => `${lang}: ${key}`),
    )
    expect(empty).toEqual([])
  })
})

describe('error codes (emudome_core::ErrorCode)', () => {
  it('each has errors.<code> in both files', () => {
    expect(codes.length).toBeGreaterThan(0)
    expect(missing(codes.map(code => `errors.${code}`))).toEqual([])
  })

  it('every errors.<code> belongs to a code', () => {
    const known = new Set([...codes, ...INTERFACE_ERRORS])
    const stray = [...files.en.keys()]
      .filter(key => key.startsWith('errors.'))
      .map(key => key.split('.')[1]!)
      .filter(code => !known.has(code))
    expect(stray).toEqual([])
  })
})

describe('the settings catalogue', () => {
  it('every entry has its name and description, every option its text', () => {
    const keys = catalog.entries.flatMap(e => [
      msg.name(e.key),
      msg.desc(e.key),
      ...e.options.map(id => msg.option(e.key, id)),
    ])
    expect(missing(keys)).toEqual([])
  })

  it('every page, section and group has its text', () => {
    const keys = [
      ...catalog.pages.flatMap(p => [msg.page(p), msg.summary(p)]),
      ...catalog.sections.map(s => msg.section(s.page, s.id)),
      ...catalog.groups.map(g => msg.group(g)),
    ]
    expect(missing(keys)).toEqual([])
  })

  it('no setting text is left for a key the catalogue does not have (a renamed key)', () => {
    const known = new Set(catalog.entries.map(e => e.key))
    const ui = new Set(['pages', 'sections', 'groups'])
    const stray = new Set<string>()
    for (const key of files.en.keys()) {
      const [root, prefix, name] = key.split('.')
      if (root !== 'settings' || ui.has(prefix!)) continue
      if (!known.has(`${prefix}.${name}`)) stray.add(`${prefix}.${name}`)
    }
    expect([...stray]).toEqual([])
  })

  it('options without an entry option are not left behind', () => {
    const stray: string[] = []
    for (const e of catalog.entries) {
      const wanted = new Set(e.options.map(id => msg.option(e.key, id)))
      const prefix = `${msg.name(e.key).slice(0, -'.name'.length)}.option.`
      for (const key of files.en.keys())
        if (key.startsWith(prefix) && !wanted.has(key) && e.options.length > 0) stray.push(key)
    }
    expect(stray).toEqual([])
  })
})
