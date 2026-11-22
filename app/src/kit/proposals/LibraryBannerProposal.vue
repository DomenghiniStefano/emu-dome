<script setup lang="ts">
// Kit proposal (library group, the banners over the grid): the scan running, what it added (L11)
// and a folder not reachable (E01) as one banner shape: a row with the same edge, corners and
// padding, neutral or warning. It wraps in a narrow window (the action under the text instead of
// the text squeezed into a column of letters), and «N giochi aggiunti» is one row with its counts
// instead of a large glass panel. Only shown on the Kit page.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderX, X } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import type { ConsoleId } from '@/emu'
import { consoleCounts } from '@/lib/consoleCounts'

// One props object, each field read only by its kind: Vue turns a union of props into every field
// required, which warns for the kinds that leave them out.
const props = defineProps<{
  kind: 'progress' | 'summary' | 'unreachable'
  found?: number | null
  added?: number
  perConsole?: Partial<Record<ConsoleId, number>>
  path?: string
  count?: number
}>()

const { t } = useI18n()

const counts = computed(() =>
  props.kind === 'summary' ? consoleCounts(props.perConsole ?? {}, { hideEmpty: true }) : [],
)
</script>

<template>
  <div
    class="flex flex-wrap items-center gap-x-12 gap-y-8 rounded-10 border px-14 py-10"
    :class="
      props.kind === 'unreachable'
        ? 'border-warning-edge bg-warning-tint'
        : 'border-line bg-surface'
    "
    role="status"
  >
    <template v-if="props.kind === 'progress'">
      <span class="font-medium">{{ t('library.scan.title') }}</span>
      <Progress class="max-w-scan min-w-0 flex-1" :label="t('library.scan.title')" />
      <span v-if="props.found != null" class="text-ink-muted tabular-nums">
        {{ t('library.scan.found', { found: props.found }, props.found) }}
      </span>
    </template>
    <template v-else-if="props.kind === 'summary'">
      <span class="font-semibold">
        {{ t('library.added.title', { n: props.added ?? 0 }, props.added ?? 0) }}
      </span>
      <span class="min-w-0 flex-1 text-ink-muted tabular-nums">
        <template v-for="(c, i) in counts" :key="c.id">
          <template v-if="i > 0"> · </template>
          <span class="font-semibold text-ink">{{ c.count }}</span> {{ t(c.label) }}
        </template>
      </span>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconSmall"
        :aria-label="t('library.added.close')"
        :title="t('library.added.close')"
      >
        <X :size="16" :stroke-width="1.75" aria-hidden="true" />
      </Button>
    </template>
    <template v-else>
      <span class="flex min-w-0 grow basis-menu-min items-start gap-12">
        <FolderX
          class="flex-none text-warning"
          :size="18"
          :stroke-width="1.75"
          aria-hidden="true"
        />
        <span class="min-w-0 wrap-anywhere">{{
          t(
            'library.unreachable.text',
            { path: props.path ?? '', count: props.count ?? 0 },
            props.count ?? 0,
          )
        }}</span>
      </span>
      <Button class="ml-auto">{{ t('library.unreachable.choose') }}</Button>
    </template>
  </div>
</template>
