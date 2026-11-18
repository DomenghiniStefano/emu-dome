import { describe, expect, it } from 'vitest'
import type { FolderSaves } from '../../emu'
import { canImport, importLine } from './imports'

const folder = (found: number, matches: number): FolderSaves => ({
  folder: 'C:/Saves',
  found,
  matches: Array.from({ length: matches }, () => ({}) as FolderSaves['matches'][number]),
})

describe('importLine', () => {
  it('gives the hint before a folder is picked', () => {
    expect(importLine({ found: null, imported: null })).toEqual({ kind: 'hint' })
  })
  it('tells what the folder holds, and how much matches when something was found', () => {
    expect(importLine({ found: folder(0, 0), imported: null })).toEqual({
      kind: 'found',
      n: 0,
      folder: 'C:/Saves',
      matched: null,
    })
    expect(importLine({ found: folder(5, 2), imported: null })).toEqual({
      kind: 'found',
      n: 5,
      folder: 'C:/Saves',
      matched: 2,
    })
  })
  it('counts the imported ones at the end', () => {
    expect(importLine({ found: folder(5, 2), imported: 2 })).toEqual({ kind: 'imported', n: 2 })
  })
})

describe('canImport', () => {
  it('imports only saves matching the library, once', () => {
    expect(canImport({ found: null, imported: null })).toBe(false)
    expect(canImport({ found: folder(3, 0), imported: null })).toBe(false)
    expect(canImport({ found: folder(3, 1), imported: null })).toBe(true)
    expect(canImport({ found: folder(3, 1), imported: 1 })).toBe(false)
  })
})
