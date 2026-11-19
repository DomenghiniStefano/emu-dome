// First run S04: the A/B layout (`pad.ab`) and the press test of the first controller connected.
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type PadInfo, type PadState, type Pos, type RemapRow } from '@/emu'
import { abLayout, buttonName, FACES, posName, type AbLayout } from '@/lib/controls'
import { buttonsUsing, newlyHeld } from '@/components/controls/tester'
import { useGlobalSettings } from '@/components/controls/useGlobalSettings'
import type { ErrorSink } from '@/i18n/labels'
import { GLOBAL, readSetting } from '@/lib/settingsModel'

export function usePadTester(
  pads: Ref<PadInfo[]>,
  /** Whether the test is on screen: presses count only then. */
  active: () => boolean,
  onError: ErrorSink,
) {
  const { t } = useI18n()
  const settings = useGlobalSettings(onError)
  const pad = computed(() => pads.value[0] ?? null)
  const ab = ref<AbLayout>('nintendo')
  const remap = ref<RemapRow[]>([])
  /** The last press: what it is on the controller and what the game gets. */
  const pressed = ref<{ which: string; button: string | null } | null>(null)
  let prevHeld = new Set<Pos>()

  async function load() {
    try {
      ab.value = abLayout((await readSetting('pad.ab'))?.value)
      if (pad.value) remap.value = (await emu.controls.bindings(pad.value.family, GLOBAL)).remap
    } catch (e) {
      onError(e)
    }
  }

  async function setAb(id: AbLayout) {
    ab.value = id
    await settings.set('pad.ab', id)
  }

  /** The buttons held now (while the test listens): a new press shows what it does. */
  function onPadState(states: PadState[]) {
    const p = pad.value
    if (!p || !active()) return
    const held = new Set<Pos>(states.find(s => s.id === p.id)?.held ?? [])
    for (const pos of newlyHeld(prevHeld, held)) {
      const button = buttonsUsing(remap.value, pos)[0]
      pressed.value = {
        which: FACES.includes(pos)
          ? t(`firstRun.controller.which.${pos}`)
          : posName(p.family, pos, false),
        button: button ? buttonName(button) : null,
      }
    }
    prevHeld = held
  }

  /** The test starts or stops with the step (Rust sends the pad states only while it listens). */
  function listen(on: boolean) {
    pressed.value = null
    prevHeld = new Set()
    void emu.controls.watch(on)
    if (on) void load()
  }

  return { pad, ab, pressed, load, setAb, onPadState, listen }
}
