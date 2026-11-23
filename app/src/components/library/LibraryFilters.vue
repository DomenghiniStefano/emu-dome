<script setup lang="ts">
// Filter and sort panel (board L03, Stati), inside its Popover (SortFilterButton): the order comes
// from the `library.sort` setting (the parent reads and writes it) as a RadioGroup, the «Mostra»
// switches, the library view's own state, as Checkboxes. This is the panel's body, which scrolls
// when the window is short; «Ripristina» is its footer, kept in view (SortFilterButton, #106).
// Controller / keyboard: the popover's own nav scope while open, the order chosen focused first;
// B / Esc / a click outside close it.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Checkbox } from '@/design-system/components/checkbox'
import { RadioGroup, RadioGroupItem } from '@/design-system/components/radio-group'
import { SHOW_KEYS, SORTS, type ShowKey, type Sort } from './librarySort'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

defineProps<{ sort: Sort; show: Record<ShowKey, boolean> }>()
const emit = defineEmits<{ sort: [Sort]; toggle: [ShowKey] }>()

const { t } = useI18n()

function choose(value: unknown) {
  const s = SORTS.find(x => x === value)
  if (s) emit('sort', s)
}
</script>

<template>
  <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('library.filters.sortBy') }}</Heading>
  <RadioGroup
    :model-value="sort"
    orientation="vertical"
    class="flex flex-col gap-2"
    :aria-label="t('library.filters.sortBy')"
    @update:model-value="choose"
  >
    <RadioGroupItem v-for="s in SORTS" :key="s" :value="s">
      <Button
        :variant="ButtonVariant.Menu"
        :size="ButtonSize.MenuRow"
        class="group -mx-6"
        :data-selected="s === sort ? '' : undefined"
        :data-nav-initial="s === sort ? '' : undefined"
      >
        <span
          class="size-14 shrink-0 rounded-full border border-line-strong group-data-selected:border-check group-data-selected:border-accent-text"
          aria-hidden="true"
        ></span>
        {{ t(`library.sort.${s}`) }}
      </Button>
    </RadioGroupItem>
  </RadioGroup>
  <Heading as="h2" :variant="HeadingVariant.Overline">{{ t('library.filters.show') }}</Heading>
  <div class="flex flex-col gap-2">
    <Checkbox
      v-for="k in SHOW_KEYS"
      :key="k"
      :model-value="show[k]"
      @update:model-value="emit('toggle', k)"
    >
      <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
        <span
          class="size-14 shrink-0 rounded-badge"
          :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
          aria-hidden="true"
        ></span>
        {{ t(`library.show.${k}`) }}
      </Button>
    </Checkbox>
  </div>
</template>
