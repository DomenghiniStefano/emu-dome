import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { i18n } from '@/i18n'
import { errorText } from '@/i18n/labels'
import { setErrorSink } from '@/lib/errors'
import ToastStack from '@/components/toasts/ToastStack.vue'
import { ToastKind } from '@/components/toasts/toasts'
import { createToasts, provideToasts, useErrorSink, useToasts, type Toasts } from './useToasts'

vi.mock('@/emu', () => ({ emu: {} }))

afterEach(() => {
  setErrorSink(null)
  vi.useRealTimers()
  vi.restoreAllMocks()
})

/** Mounts `child` under a root that provides `toasts`, as App.vue does. */
function under(toasts: Toasts, child: () => void, extra?: () => ReturnType<typeof h>) {
  const Child = defineComponent({
    setup() {
      child()
      return () => h('div')
    },
  })
  return mount(
    defineComponent({
      setup() {
        provideToasts(toasts)
        return () => h('div', [h(Child), extra?.()])
      },
    }),
    { global: { plugins: [i18n] } },
  )
}

describe('the toasts', () => {
  it('a toast pushed before the stack is mounted shows once it is, with its full time', async () => {
    vi.useFakeTimers()
    const toasts = createToasts()
    toasts.fail('early')
    vi.advanceTimersByTime(10_000)
    expect(toasts.list.value.map(t => t.text)).toEqual(['early'])

    const w = under(
      toasts,
      () => {},
      () => h(ToastStack),
    )
    await w.vm.$nextTick()
    expect(w.text()).toContain('early')
    // An error outside the game stays 5 s from when the stack shows it.
    vi.advanceTimersByTime(4_999)
    expect(toasts.list.value).toHaveLength(1)
    vi.advanceTimersByTime(1)
    expect(toasts.list.value).toHaveLength(0)
    w.unmount()
  })

  it('a component below the provider tells the user without an emit', () => {
    const toasts = createToasts()
    vi.spyOn(console, 'error').mockImplementation(() => {})
    under(toasts, () => {
      useToasts().notify('hello', { ms: 100 })
      useErrorSink()({ code: 'noGame' })
    })
    expect(toasts.list.value.map(t => [t.text, t.kind])).toEqual([
      ['hello', ToastKind.Info],
      [errorText({ code: 'noGame' }), ToastKind.Error],
    ])
    expect(console.error).toHaveBeenCalledWith({ code: 'noGame' })
  })

  it('with no provider, an error still reaches the app-wide sink and useToasts says so', () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const sink = vi.fn()
    setErrorSink(sink)
    let thrown: unknown = null
    mount(
      defineComponent({
        setup() {
          useErrorSink()({ code: 'noGame' })
          try {
            useToasts()
          } catch (e) {
            thrown = e
          }
          return () => h('div')
        },
      }),
    )
    expect(sink).toHaveBeenCalledWith({ code: 'noGame' })
    expect(thrown).toBeInstanceOf(Error)
  })

  it('dismiss takes one toast away', () => {
    const toasts = createToasts()
    toasts.notify('a')
    toasts.notify('b')
    const first = toasts.list.value[0]
    if (first) toasts.dismiss(first.id)
    expect(toasts.list.value.map(t => t.text)).toEqual(['b'])
  })
})
