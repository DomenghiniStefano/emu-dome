<script setup lang="ts">
// C03 «Scorciatoie» (board «Controlli»): the shortcut button (`pad.mod`, as a settings row) and
// the shortcuts held with it (`pad.combos`, «Tutti i giochi»). «Cambia» captures the button to
// press with the shortcut button, like the remap rows; a button another shortcut uses asks to
// swap them. The rows come from `DEFAULT_COMBOS` (Rust): the reserved ones, whose feature does
// not exist yet, show their button with a badge and cannot be changed.
// On the keyboard the shortcuts are the fixed keys of keyboard.ts, listed read-only.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ComboRow,
  type HotkeyId,
  type HotkeyStatus,
  type PadFamily,
  type PadState,
  type Pos,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
} from '../emu'
import { errorText } from '../i18n/labels'
import { combosList, comboText, hotkeyName, keyName, posName, useCapture } from '../controls'
import type { CaptureDevice } from '../controls'
import { KEYBOARD_SHORTCUTS } from '../keyboard'
import SettingRowView from './SettingRow.vue'

const props = defineProps<{
  family: PadFamily
  device: CaptureDevice
  states: PadState[]
  schema: SettingsSchema
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

const GLOBAL = { level: 'global' } as const

const rows = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
async function load() {
  // The keyboard has no controller shortcuts: the names are those of a generic pad.
  const family = props.family === 'keyboard' ? 'generic' : props.family
  try {
    const b = await emu.controls.bindings(family, GLOBAL)
    rows.value = b.combos
    modifier.value = b.modifier
  } catch (e) {
    emit('error', errorText(e))
  }
}
const padFamily = computed<PadFamily>(() =>
  props.family === 'keyboard' ? 'generic' : props.family,
)

const modEntry = computed<SettingEntry | undefined>(() =>
  props.schema.entries.find(e => e.key === 'pad.mod'),
)
async function setSetting(key: string, value: SettingValue) {
  try {
    await emu.settings.set(key, GLOBAL, value)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function resetSetting(key: string) {
  try {
    await emu.settings.reset(key, GLOBAL)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- Capture and swap ----
interface Conflict {
  hotkey: HotkeyId
  other: HotkeyId
  pos: Pos
}
const conflict = ref<Conflict | null>(null)
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
    await resetSetting('pad.combos')
  else await setSetting('pad.combos', list)
}

async function assign(hotkey: HotkeyId, pos: Pos) {
  // The shortcut button itself cannot be a shortcut.
  if (pos === modifier.value) return
  const other = rows.value.find(r => r.hotkey !== hotkey && r.pos === pos)
  if (other) {
    conflict.value = { hotkey, other: other.hotkey, pos }
    await nextTick()
    document.querySelector<HTMLElement>('[data-conflict-swap]')?.focus()
    return
  }
  await write({ [hotkey]: pos })
}
async function swap() {
  const c = conflict.value
  if (!c) return
  conflict.value = null
  const mine = rows.value.find(r => r.hotkey === c.hotkey)?.pos ?? null
  await write({ [c.hotkey]: c.pos, [c.other]: mine })
}

/** The badge of a reserved row: «In arrivo», or «Con Nintendo DS». */
const badge = (status: HotkeyStatus) => (status === 'later' ? 'soon' : 'nds')
/** Fullscreen has no controller shortcut: its key, from the keyboard table. */
const fullscreenKey = keyName(KEYBOARD_SHORTCUTS.find(k => k.action === 'fullscreen')?.code ?? '')

let unlisten: (() => void) | null = null
onMounted(async () => {
  await load()
  unlisten = await emu.settings.onChanged(e => {
    if (e.key === 'pad.combos' || e.key === 'pad.mod') void load()
  })
})
onBeforeUnmount(() => unlisten?.())
</script>

<template>
  <div class="shortcuts">
    <template v-if="family !== 'keyboard'">
      <div v-if="modEntry" class="card mod">
        <SettingRowView
          :entry="modEntry"
          :row="globalRows['pad.mod']"
          :schema="schema"
          level="global"
          :level-name="() => t('settingsUi.scope.global')"
          :scoped="false"
          @set="v => setSetting('pad.mod', v)"
          @reset="resetSetting('pad.mod')"
          @error="text => emit('error', text)"
        />
        <p class="note">{{ t('controls.modNote') }}</p>
      </div>

      <div class="card list">
        <template v-for="(r, i) in rows" :key="r.hotkey">
          <div class="tr" :class="{ later: r.status !== 'active' }">
            <span class="action">{{ hotkeyName(r.hotkey) }}</span>
            <span class="combo num">
              <span v-if="r.status !== 'active'" class="badge">
                {{ t(`controls.badge.${badge(r.status)}`) }}
              </span>
              <template v-if="capture.target.value === r.hotkey">
                {{ t('controls.pressButton', { n: capture.left.value }) }}
              </template>
              <template v-else-if="r.hotkey === 'menu'">
                {{ t('controls.menuCombo', { combo: comboText(padFamily, modifier, r.pos) }) }}
              </template>
              <template v-else>{{ comboText(padFamily, modifier, r.pos) }}</template>
            </span>
            <span v-if="r.status !== 'active'"></span>
            <button
              v-else
              class="ghost"
              :class="{ listening: capture.target.value === r.hotkey }"
              :data-nav-initial="i === 0 ? '' : undefined"
              :aria-label="t('controls.changeShortcut', { action: hotkeyName(r.hotkey) })"
              @click="change(r.hotkey)"
            >
              {{ t('controls.change') }}
            </button>
          </div>
          <div v-if="conflict && conflict.hotkey === r.hotkey" class="conflict" role="alert">
            <span class="conflict-text">
              {{
                t('controls.comboConflict', {
                  source: posName(padFamily, conflict.pos, false),
                  action: hotkeyName(conflict.other),
                })
              }}
            </span>
            <button class="btn" data-conflict-swap @click="swap">{{ t('controls.swap') }}</button>
            <button class="btn" @click="conflict = null">{{ t('controls.cancel') }}</button>
          </div>
        </template>
        <div class="tr later">
          <span class="action">{{ t('controls.keys.fullscreen') }}</span>
          <span class="combo num">{{ t('controls.keyboardOnly', { key: fullscreenKey }) }}</span>
          <span></span>
        </div>
      </div>
    </template>

    <div v-else class="card list">
      <div
        v-for="k in KEYBOARD_SHORTCUTS"
        :key="k.code"
        class="tr"
        :class="{ later: k.status !== 'active' }"
      >
        <span class="action">{{ hotkeyName(k.action) }}</span>
        <span class="combo num">
          <span v-if="k.status !== 'active'" class="badge">{{ t('controls.badge.soon') }}</span>
          <kbd>{{ keyName(k.code) }}</kbd>
        </span>
        <span></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shortcuts {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.mod {
  padding: 0 0 12px;
}
.note {
  margin: 0;
  padding: 0 16px;
  font-size: 12px;
  color: var(--ink-muted);
}
.tr {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto 72px;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-top: 1px solid var(--line);
}
.tr:first-child {
  border-top: 0;
}
.combo {
  min-width: 150px;
  text-align: right;
  font-weight: 600;
  display: inline-flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
}
.later .action {
  color: var(--ink-muted);
}
.badge {
  font-size: 11px;
  font-weight: 500;
  padding: 0 6px;
  border-radius: 4px;
  border: 1px dashed var(--line-strong);
  color: var(--ink-muted);
}
.ghost {
  height: 28px;
  padding: 0 8px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink-muted);
}
.ghost:hover {
  background: var(--surface-hover);
}
.ghost.listening {
  color: var(--accent-text);
  outline: 1px dashed var(--accent-text);
}
.conflict {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: color-mix(in srgb, var(--warning) 10%, transparent);
  border-top: 1px solid color-mix(in srgb, var(--warning) 33%, transparent);
}
.conflict-text {
  flex: 1;
}
</style>
