// First run S04: the A/B layout (`pad.ab`) and the press test of the first controller connected.
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type PadInfo, type PadState, type Pos, type RemapRow } from '../emu'
import { buttonName, posName } from '../controls'
import { errorText } from '../i18n/labels'

/** The A/B layout: as on Nintendo's consoles, or as printed on the controller. */
export type AbLayout = 'nintendo' | 'printed'

const FACES: Pos[] = ['south', 'east', 'west', 'north']

export function usePadTester(
  pads: Ref<PadInfo[]>,
  /** Whether the test is on screen: presses count only then. */
  active: () => boolean,
  onError: (text: string) => void,
) {
  const { t } = useI18n()
  const pad = computed(() => pads.value[0] ?? null)
  const ab = ref<AbLayout>('nintendo')
  const remap = ref<RemapRow[]>([])
  /** The last press: what it is on the controller and what the game gets. */
  const pressed = ref<{ which: string; button: string | null } | null>(null)
  let prevHeld = new Set<Pos>()

  async function load() {
    try {
      const rows = await emu.settings.page('pad', { level: 'global' })
      ab.value = rows.find(r => r.key === 'pad.ab')?.value === 'printed' ? 'printed' : 'nintendo'
      if (pad.value)
        remap.value = (await emu.controls.bindings(pad.value.family, { level: 'global' })).remap
    } catch (e) {
      onError(errorText(e))
    }
  }

  async function setAb(id: AbLayout) {
    ab.value = id
    try {
      await emu.settings.set('pad.ab', { level: 'global' }, id)
    } catch (e) {
      onError(errorText(e))
    }
  }

  /** The buttons held now (while the test listens): a new press shows what it does. */
  function onPadState(states: PadState[]) {
    const p = pad.value
    if (!p || !active()) return
    const held = new Set<Pos>(states.find(s => s.id === p.id)?.held ?? [])
    for (const pos of held) {
      if (prevHeld.has(pos)) continue
      const button = remap.value.find(r => r.source === pos)?.button
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
