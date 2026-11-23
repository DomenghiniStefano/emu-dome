<script setup lang="ts">
// Game details › Salvataggi (L05, board «Dettagli» desktop and «Tel-Dettagli»): on the left the
// quick saves of this game as slot tiles (the one SlotTile, #59 item 3: «Gioca da qui» and «…»,
// which opens V03), on the right the in-game save (InGameSaveCard).
// Phone in portrait: the slots two per row, then the in-game save card.
// The board's Google Drive line waits for the sync (Cloud).
//
// Controller / keyboard: the shell's nav scope; V03 opens its own.
import { nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT, type GameSaves, type LibraryGame, type QuickSave } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { saveWhenLabel } from '@/i18n/labels'
import { loadGameSaves } from './gameSaves'
import InGameSaveCard from './InGameSaveCard.vue'
import QuickSaveDialog from '@/components/saves/QuickSaveDialog.vue'
import SlotTile from '@/components/saves/SlotTile.vue'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Gioca da qui» on a slot. */
  play: [slot: number]
  backups: []
  import: []
  /** A quick save was deleted: «Riprendi» may change. */
  changed: []
}>()

const { t } = useI18n()
const failWith = useErrorSink()

const saves = ref<GameSaves | null>(null)

async function load() {
  const id = props.game.id
  try {
    const loaded = await loadGameSaves(props.game)
    // Rule 7: the details show another game now; its own load shows.
    if (props.game.id === id) saves.value = loaded
  } catch (e) {
    if (props.game.id === id) failWith(e)
  }
}
watch(
  () => `${props.game.id}:${props.game.lastPlayed}`,
  () => void load(),
)
onMounted(load)

const slotName = (slot: number) => (slot === AUTO_SLOT ? t('qm.auto') : t('qm.slot', { slot }))

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
  <div
    class="grid grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)] items-start gap-16 touch:grid-cols-1 touch:gap-10"
  >
    <section class="flex flex-col gap-8">
      <div class="flex items-center justify-between gap-8">
        <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('saves.quickTitle') }}</Heading>
        <span v-if="!phone" class="text-caption text-ink-muted">
          {{ t('details.saves.quickHint') }}
        </span>
      </div>
      <div
        v-if="saves && saves.quick.length"
        class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-card-min),1fr))] gap-8 touch:grid-cols-2"
      >
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
      <p v-else-if="saves" class="m-0 text-ink-muted">{{ t('saves.noQuick') }}</p>
    </section>

    <InGameSaveCard
      :game="game"
      :saves="saves"
      :phone="phone"
      @backups="emit('backups')"
      @import="emit('import')"
    />

    <QuickSaveDialog
      v-if="quick"
      :game="game"
      :quick="quick"
      @close="closeQuick"
      @play="playQuick"
      @changed="onChanged"
    />
  </div>
</template>
