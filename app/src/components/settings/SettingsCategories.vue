<script setup lang="ts">
// The Settings section's left column: the title, the search, «In primo piano» (T01) and the
// schema's pages by group, each with how many of its settings are changed at the level edited.
// The page shown is marked (`aria-current`) only while no search is typed. In a short window the
// entries give up height before the column scrolls (`shrink`, as the list always did; whether it
// should scroll instead is a design decision).
// The phone held upright (#29, settingsPane.ts): the list alone, full width, scrolling; while a
// search is typed, only the title and the field, the results under them.
import { useI18n } from 'vue-i18n'
import { Search } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Input, InputVariant } from '@/components/ui/input'
import { cn } from '@/lib/cn'
import type { SettingsSchema } from '@/emu'
import { msg } from '@/lib/settingsModel'
import { OVERVIEW } from './settingsPage'
import { SettingsPane } from './settingsPane'
import { Heading, HeadingVariant } from '@/components/ui/heading'

const props = defineProps<{
  schema: SettingsSchema
  page: string
  /** Settings changed at the level edited, on a page. */
  changedOn: (page: string) => number
  /** Which panes the section shows (the phone held upright: one at a time). */
  pane?: SettingsPane
}>()
const query = defineModel<string>('query', { required: true })
const emit = defineEmits<{ pick: [string] }>()
const { t } = useI18n()

/** The page shown, while no search is typed. */
const current = (p: string) => (p === props.page && !query.value ? 'page' : undefined)

/** Beside the page on the desk; full width upright; the field alone over a search's results. */
const placement: Record<SettingsPane, string> = {
  [SettingsPane.Both]: 'basis-sidebar overflow-y-auto border-r py-12',
  [SettingsPane.Page]: 'basis-sidebar overflow-y-auto border-r py-12',
  [SettingsPane.List]: 'min-h-0 flex-1 overflow-y-auto py-12',
  [SettingsPane.Search]: 'pt-12 pb-0',
}
</script>

<template>
  <nav
    :class="
      cn('flex flex-none flex-col gap-12 border-line px-8', placement[pane ?? SettingsPane.Both])
    "
    :aria-label="t('settingsUi.categories')"
  >
    <div class="flex flex-col gap-8">
      <Heading as="h1" class="px-8">{{ t('settingsUi.title') }}</Heading>
      <label
        class="flex h-control items-center gap-8 rounded-control border border-line-strong bg-surface-raised px-8 text-ink-muted"
      >
        <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
        <span class="sr-only">{{ t('settingsUi.search') }}</span>
        <Input
          v-model="query"
          :variant="InputVariant.Bare"
          type="search"
          :placeholder="t('settingsUi.searchPlaceholder')"
        />
      </label>
    </div>
    <template v-if="pane !== SettingsPane.Search">
      <Button
        :variant="ButtonVariant.NavHead"
        :size="ButtonSize.NavItem"
        class="shrink"
        :aria-current="current(OVERVIEW)"
        :data-nav-initial="page === OVERVIEW ? '' : undefined"
        @click="emit('pick', OVERVIEW)"
      >
        <span class="flex-1 text-left">{{ t('settingsUi.overview.name') }}</span>
      </Button>
      <div v-for="g in schema.groups" :key="g.id" class="flex flex-col gap-2">
        <Heading as="h2" :variant="HeadingVariant.Overline" class="px-8 pb-4">{{
          t(msg.group(g.id))
        }}</Heading>
        <Button
          v-for="p in g.pages"
          :key="p"
          :variant="ButtonVariant.NavItem"
          :size="ButtonSize.NavItem"
          class="shrink"
          :aria-current="current(p)"
          :data-nav-initial="p === page ? '' : undefined"
          @click="emit('pick', p)"
        >
          <span class="flex-1 text-left">{{ t(msg.page(p)) }}</span>
          <span v-if="changedOn(p)" class="text-caption font-medium text-accent-text tabular-nums">
            {{ changedOn(p) }}
          </span>
        </Button>
      </div>
    </template>
  </nav>
</template>
