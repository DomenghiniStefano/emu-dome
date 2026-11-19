<script setup lang="ts">
// C03 «Scorciatoie» (board «Controlli»): the shortcut button (`pad.mod`, as a settings row) and
// the shortcuts held with it (`pad.combos`, «Tutti i giochi»). «Cambia» captures the button to
// press with the shortcut button, like the remap rows; a button another shortcut uses asks to
// swap them. The rows come from `DEFAULT_COMBOS` (Rust): the reserved ones, whose feature does
// not exist yet, show their button with a badge and cannot be changed.
// On the keyboard the shortcuts are the fixed keys of keyboard.ts, listed read-only.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ComboRow,
  type HotkeyId,
  type HotkeyStatus,
  type PadFamily,
  type PadState,
  type Pos,
  type SettingRow,
  type SettingsSchema,
} from '../emu'
import { useSettingsChanged } from '../composables/useSettingsChanged'
import { errorText, type ErrorSink } from '../i18n/labels'
import { combosList, comboText, hotkeyName, keyName, posName, useCapture } from '../controls'
import type { CaptureDevice } from '../controls'
import { KEYBOARD_SHORTCUTS } from '../keyboard'
import { GLOBAL } from '../settingsModel'
import { CaptureField, CaptureFieldVariant } from '@/components/ui/capture-field'
import { Card, CardVariant } from '@/components/ui/card'
import { Kbd } from '@/components/ui/kbd'
import SettingRowView from './SettingRow.vue'
import ConflictBanner from './controls/ConflictBanner.vue'
import ShortcutRow from './controls/ShortcutRow.vue'
import { useBindingConflict } from './controls/useBindingConflict'
import { useGlobalSettings } from './controls/useGlobalSettings'

const props = defineProps<{
  family: PadFamily
  device: CaptureDevice
  states: PadState[]
  schema: SettingsSchema
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()
const failWith: ErrorSink = e => emit('error', errorText(e))
const settings = useGlobalSettings(failWith)

/** The keyboard has no controller shortcuts: the names are those of a generic pad. */
const padFamily = computed<PadFamily>(() =>
  props.family === 'keyboard' ? 'generic' : props.family,
)
const rows = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
async function load() {
  try {
    const b = await emu.controls.bindings(padFamily.value, GLOBAL)
    rows.value = b.combos
    modifier.value = b.modifier
  } catch (e) {
    failWith(e)
  }
}

const modEntry = computed(() => props.schema.entries.find(e => e.key === 'pad.mod'))

// ---- Capture and swap ----
const capture = useCapture({
  device: () => props.device,
  state: id => props.states.find(s => s.id === id),
  done: (id, source) => void assign(id as HotkeyId, source as Pos),
})
watch(
  () => props.states,
  s => capture.onPads(s),
)

function change(hotkey: HotkeyId) {
  conflict.value = null
  if (capture.target.value === hotkey) return capture.stop()
  capture.start(hotkey)
}

async function write(changes: Partial<Record<HotkeyId, Pos | null>>) {
  const list = combosList(rows.value, changes)
  if (list.length === 0 && props.globalRows['pad.combos']?.source === 'global')
    await settings.reset('pad.combos')
  else await settings.set('pad.combos', list)
}

const binding = useBindingConflict({
  rows: () => rows.value.map(r => ({ id: r.hotkey, source: r.pos })),
  write,
})
const { conflict, swap } = binding
function assign(hotkey: HotkeyId, pos: Pos) {
  // The shortcut button itself cannot be a shortcut.
  if (pos !== modifier.value) return binding.assign(hotkey, pos)
}

/** The badge of a reserved row: «In arrivo», or «Con Nintendo DS». */
const badgeOf = (status: HotkeyStatus) =>
  status === 'active' ? undefined : t(`controls.badge.${status === 'later' ? 'soon' : 'nds'}`)
/** What a row says to press: the countdown while it listens, the menu's Home beside its combo. */
function comboOf(r: ComboRow): string {
  if (capture.target.value === r.hotkey) return t('controls.pressButton', { n: capture.left.value })
  const combo = comboText(padFamily.value, modifier.value, r.pos)
  return r.hotkey === 'menu' ? t('controls.menuCombo', { combo }) : combo
}
/** Fullscreen has no controller shortcut: its key, from the keyboard table. */
const fullscreenKey = keyName(KEYBOARD_SHORTCUTS.find(k => k.action === 'fullscreen')?.code ?? '')

onMounted(() => void load())
useSettingsChanged(['pad.combos', 'pad.mod'], () => void load())
</script>

<template>
  <div class="flex flex-col gap-12">
    <template v-if="family !== 'keyboard'">
      <Card v-if="modEntry" :variant="CardVariant.List" class="pb-12">
        <SettingRowView
          :entry="modEntry"
          :row="globalRows['pad.mod']"
          :schema="schema"
          level="global"
          :level-name="() => t('settingsUi.scope.global')"
          :scoped="false"
          @set="v => settings.set('pad.mod', v)"
          @reset="settings.reset('pad.mod')"
          @error="onError"
        />
        <p class="m-0 px-16 text-caption-tall text-ink-muted">
          {{ t('controls.modNote') }}
        </p>
      </Card>

      <Card :variant="CardVariant.List">
        <template v-for="(r, i) in rows" :key="r.hotkey">
          <ShortcutRow
            :action="hotkeyName(r.hotkey)"
            :muted="r.status !== 'active'"
            :badge="badgeOf(r.status)"
          >
            {{ comboOf(r) }}
            <template #change>
              <span v-if="r.status !== 'active'"></span>
              <CaptureField
                v-else
                :variant="CaptureFieldVariant.Quiet"
                :listening="capture.target.value === r.hotkey"
                :data-nav-initial="i === 0 ? '' : undefined"
                :aria-label="t('controls.changeShortcut', { action: hotkeyName(r.hotkey) })"
                @click="change(r.hotkey)"
              >
                {{ t('controls.change') }}
              </CaptureField>
            </template>
          </ShortcutRow>
          <ConflictBanner
            v-if="conflict && conflict.id === r.hotkey"
            :text="
              t('controls.comboConflict', {
                source: posName(padFamily, conflict.source, false),
                action: hotkeyName(conflict.other),
              })
            "
            @swap="swap"
            @cancel="conflict = null"
          />
        </template>
        <ShortcutRow :action="t('controls.keys.fullscreen')" muted>
          {{ t('controls.keyboardOnly', { key: fullscreenKey }) }}
        </ShortcutRow>
      </Card>
    </template>

    <Card v-else :variant="CardVariant.List">
      <ShortcutRow
        v-for="k in KEYBOARD_SHORTCUTS"
        :key="k.code"
        :action="hotkeyName(k.action)"
        :muted="k.status !== 'active'"
        :badge="k.status !== 'active' ? t('controls.badge.soon') : undefined"
      >
        <Kbd>{{ keyName(k.code) }}</Kbd>
      </ShortcutRow>
    </Card>
  </div>
</template>
