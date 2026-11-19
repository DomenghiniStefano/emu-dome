<script setup lang="ts">
// Filter and sort panel (board L03, Stati): the order comes from the `library.sort` setting
// (the parent reads and writes it), the «Mostra» switches are the library view's own state.
// Controller / keyboard: its own nav scope while open; B / Esc / a click outside close it.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useNavScope } from '../nav'
import { SHOW_KEYS, SORTS, type ShowKey, type Sort } from '../librarySort'
import { useDismiss } from './library/useDismiss'

defineProps<{ sort: Sort; show: Record<ShowKey, boolean> }>()
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey]; reset: []; close: [] }>()

const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, escape: true, back: () => emit('close') })
// The parent holds the panel next to its toggle button, which toggles it itself.
useDismiss(root, () => emit('close'))
</script>

<template>
  <div
    ref="root"
    class="glass-thick absolute top-[calc(100%+var(--spacing-8))] right-0 z-raised flex w-menu flex-col gap-14 p-16"
    role="dialog"
    :aria-label="t('library.filters.label')"
  >
    <div class="text-overline text-ink-muted">{{ t('library.filters.sortBy') }}</div>
    <div class="flex flex-col gap-2" role="radiogroup" :aria-label="t('library.filters.sortBy')">
      <Button
        v-for="s in SORTS"
        :key="s"
        :variant="ButtonVariant.Menu"
        :size="ButtonSize.MenuRow"
        class="group -mx-6"
        role="radio"
        :aria-checked="s === sort"
        :data-selected="s === sort ? '' : undefined"
        :data-nav-initial="s === sort ? '' : undefined"
        @click="emit('sort', s)"
      >
        <span
          class="size-14 shrink-0 rounded-full border border-line-strong group-data-selected:border-check group-data-selected:border-accent-text"
          aria-hidden="true"
        ></span>
        {{ t(`library.sort.${s}`) }}
      </Button>
    </div>
    <div class="text-overline text-ink-muted">{{ t('library.filters.show') }}</div>
    <div class="flex flex-col gap-2">
      <Button
        v-for="k in SHOW_KEYS"
        :key="k"
        :variant="ButtonVariant.Menu"
        :size="ButtonSize.MenuRow"
        class="-mx-6"
        role="checkbox"
        :aria-checked="show[k]"
        @click="emit('toggle', k)"
      >
        <span
          class="size-14 shrink-0 rounded-4"
          :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
          aria-hidden="true"
        ></span>
        {{ t(`library.show.${k}`) }}
      </Button>
    </div>
    <Button @click="emit('reset')">{{ t('library.filters.reset') }}</Button>
  </div>
</template>
