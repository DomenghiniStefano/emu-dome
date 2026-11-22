import { ArrowRight, File as FileIcon, Gamepad2, Hand, Rows2 } from '@lucide/vue'
import { describe, expect, it } from 'vitest'
import { rowActionIcon } from './rowActionIcon'

describe('rowActionIcon', () => {
  it('picks the icon by the kind of the row', () => {
    expect(rowActionIcon({ type: 'bindings', table: 'remap' })).toBe(Gamepad2)
    expect(rowActionIcon({ type: 'screenLayout' })).toBe(Rows2)
    expect(rowActionIcon({ type: 'touchLayout' })).toBe(Hand)
    expect(rowActionIcon({ type: 'file' })).toBe(FileIcon)
    expect(rowActionIcon({ type: 'action', style: 'button' })).toBe(ArrowRight)
    expect(rowActionIcon({ type: 'action', style: 'link' })).toBe(ArrowRight)
  })

  it('gives none to a kind with a control of its own', () => {
    expect(rowActionIcon({ type: 'toggle' })).toBeUndefined()
    expect(rowActionIcon({ type: 'text' })).toBeUndefined()
  })
})
