<script setup lang="ts">
// Quick saves (G07 on the main screen, G08 «Altri slot e salvataggi»): the slots to choose from,
// Carica and Salva for the chosen one, the line a save made here leaves with its «Annulla», and on
// G08 where the in-game save's backups are.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArchiveRestore, RotateCcw, Save } from '@lucide/vue'
import type { SlotInfo } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import PageRow from './PageRow.vue'
import QuickSlotTile from './QuickSlotTile.vue'
import SectionLabel from './SectionLabel.vue'
import { loadLabel } from './slots'
import { useSay } from './useSay'
import { FocusMark } from './view'

const props = defineProps<{
  slots: SlotInfo[]
  chosen: number
  chosenEmpty: boolean
  autoChosen: boolean
  /** The slot a save made here went to, while its «Annulla» is offered. */
  savedSlot: number | null
  /** G08: every slot, and the backups' row. */
  all?: boolean
  touch?: boolean
}>()
const emit = defineEmits<{
  pick: [number]
  more: [Event]
  load: []
  save: []
  undoSave: []
}>()

const { t } = useI18n()
const say = useSay()
const columns = computed(() => {
  if (props.touch) return 'grid-cols-3'
  return props.all ? 'grid-cols-4' : 'grid-cols-5'
})
const actionSize = computed(() => (props.touch ? ButtonSize.MenuTouch : ButtonSize.Menu))
const moreMark = { [FocusMark.MoreSlots]: '' }
</script>

<template>
  <section class="flex flex-col gap-10" :aria-label="t('qm.savesTitle')">
    <div v-if="!all" class="flex items-baseline justify-between gap-8">
      <SectionLabel :touch="touch">{{ t('qm.savesTitle') }}</SectionLabel>
      <Button
        v-bind="moreMark"
        :variant="ButtonVariant.Link"
        :size="touch ? ButtonSize.TextTouch : ButtonSize.Text"
        @click="emit('more', $event)"
      >
        {{ touch ? t('qm.moreShort') : t('qm.more') }}
      </Button>
    </div>
    <div class="grid gap-8" :class="columns" role="radiogroup" :aria-label="t('qm.slotsLabel')">
      <QuickSlotTile
        v-for="s in slots"
        :key="s.slot"
        :info="s"
        :chosen="s.slot === chosen"
        :touch="touch"
        @pick="emit('pick', s.slot)"
      />
    </div>
    <div class="grid grid-cols-[1fr_1fr] gap-8">
      <Button
        :variant="ButtonVariant.Soft"
        :size="actionSize"
        class="min-w-0"
        :disabled="chosenEmpty"
        @click="emit('load')"
      >
        <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
        {{ say(loadLabel(chosen, chosenEmpty)) }}
      </Button>
      <span
        v-if="autoChosen"
        class="flex h-[44px] items-center justify-center rounded-10 border border-dashed border-line-strong px-8 text-center text-(length:--text-caption) text-ink-muted"
      >
        {{ t('qm.autoUpdates') }}
      </span>
      <Button
        v-else
        :variant="ButtonVariant.Soft"
        :size="actionSize"
        class="min-w-0"
        @click="emit('save')"
      >
        <Save :size="18" :stroke-width="1.75" aria-hidden="true" />
        {{ t('qm.saveSlot', { slot: chosen }) }}
      </Button>
    </div>
    <div
      v-if="savedSlot !== null"
      class="flex min-h-20 items-center gap-8 font-medium text-success"
      role="status"
    >
      {{ t('qm.saved', { slot: savedSlot }) }}
      <Button
        :variant="ButtonVariant.Link"
        :size="ButtonSize.Text"
        class="px-4"
        @click="emit('undoSave')"
      >
        {{ t('qm.undoSave') }}
      </Button>
    </div>
    <!-- G08: the in-game save's backups live in Saves (V04), with the game closed. -->
    <template v-if="all">
      <PageRow :icon="ArchiveRestore" :label="t('qm.backup')" disabled />
      <span class="text-caption text-ink-muted">{{ t('qm.backupNotHere') }}</span>
    </template>
  </section>
</template>
