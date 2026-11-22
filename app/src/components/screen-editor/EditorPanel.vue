<script setup lang="ts">
// The C09 editor's side panel (thick glass, the owner's mockup, option A): where to start from,
// the selected screen's place and size in per cent, keeping the proportions, the grid, which
// screen is in front, where the layout is saved, Annulla / Salva. «Riduci» (Select) folds it into
// the pill at the top. The state is the editor's (useScreenEditor); this panel only shows it.
import { useI18n } from 'vue-i18n'
import type { CustomScreen, ScreenLayoutId } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { Input, InputVariant } from '@/components/ui/input'
import { Kbd, KbdSize } from '@/components/ui/kbd'
import { LabelledBlock } from '@/components/ui/labelled-block'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import { Switch } from '@/components/ui/switch'
import { posName } from '@/lib/controls'
import { inputMode } from '@/lib/inputMode'
import { fromPercent, percent, type EditorScreen, type Field } from './screenEditor'
import { PANEL_ATTR, type SaveTarget } from './useScreenEditor'

defineProps<{
  /** The selected screen; none before the layout is read. */
  screen: CustomScreen | undefined
  /** Its full name («Schermo superiore»). */
  screenName: string
  /** The short name of each screen, for «In primo piano». */
  names: Record<EditorScreen, string>
  /** The screen in front; null with one screen only (no choice then). */
  front: EditorScreen | null
  twoScreens: boolean
  /** A game is open: «Si applica a» chooses; else `scopeName` says where it goes. */
  live: boolean
  consoleScope: string
  scopeName: string
  busy: boolean
  canSave: boolean
}>()
const emit = defineEmits<{
  start: [ScreenLayoutId]
  field: [Field, number]
  front: [EditorScreen]
  collapse: []
  cancel: []
  save: []
}>()
const keep = defineModel<boolean>('keep', { required: true })
const snap = defineModel<boolean>('snap', { required: true })
const target = defineModel<SaveTarget>('target', { required: true })

const { t, locale } = useI18n()

/** «Parti da»: the layouts Rust composes the screens from. */
const STARTS: ScreenLayoutId[] = ['vertical', 'side', 'big_small', 'top_only']
const FIELDS: Field[] = ['x', 'y', 'w', 'h']
const panelMark = { [PANEL_ATTR]: '' }

const shown = (v: number) =>
  new Intl.NumberFormat(locale.value, { maximumFractionDigits: 1 }).format(percent(v))

/** A field left: its number, or back to what it was when it is none. */
function onField(field: Field, e: Event, s: CustomScreen) {
  const input = e.target as HTMLInputElement
  const v = fromPercent(input.value)
  if (v === null) input.value = shown(s[field])
  else emit('field', field, v)
}
function onFront(id: string) {
  if (id === 'top' || id === 'bottom') emit('front', id)
}
function onTarget(to: string) {
  if (to === 'console' || to === 'game') target.value = to
}
</script>

<template>
  <div
    v-bind="panelMark"
    class="flex max-h-[calc(100vh-var(--spacing-window-margin))] w-editor-panel flex-col gap-12 overflow-y-auto p-card-pad glass-thick"
  >
    <div class="flex items-start gap-8">
      <div class="flex min-w-0 flex-1 flex-col gap-4">
        <Heading as="h2" :variant="HeadingVariant.PageTitle">{{ t('screenEditor.title') }}</Heading>
        <span class="text-caption text-ink-muted">{{ t('screenEditor.lead') }}</span>
      </div>
      <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Small" @click="emit('collapse')">
        {{ t('screenEditor.collapse') }}
        <Kbd v-if="inputMode === 'controller'" :size="KbdSize.Small">
          {{ posName('generic', 'back', false) }}
        </Kbd>
      </Button>
    </div>

    <LabelledBlock :title="t('screenEditor.startFrom')" title-as="h3">
      <div class="grid grid-cols-2 gap-8">
        <Button
          v-for="id in STARTS"
          :key="id"
          :variant="ButtonVariant.Soft"
          class="min-w-0"
          @click="emit('start', id)"
        >
          <span class="truncate">{{ t(`screenEditor.start.${id}`) }}</span>
        </Button>
      </div>
    </LabelledBlock>

    <LabelledBlock v-if="screen" :title="screenName" title-as="h3">
      <div class="grid grid-cols-2 gap-8">
        <label v-for="f in FIELDS" :key="f" class="flex min-w-0 flex-col gap-4">
          <span class="text-caption text-ink-muted">{{ t(`screenEditor.field.${f}`) }}</span>
          <Input
            :variant="InputVariant.Block"
            class="min-w-0"
            inputmode="decimal"
            :model-value="shown(screen[f])"
            @change="(e: Event) => onField(f, e, screen!)"
          />
        </label>
      </div>
      <label class="flex items-center justify-between gap-12">
        <span>{{ t('screenEditor.keep') }}</span>
        <Switch v-model="keep" />
      </label>
      <label class="flex items-center justify-between gap-12">
        <span>{{ t('screenEditor.snap') }}</span>
        <Switch v-model="snap" />
      </label>
      <div v-if="twoScreens && front" class="flex items-center justify-between gap-12">
        <span>{{ t('screenEditor.front') }}</span>
        <Segmented :model-value="front" @update:model-value="onFront">
          <SegmentedItem value="top">{{ names.top }}</SegmentedItem>
          <SegmentedItem value="bottom">{{ names.bottom }}</SegmentedItem>
        </Segmented>
      </div>
    </LabelledBlock>

    <LabelledBlock :title="t('screenEditor.appliesTo')" title-as="h3">
      <Segmented v-if="live" :model-value="target" @update:model-value="onTarget">
        <SegmentedItem value="console">{{ consoleScope }}</SegmentedItem>
        <SegmentedItem value="game">{{ t('screenEditor.thisGame') }}</SegmentedItem>
      </Segmented>
      <span v-else>{{ scopeName }}</span>
    </LabelledBlock>

    <div class="flex justify-end gap-8">
      <Button :variant="ButtonVariant.Ghost" :disabled="busy" @click="emit('cancel')">
        {{ t('screenEditor.cancel') }}
      </Button>
      <Button :variant="ButtonVariant.Primary" :disabled="busy || !canSave" @click="emit('save')">
        {{ t('screenEditor.save') }}
      </Button>
    </div>
  </div>
</template>
