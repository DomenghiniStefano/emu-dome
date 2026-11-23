<script setup lang="ts">
// The settings pages in TV mode (#30, the owner's option B: the desk's structure, larger): the
// schema's groups and pages as a column on the left, the page shown on accent-soft, how many of
// its settings are changed at the level edited. As on the desk, moving onto a page does not
// switch it, confirming it does; the column scrolls when the screen is short.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import type { SettingsSchema } from '@/emu'
import { msg } from '@/lib/settingsModel'

defineProps<{
  schema: SettingsSchema
  page: string
  /** Settings changed at the level edited, on a page. */
  changedOn: (page: string) => number
}>()
const emit = defineEmits<{ pick: [string] }>()
const { t } = useI18n()
</script>

<template>
  <!-- Room around the entries for the focus ring (4 + 4), which the scrolling column would cut. -->
  <nav
    class="flex w-tv-sidebar flex-none flex-col gap-tv-shelf-gap overflow-y-auto p-8 pb-tv-shelf-gap [scrollbar-width:none]"
    :aria-label="t('settingsUi.categories')"
  >
    <div v-for="g in schema.groups" :key="g.id" class="flex flex-col gap-tv-row-gap">
      <h2 class="m-0 px-tv-row-y text-tv-overline text-ink-muted uppercase">
        {{ t(msg.group(g.id)) }}
      </h2>
      <Button
        v-for="p in g.pages"
        :key="p"
        :variant="ButtonVariant.NavItem"
        :size="ButtonSize.TvNavItem"
        :aria-current="p === page ? 'page' : undefined"
        :data-nav-initial="p === page ? '' : undefined"
        @click="emit('pick', p)"
      >
        <span class="min-w-0 flex-1 truncate text-left">{{ t(msg.page(p)) }}</span>
        <span v-if="changedOn(p)" class="font-semibold text-accent-text tabular-nums">
          {{ changedOn(p) }}
        </span>
      </Button>
    </div>
  </nav>
</template>
