<script setup lang="ts">
// The open popover's nav scope (nav.ts): mounted inside Reka's content, so it exists exactly while
// the popover is open. The steps stay inside it, its `data-nav-initial` control takes the first
// focus, and B (`back`) closes it, not the screen behind it.
import { ref } from 'vue'
import { injectPopoverRootContext } from 'reka-ui'
import { useNavScope } from '@/nav'

const popover = injectPopoverRootContext()
const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => popover.onOpenChange(false) })
</script>

<template>
  <div ref="root" class="contents">
    <slot />
  </div>
</template>
