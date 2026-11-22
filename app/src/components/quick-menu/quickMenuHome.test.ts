import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import { i18n } from '@/i18n'
import { NORMAL } from '@/lib/speeds'
import QuickMenuHome from './QuickMenuHome.vue'
import { modeSpeeds, speedChoices } from './speedChoices'
import type { Tool } from './tools'
import { FocusMark, QuickMenuView } from './view'

vi.mock('@/emu', () => ({ emu: {} }))

// The focus comes back to the main-screen control that opened a page by its `FocusMark.Opener`
// (useQuickMenuNavigation.ts): every control that opens a page carries one, unique on the screen.
function home(sheet: boolean) {
  const { ff, slow } = modeSpeeds(null, null)
  return mount(QuickMenuHome, {
    props: {
      choices: speedChoices(ff, slow),
      speed: NORMAL,
      ffOption: null,
      slowOption: null,
      sheet,
      phone: sheet,
      dualScreen: false,
      muted: false,
      fullscreen: false,
      screens: null,
    },
    global: { plugins: [i18n] },
    attachTo: document.body,
  })
}

let mounted: VueWrapper | null = null
afterEach(() => {
  mounted?.unmount()
  mounted = null
})

describe('quick menu main screen', () => {
  it.each([false, true])('marks every control that opens a page (sheet: %s)', async sheet => {
    const wrapper = home(sheet)
    mounted = wrapper
    const marks = wrapper.findAll(`[${FocusMark.Opener}]`).map(b => b.attributes(FocusMark.Opener))
    expect(new Set(marks).size).toBe(marks.length)

    // Each control that opens a page (or runs a tool: Riavvia opens G11) is marked with it.
    for (const button of wrapper.findAll('button')) {
      const opens = wrapper.emitted('open')?.length ?? 0
      const runs = wrapper.emitted('run')?.length ?? 0
      await button.trigger('click')
      const open = wrapper.emitted('open') as [QuickMenuView, Event][] | undefined
      const run = wrapper.emitted('run') as [Tool, Event][] | undefined
      if ((open?.length ?? 0) > opens)
        expect(button.attributes(FocusMark.Opener)).toBe(open?.at(-1)?.[0])
      if ((run?.length ?? 0) > runs)
        expect(button.attributes(FocusMark.Opener)).toBe(run?.at(-1)?.[0].id)
    }
    expect(marks).toContain(QuickMenuView.Speed)
    expect(marks).toContain(QuickMenuView.Screens)
    expect(marks).toContain(QuickMenuView.Controls)
    expect(marks).toContain('restart')
  })
})
