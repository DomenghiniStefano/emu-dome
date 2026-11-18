<script setup lang="ts">
// The dialog's nav scope (nav.ts): mounted inside Reka's content, so it exists exactly while the
// dialog is open. While it is the latest scope every step goes to the dialog: the focus moves
// spatially among its controls, and `back` (B on the controller) closes it.
import { ref } from 'vue'
import { injectDialogRootContext } from 'reka-ui'
import { useNavScope } from '@/nav'

const dialog = injectDialogRootContext()
const close = () => dialog.onOpenChange(false)
const root = ref<HTMLElement | null>(null)
useNavScope(root, { back: close })
</script>

<template>
  <div ref="root" class="contents">
    <slot />
  </div>
</template>
