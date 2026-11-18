// The phone's portrait layout (Tel-PrimoAvvio), decided like the quick menu does: a phone (the
// schema's device) held upright.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { loadSchema } from '../settingsModel'

export function usePhoneLayout() {
  const phoneDevice = ref(false)
  const portraitQuery = window.matchMedia('(orientation: portrait)')
  const portrait = ref(portraitQuery.matches)
  const onPortrait = (e: MediaQueryListEvent) => (portrait.value = e.matches)
  onMounted(() => portraitQuery.addEventListener('change', onPortrait))
  onBeforeUnmount(() => portraitQuery.removeEventListener('change', onPortrait))

  /** Asks the schema which device this is (it can throw: the caller reports it). */
  async function detect() {
    phoneDevice.value = (await loadSchema()).device === 'phone'
  }

  return { phone: computed(() => phoneDevice.value && portrait.value), detect }
}
