<script setup lang="ts">
// V03, a quick save's details (board «Salvataggi»), opened from «…» on a slot tile in the Saves
// section (V02) and in Game details › Salvataggi (L05): the thumbnail, when and how big,
// «Gioca da qui», «Esporta…» and, except for the Automatico slot, «Elimina» with its X04
// confirmation (a deleted quick save cannot be recovered).
//
// Controller / keyboard: each dialog (FlowDialog) opens its own nav scope.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT, emu, type LibraryGame, type QuickSave } from '@/emu'
import { errorText, saveWhenLabel, sizeLabel } from '@/i18n/labels'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import SaveThumb from './SaveThumb.vue'
import { useSlotName } from './useSlotName'

const props = defineProps<{ game: LibraryGame; quick: QuickSave }>()
const emit = defineEmits<{
  close: []
  /** «Gioca da qui»: open the game on this slot. */
  play: [slot: number]
  /** The slot was deleted: the saves shown are out of date. */
  changed: []
  error: [string]
  notice: [string]
}>()

const { t } = useI18n()
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
      emit('notice', t('saves.exported'))
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function deleteIt() {
  // Emits before `close`: the parent unmounts this dialog on it.
  try {
    await emu.saves.deleteQuick(props.game.id, props.quick.slot)
    emit('notice', t('saves.deleted'))
  } catch (e) {
    emit('error', errorText(e))
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
    <SaveThumb :src="quick.thumbUrl" class="aspect-4/3 rounded-10" />
    <p class="m-0 tabular-nums">{{ savedLine }}</p>
    <p class="m-0 text-caption">{{ t('saves.quick.overwriteHint') }}</p>
    <template #actions>
      <!-- The Automatico slot cannot be deleted (#59 item 3): EmuDome fills it by itself. -->
      <Button
        v-if="quick.slot !== AUTO_SLOT"
        :size="ButtonSize.Medium"
        @click="confirmDelete = true"
      >
        {{ t('saves.quick.delete') }}
      </Button>
      <Button :size="ButtonSize.Medium" @click="exportIt">{{ t('saves.export') }}</Button>
      <Button :size="ButtonSize.Medium" @click="emit('close')">
        {{ t('saves.quick.close') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
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
      <Button :variant="ButtonVariant.Danger" :size="ButtonSize.Medium" @click="deleteIt">
        {{ t('saves.quick.deleteConfirm', { slot: quick.slot }) }}
      </Button>
    </template>
  </FlowDialog>
</template>
