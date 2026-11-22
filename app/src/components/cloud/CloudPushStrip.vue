<script setup lang="ts">
// G14 after a game closed (Trello #31): its saves go up to Google Drive in the background (the
// close flow never waits for it). The closing's strip (#86), bottom centre, with the upload icon
// while they go and a tick for a moment once they are there. It takes no clicks: the library is
// usable meanwhile.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { CloudUpload } from '@lucide/vue'
import ClosingStrip from '@/components/closing/ClosingStrip.vue'
import { progressOf } from './cloud'
import { cloud } from './cloudState'

const { t } = useI18n()
const pushing = computed(() => progressOf(cloud.progress, 'pushing', null) !== null)
</script>

<template>
  <div
    v-if="pushing || cloud.pushed"
    class="pointer-events-none fixed inset-x-0 bottom-16 z-toast flex justify-center px-16"
  >
    <ClosingStrip
      :saved="!pushing"
      :icon="CloudUpload"
      :busy="t('cloud.push.busy')"
      :hint="t('cloud.push.hint')"
      :done="t('cloud.push.done')"
    />
  </div>
</template>
