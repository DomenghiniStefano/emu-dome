<script setup lang="ts">
// The open list's nav scope (nav.ts): B (`back`) closes the list and not the screen behind it.
// Reka itself puts the focus on the chosen option and moves it on the arrows (nav.ts hands them
// over), so no autofocus here.
//
// Reka renders a closed Select's content too, out of the page, to know its options: this
// component is mounted then as well. Its scope only has a root while the list is open, and a scope
// without a root receives no steps (nav.ts `dispatchNav`).
import { ref, watchEffect } from 'vue'
import { injectSelectRootContext } from 'reka-ui'
import { useNavScope } from '@/nav'

const select = injectSelectRootContext()
const el = ref<HTMLElement | null>(null)
const root = ref<HTMLElement | null>(null)
watchEffect(() => (root.value = select.open.value ? el.value : null))
useNavScope(root, { back: () => select.onOpenChange(false) })
</script>

<template>
  <div ref="el" class="contents">
    <slot />
  </div>
</template>
