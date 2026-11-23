<script setup lang="ts">
// L10 «Come appare»: «Nome mostrato» (`library.setName`; blank goes back to the ROM's title, shown
// as the field's placeholder), the cover with «Cambia…» (L12), «Nascondi dalla libreria».
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Image as ImageIcon } from '@lucide/vue'
import { emu, type LibraryGame } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { Input, InputVariant } from '@/design-system/components/input'
import { Switch } from '@/design-system/components/switch'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import DetailsAction from './DetailsAction.vue'
import DetailsCard from './DetailsCard.vue'
import { coverDescription } from './gameFiles'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{
  /** «Cambia…»: L12. */
  cover: []
}>()

const { t } = useI18n()
const failWith = useErrorSink()

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
    failWith(e)
  }
}

async function setHidden(hidden: boolean) {
  try {
    await emu.library.setFlags(props.game.id, { hidden })
  } catch (e) {
    failWith(e)
  }
}

const coverLine = computed(() => coverDescription(props.game, t))
</script>

<template>
  <DetailsCard class="gap-10">
    <Heading as="h2" :variant="HeadingVariant.Overline">{{
      t('details.files.looksTitle')
    }}</Heading>
    <label class="flex flex-col gap-6">
      <span class="font-medium">{{ t('details.files.name') }}</span>
      <Input
        v-model="name"
        :variant="InputVariant.Block"
        type="text"
        maxlength="128"
        :placeholder="game.romTitle"
        @change="saveName"
        @keydown.enter="saveName"
      />
    </label>
    <div class="flex items-center gap-10">
      <GameCover
        :size="CoverSize.Row"
        class="flex-none"
        :game="game"
        :badge="false"
        :title="false"
      />
      <div class="min-w-0 flex-1">
        <div class="font-medium">{{ t('details.files.cover') }}</div>
        <div class="text-caption text-ink-muted">{{ coverLine }}</div>
      </div>
      <DetailsAction :icon="ImageIcon" @click="emit('cover')">
        {{ t('details.files.coverChange') }}
      </DetailsAction>
    </div>
    <div class="flex items-center gap-10">
      <span class="min-w-0 flex-1 font-medium">{{ t('library.menu.hide') }}</span>
      <Switch
        :model-value="game.hidden"
        :aria-label="t('library.menu.hide')"
        @update:model-value="setHidden"
      />
    </div>
  </DetailsCard>
</template>
