<script setup lang="ts">
// G03 inside the starting scene (Trello #31): the newest saves come down from Drive before the
// game opens. «Gioca senza aspettare» stops the pull before its next file and the game opens with
// this device's saves. Mounted only while a pull runs (GameFlowOverlays), with a nav scope of
// its own so A presses the button.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type CloudProgress } from '@/emu'
import { Button, ButtonSize } from '@/components/ui/button'
import { useNavScope } from '@/nav'
import { failCloud } from './cloudState'

const props = defineProps<{ progress: CloudProgress }>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true })

const skipped = ref(false)
async function skip() {
  const id = props.progress.game
  if (!id || skipped.value) return
  skipped.value = true
  try {
    await emu.cloud.skipPull(id)
  } catch (e) {
    failCloud(e)
  }
}
</script>

<template>
  <div ref="root" class="flex flex-col items-center gap-10">
    <p class="m-0 text-center text-ink-muted">
      {{ skipped ? t('cloud.pull.skipped') : t('cloud.pull.text') }}
    </p>
    <Button :size="ButtonSize.Medium" :disabled="skipped" data-nav-initial @click="skip">
      {{ t('cloud.pull.skip') }}
    </Button>
  </div>
</template>
