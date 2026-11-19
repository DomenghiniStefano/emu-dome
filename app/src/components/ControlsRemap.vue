<script setup lang="ts">
// C02 «Rimappatura dei pulsanti» (board «Controlli», CaptureField): which button of the chosen
// device gives each game button, at «Tutti i giochi», «Giochi <console>» or «Solo questo gioco».
// A row (click, A or Enter) starts the capture: 5 s to press the button; B or Esc held 1 s
// cancel. A button already used by another row asks «… è già … Scambiali?».
// Beside it the tester: every position held lights up, and the last press says what it does.
//
// Values: `pad.remap` (overrides per controller type, `crates/emudome-core/src/bindings.rs`),
// resolved with the defaults by `emu.controls.bindings`.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type Button as GameButton,
  type ConsoleId,
  type PadFamily,
  type PadState,
  type RemapRow,
  type SettingRow,
  type SettingsScope,
} from '../emu'
import { errorText } from '../i18n/labels'
import type { GameFocus } from '../gameDetails'
import { library } from '../library'
import {
  buttonName,
  posName,
  remapList,
  remapWithout,
  sourceName,
  testerCells,
  useCapture,
  type CaptureDevice,
} from '../controls'
import { Button, ButtonFade } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import ConflictBanner from './controls/ConflictBanner.vue'
import RemapTableRow from './controls/RemapTableRow.vue'
import RemapTesterCard from './controls/RemapTesterCard.vue'
import RemapToolbar from './controls/RemapToolbar.vue'
import {
  RemapLevel,
  lastPlayedConsole,
  levelScope,
  remapLevels,
  type RemapLevel as Level,
} from './controls/levels'
import { remapRowStatus, sourceUses } from './controls/remapTable'
import { buttonsUsing } from './controls/tester'
import { useButtonTester } from './controls/useButtonTester'
import { useGlobalSettings } from './controls/useGlobalSettings'

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
const onError = (text: string) => emit('error', text)

// ---- Level: «Tutti i giochi», «Giochi <console>», «Solo questo gioco» ----
const level = ref<Level>(RemapLevel.Global)
const contextConsole = ref<ConsoleId | null>(null)
const gameScope = ref<SettingsScope | null>(null)
async function findContext() {
  if (props.game) {
    gameScope.value = props.game.scope
    contextConsole.value = props.game.scope.console
    level.value = RemapLevel.Game
    return
  }
  gameScope.value = await emu.settings.gameScope().catch(() => null)
  const open = gameScope.value
  contextConsole.value =
    open && open.level === 'game' ? open.console : lastPlayedConsole(library.games)
}
const levels = computed(() => remapLevels(contextConsole.value, gameScope.value))
const scope = computed(() => levelScope(level.value, contextConsole.value, gameScope.value))

// ---- The table at this level ----
const rows = ref<RemapRow[]>([])
/** `pad.remap` as stored or inherited at this level (the list a change starts from). */
const stored = ref<string[]>([])
const storedHere = ref(false)
async function load() {
  try {
    const [b, page] = await Promise.all([
      emu.controls.bindings(props.family, scope.value),
      emu.settings.page('pad', scope.value),
    ])
    rows.value = b.remap
    const row = page.find(r => r.key === 'pad.remap')
    stored.value = Array.isArray(row?.value) ? row.value : []
    storedHere.value = row?.source === scope.value.level
  } catch (e) {
    onError(errorText(e))
  }
}
watch(scope, () => void load(), { deep: true })

async function write(list: string[]) {
  try {
    if (list.length === 0 && level.value === RemapLevel.Global && storedHere.value)
      await emu.settings.reset('pad.remap', scope.value)
    else await emu.settings.set('pad.remap', scope.value, list)
  } catch (e) {
    onError(errorText(e))
  }
}

const uses = computed(() => sourceUses(rows.value))
const name = (source: string | null) => sourceName(props.family, source)

// ---- Capture and conflicts ----
interface Conflict {
  button: GameButton
  other: GameButton
  source: string
}
const conflict = ref<Conflict | null>(null)

const capture = useCapture({
  device: () => props.device,
  state: id => props.states.find(s => s.id === id),
  done: (id, source) => void assign(id as GameButton, source),
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

async function assign(button: GameButton, source: string) {
  const row = rows.value.find(r => r.button === button)
  if (!row || row.source === source) return
  const other = rows.value.find(r => r.button !== button && r.source === source)
  if (other) {
    conflict.value = { button, other: other.button, source }
    return
  }
  tester.last.value = t('controls.assigned', { source: name(source), button: buttonName(button) })
  await write(remapList(stored.value, props.family, rows.value, { [button]: source }))
}

async function swap() {
  const c = conflict.value
  if (!c) return
  conflict.value = null
  const mine = rows.value.find(r => r.button === c.button)?.source ?? null
  tester.last.value = t('controls.assigned', {
    source: name(c.source),
    button: buttonName(c.button),
  })
  await write(
    remapList(stored.value, props.family, rows.value, { [c.button]: c.source, [c.other]: mine }),
  )
}

async function restoreDefaults() {
  conflict.value = null
  await write(remapWithout(stored.value, props.family))
}

// ---- A/B layout (`pad.ab`, «Tutti i giochi») ----
const settings = useGlobalSettings(onError)
const ab = computed({
  get: () => String(props.globalRows['pad.ab']?.value ?? 'nintendo'),
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

let unlisten: (() => void) | null = null
onMounted(async () => {
  await findContext()
  await load()
})
onMounted(async () => {
  unlisten = await emu.settings.onChanged(e => {
    if (e.key.startsWith('pad.')) void load()
  })
})
onBeforeUnmount(() => unlisten?.())
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
            v-if="conflict && conflict.button === r.button"
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
        <RemapTesterCard
          :hint="hint"
          :cells="cells"
          :held="tester.held.value"
          :last="tester.last.value"
        />
        <Card class="gap-10">
          <div class="text-overline text-ink-muted">{{ t('controls.otherActions') }}</div>
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
