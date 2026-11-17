<script setup lang="ts">
// C04 «Stick e grilletti» (board «Controlli»): the `pad.*` rows of sticks, triggers and menus,
// as settings rows at «Tutti i giochi» (the same rows as the Settings pages), and the live
// stick: the dashed circle is the dead zone, the dot the stick's position now.
// They take effect at once: the controller thread reloads them on `settings://changed`.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type PadState,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
} from '../emu'
import { errorText } from '../i18n/labels'
import { isShown } from '../settingsModel'
import type { CaptureDevice } from '../controls'
import SettingRowView from './SettingRow.vue'

const props = defineProps<{
  device: CaptureDevice
  states: PadState[]
  schema: SettingsSchema
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

const GROUPS: { id: string; keys: string[] }[] = [
  { id: 'sticks', keys: ['pad.dead', 'pad.stickdpad', 'pad.trig', 'pad.rumble'] },
  { id: 'menus', keys: ['pad.confirm', 'pad.repeat'] },
]
const groups = computed(() =>
  GROUPS.map(g => ({
    id: g.id,
    entries: g.keys
      .map(k => props.schema.entries.find(e => e.key === k))
      .filter((e): e is SettingEntry => !!e && isShown(e, props.schema)),
  })),
)

const GLOBAL = { level: 'global' } as const
async function set(key: string, value: SettingValue) {
  try {
    await emu.settings.set(key, GLOBAL, value)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function reset(key: string) {
  try {
    await emu.settings.reset(key, GLOBAL)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- Live stick ----
const SIZE = 120
const DOT = 12
const dead = computed(() => Number(props.globalRows['pad.dead']?.value ?? 15))
const stick = computed(() => {
  const id = props.device.kind === 'pad' ? props.device.id : null
  const axes = props.states.find(s => s.id === id)?.axes
  return { x: (axes?.[0] ?? 0) / 32767, y: (axes?.[1] ?? 0) / 32767 }
})
const dotStyle = computed(() => {
  const r = (SIZE - DOT) / 2
  return {
    left: `${r + stick.value.x * r}px`,
    top: `${r + stick.value.y * r}px`,
  }
})
const deadStyle = computed(() => {
  const d = (SIZE * dead.value) / 100
  return { width: `${d}px`, height: `${d}px` }
})
</script>

<template>
  <div class="sticks">
    <section v-for="g in groups" :key="g.id" class="group">
      <div class="overline">{{ t(`settings.sections.pad.${g.id}`) }}</div>
      <div class="card">
        <SettingRowView
          v-for="e in g.entries"
          :key="e.key"
          :entry="e"
          :row="globalRows[e.key]"
          :schema="schema"
          level="global"
          :level-name="() => t('settingsUi.scope.global')"
          :scoped="false"
          @set="v => set(e.key, v)"
          @reset="reset(e.key)"
          @error="text => emit('error', text)"
        />
      </div>
    </section>

    <div class="card live">
      <div
        class="pad"
        role="img"
        :aria-label="t('controls.stickLabel')"
        :style="{ width: `${SIZE}px`, height: `${SIZE}px` }"
      >
        <span class="dead" :style="deadStyle"></span>
        <span class="dot" :style="dotStyle"></span>
      </div>
      <div class="muted">{{ t('controls.stickHint') }}</div>
    </div>
  </div>
</template>

<style scoped>
.sticks {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.live {
  padding: 16px;
  display: flex;
  gap: 24px;
  align-items: center;
}
.pad {
  position: relative;
  flex: 0 0 auto;
  border-radius: 999px;
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.dead {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  border-radius: 999px;
  border: 1px dashed var(--line-strong);
}
.dot {
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 999px;
  background: var(--accent-text);
}
.muted {
  color: var(--ink-muted);
}
</style>
