import { describe, expect, it } from 'vitest'
import { cn } from './cn'

describe('cn', () => {
  it('keeps a type step beside a text colour', () => {
    expect(cn('text-label', 'text-ink')).toBe('text-label text-ink')
  })

  it('lets a later type step replace an earlier one', () => {
    expect(cn('text-label', 'text-cta')).toBe('text-cta')
  })

  it('lets a later colour replace an earlier one', () => {
    expect(cn('bg-surface', 'bg-surface-raised')).toBe('bg-surface-raised')
    expect(cn('text-ink', 'text-accent-text')).toBe('text-accent-text')
  })

  it('lets a later spacing token replace an earlier one', () => {
    expect(cn('h-control', 'h-control-lg')).toBe('h-control-lg')
    expect(cn('px-12', 'px-18')).toBe('px-18')
  })

  it('lets a later radius replace an earlier one', () => {
    expect(cn('rounded-6', 'rounded-10')).toBe('rounded-10')
  })

  it('lets a later shadow replace an earlier one', () => {
    expect(cn('shadow-glow-sm', 'shadow-float')).toBe('shadow-float')
  })

  it('lets a later weight replace an earlier one', () => {
    expect(cn('font-medium', 'font-semibold')).toBe('font-semibold')
  })

  it('lets a later duration and easing replace earlier ones', () => {
    expect(cn('duration-fast ease-out', 'duration-panel ease-spring')).toBe(
      'duration-panel ease-spring',
    )
  })

  it('lets a later opacity token replace an earlier one, numbers included', () => {
    expect(cn('opacity-disabled', 'opacity-locked')).toBe('opacity-locked')
    expect(cn('opacity-disabled', 'opacity-0')).toBe('opacity-0')
  })

  it('lets a later stacking layer replace an earlier one', () => {
    expect(cn('z-menu', 'z-dialog')).toBe('z-dialog')
  })

  it('lets a caller replace a border width without losing its colour', () => {
    expect(cn('border border-line', 'border-0')).toBe('border-line border-0')
  })

  it('drops falsy inputs', () => {
    expect(cn('text-label', false, undefined, 'text-ink')).toBe('text-label text-ink')
  })
})
