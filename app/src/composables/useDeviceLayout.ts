// The device a screen lays itself out for, decided in one place:
// - `device`: what the settings schema says this is (`desktop` / `phone`), null until it is read;
// - `portrait`: the window is upright now, following rotations;
// - `phone`: the phone's portrait layout (boards «Tel-…»): a phone held upright;
// - `touchDensity`: the phone, upright or on its side: the touch density (44 px targets, the
//   phone's type), set on <html> by App.vue (useDensity.ts);
// - `nav`: where the shell's sections sit (lib/phoneLayout.ts: rail, tab bar, side column);
// - `camera`: with the phone on its side, the window's edge the camera is on, following turns;
// - `coarse`: a touch screen is the main pointer (on-screen controls, no keyboard hints).
import { computed, onMounted, onScopeDispose, ref } from 'vue'
import type { SettingDevice } from '@/emu'
import { cameraSide, shellNav } from '@/lib/phoneLayout'
import { loadSchema } from '@/lib/settingsModel'

/** Whether a touch screen is the main pointer. Fixed for the app's run. */
export const coarsePointer = (): boolean => window.matchMedia('(pointer: coarse)').matches

/** `screen.orientation`, where the browser has it. */
const screenOrientation = (): ScreenOrientation | undefined =>
  typeof screen !== 'undefined' ? screen.orientation : undefined

export function useDeviceLayout() {
  const device = ref<SettingDevice | null>(null)

  const query = window.matchMedia('(orientation: portrait)')
  const portrait = ref(query.matches)
  const follow = (e: MediaQueryListEvent) => (portrait.value = e.matches)
  query.addEventListener('change', follow)
  onScopeDispose(() => query.removeEventListener('change', follow))

  // The way the phone is turned: the orientation's own event, and the window's resize as well
  // (a half turn from one side to the other changes no size, a quarter turn both).
  const orientation = ref(screenOrientation()?.type)
  const turned = () => (orientation.value = screenOrientation()?.type)
  screenOrientation()?.addEventListener('change', turned)
  window.addEventListener('resize', turned)
  onScopeDispose(() => {
    screenOrientation()?.removeEventListener('change', turned)
    window.removeEventListener('resize', turned)
  })

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
    touchDensity: computed(() => device.value === 'phone'),
    nav: computed(() => shellNav(device.value, portrait.value)),
    camera: computed(() => cameraSide(orientation.value)),
    coarse: coarsePointer(),
  }
}
