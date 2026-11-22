<script setup lang="ts">
// G05 «Aperto su un altro dispositivo» (Trello #31, mockup section 3): the game is open on
// another device since N minutes. «Gioca comunque» opens it here (or plays on, when it already
// runs: opened without the check), «Annulla» (or B) does not (or closes it). Outside the game Esc
// is «Annulla» too; in game Esc keeps its meaning (the quick menu).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import { minutesSince } from './cloud'
import type { InUseAsk } from './cloudState'

const props = defineProps<{ ask: InUseAsk }>()
const emit = defineEmits<{ answer: [play: boolean] }>()
const { t } = useI18n()

const minutes = computed(() => minutesSince(props.ask.since))
</script>

<template>
  <FlowDialog
    :title="t('cloud.inUse.title', { title: ask.title, device: ask.device })"
    :escape-closes="!ask.opened"
    @back="emit('answer', false)"
  >
    <p class="m-0">{{ t('cloud.inUse.text', { n: minutes }, minutes) }}</p>
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('answer', false)">
        {{ t('cloud.inUse.cancel') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('answer', true)"
      >
        {{ t('cloud.inUse.play') }}
      </Button>
    </template>
  </FlowDialog>
</template>
