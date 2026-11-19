<script setup lang="ts">
// The shell's rail (#59 item 1): the five sections, icon over name, Settings alone at the bottom.
// Moving onto an item does not switch section, confirming it does. The current section's item
// takes the first focus only when the section has no first control of its own.
import { ref, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Cloud, Gamepad2, Library, Save, Settings2 } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { SECTIONS, type Section } from '../../sections'

const props = defineProps<{
  current: Section
  /** The current section focuses a control of its own first. */
  contentHasInitial: boolean
}>()
const emit = defineEmits<{ pick: [Section] }>()

const { t } = useI18n()

const ICONS: Record<Section, Component> = {
  library: Library,
  saves: Save,
  controls: Gamepad2,
  cloud: Cloud,
  settings: Settings2,
}

const isCurrent = (s: Section) => s === props.current

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
    class="flex shrink-0 basis-[76px] flex-col items-center gap-4 border-r border-line bg-glass-thin py-12"
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
        <component :is="ICONS[s]" :size="20" :stroke-width="1.75" aria-hidden="true" />
        <!-- 10/12 is off the type scale (the smallest step is 11/14: Cowork). -->
        <span class="text-[10px]/[12px] font-medium whitespace-nowrap">{{ t(`shell.${s}`) }}</span>
      </Button>
    </template>
  </nav>
</template>
