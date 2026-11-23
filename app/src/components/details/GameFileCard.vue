<script setup lang="ts">
// L10 «File del gioco»: where the file is on this device and its size, then region, version, game
// code, the identity (SHA-1, or the title ID of a 3DS game) and the emulation engine.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { LibraryGame, SettingDevice } from '@/emu'
import { sizeLabel } from '@/i18n/labels'
import DetailsCard from './DetailsCard.vue'
import { deviceLine, gameFacts } from './gameFiles'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const props = defineProps<{ game: LibraryGame; device?: SettingDevice | null }>()

const { t } = useI18n()

const onDevice = computed(() => deviceLine(props.device, t))
const facts = computed(() => gameFacts(props.game, t))
</script>

<template>
  <DetailsCard class="gap-10">
    <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('details.files.fileTitle') }}</Heading>
    <div class="flex flex-col gap-2">
      <span class="font-medium">{{ onDevice }}</span>
      <span class="text-caption break-all text-ink-muted">
        {{ game.path }} · {{ sizeLabel(game.size) }}
      </span>
    </div>
    <dl
      class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-12 gap-y-2 text-caption text-ink-muted"
    >
      <template v-for="f in facts" :key="f.label">
        <dt>{{ f.label }}</dt>
        <!-- The identity in the system's monospace. -->
        <dd class="m-0" :class="f.mono && 'font-mono break-all'">
          {{ f.value }}
        </dd>
      </template>
    </dl>
  </DetailsCard>
</template>
