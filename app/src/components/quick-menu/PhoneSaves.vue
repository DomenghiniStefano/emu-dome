<script setup lang="ts">
// The phone quick menu's «Altro › Salvataggi» (owner, 2026-11-22): every slot in three columns,
// the automatic one and the quick one tagged; for the chosen slot short fixed actions («Carica»,
// «Salva», «Rendi rapido»: the slot is already written on its tile, so no label overflows); the
// two undos, always here, disabled when there is nothing to undo; where the in-game save's
// backups are. Carica loads and resumes, like «Carica rapido».
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArchiveRestore, RotateCcw, Save, Undo2, Zap } from '@lucide/vue'
import { AUTO_SLOT, type SlotInfo, type StateLoadBlock } from '@/emu'
import { stateLoadBlockText } from '@/i18n/labels'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { CardVariant } from '@/components/ui/card'
import { LabelledBlock } from '@/components/ui/labelled-block'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import PageRow from './PageRow.vue'
import QuickSlotTile from './QuickSlotTile.vue'
import SavedLine from './SavedLine.vue'
import { canMakeQuick } from './phoneMenu'

const props = defineProps<{
  slots: SlotInfo[]
  chosen: number
  chosenEmpty: boolean
  autoChosen: boolean
  /** The quick slot (Rust's current slot). */
  quickSlot: number
  /** A save made here, while its «Annulla» line shows. */
  savedSlot: number | null
  loadBlock: StateLoadBlock | null
  /** The slot of a load that can be undone (`saves.undo` on), else null. */
  undoLoadSlot: number | null
  /** The slot of the last save that can be undone, else null. */
  lastSaved: number | null
}>()
const emit = defineEmits<{
  pick: [number]
  load: []
  save: []
  makeQuick: []
  undoSave: []
  undoLoad: []
}>()

const { t } = useI18n()

function tagOf(slot: number): string | undefined {
  if (slot === AUTO_SLOT) return t('qm.autoBadge')
  return slot === props.quickSlot ? t('qm.quickBadge') : undefined
}
const savedThumb = computed(
  () => props.slots.find(s => s.slot === props.savedSlot)?.thumbUrl ?? null,
)
const undoLoadValue = computed(() =>
  props.undoLoadSlot === null
    ? undefined
    : props.undoLoadSlot === AUTO_SLOT
      ? t('qm.auto')
      : t('qm.slot', { slot: props.undoLoadSlot }),
)
</script>

<template>
  <LabelledBlock :title="t('qm.slotsTitle')" :aria-label="t('qm.slotsLabel')">
    <RadioGroup
      :model-value="chosen"
      class="grid grid-cols-3 gap-6"
      :aria-label="t('qm.slotsLabel')"
      @update:model-value="v => emit('pick', Number(v))"
    >
      <RadioGroupItem v-for="s in slots" :key="s.slot" :value="s.slot">
        <QuickSlotTile
          :info="s"
          :chosen="s.slot === chosen"
          :tag="tagOf(s.slot)"
          :variant="CardVariant.TileQuiet"
          short-date
        />
      </RadioGroupItem>
    </RadioGroup>
    <div class="grid grid-cols-[1fr_1fr_auto] gap-6">
      <Button
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Tool"
        :fade="ButtonFade.Dim"
        class="min-w-0"
        :disabled="chosenEmpty || loadBlock !== null"
        :icon="RotateCcw"
        @click="emit('load')"
      >
        {{ t('qm.load') }}
      </Button>
      <Button
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Tool"
        :fade="ButtonFade.Dim"
        class="min-w-0"
        :disabled="autoChosen"
        :icon="Save"
        @click="emit('save')"
      >
        {{ t('qm.save') }}
      </Button>
      <Button
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Tool"
        :fade="ButtonFade.Dim"
        :disabled="!canMakeQuick(chosen, quickSlot)"
        :icon="Zap"
        :title="t('qm.makeQuickLong')"
        @click="emit('makeQuick')"
      >
        {{ t('qm.makeQuick') }}
      </Button>
    </div>
    <span v-if="autoChosen" class="text-caption text-ink-muted">{{ t('qm.autoUpdates') }}</span>
    <span v-if="loadBlock !== null" class="text-caption text-ink-muted">
      {{ stateLoadBlockText(loadBlock) }}
    </span>
    <SavedLine
      v-if="savedSlot !== null"
      :saved="savedSlot"
      :thumb="savedThumb"
      @undo="emit('undoSave')"
    />
  </LabelledBlock>

  <LabelledBlock :title="t('qm.undoTitle')" :aria-label="t('qm.undoTitle')">
    <div class="flex flex-col gap-2">
      <PageRow
        :icon="Undo2"
        :label="t('qm.undoLoad')"
        :value="undoLoadValue"
        :leads="false"
        :disabled="undoLoadSlot === null"
        @click="emit('undoLoad')"
      />
      <PageRow
        :icon="Undo2"
        :label="t('qm.undoLastSave')"
        :value="lastSaved === null ? undefined : t('qm.slot', { slot: lastSaved })"
        :leads="false"
        :disabled="lastSaved === null"
        @click="emit('undoSave')"
      />
    </div>
  </LabelledBlock>

  <!-- The in-game save's backups live in Saves (V04), with the game closed. -->
  <PageRow :icon="ArchiveRestore" :label="t('qm.backup')" :leads="false" disabled />
  <span class="text-caption text-ink-muted">{{ t('qm.backupNotHere') }}</span>
</template>
