<script setup lang="ts">
// A multiple choice (`multi`): the chosen options in order, numbered; «Modifica l’elenco» turns
// every option into a chip to switch on or off, «Fatto» goes back.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Chip, ChipVariant } from '@/components/ui/chip'
import { toggleChosen } from './settingRowModel'
import type { RowOption } from './useSettingRow'

const props = defineProps<{
  chosen: string[]
  options: RowOption[]
  /** The text of a chosen option (it may not be among this device's options). */
  optionText: (id: string) => string
  editText: string
  disabled: boolean
}>()
const emit = defineEmits<{ set: [string[]] }>()
const { t } = useI18n()

const editing = ref(false)
function toggle(id: string) {
  emit(
    'set',
    toggleChosen(
      props.chosen,
      props.options.map(o => o.id),
      id,
    ),
  )
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-8" :class="{ 'opacity-locked': disabled }">
    <template v-if="!editing">
      <Chip v-for="(id, i) in chosen" :key="id" as="span" :variant="ChipVariant.Option">
        <span class="text-ink-muted tabular-nums">{{ i + 1 }}</span>
        {{ optionText(id) }}
      </Chip>
      <Button
        :variant="ButtonVariant.GhostAccent"
        :fade="ButtonFade.None"
        :size="ButtonSize.Small"
        :disabled="disabled"
        @click="editing = true"
      >
        {{ editText }}
      </Button>
    </template>
    <template v-else>
      <Chip
        v-for="o in options"
        :key="o.id"
        :variant="ChipVariant.Option"
        :selected="chosen.includes(o.id)"
        role="checkbox"
        :aria-checked="chosen.includes(o.id)"
        :disabled="disabled"
        @click="toggle(o.id)"
      >
        {{ o.text }}
      </Chip>
      <Button
        :variant="ButtonVariant.GhostAccent"
        :fade="ButtonFade.None"
        :size="ButtonSize.Small"
        @click="editing = false"
      >
        {{ t('settingsUi.done') }}
      </Button>
    </template>
  </div>
</template>
