// The look-and-feel settings that apply to the whole interface (T02 Generale, T16 Accessibilità),
// all at the Global level, followed live through `settings://changed`:
// - `general.accent`: the accent tokens of accent.ts, written as a <style> after styles.css;
// - `a11y.textsize`: the whole interface scaled (CSS `zoom` on <html>, so the layout reflows);
//   the game is not scaled: layers sized in px from window sizes divide by `uiZoom`;
// - `a11y.motion`: `data-motion="reduce"` on <html> (styles.css keeps only short fades);
// - `a11y.transp`: `data-transp="reduce"` on <html> (glass becomes solid surface, haze off).
// «Come il sistema» follows `prefers-reduced-motion` / `prefers-reduced-transparency`.
import { ref } from 'vue'
import { accentTokens } from './accent'
import { onSettingsChanged } from '@/composables/useSettingsChanged'
import { emu, type SettingRow, type SettingValue } from '@/emu'
import { GLOBAL } from './settingsModel'

/** The interface's scale (`a11y.textsize` / 100). */
export const uiZoom = ref(1)
/** Movement replaced by short fades (`a11y.motion`, or the system's preference). */
export const reducedMotion = ref(false)
/** Solid surfaces instead of glass (`a11y.transp`, or the system's preference). */
export const reducedTransparency = ref(false)

const KEYS = ['general.accent', 'a11y.textsize', 'a11y.motion', 'a11y.transp']
const systemMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
const systemTransp = window.matchMedia('(prefers-reduced-transparency: reduce)')

let values: Record<string, SettingValue | null> = {}

/** `system` follows the media query, `on` / `off` force it. */
function choice(v: SettingValue | null | undefined, system: MediaQueryList): boolean {
  if (v === 'on') return true
  if (v === 'off') return false
  return system.matches
}

function applyAccent(swatch: SettingValue | null | undefined) {
  const id = 'emudome-accent'
  let el = document.getElementById(id) as HTMLStyleElement | null
  const tokens = typeof swatch === 'string' ? accentTokens(swatch) : null
  if (!tokens) {
    el?.remove()
    return
  }
  if (!el) {
    el = document.createElement('style')
    el.id = id
    document.head.appendChild(el)
  }
  const block = (sel: string, t: Record<string, string>) =>
    `${sel}{${Object.entries(t)
      .map(([k, v]) => `${k}:${v}`)
      .join(';')}}`
  el.textContent = [
    block(':root', tokens.dark),
    block(":root[data-theme='dark']", tokens.dark),
    block(":root[data-theme='black']", tokens.black),
    block(":root[data-theme='light']", tokens.light),
  ].join('\n')
}

function apply() {
  const root = document.documentElement
  applyAccent(values['general.accent'])
  const size = Number(values['a11y.textsize'] ?? 100)
  uiZoom.value = Number.isFinite(size) && size > 0 ? size / 100 : 1
  root.style.zoom = uiZoom.value === 1 ? '' : String(uiZoom.value)
  reducedMotion.value = choice(values['a11y.motion'], systemMotion)
  reducedTransparency.value = choice(values['a11y.transp'], systemTransp)
  if (reducedMotion.value) root.dataset.motion = 'reduce'
  else delete root.dataset.motion
  if (reducedTransparency.value) root.dataset.transp = 'reduce'
  else delete root.dataset.transp
}

async function refresh() {
  try {
    const rows: SettingRow[] = (
      await Promise.all(['general', 'a11y'].map(p => emu.settings.page(p, GLOBAL)))
    ).flat()
    values = Object.fromEntries(rows.filter(r => KEYS.includes(r.key)).map(r => [r.key, r.value]))
  } catch (e) {
    console.warn('appearance settings unavailable, keeping the defaults', e)
  }
  apply()
}

/** Applies the appearance settings now and follows their changes and the system's. */
export function startAppearance() {
  void refresh()
  systemMotion.addEventListener('change', apply)
  systemTransp.addEventListener('change', apply)
  void onSettingsChanged(KEYS, () => void refresh())
}
