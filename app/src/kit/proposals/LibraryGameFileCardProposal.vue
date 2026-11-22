<script setup lang="ts">
// Kit proposal (library group, «Game file card»): when the file is not reachable the card says
// so where it now says «Su questo computer» (the same words as the details' head), and the path
// wraps at its separators and spaces first (`wrap-anywhere`) instead of in the middle of a word
// (`break-all`). Only shown on the Kit page.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { TriangleAlert } from '@lucide/vue'
import DetailsCard from '@/components/details/DetailsCard.vue'
import { deviceLine, gameFacts } from '@/components/details/gameFiles'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import type { LibraryGame, SettingDevice } from '@/emu'
import { sizeLabel } from '@/i18n/labels'

const props = defineProps<{ game: LibraryGame; device?: SettingDevice | null }>()

const { t } = useI18n()

const onDevice = computed(() => deviceLine(props.device, t))
const facts = computed(() => gameFacts(props.game, t))
</script>

<template>
  <DetailsCard class="gap-10">
    <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('details.files.fileTitle') }}</Heading>
    <div class="flex flex-col gap-2">
      <span v-if="game.available" class="font-medium">{{ onDevice }}</span>
      <span v-else class="flex items-start gap-6 font-medium text-warning">
        <TriangleAlert class="mt-px flex-none" :size="16" :stroke-width="1.75" aria-hidden="true" />
        {{ t('errors.libraryGameUnavailable') }}
      </span>
      <span class="text-caption wrap-anywhere text-ink-muted">
        {{ game.path }} · {{ sizeLabel(game.size) }}
      </span>
    </div>
    <dl
      class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-12 gap-y-2 text-caption text-ink-muted"
    >
      <template v-for="f in facts" :key="f.label">
        <dt>{{ f.label }}</dt>
        <dd class="m-0" :class="f.mono && 'font-mono break-all'">{{ f.value }}</dd>
      </template>
    </dl>
  </DetailsCard>
</template>
