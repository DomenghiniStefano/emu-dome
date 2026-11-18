<script setup lang="ts">
// Game details › Salvataggi (L05, board «Dettagli» desktop and «Tel-Dettagli»): on the left the
// quick saves of this game as slot tiles (the one SlotTile, #59 item 3: «Gioca da qui» and «…»,
// which opens V03), on the right the in-game save (last change, backups) with «Mostra i backup»
// (the Saves section on this game), «Importa…» (V05 for this game) and «Esporta…».
// Phone in portrait: the slots two per row, then the in-game save card.
// The board's Google Drive line waits for the sync (Cloud).
//
// Controller / keyboard: the shell's nav scope; V03 opens its own.
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { TriangleAlert } from '@lucide/vue'
import { AUTO_SLOT, emu, type GameSaves, type LibraryGame, type QuickSave } from '../emu'
import { errorText, saveWhenLabel } from '../i18n/labels'
import QuickSaveDialog from './QuickSaveDialog.vue'
import SlotTile from './SlotTile.vue'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Gioca da qui» on a slot. */
  play: [slot: number]
  backups: []
  import: []
  /** A quick save was deleted: «Riprendi» may change. */
  changed: []
  error: [string]
  notice: [string]
}>()

const { t } = useI18n()

const saves = ref<GameSaves | null>(null)

async function load() {
  try {
    const rows = await emu.settings.page('saves', { level: 'global' })
    const slots = Number(rows.find(r => r.key === 'saves.slots')?.value ?? 10) || 10
    saves.value = await emu.saves.game(props.game.path, slots)
  } catch (e) {
    emit('error', errorText(e))
  }
}
watch(
  () => `${props.game.path}:${props.game.lastPlayed}`,
  () => void load(),
)
onMounted(load)

const slotName = (slot: number) => (slot === AUTO_SLOT ? t('qm.auto') : t('qm.slot', { slot }))

const lastChange = computed(() => {
  const g = saves.value?.inGame
  return g ? saveWhenLabel(g.modified ?? 0) : null
})
const backupsLine = computed(() => {
  const list = saves.value?.backups ?? []
  const oldest = list[list.length - 1]
  return oldest
    ? t(
        'details.saves.backupsLine',
        { n: list.length, when: saveWhenLabel(oldest.at) },
        list.length,
      )
    : t('saves.noBackups')
})

async function exportInGame() {
  try {
    if (await emu.saves.export(props.game.path, { kind: 'inGame' }))
      emit('notice', t('saves.exported'))
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- V03 ----
const quick = ref<QuickSave | null>(null)
let quickReturn: HTMLElement | null = null
function openQuick(q: QuickSave, e: Event) {
  quickReturn = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  quick.value = q
}
function closeQuick() {
  quick.value = null
  void nextTick(() => quickReturn?.focus())
}
function playQuick(slot: number) {
  quick.value = null
  emit('play', slot)
}
async function onChanged() {
  await load()
  emit('changed')
}
</script>

<template>
  <div class="saves" :class="{ phone }">
    <section class="quick">
      <div class="head">
        <h2 class="overline">{{ t('saves.quickTitle') }}</h2>
        <span v-if="!phone" class="muted small">{{ t('details.saves.quickHint') }}</span>
      </div>
      <div v-if="saves && saves.quick.length" class="slots">
        <SlotTile
          v-for="q in saves.quick"
          :key="q.slot"
          :name="slotName(q.slot)"
          :when="q.savedAt !== null ? saveWhenLabel(q.savedAt) : null"
          :thumb-url="q.thumbUrl"
          :outdated="q.outdated"
          @play="emit('play', q.slot)"
          @more="e => openQuick(q, e)"
        />
      </div>
      <p v-else-if="saves" class="muted">{{ t('saves.noQuick') }}</p>
    </section>

    <aside class="card">
      <h2 class="overline">{{ t('saves.inGameTitle') }}</h2>
      <template v-if="phone">
        <div class="num">
          {{
            lastChange
              ? t('details.saves.lastChangeLine', { when: lastChange })
              : t('saves.noInGame')
          }}
        </div>
      </template>
      <template v-else>
        <div class="muted">{{ t('details.saves.inGameLead') }}</div>
        <div class="pair num">
          <span>{{ t('details.saves.lastChange') }}</span>
          <span class="muted">{{ lastChange ?? t('saves.noInGame') }}</span>
        </div>
        <div class="pair">
          <span>{{ t('details.saves.backups') }}</span>
          <span class="muted num">{{ backupsLine }}</span>
        </div>
      </template>
      <!-- E04: the unreadable in-game save was set aside. -->
      <div v-if="saves?.broken.length" class="warning small">
        <TriangleAlert :size="16" :stroke-width="1.75" aria-hidden="true" />
        <span>{{ t('saves.brokenNote', { file: saves.broken[0] }) }}</span>
      </div>
      <div class="buttons">
        <button class="btn" @click="emit('backups')">{{ t('details.saves.showBackups') }}</button>
        <button class="btn" @click="emit('import')">{{ t('saves.import') }}</button>
        <button v-if="!phone && saves?.inGame" class="btn" @click="exportInGame">
          {{ t('saves.export') }}
        </button>
      </div>
    </aside>

    <QuickSaveDialog
      v-if="quick"
      :game="game"
      :quick="quick"
      @close="closeQuick"
      @play="playQuick"
      @changed="onChanged"
      @error="text => emit('error', text)"
      @notice="text => emit('notice', text)"
    />
  </div>
</template>

<style scoped>
.saves {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.quick {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.overline {
  margin: 0;
}
.slots {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
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
.pair {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.pair > :last-child {
  text-align: right;
}
.warning {
  display: flex;
  gap: 8px;
  color: var(--warning);
}
.buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

/* Phone in portrait (board «Tel-Dettagli»). */
.saves.phone {
  grid-template-columns: minmax(0, 1fr);
  gap: 10px;
}
.phone .slots {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.phone .card {
  border-radius: var(--radius-16);
  padding: 14px;
  gap: 6px;
  font-size: 14px;
}
.phone .buttons {
  margin-top: 0;
}
.phone .btn {
  min-height: 44px;
  padding: 0 14px;
  border-radius: var(--radius-10);
}
</style>
