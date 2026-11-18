<script setup lang="ts">
// An on / off switch (SettingRow `.switch`) on Reka's SwitchRoot. Reka toggles on its own Enter;
// nav.ts leaves that Enter to it and sends the controller's A as the same Enter (Decision 6 of
// docs/specs/2026-11-18-design-system-foundations.md), so one press is one toggle.
import type { SwitchRootEmits, SwitchRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { SwitchRoot, SwitchThumb, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<SwitchRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<SwitchRootEmits>()

const delegated = computed(() => {
  const { class: _class, ...rest } = props
  return rest
})
const forwarded = useForwardPropsEmits(delegated, emits)
</script>

<template>
  <!-- 36 x 20 with a 16px knob (SettingRow): off, an edged track on surface-raised with the knob
       at the start; on, the accent's track with its glow and the knob at the end. -->
  <SwitchRoot
    data-slot="switch"
    v-bind="forwarded"
    :class="
      cn(
        'flex h-20 w-switch shrink-0 cursor-pointer justify-start rounded-full border border-line-strong bg-surface-raised p-2 data-[state=checked]:justify-end data-[state=checked]:border-0 data-[state=checked]:bg-accent-track data-[state=checked]:shadow-glow-sm disabled:cursor-not-allowed',
        props.class,
      )
    "
  >
    <SwitchThumb data-slot="switch-thumb" class="block size-16 rounded-full bg-knob" />
  </SwitchRoot>
</template>
