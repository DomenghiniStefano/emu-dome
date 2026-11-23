<script setup lang="ts">
// The phone's tab bar (#29, the owner's mockup «EmuDome sul telefono», option A): held upright,
// the five sections at the bottom, in reach of the thumb, in the rail's order. Each tab is at
// least the touch minimum, icon over name; the current one in accent text. Below it the room
// the system's navigation bar takes. Like the rail, the current tab takes the first focus only
// when the section has no first control of its own.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { SECTION_ICONS, SECTIONS, type Section } from './sections'

const props = defineProps<{
  current: Section
  /** The current section focuses a control of its own first. */
  contentHasInitial: boolean
}>()
const emit = defineEmits<{ pick: [Section] }>()

const { t } = useI18n()

const isCurrent = (s: Section) => s === props.current

const nav = ref<HTMLElement | null>(null)
/** The focus to the current section's tab. */
function focusCurrent() {
  nav.value?.querySelector<HTMLElement>('[aria-current="page"]')?.focus()
}
defineExpose({ focusCurrent })
</script>

<template>
  <nav
    ref="nav"
    class="flex shrink-0 items-stretch border-t border-line px-4 pt-4 pb-tabbar-bottom glass-thick-flat"
    :aria-label="t('shell.sections')"
  >
    <Button
      v-for="s in SECTIONS"
      :key="s"
      :variant="ButtonVariant.PhoneTab"
      :size="ButtonSize.PhoneTab"
      :aria-current="isCurrent(s) ? 'page' : undefined"
      :data-nav-initial="isCurrent(s) && !contentHasInitial ? '' : undefined"
      @click="emit('pick', s)"
    >
      <component :is="SECTION_ICONS[s]" :size="24" :stroke-width="1.75" aria-hidden="true" />
      <span class="max-w-full truncate text-fine font-medium">{{ t(`shell.${s}`) }}</span>
    </Button>
  </nav>
</template>
