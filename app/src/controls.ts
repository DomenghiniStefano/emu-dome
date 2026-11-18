// What the Controls section (C01–C04) needs beyond the bridge: names of buttons, positions
// and keys in the current language, the shape of the `pad.remap` / `pad.combos` values when
// writing them, and the capture field's logic (CaptureField, design system).
//
// The tables themselves (defaults, what is remapped) come resolved from Rust
// (`emu.controls.bindings`): nothing here lists default buttons.
import { onBeforeUnmount, ref } from 'vue'
import { i18n, t } from './i18n'
import {
  emu,
  type Button,
  type ComboRow,
  type HotkeyId,
  type PadFamily,
  type PadState,
  type Pos,
  type RemapRow,
} from './emu'
import type { KeyAction } from './keyboard'

const known = (key: string) => i18n.global.te(key)

const FACES: Pos[] = ['south', 'east', 'west', 'north']

/** The 12 buttons of the tester (C01, C02): faces, shoulders, triggers, Start / Select, sticks. */
const TESTER_POSITIONS: Pos[] = [
  'south',
  'east',
  'west',
  'north',
  'l1',
  'r1',
  'l2',
  'r2',
  'start',
  'back',
  'l3',
  'r3',
]

/** The tester's cells: the 12 positions of a controller, or the keys the keyboard uses. */
export function testerCells(family: PadFamily, remap: RemapRow[]): { id: string; label: string }[] {
  if (family !== 'keyboard')
    return TESTER_POSITIONS.map(p => ({ id: p, label: posName(family, p, false) }))
  const keys = remap.map(r => r.source).filter((s): s is string => !!s)
  return [...new Set(keys)].map(k => ({ id: k, label: keyName(k) }))
}

/** The name of a position on a controller of `family` («B (destra)», «LB», «Croce su»). */
export function posName(family: PadFamily, pos: Pos, withSide = true): string {
  const own = `controls.pos.${family}.${pos}`
  const name = known(own) ? t(own) : t(`controls.pos.common.${pos}`)
  // Face buttons with a letter or a symbol also say where they are.
  if (withSide && family !== 'generic' && FACES.includes(pos))
    return t('controls.withSide', { name, side: t(`controls.side.${pos}`) })
  return name
}

/** The name of a key from its `KeyboardEvent.code` («X», «1», «Invio»). */
export function keyName(code: string): string {
  const letter = /^Key([A-Z])$/.exec(code)
  if (letter) return letter[1]!
  const digit = /^(?:Digit|Numpad)(\d)$/.exec(code)
  if (digit) return digit[1]!
  if (/^F\d{1,2}$/.test(code)) return code
  return known(`controls.key.${code}`) ? t(`controls.key.${code}`) : code
}

/** The name of a source in a remap table: a position, a key, or «Nessuno». */
export function sourceName(family: PadFamily, source: string | null): string {
  if (!source) return t('controls.none')
  return family === 'keyboard' ? keyName(source) : posName(family, source as Pos)
}

/** The name of a game button («A», «ZL (solo 3DS)», «Croce su»). */
export const buttonName = (b: Button) => t(`controls.button.${b}`)

/** The long name of a shortcut (C03, G10: «Salvataggio rapido nello slot attuale»). */
export const hotkeyName = (h: KeyAction) =>
  h === 'fullscreen' ? t('controls.keys.fullscreen') : t(`controls.hotkey.${h}`)

/** The short name of a line of «Scorciatoie da ricordare» (C01: «Salvataggio rapido»). */
export const shortName = (a: KeyAction | 'slot') => t(`controls.short.${a}`)

/** «L3 + RB»: a shortcut as the user presses it. */
export function comboText(family: PadFamily, modifier: Pos, pos: Pos | null): string {
  if (!pos) return t('controls.none')
  return t('controls.combo', {
    modifier: posName(family, modifier, false),
    button: posName(family, pos, false),
  })
}

/** «L3 + ← →»: the previous / next slot pair on one line (C01); other buttons «L3 + X / L3 + Y». */
export function slotPairText(
  family: PadFamily,
  modifier: Pos,
  prev: Pos | null,
  next: Pos | null,
): string {
  if (prev === 'left' && next === 'right')
    return t('controls.comboArrows', { modifier: posName(family, modifier, false) })
  return [prev, next]
    .filter((p): p is Pos => p !== null)
    .map(p => comboText(family, modifier, p))
    .join(' / ')
}

/**
 * The `pad.remap` list after giving `changes` (button → source, null for nothing) to `family`.
 * Items of other families and of untouched buttons stay; a source equal to the default is
 * dropped (only overrides are stored, `crates/emudome-core/src/bindings.rs`).
 */
export function remapList(
  current: string[],
  family: PadFamily,
  rows: RemapRow[],
  changes: Partial<Record<Button, string | null>>,
): string[] {
  const touched = new Set(Object.keys(changes))
  const kept = current.filter(item => {
    const [fam, rest] = item.split(':')
    const button = rest?.split('=')[0]
    return fam !== family || !button || !touched.has(button)
  })
  for (const row of rows) {
    if (!touched.has(row.button)) continue
    const source = changes[row.button] ?? null
    if (source !== row.default) kept.push(`${family}:${row.button}=${source ?? 'none'}`)
  }
  return kept
}

/** The `pad.remap` list without `family`'s items («Ripristina i pulsanti predefiniti»). */
export function remapWithout(current: string[], family: PadFamily): string[] {
  return current.filter(item => !item.startsWith(`${family}:`))
}

/** The `pad.combos` list for the shortcuts `rows` after `changes` (overrides only). */
export function combosList(rows: ComboRow[], changes: Partial<Record<HotkeyId, Pos | null>>) {
  return rows
    .map(r => ({ ...r, pos: r.hotkey in changes ? (changes[r.hotkey] ?? null) : r.pos }))
    .filter(r => r.pos !== r.default)
    .map(r => `${r.hotkey}=${r.pos ?? 'none'}`)
}

/** How long the capture waits, and how long B or Esc must be held to cancel it. */
export const CAPTURE_MS = 5000
export const HOLD_TO_CANCEL_MS = 1000

/** What a capture listens to: one controller (with its menu «back» position) or the keyboard. */
export type CaptureDevice = { kind: 'pad'; id: number; back: Pos } | { kind: 'keyboard' }

/**
 * The capture field: `start(id)` waits up to 5 s for a button of the device and calls
 * `done(id, source)` with a position or a key code. B (the menu's back) and Esc cancel only
 * when held for 1 s, so a short B press is assigned like any other button. While it runs, the
 * controllers neither play nor navigate (`gamepad_capture`) and keys do not reach the app.
 */
export function useCapture(options: {
  device: () => CaptureDevice
  /** The pad's latest state, for the buttons already held when the capture starts. */
  state: (id: number) => PadState | undefined
  done: (id: string, source: string) => void
}) {
  /** The row being captured, null when idle. */
  const target = ref<string | null>(null)
  /** Whole seconds left, for the countdown. */
  const left = ref(0)

  let deadline = 0
  let timer: number | undefined
  let prev = new Set<Pos>()
  /** Held when the capture started: ignored until released (the A that clicked the row). */
  let ignored = new Set<Pos>()
  let backSince: number | null = null
  let escSince: number | null = null

  function tick() {
    const now = Date.now()
    if (backSince !== null && now - backSince >= HOLD_TO_CANCEL_MS) return stop()
    if (escSince !== null && now - escSince >= HOLD_TO_CANCEL_MS) return stop()
    // A B being held when time runs out still gets its second to decide.
    if (now >= deadline && backSince === null && escSince === null) return stop()
    left.value = Math.max(1, Math.ceil((deadline - now) / 1000))
  }

  function finish(source: string) {
    const id = target.value
    stop()
    if (id) options.done(id, source)
  }

  function onKey(e: KeyboardEvent, down: boolean) {
    if (!target.value) return
    e.preventDefault()
    e.stopImmediatePropagation()
    if (e.code === 'Escape') {
      if (down && !e.repeat) escSince = Date.now()
      if (!down) escSince = null
      return
    }
    if (down && !e.repeat && options.device().kind === 'keyboard') finish(e.code)
  }
  const keydown = (e: KeyboardEvent) => onKey(e, true)
  const keyup = (e: KeyboardEvent) => onKey(e, false)

  function start(id: string) {
    stop()
    const device = options.device()
    target.value = id
    deadline = Date.now() + CAPTURE_MS
    left.value = Math.ceil(CAPTURE_MS / 1000)
    backSince = null
    escSince = null
    if (device.kind === 'pad') {
      prev = new Set(options.state(device.id)?.held ?? [])
      ignored = new Set(prev)
      void emu.controls.capture(true)
    }
    window.addEventListener('keydown', keydown, { capture: true })
    window.addEventListener('keyup', keyup, { capture: true })
    timer = window.setInterval(tick, 100)
  }

  function stop() {
    if (target.value === null) return
    target.value = null
    window.clearInterval(timer)
    window.removeEventListener('keydown', keydown, { capture: true })
    window.removeEventListener('keyup', keyup, { capture: true })
    if (options.device().kind === 'pad') void emu.controls.capture(false)
  }

  /** Feed every `emu://pad-state` here. */
  function onPads(states: PadState[]) {
    const device = options.device()
    if (!target.value || device.kind !== 'pad') return
    const held = new Set(states.find(s => s.id === device.id)?.held ?? [])
    for (const pos of prev) {
      if (held.has(pos)) continue
      ignored.delete(pos)
      // B released within the second: it is the button to assign.
      if (pos === device.back && backSince !== null) return finish(pos)
    }
    for (const pos of held) {
      if (prev.has(pos) || ignored.has(pos)) continue
      if (pos === device.back) backSince = Date.now()
      else return finish(pos)
    }
    prev = held
  }

  onBeforeUnmount(stop)
  return { target, left, start, stop, onPads }
}
