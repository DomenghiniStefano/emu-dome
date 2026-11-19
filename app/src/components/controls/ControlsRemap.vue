<script setup lang="ts">
// C02 «Rimappatura dei pulsanti» (board «Controlli», CaptureField): which button of the chosen
// device gives each game button, at «Tutti i giochi», «Giochi <console>» or «Solo questo gioco».
// A row (click, A or Enter) starts the capture: 5 s to press the button; B or Esc held 1 s
// cancel. A button already used by another row asks «… è già … Scambiali?».
// Beside it the tester: every position held lights up, and the last press says what it does.
//
// Values: `pad.remap` (overrides per controller type, `crates/emudome-core/src/bindings.rs`),
// resolved with the defaults by `emu.controls.bindings`; the level and its table are
// controls/useRemapLevel.ts's, the capture's conflicts controls/useBindingConflict.ts's.
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { type Button as GameButton, type PadFamily, type PadState, type SettingRow } from '@/emu'
import { errorText, type ErrorSink } from '@/i18n/labels'
import type { GameFocus } from '@/lib/gameDetails'
import {
  abLayout,
  buttonName,
  posName,
  sourceName,
  testerCells,
  useCapture,
  type CaptureDevice,
} from '@/lib/controls'
import { Button, ButtonFade } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import ConflictBanner from './ConflictBanner.vue'
import RemapTableRow from './RemapTableRow.vue'
import TesterCard from './TesterCard.vue'
import RemapToolbar from './RemapToolbar.vue'
import { remapRowStatus, sourceUses } from './remapTable'
import { buttonsUsing } from './tester'
import { useBindingConflict } from './useBindingConflict'
import { useButtonTester } from './useButtonTester'
import { useGlobalSettings } from './useGlobalSettings'
import { useRemapLevel } from './useRemapLevel'
import { Heading, HeadingVariant } from '@/components/ui/heading'

const props = defineProps<{
  family: PadFamily
  device: CaptureDevice
  states: PadState[]
  globalRows: Record<string, SettingRow>
  /** Opened from a game's details (L07): its «Solo questo gioco», chosen on arrival. */
  game?: GameFocus | null
}>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()
const failWith: ErrorSink = e => emit('error', errorText(e))

// ---- The table at the level being edited ----
const {
  level,
  contextConsole,
  levels,
  rows,
  change,
  restoreDefaults: restoreLevel,
} = useRemapLevel({
  family: () => props.family,
  game: () => props.game,
  onError: failWith,
})
const uses = computed(() => sourceUses(rows.value))
const name = (source: string | null) => sourceName(props.family, source)

// ---- Capture and conflicts ----
const capture = useCapture({
  device: () => props.device,
  state: id => props.states.find(s => s.id === id),
  done: (id, source) => void assign(id as GameButton, source),
})
const { conflict, assign, swap } = useBindingConflict({
  rows: () => rows.value.map(r => ({ id: r.button, source: r.source })),
  write: change,
  assigned: (button, source) =>
    (tester.last.value = t('controls.assigned', {
      source: name(source),
      button: buttonName(button),
    })),
})

// ---- Tester: what is held lights up; the last press says what it does in the game ----
function describe(source: string) {
  const users = buttonsUsing(rows.value, source).map(buttonName)
  return users.length
    ? t('controls.pressInGame', { source: name(source), button: users.join(', ') })
    : t('controls.pressUnused', { source: name(source) })
}
const tester = useButtonTester({
  padId: () => (props.device.kind === 'pad' ? props.device.id : null),
  describe,
  paused: () => capture.target.value !== null,
})
/** The tester's cells: the 12 buttons of a controller, or the keys in use on the keyboard. */
const cells = computed(() => testerCells(props.family, rows.value))
// The capture first: a press that ends it is then described by the tester.
watch(
  () => props.states,
  s => {
    capture.onPads(s)
    tester.onPads(s)
  },
)

function startCapture(button: GameButton) {
  conflict.value = null
  if (capture.target.value === button) return capture.stop()
  capture.start(button)
}

async function restoreDefaults() {
  conflict.value = null
  await restoreLevel()
}

// ---- A/B layout (`pad.ab`, «Tutti i giochi») ----
const settings = useGlobalSettings(failWith)
const ab = computed({
  get: () => abLayout(props.globalRows['pad.ab']?.value),
  set: id => void settings.set('pad.ab', id),
})

const hint = computed(() => {
  const target = capture.target.value
  if (!target) return t('controls.testerHint')
  const back = props.device.kind === 'pad' ? posName(props.family, props.device.back, false) : 'Esc'
  return t(props.device.kind === 'pad' ? 'controls.captureHintPad' : 'controls.captureHintKey', {
    button: buttonName(target as GameButton),
    back,
  })
})
</script>

<template>
  <div class="flex flex-col gap-18">
    <RemapToolbar
      v-model:level="level"
      v-model:ab="ab"
      :levels="levels"
      :context-console="contextConsole"
      :with-ab="family !== 'keyboard'"
    />

    <div class="grid grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)] items-start gap-20">
      <Card as="section" :variant="CardVariant.List">
        <div
          class="grid grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_auto] items-center gap-12 bg-surface-raised px-16 py-8 text-overline text-ink-muted"
        >
          <span>{{ t('controls.inGame') }}</span>
          <span>{{ family === 'keyboard' ? t('controls.onKeyboard') : t('controls.onPad') }}</span>
          <span></span>
        </div>
        <template v-for="(r, i) in rows" :key="r.button">
          <RemapTableRow
            :button="buttonName(r.button)"
            :source="name(r.source)"
            :status="remapRowStatus(r, capture.target.value === r.button, uses)"
            :listening="capture.target.value === r.button"
            :seconds-left="capture.left.value"
            :keyboard="family === 'keyboard'"
            :nav-initial="i === 0"
            @capture="startCapture(r.button)"
          />
          <ConflictBanner
            v-if="conflict && conflict.id === r.button"
            :text="
              t('controls.conflict', {
                source: name(conflict.source),
                button: buttonName(conflict.other),
              })
            "
            @swap="swap"
            @cancel="conflict = null"
          />
        </template>
      </Card>

      <aside class="flex flex-col gap-12">
        <TesterCard
          :hint="hint"
          :cells="cells"
          :held="tester.held.value"
          :last="tester.last.value"
          :idle="t('controls.noPress')"
        />
        <Card class="gap-10">
          <Heading as="h2" :variant="HeadingVariant.Overline">{{
            t('controls.otherActions')
          }}</Heading>
          <Button class="self-start" @click="restoreDefaults">
            {{ t('controls.restoreDefaults') }}
          </Button>
          <Button
            :fade="ButtonFade.Dim"
            class="self-start"
            disabled
            :title="t('settingsUi.notYet')"
          >
            {{ t('controls.copyFrom') }}
          </Button>
          <Button
            :fade="ButtonFade.Dim"
            class="self-start"
            disabled
            :title="t('settingsUi.notYet')"
          >
            {{ t('controls.turbo') }}
          </Button>
        </Card>
      </aside>
    </div>
  </div>
</template>
