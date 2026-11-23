<script setup lang="ts">
// G04 «Due versioni dello stesso salvataggio» (Trello #31, mockup section 3): the game's save
// changed here and on another device since the last sync. The two versions side by side
// (device, date, play time; the newer chosen first), «Continua con …» keeps the chosen one,
// «Tieni entrambe» keeps this device's and Drive's copy among the backups. B / Esc put the
// question off: it stays in the status card. Shown outside the game only (CloudFlowOverlays).
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, Copy, Save } from '@lucide/vue'
import { emu, type CloudChoice, type CloudGameConflict, type CloudVersion } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { RadioGroup, RadioGroupItem } from '@/design-system/components/radio-group'
import FlowDialog from '@/components/game/FlowDialog.vue'
import { playTimeLabel, saveWhenLabel } from '@/i18n/labels'
import { applyStatus, dropConflict, failCloud } from './cloudState'

const props = defineProps<{ conflict: CloudGameConflict }>()
const { t } = useI18n()

type Side = Exclude<CloudChoice, 'both'>
const SIDES: Side[] = ['local', 'remote']
const chosen = ref<Side>(
  props.conflict.remote.modified > props.conflict.local.modified ? 'remote' : 'local',
)
const busy = ref(false)

const deviceName = (side: Side) =>
  side === 'local'
    ? t('cloud.devices.this')
    : (props.conflict.remote.device ?? t('cloud.conflict.otherName'))
const version = (side: Side): CloudVersion => props.conflict[side]
const meta = (v: CloudVersion) =>
  v.playTime === null
    ? saveWhenLabel(v.modified)
    : t('cloud.conflict.meta', {
        when: saveWhenLabel(v.modified),
        time: playTimeLabel(v.playTime, true),
      })
const continueText = computed(() =>
  chosen.value === 'local'
    ? t('cloud.conflict.continueHere')
    : t('cloud.conflict.continueWith', { device: deviceName('remote') }),
)

function choose(value: unknown) {
  const side = SIDES.find(s => s === value)
  if (side) chosen.value = side
}

async function resolve(choice: CloudChoice) {
  if (busy.value) return
  busy.value = true
  try {
    applyStatus(await emu.cloud.resolve(props.conflict.id, choice))
    dropConflict(props.conflict.id)
  } catch (e) {
    failCloud(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <FlowDialog
    :title="t('cloud.conflict.title', { title: conflict.title })"
    escape-closes
    @back="dropConflict(conflict.id)"
  >
    <p class="m-0">
      {{
        t('cloud.conflict.text', {
          device: conflict.remote.device ?? t('cloud.conflict.otherDevice'),
        })
      }}
    </p>
    <RadioGroup
      :model-value="chosen"
      class="grid grid-cols-2 gap-12"
      :aria-label="t('cloud.conflict.versions')"
      @update:model-value="choose"
    >
      <RadioGroupItem v-for="side in SIDES" :key="side" :value="side">
        <Card
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="chosen === side"
          class="gap-8"
          :data-nav-initial="chosen === side ? '' : undefined"
        >
          <span
            class="grid aspect-3/2 w-full place-items-center rounded-control bg-surface-raised text-ink-muted"
            aria-hidden="true"
          >
            <Save :size="24" :stroke-width="1.75" />
          </span>
          <span class="font-semibold text-ink wrap-anywhere">{{ deviceName(side) }}</span>
          <span class="text-caption text-ink-muted tabular-nums">{{ meta(version(side)) }}</span>
        </Card>
      </RadioGroupItem>
    </RadioGroup>
    <template #actions>
      <Button :size="ButtonSize.Medium" :icon="Copy" :disabled="busy" @click="resolve('both')">
        {{ t('cloud.conflict.keepBoth') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :icon="Check"
        :disabled="busy"
        @click="resolve(chosen)"
      >
        {{ continueText }}
      </Button>
    </template>
  </FlowDialog>
</template>
