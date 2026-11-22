<script setup lang="ts">
// The shell's rail (#59 item 1): the five sections, icon over name, Settings alone at the bottom.
// Moving onto an item does not switch section, confirming it does. The current section's item
// takes the first focus only when the section has no first control of its own.
//
// The phone on its side (#29): the same rail as a column on the camera's edge (`edge`), keeping
// the notch free; it moves to the other edge when the phone is turned the other way.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { cn } from '@/lib/cn'
import type { CameraSide } from '@/lib/phoneLayout'
import { SECTION_ICONS, SECTIONS, type Section } from './sections'

const props = defineProps<{
  current: Section
  /** The current section focuses a control of its own first. */
  contentHasInitial: boolean
  /** The phone on its side: the window's edge the rail sits on, the camera's. */
  edge?: CameraSide
}>()
const emit = defineEmits<{ pick: [Section] }>()

const { t } = useI18n()

const isCurrent = (s: Section) => s === props.current

/** Where the rail sits: the desktop's left, or the camera's edge with room for the notch, its
 * glass under the status bar and its items below it. */
const placement = {
  desktop: 'basis-rail border-r py-12',
  left: 'border-r pt-[calc(var(--spacing-8)+var(--spacing-safe-top))] pr-6 pb-8 pl-camera-left',
  right: 'border-l pt-[calc(var(--spacing-8)+var(--spacing-safe-top))] pr-camera-right pb-8 pl-6',
} as const

const nav = ref<HTMLElement | null>(null)
/** Back from a section's content: the focus returns to its item. */
function focusCurrent() {
  nav.value?.querySelector<HTMLElement>('[aria-current="page"]')?.focus()
}
defineExpose({ focusCurrent })
</script>

<template>
  <!-- The rail's 76px width is its own (the design system's `sidebar` is 56: Cowork). -->
  <nav
    ref="nav"
    :class="
      cn(
        'flex shrink-0 flex-col items-center gap-4 border-line bg-glass-thin',
        placement[edge ?? 'desktop'],
      )
    "
    :aria-label="t('shell.sections')"
  >
    <template v-for="s in SECTIONS" :key="s">
      <div v-if="s === 'settings'" class="flex-1"></div>
      <Button
        :variant="ButtonVariant.Rail"
        :size="ButtonSize.Rail"
        :aria-current="isCurrent(s) ? 'page' : undefined"
        :data-nav-initial="isCurrent(s) && !contentHasInitial ? '' : undefined"
        @click="emit('pick', s)"
      >
        <component :is="SECTION_ICONS[s]" :size="20" :stroke-width="1.75" aria-hidden="true" />
        <!-- 10/12 is off the type scale (the smallest step is 11/14: Cowork). -->
        <span class="text-rail font-medium whitespace-nowrap">{{ t(`shell.${s}`) }}</span>
      </Button>
    </template>
  </nav>
</template>
