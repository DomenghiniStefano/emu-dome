<script setup lang="ts">
// V03, a quick save's details (board «Salvataggi»), opened from «…» on a slot tile in the Saves
// section (V02) and in Game details › Salvataggi (L05): the thumbnail, when and how big,
// «Gioca da qui», «Esporta…» and, except for the Automatico slot, «Elimina» with its X04
// confirmation (a deleted quick save cannot be recovered), asked only while
// `general.confirmdel` is on (askBeforeDelete.ts).
//
// Controller / keyboard: each dialog (FlowDialog) opens its own nav scope.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play, Share, Trash2 } from '@lucide/vue'
import { AUTO_SLOT, emu, type LibraryGame, type QuickSave } from '@/emu'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { saveWhenLabel, sizeLabel } from '@/i18n/labels'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import SaveThumb from './SaveThumb.vue'
import { askBeforeDelete } from './askBeforeDelete'
import { useSlotName } from './useSlotName'

const props = defineProps<{ game: LibraryGame; quick: QuickSave }>()
const emit = defineEmits<{
  close: []
  /** «Gioca da qui»: open the game on this slot. */
  play: [slot: number]
  /** The slot was deleted: the saves shown are out of date. */
  changed: []
}>()

const { t } = useI18n()
const toasts = useToasts()
const failWith = useErrorSink()
const slotName = useSlotName()
const confirmDelete = ref(false)

/** When it was saved and how big it is; the size alone when the date is unknown. */
const savedLine = computed(() =>
  props.quick.savedAt !== null
    ? t('saves.inGameLine', {
        when: saveWhenLabel(props.quick.savedAt),
        size: sizeLabel(props.quick.size),
      })
    : sizeLabel(props.quick.size),
)

async function exportIt() {
  try {
    if (await emu.saves.export(props.game.id, { kind: 'quick', slot: props.quick.slot }))
      toasts.notify(t('saves.exported'))
  } catch (e) {
    failWith(e)
  }
}

/** «Elimina»: the X04 question first, unless `general.confirmdel` is off. */
async function startDelete() {
  if (await askBeforeDelete()) confirmDelete.value = true
  else await deleteIt()
}

async function deleteIt() {
  // Emits `changed` before `close`: the parent unmounts this dialog on it.
  try {
    await emu.saves.deleteQuick(props.game.id, props.quick.slot)
    toasts.notify(t('saves.deleted'))
  } catch (e) {
    failWith(e)
  }
  emit('changed')
  emit('close')
}
</script>

<template>
  <FlowDialog
    v-if="!confirmDelete"
    :title="slotName(quick.slot)"
    escape-closes
    @back="emit('close')"
  >
    <SaveThumb :src="quick.thumbUrl" class="aspect-4/3 rounded-card" />
    <p class="m-0 tabular-nums">{{ savedLine }}</p>
    <p class="m-0 text-caption">{{ t('saves.quick.overwriteHint') }}</p>
    <template #actions>
      <!-- The Automatico slot cannot be deleted (#59 item 3): EmuDome fills it by itself. -->
      <Button
        v-if="quick.slot !== AUTO_SLOT"
        :size="ButtonSize.Medium"
        :icon="Trash2"
        @click="startDelete"
      >
        {{ t('saves.quick.delete') }}
      </Button>
      <Button :size="ButtonSize.Medium" :icon="Share" @click="exportIt">
        {{ t('saves.export') }}
      </Button>
      <!-- The quiet way out stays words (#104), so the main action stands out. -->
      <Button :size="ButtonSize.Medium" @click="emit('close')">
        {{ t('saves.quick.close') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :icon="Play"
        icon-filled
        data-nav-initial
        @click="emit('play', quick.slot)"
      >
        {{ t('saves.quick.load') }}
      </Button>
    </template>
  </FlowDialog>

  <FlowDialog
    v-else
    :title="t('saves.quick.deleteTitle', { slot: quick.slot })"
    tone="danger"
    escape-closes
    @back="confirmDelete = false"
  >
    <p class="m-0">{{ t('saves.quick.deleteText') }}</p>
    <template #actions>
      <Button :size="ButtonSize.Medium" data-nav-initial @click="confirmDelete = false">
        {{ t('saves.quick.cancel') }}
      </Button>
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :icon="Trash2"
        @click="deleteIt"
      >
        {{ t('saves.quick.deleteConfirm', { slot: quick.slot }) }}
      </Button>
    </template>
  </FlowDialog>
</template>
