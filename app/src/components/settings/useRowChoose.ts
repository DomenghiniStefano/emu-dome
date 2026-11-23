// What the button of a file or action row does, the same on the desk's row (SettingRow) and the
// TV's (TvSettingRow): the Google account (`cloud.acct`, #31) opens the Cloud section; a row of a
// feature that lives in another section (ROW_JUMPS, #112) leads there through the shell; an action
// with a backend (ROW_ACTIONS: `about.upd`, `about.log`) runs here and says when it is done; a
// file row with its own picker (FILE_PICKERS) opens it here; any other row is the page's to carry
// out (`video.surroundImg`'s picker at the page's scope, `about.lic`'s licences page), which the
// row asks for with `choose`. Before Trello #112 the TV row sent every one of them to the page,
// which opened the background image's picker for «Cerca aggiornamenti» and «Licenze».
import { useI18n } from 'vue-i18n'
import type { SettingKey } from '@/emu'
import { useShellJump } from '@/components/shell/useShellJump'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { FILE_PICKERS, ROW_ACTIONS } from '@/lib/settingsModel'
import { ROW_JUMPS } from './rowJumps'

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
  // Outside the shell (the Kit page) there is nowhere to lead to.
  const jumpTo = useShellJump()

  return async function choose() {
    if (disabled()) return
    const k = key()
    if (k === 'cloud.acct') return page.openCloud()
    const jump = ROW_JUMPS[k]
    if (jump) return jumpTo?.(jump)
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
