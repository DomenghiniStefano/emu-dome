// The device a screen lays itself out for, decided in one place:
// - `device`: what the settings schema says this is (`desktop` / `phone`), null until it is read;
// - `portrait`: the window is upright now, following rotations;
// - `phone`: the phone's portrait layout (boards «Tel-…»): a phone held upright;
// - `coarse`: a touch screen is the main pointer (on-screen controls, no keyboard hints).
import { computed, onMounted, onScopeDispose, ref } from 'vue'
import type { SettingDevice } from '../emu'
import { loadSchema } from '../settingsModel'

/** Whether a touch screen is the main pointer. Fixed for the app's run. */
export const coarsePointer = (): boolean => window.matchMedia('(pointer: coarse)').matches

export function useDeviceLayout() {
  const device = ref<SettingDevice | null>(null)

  const query = window.matchMedia('(orientation: portrait)')
  const portrait = ref(query.matches)
  const follow = (e: MediaQueryListEvent) => (portrait.value = e.matches)
  query.addEventListener('change', follow)
  onScopeDispose(() => query.removeEventListener('change', follow))

  // The schema is read once per app (cached). Unreadable: the desktop layout, and the screen's
  // own loads report the failure.
  onMounted(async () => {
    try {
      device.value = (await loadSchema()).device
    } catch {
      device.value = null
    }
  })

  return {
    device,
    portrait,
    phone: computed(() => device.value === 'phone' && portrait.value),
    coarse: coarsePointer(),
  }
}
