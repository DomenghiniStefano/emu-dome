// The game details' layout: the phone's (board «Tel-Dettagli») when the device is a phone, as the
// settings schema tells it, and the screen is in portrait; the desktop's otherwise.
import { computed, onBeforeUnmount, ref } from 'vue'
import type { SettingsSchema } from '../../emu'
import { loadSchema } from '../../settingsModel'

export function usePhoneLayout() {
  const schema = ref<SettingsSchema | null>(null)
  const portraitQuery = window.matchMedia('(orientation: portrait)')
  const portrait = ref(portraitQuery.matches)
  const onOrientation = (e: MediaQueryListEvent) => (portrait.value = e.matches)
  portraitQuery.addEventListener('change', onOrientation)
  onBeforeUnmount(() => portraitQuery.removeEventListener('change', onOrientation))

  /** Reads the device from the settings schema (once per app: the schema is cached). */
  async function loadDevice() {
    schema.value = await loadSchema()
  }

  return {
    /** The device, null until `loadDevice` has read it. */
    device: computed(() => schema.value?.device ?? null),
    phone: computed(() => schema.value?.device === 'phone' && portrait.value),
    loadDevice,
  }
}
