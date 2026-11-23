<script setup lang="ts">
// The C06 editor's toolbar (the owner's mockup, option A): thick glass at the top, undo / redo,
// add a control, size, opacity, behaviour, remove, back to the default, leave without saving,
// «Riduci» (folds it into the pill) and the main action «Salva». On a narrow phone it never
// overflows (owner): the tools that do not fit go into «Altro» (editorToolbar.ts decides from
// each button's width, measured on a hidden copy of the row). The tools are their icons alone
// (#104), their words the name and tooltip; in «Altro» icon and words.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Blend,
  ChevronUp,
  Ellipsis,
  Plus,
  Redo2,
  Repeat2,
  RotateCcw,
  Save,
  Scaling,
  Trash2,
  Undo2,
  X,
} from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
} from '@/design-system/components/dropdown-menu'
import { fitToolbar, TOOLS, type Tool, type ToolbarFit } from './editorToolbar'
import type { ToolPanel } from './useTouchEditor'

const props = defineProps<{
  canUndo: boolean
  canRedo: boolean
  /** A control is picked (size, opacity, behaviour act on it). */
  picked: boolean
  /** The picked control can be removed (not the menu). */
  canRemove: boolean
  /** Some of the console's controls are not on screen. */
  canAdd: boolean
  panel: ToolPanel | null
  busy: boolean
  canSave: boolean
}>()
const emit = defineEmits<{
  undo: []
  redo: []
  panel: [ToolPanel]
  remove: []
  reset: []
  discard: []
  collapse: []
  save: []
}>()
const { t, locale } = useI18n()

const PANELS: Partial<Record<Tool, ToolPanel>> = {
  add: 'add',
  size: 'size',
  opacity: 'opacity',
  behaviour: 'behaviour',
}
/** Every tool as its icon alone (#104), its words its name and tooltip, and in «Altro» beside them. */
const ICONS: Record<Tool, Component> = {
  undo: Undo2,
  redo: Redo2,
  add: Plus,
  size: Scaling,
  opacity: Blend,
  behaviour: Repeat2,
  remove: Trash2,
  reset: RotateCcw,
  discard: X,
}

function disabled(tool: Tool): boolean {
  switch (tool) {
    case 'undo':
      return !props.canUndo
    case 'redo':
      return !props.canRedo
    case 'add':
      return !props.canAdd
    case 'size':
    case 'opacity':
    case 'behaviour':
      return !props.picked
    case 'remove':
      return !props.canRemove
    case 'reset':
    case 'discard':
      return props.busy
  }
}

function run(tool: Tool) {
  const panel = PANELS[tool]
  if (panel) return emit('panel', panel)
  switch (tool) {
    case 'undo':
      return emit('undo')
    case 'redo':
      return emit('redo')
    case 'remove':
      return emit('remove')
    case 'reset':
      return emit('reset')
    case 'discard':
      return emit('discard')
    case 'add':
    case 'size':
    case 'opacity':
    case 'behaviour':
      return
  }
}

// ---- What fits: each button measured on the hidden copy, the room from the bar's place ----
const place = ref<HTMLElement | null>(null)
const bar = ref<HTMLElement | null>(null)
const measure = ref<HTMLElement | null>(null)
const fit = ref<ToolbarFit>({ shown: [...TOOLS], more: [] })

function refit() {
  const m = measure.value
  const p = place.value
  const b = bar.value
  if (!m || !p || !b) return
  const width = (name: string) =>
    m.querySelector<HTMLElement>(`[data-measure="${name}"]`)?.offsetWidth ?? 0
  const style = getComputedStyle(b)
  const frame =
    parseFloat(style.paddingLeft) +
    parseFloat(style.paddingRight) +
    parseFloat(style.borderLeftWidth) +
    parseFloat(style.borderRightWidth)
  const tools = Object.fromEntries(TOOLS.map(tool => [tool, width(tool)])) as Record<Tool, number>
  fit.value = fitToolbar(p.clientWidth - frame, {
    tools,
    more: width('more'),
    collapse: width('collapse'),
    save: width('save'),
    gap: parseFloat(getComputedStyle(m).columnGap) || 0,
  })
}

let observer: ResizeObserver | null = null
onMounted(() => {
  void nextTick(refit)
  observer = new ResizeObserver(refit)
  if (place.value) observer.observe(place.value)
})
onBeforeUnmount(() => observer?.disconnect())
watch(locale, () => void nextTick(refit))

const moreOpen = ref(false)
const moreButton = ref<{ $el: HTMLElement } | null>(null)
const moreAnchor = computed(() => moreButton.value?.$el ?? null)
function fromMenu(tool: Tool) {
  moreOpen.value = false
  run(tool)
}
const label = (tool: Tool) => t(`touch.editor.${tool}`)
</script>

<template>
  <div ref="place" class="flex justify-center">
    <div
      ref="bar"
      class="relative flex items-center gap-4 rounded-card border border-line p-6 shadow-glass glass-thick-flat"
    >
      <template v-for="tool in fit.shown" :key="tool">
        <Button
          :size="ButtonSize.ToolIcon"
          :variant="PANELS[tool] ? ButtonVariant.Pick : ButtonVariant.Soft"
          :icon="ICONS[tool]"
          :label="label(tool)"
          :aria-pressed="PANELS[tool] ? panel === PANELS[tool] : undefined"
          :disabled="disabled(tool)"
          @click="run(tool)"
        />
      </template>
      <Button
        v-if="fit.more.length > 0"
        ref="moreButton"
        :size="ButtonSize.ToolIcon"
        :variant="ButtonVariant.Soft"
        :icon="Ellipsis"
        :label="t('touch.editor.more')"
        :aria-expanded="moreOpen"
        @click="moreOpen = !moreOpen"
      />
      <Button
        :size="ButtonSize.Tool"
        :variant="ButtonVariant.Soft"
        :icon="ChevronUp"
        @click="emit('collapse')"
      >
        {{ t('touch.editor.collapse') }}
      </Button>
      <Button
        :size="ButtonSize.Tool"
        :variant="ButtonVariant.Primary"
        :icon="Save"
        :disabled="busy || !canSave"
        @click="emit('save')"
      >
        {{ t('touch.editor.save') }}
      </Button>

      <!-- The copy every button is measured on: never seen, never reached, and in a box of no
           size, so it never widens what scrolls around it. -->
      <div class="invisible pointer-events-none absolute top-0 left-0 size-0 overflow-hidden">
        <div ref="measure" class="flex gap-4" aria-hidden="true" inert>
          <Button
            v-for="tool in TOOLS"
            :key="tool"
            :data-measure="tool"
            :size="ButtonSize.ToolIcon"
            :icon="ICONS[tool]"
            :label="label(tool)"
            tabindex="-1"
          />
          <Button
            data-measure="more"
            :size="ButtonSize.ToolIcon"
            :icon="Ellipsis"
            :label="t('touch.editor.more')"
            tabindex="-1"
          />
          <Button data-measure="collapse" :size="ButtonSize.Tool" :icon="ChevronUp" tabindex="-1">
            {{ t('touch.editor.collapse') }}
          </Button>
          <Button
            data-measure="save"
            :size="ButtonSize.Tool"
            :variant="ButtonVariant.Primary"
            :icon="Save"
            tabindex="-1"
          >
            {{ t('touch.editor.save') }}
          </Button>
        </div>
      </div>
    </div>

    <DropdownMenu v-if="moreOpen" :open="true" @update:open="o => o || (moreOpen = false)">
      <DropdownMenuContent :anchor="moreAnchor" :aria-label="t('touch.editor.more')">
        <DropdownMenuItem
          v-for="(tool, i) in fit.more"
          :key="tool"
          :disabled="disabled(tool)"
          @select.prevent="fromMenu(tool)"
        >
          <Button
            :variant="ButtonVariant.Menu"
            :size="ButtonSize.MenuItem"
            :icon="ICONS[tool]"
            :disabled="disabled(tool)"
            :data-nav-initial="i === 0 ? '' : undefined"
          >
            {{ label(tool) }}
          </Button>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
