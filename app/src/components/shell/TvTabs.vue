<script setup lang="ts">
// The sections as tabs at the top, in TV mode (#30, mockup «TV-Libreria»): in place of the rail,
// L1 and R1 at their sides (they switch section from anywhere: AppShell), the current one an ink
// pill, the connected controller's name on the right. As on the rail, moving onto a tab does not
// switch section, confirming it does; the current tab takes the first focus only when the
// section has no first control of its own.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { uiMode } from '@/composables/useUiMode'
import { gamepadName } from '@/i18n/labels'
import { SECTIONS, type Section } from './sections'
import TvGlyph from './TvGlyph.vue'

const props = defineProps<{
  current: Section
  /** The current section focuses a control of its own first. */
  contentHasInitial: boolean
}>()
const emit = defineEmits<{ pick: [Section] }>()

const { t } = useI18n()

const isCurrent = (s: Section) => s === props.current

const nav = ref<HTMLElement | null>(null)
/** Back from a section's content: the focus returns to its tab. */
function focusCurrent() {
  nav.value?.querySelector<HTMLElement>('[aria-current="page"]')?.focus()
}
/** Whether the focus is on a tab (L1 / R1 then moves it to the new current one). */
const hasFocus = () => nav.value?.contains(document.activeElement) ?? false
defineExpose({ focusCurrent, hasFocus })
</script>

<template>
  <!-- Three columns: the tabs stay centred, the controller's name shrinks before reaching them. -->
  <header
    class="grid shrink-0 basis-tv-tabs grid-cols-[1fr_auto_1fr] items-center gap-24 px-tv-edge"
  >
    <span aria-hidden="true"></span>
    <nav ref="nav" class="flex items-center gap-24" :aria-label="t('shell.sections')">
      <TvGlyph action="prev" aria-hidden="true" />
      <Button
        v-for="s in SECTIONS"
        :key="s"
        :variant="ButtonVariant.TvTab"
        :size="ButtonSize.TvTab"
        :aria-current="isCurrent(s) ? 'page' : undefined"
        :data-nav-initial="isCurrent(s) && !contentHasInitial ? '' : undefined"
        @click="emit('pick', s)"
      >
        {{ t(`shell.${s}`) }}
      </Button>
      <TvGlyph action="next" aria-hidden="true" />
    </nav>
    <span class="min-w-0 truncate text-right text-tv-hint text-ink-muted">
      {{ uiMode.pad ? gamepadName(uiMode.pad) : '' }}
    </span>
  </header>
</template>
