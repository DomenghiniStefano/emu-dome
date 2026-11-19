// Fast forward and slow motion follow the settings (`speed.ffval`, `speed.slow`, G16), the same
// for the quick menu, the Tab key and the controller combos (those read them in Rust).
import type { Speed } from '@/emu'

export const NORMAL: Speed = { kind: 'factor', value: 1 }
/** The defaults of `speed.ffval` and `speed.slow`. */
export const FF_DEFAULT: Speed = { kind: 'factor', value: 4 }
export const SLOW_DEFAULT: Speed = { kind: 'factor', value: 0.5 }

/** A speed option of the settings (`max`, `0.5`, `4`…); null for anything else. */
export function speedFromOption(id: unknown): Speed | null {
  if (id === 'max') return { kind: 'max' }
  if (typeof id !== 'string' || id.trim() === '') return null
  const value = Number(id)
  return Number.isFinite(value) && value > 0 ? { kind: 'factor', value } : null
}

export function sameSpeed(a: Speed, b: Speed): boolean {
  if (a.kind === 'max' || b.kind === 'max') return a.kind === b.kind
  return a.value === b.value
}
