import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h } from 'vue'
import { catchEverything, ignore, report, reportError, setErrorSink, settle } from './errors'

const flush = () => new Promise(r => setTimeout(r, 0))

afterEach(() => {
  setErrorSink(null)
  vi.restoreAllMocks()
})

function sinkSpy() {
  vi.spyOn(console, 'error').mockImplementation(() => {})
  vi.spyOn(console, 'warn').mockImplementation(() => {})
  const sink = vi.fn()
  setErrorSink(sink)
  return sink
}

describe('the error sink', () => {
  it('report shows a failure, ignore only logs it, a success shows nothing', async () => {
    const sink = sinkSpy()
    report(Promise.reject({ code: 'noGame' }))
    ignore(Promise.reject(new Error('quiet')), 'nobody needs to know')
    report(Promise.resolve(1))
    await flush()
    expect(sink.mock.calls).toEqual([[{ code: 'noGame' }]])
    expect(console.warn).toHaveBeenCalledWith('nobody needs to know', new Error('quiet'))
  })

  it('settle reports a failed step and lets the next one run', async () => {
    const sink = sinkSpy()
    const steps: string[] = []
    await settle(Promise.reject(new Error('first')))
    steps.push('second')
    expect(steps).toEqual(['second'])
    expect(sink).toHaveBeenCalledOnce()
  })

  it('before the toasts exist a failure is logged, not lost', () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    reportError(new Error('early'))
    expect(console.error).toHaveBeenCalledWith(new Error('early'))
  })

  it('catches an error in a component and a rejection nobody awaited', () => {
    const sink = sinkSpy()
    const target = new EventTarget() as unknown as Window
    const app = createApp(
      defineComponent({
        setup() {
          throw new Error('render')
        },
        render: () => h('div'),
      }),
    )
    catchEverything(app, target)
    app.mount(document.createElement('div'))
    const rejection = Object.assign(new Event('unhandledrejection', { cancelable: true }), {
      reason: { code: 'appClosing' },
    })
    target.dispatchEvent(rejection)
    expect(sink.mock.calls).toEqual([[new Error('render')], [{ code: 'appClosing' }]])
    expect(rejection.defaultPrevented).toBe(true)
  })
})
