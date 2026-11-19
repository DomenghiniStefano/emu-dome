<script setup lang="ts">
// The Settings section's left column: the title, the search, «In primo piano» (T01) and the
// schema's pages by group, each with how many of its settings are changed at the level edited.
// The page shown is marked (`aria-current`) only while no search is typed. In a short window the
// entries give up height before the column scrolls (`shrink`, as the list always did: Cowork to
// decide whether it should scroll instead).
import { useI18n } from 'vue-i18n'
import { Search } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Input, InputVariant } from '@/components/ui/input'
import type { SettingsSchema } from '../../emu'
import { msg } from '../../settingsModel'
import { OVERVIEW } from './settingsPage'
import { Heading, HeadingVariant } from '@/components/ui/heading'

const props = defineProps<{
  schema: SettingsSchema
  page: string
  /** Settings changed at the level edited, on a page. */
  changedOn: (page: string) => number
}>()
const query = defineModel<string>('query', { required: true })
const emit = defineEmits<{ pick: [string] }>()
const { t } = useI18n()

/** The page shown, while no search is typed. */
const current = (p: string) => (p === props.page && !query.value ? 'page' : undefined)
</script>

<template>
  <nav
    class="flex flex-none basis-sidebar flex-col gap-16 overflow-y-auto border-r border-line px-12 py-24"
    :aria-label="t('settingsUi.categories')"
  >
    <div class="flex flex-col gap-12">
      <Heading as="h1" class="px-8">{{ t('settingsUi.title') }}</Heading>
      <label
        class="flex h-control-lg items-center gap-8 rounded-8 border border-line-strong bg-surface-raised px-12 text-ink-muted"
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
  </nav>
</template>
