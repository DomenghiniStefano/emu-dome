<script setup lang="ts">
// The dialog's panel over its scrim (FlowDialog): thick glass, 520px at most, centred.
//
// With the app's navigation:
// - the first focus goes to the `data-nav-initial` control when there is one (else Reka's first
//   control), and on closing it goes back to what was focused before the dialog opened;
// - B on the controller is `back` (DialogNavScope); Escape is Reka's own;
// - Reka's focus trap only keeps Tab inside, which nav.ts never leaves either.
import type { DialogContentEmits, DialogContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed, nextTick, ref } from 'vue'
import { DialogContent, DialogOverlay, DialogPortal, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'
import DialogNavScope from './DialogNavScope.vue'

// Attributes (a `role`, an `aria-*`) go to the panel, not to the portal around it.
defineOptions({ inheritAttrs: false })

const props = defineProps<DialogContentProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DialogContentEmits>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
const forwarded = useForwardPropsEmits(delegated, emits)

const content = ref<{ $el: HTMLElement } | null>(null)
let before: HTMLElement | null = null

function onOpen(e: Event) {
  before = document.activeElement instanceof HTMLElement ? document.activeElement : null
  const initial = content.value?.$el.querySelector<HTMLElement>('[data-nav-initial]')
  if (!initial) return
  e.preventDefault()
  initial.focus()
}
// Reka's own return lost the focus to <body> when the trigger is a primitive (measured
// 2026-11-18): the element focused at opening is put back instead.
function onClose(e: Event) {
  const target = before
  before = null
  if (!target?.isConnected) return
  e.preventDefault()
  void nextTick(() => target.focus())
}
</script>

<template>
  <DialogPortal>
    <DialogOverlay
      data-slot="dialog-overlay"
      class="fixed inset-0 z-dialog grid place-items-center bg-scrim p-16"
    >
      <DialogContent
        ref="content"
        data-slot="dialog-content"
        v-bind="{ ...forwarded, ...$attrs }"
        :class="
          cn('glass-thick flex w-full max-w-dialog flex-col gap-12 p-dialog-pad', props.class)
        "
        @open-auto-focus="onOpen"
        @close-auto-focus="onClose"
      >
        <DialogNavScope>
          <slot />
        </DialogNavScope>
      </DialogContent>
    </DialogOverlay>
  </DialogPortal>
</template>
