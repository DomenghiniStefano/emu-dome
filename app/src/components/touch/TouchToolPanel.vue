<script setup lang="ts">
// The C06 editor's tool under the toolbar (thick glass): the controls to add back, or the picked
// control's size, opacity or behaviour (tap / hold / turbo, buttons only). A slider shows its
// change at once and makes one undo step when let go. The state is the editor's
// (useTouchEditor); this panel only shows it.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { TouchBehaviour, TouchControl, TouchControlId } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import { Slider } from '@/components/ui/slider'
import type { ToolPanel } from './useTouchEditor'

const props = defineProps<{
  panel: ToolPanel
  control: TouchControl | undefined
  /** The picked control is a game button (it can be held or repeated). */
  button: boolean
  addable: readonly TouchControlId[]
  sizeMin: number
  sizeMax: number
  opacityMin: number
  opacityMax: number
}>()
const emit = defineEmits<{
  add: [TouchControlId]
  size: [number, boolean]
  opacity: [number, boolean]
  behaviour: [TouchBehaviour]
}>()
const { t } = useI18n()

const BEHAVIOURS: TouchBehaviour[] = ['tap', 'hold', 'turbo']
const name = computed(() =>
  props.control ? t(`touch.editor.control.${props.control.id}`) : t('touch.editor.none'),
)
function onBehaviour(v: string) {
  const b = BEHAVIOURS.find(x => x === v)
  if (b) emit('behaviour', b)
}
</script>

<template>
  <div
    class="flex w-editor-panel max-w-full flex-col gap-10 rounded-card border border-line p-12 shadow-glass glass-thick-flat"
  >
    <template v-if="panel === 'add'">
      <span class="text-caption text-ink-muted">{{ t('touch.editor.add') }}</span>
      <div class="flex flex-wrap gap-6">
        <Button
          v-for="(id, i) in addable"
          :key="id"
          :size="ButtonSize.Tool"
          :variant="ButtonVariant.Soft"
          :data-nav-initial="i === 0 ? '' : undefined"
          @click="emit('add', id)"
        >
          {{ t(`touch.editor.control.${id}`) }}
        </Button>
      </div>
    </template>

    <template v-else-if="control">
      <div class="flex items-baseline justify-between gap-8">
        <span class="font-semibold">{{ name }}</span>
        <span class="text-caption text-ink-muted">{{ t(`touch.editor.${panel}`) }}</span>
      </div>
      <div v-if="panel === 'size'" class="flex items-center gap-12">
        <Slider
          class="min-w-0 flex-1"
          :model-value="control.size"
          :min="sizeMin"
          :max="sizeMax"
          :step="5"
          :label="t('touch.editor.size')"
          :value-text="t('touch.editor.percent', { n: control.size })"
          data-nav-initial
          @update:model-value="v => emit('size', v, false)"
          @commit="v => emit('size', v, true)"
        />
        <span class="tabular-nums">{{ t('touch.editor.percent', { n: control.size }) }}</span>
      </div>
      <div v-else-if="panel === 'opacity'" class="flex items-center gap-12">
        <Slider
          class="min-w-0 flex-1"
          :model-value="control.opacity"
          :min="opacityMin"
          :max="opacityMax"
          :step="5"
          :label="t('touch.editor.opacity')"
          :value-text="t('touch.editor.percent', { n: control.opacity })"
          data-nav-initial
          @update:model-value="v => emit('opacity', v, false)"
          @commit="v => emit('opacity', v, true)"
        />
        <span class="tabular-nums">{{ t('touch.editor.percent', { n: control.opacity }) }}</span>
      </div>
      <template v-else-if="panel === 'behaviour'">
        <Segmented
          :model-value="control.behaviour"
          :disabled="!button"
          data-nav-initial
          @update:model-value="onBehaviour"
        >
          <SegmentedItem v-for="b in BEHAVIOURS" :key="b" :value="b">
            {{ t(`touch.editor.behaviours.${b}`) }}
          </SegmentedItem>
        </Segmented>
        <span v-if="!button" class="text-caption text-ink-muted">
          {{ t('touch.editor.onlyButtons') }}
        </span>
      </template>
    </template>
  </div>
</template>
