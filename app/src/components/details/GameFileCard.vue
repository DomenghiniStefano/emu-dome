<script setup lang="ts">
// L10 «File del gioco»: where the file is on this device and its size, then region, version, game
// code, the identity (SHA-1, or the title ID of a 3DS game) and the emulation engine.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { LibraryGame, SettingDevice } from '../../emu'
import { sizeLabel } from '../../i18n/labels'
import DetailsCard from './DetailsCard.vue'
import { deviceLine, gameFacts } from './gameFiles'
import SectionLabel from './SectionLabel.vue'

const props = defineProps<{ game: LibraryGame; phone: boolean; device?: SettingDevice | null }>()

const { t } = useI18n()

const onDevice = computed(() => deviceLine(props.device, t))
const facts = computed(() => gameFacts(props.game, t))
</script>

<template>
  <DetailsCard :phone="phone" class="gap-14">
    <SectionLabel>{{ t('details.files.fileTitle') }}</SectionLabel>
    <div class="flex flex-col gap-2">
      <span class="font-medium">{{ onDevice }}</span>
      <span class="text-caption break-all text-ink-muted">
        {{ game.path }} · {{ sizeLabel(game.size) }}
      </span>
    </div>
    <dl
      class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-16 gap-y-4 text-caption text-ink-muted"
    >
      <template v-for="f in facts" :key="f.label">
        <dt>{{ f.label }}</dt>
        <!-- The identity in the system's monospace (no font token yet). -->
        <dd class="m-0" :class="f.mono && 'font-[family-name:ui-monospace,monospace] break-all'">
          {{ f.value }}
        </dd>
      </template>
    </dl>
  </DetailsCard>
</template>
