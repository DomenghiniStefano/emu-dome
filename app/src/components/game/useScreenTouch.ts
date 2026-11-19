// The DS touch screen from the mouse or a finger, while a DS game runs (see screenTouch.ts).
import { onScopeDispose, watch, type Ref } from 'vue'
import { emu, type VideoMode } from '@/emu'
import { isInterfaceTarget, pointerArgs } from './screenTouch'

/**
 * While `active` (a DS game, no menu open), the primary pointer on the game is the stylus:
 * pressed from `pointerdown` to `pointerup`, followed while it moves.
 */
export function useScreenTouch(active: () => boolean, mode: Ref<VideoMode>, insetPx: () => number) {
  let pressed: number | null = null

  function send(e: PointerEvent, down: boolean) {
    const { x, y, area } = pointerArgs(e.clientX, e.clientY, window, mode.value, insetPx())
    void emu.pointer(x, y, down, area)
  }
  function onDown(e: PointerEvent) {
    if (pressed !== null || !e.isPrimary || e.button !== 0 || isInterfaceTarget(e.target)) return
    pressed = e.pointerId
    send(e, true)
  }
  function onMove(e: PointerEvent) {
    if (e.pointerId === pressed) send(e, true)
  }
  function onUp(e: PointerEvent) {
    if (e.pointerId !== pressed) return
    pressed = null
    send(e, false)
  }

  function attach() {
    window.addEventListener('pointerdown', onDown)
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    window.addEventListener('pointercancel', onUp)
  }
  function detach() {
    window.removeEventListener('pointerdown', onDown)
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
    if (pressed !== null) {
      pressed = null
      void emu.pointer(0, 0, false, { width: 0, height: 0, inset: 0 })
    }
  }

  watch(active, on => (on ? attach() : detach()), { immediate: true })
  onScopeDispose(detach)
}
