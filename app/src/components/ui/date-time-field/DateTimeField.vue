<script setup lang="ts">
// A date and time to the minute (`v-model`: `YYYY-MM-DDTHH:MM`, a local time), on Reka's
// DateField. Closed it is a field with the date and time in words (`text`, written by the screen
// in the UI language); A, Enter or a click opens its segments in a Popover under it, the first
// one focused:
// - up / down change the focused segment, left / right move along them (stopping at the ends),
//   digits type into it;
// - A / Enter, B / Esc or a click outside close it, and the value is written then, once, if it
//   changed (not on every step), and the focus goes back to the field.
// The arrows and Enter of a segment are its own by nav.ts's rule for Reka's date segments.
import type { HTMLAttributes } from 'vue'
import { computed, nextTick, ref } from 'vue'
import { CalendarDateTime } from '@internationalized/date'
import type { DateValue } from 'reka-ui'
import { DateFieldInput, DateFieldRoot } from 'reka-ui'
import { CalendarClock } from '@lucide/vue'
import { Popover, PopoverContent, PopoverVariant } from '@/components/ui/popover'
import { cn } from '@/lib/cn'
import { dateTimeText, parseDateTimeText } from './dateTimeText'
import type { DateTimeFieldVariant } from './variants'
import { dateSegmentVariants, dateTimeFieldVariants } from './variants'

// Attributes (an `id`) go to the field, not to the popover around it.
defineOptions({ inheritAttrs: false })

const props = defineProps<{
  /** What it is, for screen readers (the setting's name). */
  label: string
  /** The value in words, shown while closed. */
  text: string
  /** The years it accepts. */
  minYear: number
  maxYear: number
  /** The UI language (BCP 47), for the segments' order and the 12 / 24-hour clock. */
  locale: string
  disabled?: boolean
  variant?: DateTimeFieldVariant
  class?: HTMLAttributes['class']
}>()
const model = defineModel<string>({ required: true })

const open = ref(false)
const draft = ref<DateValue | undefined>()
const field = ref<HTMLButtonElement | null>(null)
const minValue = computed(() => new CalendarDateTime(props.minYear, 1, 1, 0, 0))
const maxValue = computed(() => new CalendarDateTime(props.maxYear, 12, 31, 23, 59))

function show() {
  if (props.disabled) return
  draft.value = parseDateTimeText(model.value) ?? minValue.value
  open.value = true
}

/** Closes and writes the value if it changed and is a whole date and time in range. */
function close() {
  open.value = false
  const v = draft.value
  if (v && 'hour' in v && v.compare(minValue.value) >= 0 && v.compare(maxValue.value) <= 0) {
    const text = dateTimeText(v)
    if (text !== model.value) model.value = text
  }
  void nextTick(() => field.value?.focus())
}
</script>

<template>
  <Popover :open="open" @update:open="o => (o ? show() : close())">
    <button
      ref="field"
      v-bind="$attrs"
      type="button"
      data-slot="date-time-field"
      :data-variant="variant"
      :aria-label="label"
      aria-haspopup="dialog"
      :aria-expanded="open"
      :disabled="disabled"
      :class="cn(dateTimeFieldVariants({ variant }), props.class)"
      @click="open ? close() : show()"
    >
      <span class="min-w-0 flex-1 truncate">{{ text }}</span>
      <CalendarClock :size="14" :stroke-width="1.75" aria-hidden="true" />
    </button>
    <PopoverContent :anchor="field" :variant="PopoverVariant.Fit" :aria-label="label">
      <DateFieldRoot
        v-slot="{ segments }"
        v-model="draft"
        granularity="minute"
        :locale="locale"
        :min-value="minValue"
        :max-value="maxValue"
        :aria-label="label"
        class="flex flex-wrap items-center gap-2"
      >
        <template v-for="(item, i) in segments" :key="`${item.part}-${i}`">
          <DateFieldInput
            v-if="item.part === 'literal'"
            :part="item.part"
            :class="dateSegmentVariants({ literal: true })"
          >
            {{ item.value }}
          </DateFieldInput>
          <DateFieldInput
            v-else
            :part="item.part"
            :data-nav-initial="i === segments.findIndex(s => s.part !== 'literal') || undefined"
            :class="dateSegmentVariants({ literal: false })"
            @keydown.enter="close"
          >
            {{ item.value }}
          </DateFieldInput>
        </template>
      </DateFieldRoot>
    </PopoverContent>
  </Popover>
</template>
