<script setup lang="ts">
// One row of C03's shortcut list: the action, what to press (the default slot) with its badge
// when reserved, and the row's own control (the `change` slot: «Cambia», or nothing).
//
// Off the design system's scales (Cowork): the badge's 11px text on the 18px line.
defineProps<{
  action: string
  /** A reserved or keyboard-only row: the action in muted ink. */
  muted?: boolean
  /** «In arrivo», «Con Nintendo DS»: a reserved row's badge. */
  badge?: string
}>()
</script>

<template>
  <div
    class="grid grid-cols-[minmax(0,1fr)_auto_72px] items-center gap-12 border-line px-16 py-10 not-first:border-t"
  >
    <span :class="{ 'text-ink-muted': muted }">{{ action }}</span>
    <span
      class="inline-flex min-w-[150px] items-center justify-end gap-8 text-right font-semibold tabular-nums"
    >
      <span
        v-if="badge"
        class="rounded-4 border border-dashed border-line-strong px-6 text-[11px] font-medium text-ink-muted"
      >
        {{ badge }}
      </span>
      <slot />
    </span>
    <slot name="change"><span></span></slot>
  </div>
</template>
