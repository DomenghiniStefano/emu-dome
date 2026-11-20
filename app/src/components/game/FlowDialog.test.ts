import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { h, nextTick } from 'vue'
import FlowDialog from './FlowDialog.vue'

let wrapper: ReturnType<typeof mount> | null = null
afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

async function escapeOn(escapeCloses: boolean) {
  wrapper = mount(FlowDialog, {
    props: { title: 'Title', escapeCloses },
    slots: { default: () => h('p', 'Text') },
    attachTo: document.body,
  })
  await nextTick()
  await new Promise(r => setTimeout(r, 0))
  const panel = document.querySelector('[role="alertdialog"]') as HTMLElement
  panel.dispatchEvent(
    new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }),
  )
  await nextTick()
  return wrapper.emitted('back') ?? []
}

describe('FlowDialog', () => {
  it('in the game flow Escape does not close it (it is the quick menu’s key there)', async () => {
    expect(await escapeOn(false)).toHaveLength(0)
  })

  it('outside the game (Saves) Escape is its safe exit, like B', async () => {
    expect(await escapeOn(true)).toHaveLength(1)
  })
})
