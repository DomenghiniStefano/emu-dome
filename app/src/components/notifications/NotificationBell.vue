<script setup lang="ts">
// X03 (mockup «Ricerca e notifiche», approved 2026-11-21): the top bar's bell with the unread
// count (accent badge), opening a thick glass panel under it: «Notifiche» with «Segna tutte come
// lette» (its icon alone, #104), then every notification, newest first, unread ones in semibold, a coloured dot per kind.
// They stay until read: choosing one marks it read. In game only the short toasts show (G13);
// the same events are found here afterwards.
// Controller: the panel is a Popover (its nav scope: d-pad through the items, A reads one, B
// closes); closing puts the focus back on the bell.
import { computed, nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Bell, CheckCheck } from '@lucide/vue'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverVariant } from '@/components/ui/popover'
import { saveWhenLabel, timeAgoLabel } from '@/i18n/labels'
import { cn } from '@/lib/cn'
import { NoticeTone, badgeText, noticeView, noticeWhen } from './notifications'
import { useNotifications } from './useNotifications'

const { t } = useI18n()
const { list, unread, markRead, markAllRead } = useNotifications()

const DOT: Record<NoticeTone, string> = {
  [NoticeTone.Success]: 'bg-success',
  [NoticeTone.Warning]: 'bg-warning',
  [NoticeTone.Accent]: 'bg-accent',
  [NoticeTone.Faint]: 'bg-ink-faint',
}

const open = ref(false)
const button = ref<InstanceType<typeof Button> | null>(null)
const buttonEl = computed(() => (button.value?.$el as HTMLElement | undefined) ?? null)
function close() {
  open.value = false
  void nextTick(() => buttonEl.value?.focus())
}

// Read once per opening: «5 minuti fa» does not tick while the panel is open.
const now = ref(Date.now())
function toggle() {
  if (open.value) return close()
  now.value = Date.now()
  open.value = true
}
const rows = computed(() =>
  list.value.map(n => ({
    n,
    view: noticeView(n, { t }),
    when: noticeWhen(n.at, now.value, { ago: timeAgoLabel, time: saveWhenLabel }),
  })),
)
</script>

<template>
  <Popover :open="open" @update:open="o => o || close()">
    <Button
      ref="button"
      :variant="ButtonVariant.BarIcon"
      :size="ButtonSize.Icon"
      :label="t('notifications.bell', { n: unread }, unread)"
      :aria-expanded="open"
      aria-haspopup="dialog"
      @click="toggle"
    >
      <Bell :size="20" :stroke-width="1.75" aria-hidden="true" />
      <Badge
        v-if="unread > 0"
        :variant="BadgeVariant.Count"
        :size="BadgeSize.Count"
        class="absolute top-3 right-3"
        aria-hidden="true"
      >
        {{ badgeText(unread) }}
      </Badge>
    </Button>
    <PopoverContent
      :anchor="buttonEl"
      :variant="PopoverVariant.Notifications"
      :aria-label="t('notifications.title')"
    >
      <div class="flex flex-none items-center gap-8 px-8 pt-4 pb-8">
        <span class="flex-1 text-heading">{{ t('notifications.title') }}</span>
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.IconSmall"
          :icon="CheckCheck"
          :label="t('notifications.markAll')"
          :disabled="unread === 0"
          :data-nav-initial="list.length === 0 ? '' : undefined"
          @click="markAllRead"
        />
      </div>
      <div v-if="rows.length" class="flex min-h-0 flex-col divide-y divide-line overflow-auto">
        <Button
          v-for="(r, i) in rows"
          :key="r.n.id"
          :variant="ButtonVariant.Result"
          :size="ButtonSize.Notice"
          :data-nav-initial="i === 0 ? '' : undefined"
          @click="r.n.read || markRead(r.n.id)"
        >
          <span :class="cn('mt-5 size-8 flex-none rounded-full', DOT[r.view.tone])"></span>
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span :class="r.n.read ? 'font-medium' : 'font-semibold'">{{ r.view.title }}</span>
            <span class="text-caption text-ink-muted">{{ r.view.sub }}</span>
          </span>
          <span class="flex-none text-caption whitespace-nowrap text-ink-muted">{{ r.when }}</span>
        </Button>
      </div>
      <div v-else class="px-8 pb-8 text-ink-muted">{{ t('notifications.empty') }}</div>
    </PopoverContent>
  </Popover>
</template>
