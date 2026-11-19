<script setup lang="ts">
// L10 «Come appare»: «Nome mostrato» (`library.setName`; blank goes back to the ROM's title, shown
// as the field's placeholder), the cover with «Cambia…» (L12), «Nascondi dalla libreria».
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame } from '../../emu'
import { errorText } from '../../i18n/labels'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import GameCover from '../GameCover.vue'
import DetailsAction from './DetailsAction.vue'
import DetailsCard from './DetailsCard.vue'
import { coverDescription } from './gameFiles'
import SectionLabel from './SectionLabel.vue'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Cambia…»: L12. */
  cover: []
  error: [string]
}>()

const { t } = useI18n()

// ---- «Nome mostrato»: the name the user gave, blank for the ROM's own ----
const shownName = (g: LibraryGame) => (g.renamed ? g.title : '')
const name = ref(shownName(props.game))
watch(
  () => [props.game.title, props.game.renamed] as const,
  () => (name.value = shownName(props.game)),
)
async function saveName() {
  const now = name.value.trim()
  if (now === shownName(props.game)) return
  try {
    await emu.library.setName(props.game.id, now)
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function setHidden(hidden: boolean) {
  try {
    await emu.library.setFlags(props.game.id, { hidden })
  } catch (e) {
    emit('error', errorText(e))
  }
}

const coverLine = computed(() => coverDescription(props.game, t))
</script>

<template>
  <DetailsCard :phone="phone" class="gap-14">
    <SectionLabel>{{ t('details.files.looksTitle') }}</SectionLabel>
    <label class="flex flex-col gap-6">
      <span class="font-medium">{{ t('details.files.name') }}</span>
      <Input
        v-model="name"
        :class="phone && 'min-h-touch text-lead'"
        type="text"
        maxlength="128"
        :placeholder="game.romTitle"
        @change="saveName"
        @keydown.enter="saveName"
      />
    </label>
    <div class="flex items-center gap-12">
      <GameCover class="w-48 flex-none" :game="game" :badge="false" />
      <div class="min-w-0 flex-1">
        <div class="font-medium">{{ t('details.files.cover') }}</div>
        <div class="text-caption text-ink-muted">{{ coverLine }}</div>
      </div>
      <DetailsAction :phone="phone" @click="emit('cover')">
        {{ t('details.files.coverChange') }}
      </DetailsAction>
    </div>
    <div class="flex items-center gap-12">
      <span class="min-w-0 flex-1 font-medium">{{ t('library.menu.hide') }}</span>
      <Switch
        :model-value="game.hidden"
        :aria-label="t('library.menu.hide')"
        @update:model-value="setHidden"
      />
    </div>
  </DetailsCard>
</template>
