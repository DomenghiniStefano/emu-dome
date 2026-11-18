<script setup lang="ts">
// V03, a quick save's details (board «Salvataggi»), opened from «…» on a slot tile in the Saves
// section (V02) and in Game details › Salvataggi (L05): the thumbnail, when and how big,
// «Gioca da qui», «Esporta…» and, except for the Automatico slot, «Elimina» with its X04
// confirmation (a deleted quick save cannot be recovered).
//
// Controller / keyboard: each dialog (FlowDialog) opens its own nav scope.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT, emu, type LibraryGame, type QuickSave } from '../emu'
import { errorText, saveWhenLabel, sizeLabel } from '../i18n/labels'
import FlowDialog from './FlowDialog.vue'

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
const confirmDelete = ref(false)

const slotName = (slot: number) => (slot === AUTO_SLOT ? t('qm.auto') : t('qm.slot', { slot }))

async function exportIt() {
  try {
    if (await emu.saves.export(props.game.path, { kind: 'quick', slot: props.quick.slot }))
      emit('notice', t('saves.exported'))
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function deleteIt() {
  // Emits before `close`: the parent unmounts this dialog on it.
  try {
    await emu.saves.deleteQuick(props.game.path, props.quick.slot)
    emit('notice', t('saves.deleted'))
  } catch (e) {
    emit('error', errorText(e))
  }
  emit('changed')
  emit('close')
}
</script>

<template>
  <FlowDialog v-if="!confirmDelete" :title="slotName(quick.slot)" @back="emit('close')">
    <span class="big-thumb" aria-hidden="true">
      <img v-if="quick.thumbUrl" :src="quick.thumbUrl" alt="" />
    </span>
    <p class="p num">
      {{
        quick.savedAt !== null
          ? t('saves.inGameLine', {
              when: saveWhenLabel(quick.savedAt),
              size: sizeLabel(quick.size),
            })
          : sizeLabel(quick.size)
      }}
    </p>
    <p class="p small">{{ t('saves.quick.overwriteHint') }}</p>
    <template #actions>
      <!-- The Automatico slot cannot be deleted (#59 item 3): EmuDome fills it by itself. -->
      <button v-if="quick.slot !== AUTO_SLOT" class="btn" @click="confirmDelete = true">
        {{ t('saves.quick.delete') }}
      </button>
      <button class="btn" @click="exportIt">{{ t('saves.export') }}</button>
      <button class="btn" @click="emit('close')">{{ t('saves.quick.close') }}</button>
      <button class="btn btn-primary" data-nav-initial @click="emit('play', quick.slot)">
        {{ t('saves.quick.load') }}
      </button>
    </template>
  </FlowDialog>

  <FlowDialog
    v-else
    :title="t('saves.quick.deleteTitle', { slot: quick.slot })"
    tone="danger"
    @back="confirmDelete = false"
  >
    <p class="p">{{ t('saves.quick.deleteText') }}</p>
    <template #actions>
      <button class="btn" data-nav-initial @click="confirmDelete = false">
        {{ t('saves.quick.cancel') }}
      </button>
      <button class="btn danger" @click="deleteIt">
        {{ t('saves.quick.deleteConfirm', { slot: quick.slot }) }}
      </button>
    </template>
  </FlowDialog>
</template>

<style scoped>
.p {
  margin: 0;
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.btn.danger {
  color: var(--danger);
}
.big-thumb {
  display: block;
  aspect-ratio: 4 / 3;
  border-radius: var(--radius-10);
  background: #000;
  overflow: hidden;
}
.big-thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
}
</style>
