// Interface theme: `data-theme` on <html> selects the token set in styles.css.
//
// Which theme comes from Rust (src-tauri/src/theme.rs, unit-tested there): `EMUDOME_THEME`
// if set, else the `general.theme` setting, else (for "Come il sistema") the system's
// preference, which the webview knows (`prefers-color-scheme`). Asked again whenever the system
// preference or the setting changes. Without an answer (a failed call) the page keeps the dark
// tokens of :root.
import { emu, type Theme } from './emu'

const systemDark = window.matchMedia('(prefers-color-scheme: dark)')

function apply(theme: Theme) {
  document.documentElement.dataset.theme = theme
}

async function refresh() {
  try {
    apply(await emu.theme(systemDark.matches))
  } catch (e) {
    console.warn('theme unavailable, keeping the dark one', e)
  }
}

/** Applies the theme now and follows the system's and the setting's changes from then on. */
export function startTheme() {
  void refresh()
  systemDark.addEventListener('change', () => void refresh())
  void emu.settings.onChanged(e => {
    if (e.key === 'general.theme') void refresh()
  })
}
