<script setup lang="ts">
// Quick saves (G07 on the main screen, G08 «Altri slot e salvataggi»): the slots to choose from,
// Carica and Salva for the chosen one, the line a save made here leaves with its «Annulla», and on
// G08 where the in-game save's backups are.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArchiveRestore, RotateCcw, Save } from '@lucide/vue'
import type { SlotInfo } from '@/emu'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import PageRow from './PageRow.vue'
import QuickSlotTile from './QuickSlotTile.vue'
import { loadLabel } from './slots'
import { useSay } from '@/i18n/message'
import { FocusMark } from './view'
import { LabelledBlock } from '@/components/ui/labelled-block'

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
/** Every slot (G08) in rows of four, the main screen's five in one row; three on the phone. */
const columns = computed(() => (props.all ? 'grid-cols-4' : 'grid-cols-5'))
const moreMark = { [FocusMark.MoreSlots]: '' }
</script>

<template>
  <LabelledBlock :title="all ? undefined : t('qm.savesTitle')" :aria-label="t('qm.savesTitle')">
    <template v-if="!all" #aside>
      <Button
        v-bind="moreMark"
        :variant="ButtonVariant.Link"
        :size="ButtonSize.LabelLink"
        @click="emit('more', $event)"
      >
        {{ touch ? t('qm.moreShort') : t('qm.more') }}
      </Button>
    </template>
    <div
      class="grid gap-8 touch:grid-cols-3"
      :class="columns"
      role="radiogroup"
      :aria-label="t('qm.slotsLabel')"
    >
      <QuickSlotTile
        v-for="s in slots"
        :key="s.slot"
        :info="s"
        :chosen="s.slot === chosen"
        @pick="emit('pick', s.slot)"
      />
    </div>
    <div class="grid grid-cols-[1fr_1fr] gap-8">
      <Button
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Menu"
        :fade="ButtonFade.Dim"
        class="min-w-0"
        :disabled="chosenEmpty"
        @click="emit('load')"
      >
        <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
        {{ say(loadLabel(chosen, chosenEmpty)) }}
      </Button>
      <span
        v-if="autoChosen"
        class="flex h-touch items-center justify-center rounded-10 border border-dashed border-line-strong px-8 text-center text-caption-inline text-ink-muted"
      >
        {{ t('qm.autoUpdates') }}
      </span>
      <Button
        v-else
        :variant="ButtonVariant.Soft"
        :size="ButtonSize.Menu"
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
        class="mx-4"
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
  </LabelledBlock>
</template>
