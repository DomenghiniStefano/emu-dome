<script setup lang="ts">
// A long choice from the sofa (Settings in TV mode, #30): the whole screen over the blurred
// settings, the setting's name and description on top, its options as large rows, the buttons'
// hints at the bottom. The chosen option takes the first focus; A picks the focused one (a
// single choice closes the sheet, a multiple one switches it on or off and stays), B closes
// (the dialog's nav scope). An option not available yet is shown faded, says so, and cannot be
// picked.
import { Check, Square, SquareCheck } from '@lucide/vue'
import TvButtonBar from '@/components/shell/TvButtonBar.vue'
import type { TvHint } from '@/components/shell/tvHints'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogDescriptionSize,
  DialogSize,
  DialogTitle,
  DialogTitleSize,
} from '@/components/ui/dialog'

/** One option of the sheet. */
export interface SheetOption {
  id: string
  text: string
  desc?: string
  /** Shown faded with its reason, not pickable. */
  disabled?: boolean
  /** A colour to show beside the words (`general.accent`). */
  swatch?: string
}

const props = defineProps<{
  title: string
  desc: string
  options: SheetOption[]
  chosen: string[]
  /** A multiple choice: each option switches on or off, the sheet stays open. */
  multi?: boolean
}>()
const emit = defineEmits<{ pick: [id: string]; close: [] }>()

const HINTS: TvHint[] = [
  { actions: ['confirm'], label: 'tv.hint.choose' },
  { actions: ['back'], label: 'tv.hint.back' },
]

const isChosen = (id: string) => props.chosen.includes(id)
/** The option focused first: the chosen one (the first chosen of a multiple choice), else none
 * marked and the dialog focuses the first. */
const initial = (o: SheetOption) => (!o.disabled && props.chosen[0] === o.id ? '' : undefined)

function pick(o: SheetOption) {
  if (o.disabled) return
  emit('pick', o.id)
  if (!props.multi) emit('close')
}
</script>

<template>
  <Dialog :open="true" @update:open="open => !open && emit('close')">
    <DialogContent :size="DialogSize.TvSheet">
      <div class="flex min-h-0 flex-1 flex-col items-center">
        <header class="flex w-tv-sheet max-w-full flex-col gap-12 px-tv-row-x pt-tv-edge pb-24">
          <DialogTitle :size="DialogTitleSize.Tv">{{ title }}</DialogTitle>
          <DialogDescription :size="DialogDescriptionSize.Tv">{{ desc }}</DialogDescription>
        </header>
        <!-- Room around the rows for the focus ring (4 + 4), which the scrolling list would cut. -->
        <div
          class="flex min-h-0 w-tv-sheet max-w-full flex-1 flex-col gap-tv-row-gap overflow-y-auto p-8 [scrollbar-width:none]"
        >
          <Button
            v-for="o in options"
            :key="o.id"
            :variant="ButtonVariant.TvRow"
            :size="ButtonSize.TvRow"
            :fade="ButtonFade.None"
            class="flex-none"
            :aria-disabled="o.disabled || undefined"
            :aria-current="!multi && isChosen(o.id) ? 'true' : undefined"
            :aria-pressed="multi ? isChosen(o.id) : undefined"
            :data-nav-initial="initial(o)"
            @click="pick(o)"
          >
            <span
              class="flex min-w-0 flex-1 items-center gap-16"
              :class="{ 'opacity-locked': o.disabled }"
            >
              <span
                v-if="o.swatch"
                class="size-tv-glyph flex-none rounded-full border border-line"
                aria-hidden="true"
                :style="{ background: o.swatch }"
              ></span>
              <span class="flex min-w-0 flex-col gap-4">
                <span :class="isChosen(o.id) ? 'font-semibold' : undefined">{{ o.text }}</span>
                <span v-if="o.desc" class="text-tv-hint text-ink-muted">{{ o.desc }}</span>
              </span>
            </span>
            <!-- The icons as large as a controller glyph, so they grow with the TV sizes. -->
            <template v-if="multi">
              <SquareCheck
                v-if="isChosen(o.id)"
                class="size-tv-glyph flex-none"
                :stroke-width="2"
                aria-hidden="true"
              />
              <Square
                v-else
                class="size-tv-glyph flex-none text-ink-muted"
                :stroke-width="2"
                aria-hidden="true"
              />
            </template>
            <Check
              v-else-if="isChosen(o.id)"
              class="size-tv-glyph flex-none"
              :stroke-width="2.25"
              aria-hidden="true"
            />
          </Button>
        </div>
        <TvButtonBar :hints="HINTS" class="self-stretch" />
      </div>
    </DialogContent>
  </Dialog>
</template>
