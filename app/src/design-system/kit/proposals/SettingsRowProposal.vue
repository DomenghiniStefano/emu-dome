<script setup lang="ts">
// Proposal for SettingRow (Kit review; the level marks part is applied in SettingRow.vue, the
// locked look is not): the row's level marks live in the text
// column, on a line under the description («Ripristina» / «Usa il valore di …», or «Come in …»),
// so the control stays alone on the right at every level and never wraps under the text. A locked
// row fades its control, not its words: the reason stays readable in every theme.
import { Undo2 } from '@lucide/vue'
import { Badge } from '@/design-system/components/badge'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'

withDefaults(
  defineProps<{
    name: string
    desc: string
    changed?: boolean
    /** «Ripristina» / «Usa il valore di «…»», shown with «Modificato». */
    resetText?: string
    /** «Come in «…»», when the value is inherited. */
    inherited?: string
    onlyOn?: string
    locked?: boolean
  }>(),
  { changed: false, resetText: '', inherited: '', onlyOn: '', locked: false },
)
</script>

<template>
  <div class="flex flex-col gap-8 border-t border-line px-12 py-6 first:border-t-0">
    <div class="flex items-center gap-x-12">
      <div class="flex min-w-0 flex-1 flex-col">
        <div class="flex flex-wrap items-center gap-8">
          <span v-if="changed" class="size-6 rounded-full bg-accent-text" aria-hidden="true"></span>
          <span class="font-medium" :class="{ 'text-ink-muted': locked }">{{ name }}</span>
          <span v-if="changed" class="text-caption text-accent-text">Modificato</span>
          <Badge v-if="onlyOn">{{ onlyOn }}</Badge>
        </div>
        <div class="text-caption text-ink-muted">{{ desc }}</div>
        <div
          v-if="(changed && resetText) || inherited"
          class="flex items-center gap-8 text-caption"
        >
          <Button
            v-if="changed && resetText"
            :variant="ButtonVariant.Text"
            :fade="ButtonFade.None"
            :size="ButtonSize.Bare"
          >
            <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
            {{ resetText }}
          </Button>
          <span v-if="inherited" class="text-ink-muted">{{ inherited }}</span>
        </div>
      </div>
      <div class="flex flex-none items-center gap-8" :class="{ 'opacity-locked': locked }">
        <slot />
      </div>
    </div>
    <slot name="below" />
  </div>
</template>
