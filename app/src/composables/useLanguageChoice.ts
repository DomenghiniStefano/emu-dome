// The interface language as a choice (`general.lang`, «Tutti i giochi»): the system's (named in
// the language it resolves to), Italian, English.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu } from '../emu'
import { LOCALE } from '../i18n'
import { errorText } from '../i18n/labels'

export function useLanguageChoice(onError: (text: string) => void) {
  const { t } = useI18n()
  const lang = ref('system')
  const options = computed(() =>
    ['system', 'it', 'en'].map(id => ({
      id,
      text: t(`settings.general.lang.option.${id}`, {
        language: t(`settings.general.lang.option.${LOCALE}`),
      }),
    })),
  )

  /** The value saved (it can throw: the caller reports it). */
  async function load() {
    const rows = await emu.settings.page('general', { level: 'global' })
    const v = rows.find(r => r.key === 'general.lang')?.value
    if (typeof v === 'string') lang.value = v
  }

  async function choose(id: string) {
    lang.value = id
    try {
      await emu.settings.set('general.lang', { level: 'global' }, id)
    } catch (e) {
      onError(errorText(e))
    }
  }

  return { lang, options, load, choose }
}
