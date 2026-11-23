// The interface's density: `data-density="touch"` on <html> while the layout is the phone's
// (boards «Tel-*»), so the primitives and screens style their touch look with Tailwind's `touch:`
// variant (src/design-system/index.css) instead of a `phone` prop. The one place the attribute is set:
// App.vue calls this once, with the layout decision as its input.
import { watchEffect, type Ref } from 'vue'

export function useDensity(touch: Readonly<Ref<boolean>>) {
  watchEffect(() => {
    const root = document.documentElement
    if (touch.value) root.dataset.density = 'touch'
    else delete root.dataset.density
  })
}
