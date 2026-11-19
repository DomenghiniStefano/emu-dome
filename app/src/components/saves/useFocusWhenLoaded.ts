// The view's first control takes the focus once its content is there. The shell autofocuses on
// mount, before the saves are known, so the focus first sits on the rail's current item; once
// `loaded` turns true, the first `data-nav-initial` control of `root` takes it, once, unless the
// user has already moved it somewhere else.
import { nextTick, watch, type Ref } from 'vue'
import { focusInitial } from '../../nav'

export function useFocusWhenLoaded(root: Ref<HTMLElement | null>, loaded: Ref<boolean>) {
  let focused = false
  watch(loaded, async () => {
    if (focused || !loaded.value) return
    focused = true
    await nextTick()
    if (root.value) focusInitial(root.value, { unlessMoved: true })
  })
}
