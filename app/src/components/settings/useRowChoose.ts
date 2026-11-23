// What the button of a file or action row does, the same on the desk's row (SettingRow) and the
// TV's (TvSettingRow): the Google account (`cloud.acct`, #31) opens the Cloud section; an action
// with a backend (ROW_ACTIONS: `about.upd`, `about.log`) runs here and says when it is done; a
// file row with its own picker (FILE_PICKERS) opens it here; any other row is the page's to carry
// out (`video.surroundImg`'s picker at the page's scope, `about.lic`'s licences page), which the
// row asks for with `choose`. Before Trello #112 the TV row sent every one of them to the page,
// which opened the background image's picker for «Cerca aggiornamenti» and «Licenze».
import { useI18n } from 'vue-i18n'
import type { SettingKey } from '@/emu'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { FILE_PICKERS, ROW_ACTIONS } from '@/lib/settingsModel'

export interface RowChooseTargets {
  /** The Google account: open the Cloud section. */
  openCloud: () => void
  /** A row the page carries out. */
  choose: () => void
}

export function useRowChoose(
  key: () => SettingKey,
  disabled: () => boolean,
  page: RowChooseTargets,
) {
  const { t } = useI18n()
  const failWith = useErrorSink()
  const toasts = useToasts()

  return async function choose() {
    if (disabled()) return
    const k = key()
    if (k === 'cloud.acct') return page.openCloud()
    const act = ROW_ACTIONS[k]
    if (act) {
      try {
        const done = await act()
        if (done) toasts.notify(t(done.key, done.params ?? {}))
      } catch (e) {
        failWith(e)
      }
      return
    }
    const pick = FILE_PICKERS[k]
    if (!pick) return page.choose()
    try {
      await pick()
    } catch (e) {
      failWith(e)
    }
  }
}
