<script setup lang="ts">
// Game details › Controlli (L07, board «Dettagli» desktop): cards for this game's controls.
// - Controller e tastiera: whether `pad.remap` has a value of its own at «Solo questo gioco», and
//   «Rimappa i pulsanti per questo gioco» (C02 at the Game level, `controlsFocus`).
// - Pressione ripetuta (turbo): not built yet (`pad.turbo` has no backend), X05 with the reason.
// - Controlli touch (telefono): customised on the phone (Phase 4).
// - Tocco sullo schermo inferiore: Nintendo DS and 3DS games only, `pad.pointer` for this game.
// Phone in portrait: the same cards, one per row.
//
// Controller / keyboard: the shell's nav scope.
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame, type SettingRow } from '../emu'
import { gameScopeOf } from '../gameDetails'
import { errorText } from '../i18n/labels'
import { msg } from '../settingsModel'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Rimappa i pulsanti per questo gioco». */
  remap: []
  error: [string]
}>()

const { t, te } = useI18n()

const scope = computed(() => gameScopeOf(props.game))
const consoleName = computed(() => t(`console.${props.game.console}`))
/** Nintendo DS and 3DS: the bottom screen is touched. */
const twoScreens = computed(() => props.game.console === 'nds' || props.game.console === '3ds')

const rows = ref<Record<string, SettingRow>>({})
async function load() {
  const s = scope.value
  if (!s) return
  try {
    const page = await emu.settings.page('pad', s)
    rows.value = Object.fromEntries(page.map(r => [r.key, r]))
  } catch (e) {
    emit('error', errorText(e))
  }
}
watch(scope, () => void load())

/** `pad.remap` stored at this game's level. */
const ownRemap = computed(() => rows.value['pad.remap']?.source === 'game')
const pointer = computed(() => {
  const v = rows.value['pad.pointer']?.value
  const id = typeof v === 'string' ? v : 'mouse'
  const key = msg.option('pad.pointer', id)
  return te(key) ? t(key) : id
})

let unlisten: (() => void) | null = null
onMounted(async () => {
  await load()
  unlisten = await emu.settings.onChanged(c => {
    if (c.key.startsWith('pad.')) void load()
  })
})
onUnmounted(() => unlisten?.())
</script>

<template>
  <div class="controls" :class="{ phone }">
    <div class="card">
      <h2 class="overline">{{ t('details.controls.padTitle') }}</h2>
      <div class="strong">
        {{
          ownRemap
            ? t('details.controls.remapOwn')
            : t('details.controls.remapInherited', { console: consoleName })
        }}
      </div>
      <div class="muted">{{ t('details.controls.remapLead') }}</div>
      <button class="btn" :disabled="!scope" @click="emit('remap')">
        {{ t('details.controls.remap') }}
      </button>
    </div>
    <div class="card">
      <h2 class="overline">{{ t('details.controls.turboTitle') }}</h2>
      <div class="muted">{{ t('details.notYet') }}</div>
    </div>
    <div class="card">
      <h2 class="overline">{{ t('details.controls.touchTitle') }}</h2>
      <div class="muted">{{ t('details.controls.touchLead') }}</div>
    </div>
    <div v-if="twoScreens" class="card">
      <h2 class="overline">{{ t('details.controls.pointerTitle') }}</h2>
      <div class="strong">{{ pointer }}</div>
      <div class="muted">{{ t('details.controls.pointerLead') }}</div>
    </div>
  </div>
</template>

<style scoped>
.controls {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
/* An odd card out takes the whole row: no empty cell beside it. */
.card:last-child:nth-child(odd) {
  grid-column: 1 / -1;
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.overline {
  margin: 0;
}
.strong {
  font-weight: 500;
}
.muted {
  color: var(--ink-muted);
}
.btn {
  align-self: flex-start;
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Phone in portrait: one card per row. */
.controls.phone {
  grid-template-columns: minmax(0, 1fr);
}
.phone .card {
  border-radius: var(--radius-16);
  padding: 14px;
  gap: 6px;
}
.phone .btn {
  min-height: 44px;
  padding: 0 14px;
  border-radius: var(--radius-10);
}
</style>
