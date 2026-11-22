// Signing in to Google and out (Trello #31): «Accedi con Google» opens the system browser and
// waits for the redirect (the browser page's two texts come from here, in the UI language);
// «Esci» forgets the sign-in on this device. Each answer is the new status.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu } from '@/emu'
import { applyStatus, failCloud } from './cloudState'

export function useCloudAccount() {
  const { t } = useI18n()
  /** The browser is open, waiting for the user to finish there. */
  const signingIn = ref(false)

  async function signIn() {
    if (signingIn.value) return
    signingIn.value = true
    try {
      applyStatus(await emu.cloud.signIn(t('cloud.browser.done'), t('cloud.browser.failed')))
    } catch (e) {
      failCloud(e)
    } finally {
      signingIn.value = false
    }
  }

  async function signOut() {
    try {
      applyStatus(await emu.cloud.signOut())
    } catch (e) {
      failCloud(e)
    }
  }

  return { signingIn, signIn, signOut }
}
