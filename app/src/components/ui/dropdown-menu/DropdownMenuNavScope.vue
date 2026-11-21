<script setup lang="ts">
// The open menu's nav scope (nav.ts): mounted inside Reka's content, so it exists exactly while
// the menu is open. The steps stay inside it, among its items; its first item (or its
// `data-nav-initial` one) takes the first focus, and B (`back`) closes it, not the screen behind.
import { ref } from 'vue'
import { injectDropdownMenuRootContext } from 'reka-ui'
import { useNavScope } from '@/nav'

const menu = injectDropdownMenuRootContext()
const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => menu.onOpenChange(false) })
</script>

<template>
  <div ref="root" class="contents">
    <slot />
  </div>
</template>
