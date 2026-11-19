// Whether the window is in portrait now (the phone's bottom sheet), following rotations.
import { onBeforeUnmount, ref } from 'vue'

export function usePortrait() {
  const query = window.matchMedia('(orientation: portrait)')
  const portrait = ref(query.matches)
  const follow = (e: MediaQueryListEvent) => (portrait.value = e.matches)
  query.addEventListener('change', follow)
  onBeforeUnmount(() => query.removeEventListener('change', follow))
  return portrait
}
