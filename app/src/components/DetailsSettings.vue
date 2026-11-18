<script setup lang="ts">
// Game details › Impostazioni del gioco (L06, board «Dettagli» desktop and «Tel-Dettagli»): only
// the settings this game has a value of its own for (`settings_game_overrides`), each with its
// page, what every game of its console gets instead, the game's value and «Ripristina»; above
// them «Apri tutte le impostazioni di questo gioco», the Settings section at «Solo questo gioco».
// Phone in portrait: name, the game's value, the console's under it.
//
// Controller / keyboard: the shell's nav scope.
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type GameOverride,
  type LibraryGame,
  type SettingEntry,
  type SettingsSchema,
  type SettingValue,
} from '../emu'
import { gameScopeOf } from '../gameDetails'
import { errorText } from '../i18n/labels'
import { isShown, loadSchema, msg } from '../settingsModel'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Apri tutte le impostazioni di questo gioco». */
  open: []
  error: [string]
}>()

const { t, te } = useI18n()

const scope = computed(() => gameScopeOf(props.game))
const consoleName = computed(() => t(`console.${props.game.console}`))
const schema = ref<SettingsSchema | null>(null)
const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
const overrides = ref<GameOverride[]>([])
const loaded = ref(false)

async function load() {
  const s = scope.value
  if (!s) {
    overrides.value = []
    loaded.value = true
    return
  }
  try {
    schema.value ??= await loadSchema()
    overrides.value = await emu.settings.gameOverrides(s)
  } catch (e) {
    emit('error', errorText(e))
  }
  loaded.value = true
}
watch(scope, () => void load())

interface Row {
  key: string
  name: string
  where: string
  value: string
  base: string | null
}
const rows = computed<Row[]>(() =>
  overrides.value.flatMap(o => {
    const e = byKey.value.get(o.key)
    if (!e || !schema.value || !isShown(e, schema.value)) return []
    return [
      {
        key: o.key,
        name: t(msg.name(o.key)),
        where: t(msg.page(o.page)),
        value: valueText(e, o.value),
        base: o.base === null ? null : valueText(e, o.base),
      },
    ]
  }),
)

/** A value as the settings pages name it. */
function valueText(e: SettingEntry, v: SettingValue): string {
  const option = (id: string) => (te(msg.option(e.key, id)) ? t(msg.option(e.key, id)) : id)
  switch (e.kind.type) {
    case 'toggle':
      return t(v === true ? 'details.settings.on' : 'details.settings.off')
    case 'choice':
      return typeof v === 'string' ? option(v) : String(v)
    case 'multi':
      return Array.isArray(v) ? v.map(option).join(', ') : String(v)
    case 'slider':
      return t(`settingsUi.unit.${e.kind.unit}`, { v: Number(v) })
    case 'bindings': {
      const k = `settings.${e.key}.value.custom`
      return te(k) ? t(k) : ''
    }
    case 'folders':
      return Array.isArray(v) ? v.join(', ') : String(v)
    case 'file':
    case 'colour':
    case 'text':
    case 'action':
    case 'apply':
      return Array.isArray(v) ? v.join(', ') : String(v)
  }
}

async function reset(key: string) {
  const s = scope.value
  if (!s) return
  try {
    await emu.settings.reset(key, s)
  } catch (e) {
    emit('error', errorText(e))
  }
  await load()
}

let unlisten: (() => void) | null = null
onMounted(async () => {
  await load()
  unlisten = await emu.settings.onChanged(c => {
    if (c.level === 'game') void load()
  })
})
onUnmounted(() => unlisten?.())
</script>

<template>
  <div class="settings" :class="{ phone }">
    <div class="head">
      <p class="muted">
        {{
          scope
            ? t('details.settings.lead', { console: consoleName })
            : t('details.settings.noIdentity')
        }}
      </p>
      <button class="link" :disabled="!scope" @click="emit('open')">
        {{ t('details.settings.openAll') }}
      </button>
    </div>
    <div v-if="rows.length" class="card">
      <div v-for="r in rows" :key="r.key" class="row">
        <span v-if="!phone" class="dot" aria-hidden="true"></span>
        <div class="text">
          <span class="name">{{ r.name }}</span>
          <span v-if="phone" class="value">{{ r.value }}</span>
          <span v-if="r.base !== null" class="sub">
            {{
              phone
                ? t('details.settings.baseShort', { console: consoleName, value: r.base })
                : t('details.settings.base', { page: r.where, console: consoleName, value: r.base })
            }}
          </span>
        </div>
        <span v-if="!phone" class="value strong">{{ r.value }}</span>
        <button class="ghost" @click="reset(r.key)">{{ t('settingsUi.reset') }}</button>
      </div>
    </div>
    <p v-else-if="loaded && scope" class="empty muted">
      {{ t('details.settings.none', { console: consoleName }) }}
    </p>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.link {
  flex: 0 0 auto;
  padding: 0;
  border: 0;
  background: none;
  color: var(--accent-text);
  font-weight: 500;
}
.link:hover {
  text-decoration: underline;
}
.link:disabled {
  color: var(--ink-faint);
  cursor: not-allowed;
  text-decoration: none;
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
}
.row + .row {
  border-top: 1px solid var(--line);
}
.dot {
  flex: 0 0 auto;
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--accent-text);
}
.text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name {
  font-weight: 500;
}
.sub {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.value {
  color: var(--accent-text);
}
.value.strong {
  font-weight: 600;
}
.ghost {
  flex: 0 0 auto;
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
.empty {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
}

/* Phone in portrait (board «Tel-Dettagli»). */
.phone .head {
  flex-direction: column;
  align-items: flex-start;
}
.phone .link {
  min-height: 44px;
}
.phone .card,
.phone .empty {
  border-radius: var(--radius-16);
}
.phone .row {
  padding: 12px 14px;
}
.phone .value {
  font-size: 13px;
  line-height: 18px;
}
.phone .ghost {
  min-height: 44px;
}
</style>
