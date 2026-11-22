// The touch controls playing: a button's press as its behaviour says (touchPress.ts), the d-pad's
// directions, the Circle Pad, and a short vibration on each press (`touch.thapt`). Everything a
// finger holds is let go when the controls go away (`releaseAll`).
import { ref } from 'vue'
import type { Button, TouchBehaviour } from '@/emu'
import { emu } from '@/emu'
import { ignore, report } from '@/lib/errors'
import { createPresser } from './touchPress'

export function useTouchPlay(haptics: () => number) {
  const presser = createPresser((b, on) => report(emu.input(b, on)))
  /** The «hold» buttons held down now (drawn pressed). */
  const latched = ref(new Set<Button>())
  let dirs: Button[] = []
  let stickMoved = false

  function feel() {
    const strength = haptics()
    // A vibration that fails changes nothing the player needs to know: logged only.
    if (strength > 0) ignore(emu.touch.haptic(strength), 'touch: the press did not vibrate')
  }

  function down(button: Button, behaviour: TouchBehaviour) {
    if (presser.down(button, behaviour)) feel()
    if (behaviour === 'hold') {
      const next = new Set(latched.value)
      if (presser.latched(button)) next.add(button)
      else next.delete(button)
      latched.value = next
    }
  }

  function up(button: Button, behaviour: TouchBehaviour) {
    presser.up(button, behaviour)
  }

  /** The d-pad's directions now: the new ones pressed (and felt), the old ones let go. */
  function setDirs(next: Button[]) {
    for (const b of dirs) if (!next.includes(b)) presser.up(b, 'tap')
    const added = next.filter(b => !dirs.includes(b))
    for (const b of added) presser.down(b, 'tap')
    if (added.length > 0) feel()
    dirs = next
  }

  function stick(x: number, y: number) {
    stickMoved = x !== 0 || y !== 0
    report(emu.touch.stick(x, y))
  }

  function releaseAll() {
    dirs = []
    presser.releaseAll()
    latched.value = new Set()
    if (stickMoved) stick(0, 0)
  }

  return { down, up, setDirs, stick, releaseAll, latched }
}
