// Esc on a page goes back, while none of its menus or dialogs has a nav scope of its own open
// (those close on Esc themselves).
import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import { activeScopeRoot } from '../../nav'

export function useEscapeBack(root: Ref<HTMLElement | null>, back: () => void) {
  function onKey(e: KeyboardEvent) {
    if (e.code !== 'Escape' || !root.value) return
    if (!activeScopeRoot()?.contains(root.value)) return
    back()
  }
  onMounted(() => window.addEventListener('keydown', onKey))
  onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
}
