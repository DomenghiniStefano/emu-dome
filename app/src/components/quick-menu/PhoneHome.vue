<script setup lang="ts">
// The phone quick menu's first level (owner, 2026-11-22, proposal «A»): «Salva rapido» and
// «Carica rapido» on the quick slot (the controller shortcuts' slot), one tap each, and
// «Riprendi», the sheet's only amber; under them the strip («Riavvolgi» when rewind is on for the
// game, «Accelera», «Silenzia», «Chiudi il gioco» in danger ink, «Altro»). Above them, while there is one, the line of a save made
// here or of a load that can be undone. Upright: one row of three tiles. On its side: «Salva
// rapido» a row, «Carica rapido» growing into the free height with the frame it goes back to, the
// strip, «Riprendi» at the bottom under the right thumb. What each does is QuickMenu.vue's.
import { computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { Play, RotateCcw, Save } from '@lucide/vue'
import type { SlotInfo, StateLoadBlock } from '@/emu'
import { slotTimeLabel, stateLoadBlockText } from '@/i18n/labels'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import { cn } from '@/design-system/lib/cn'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import SavedLine from './SavedLine.vue'
import UndoLoadBanner from './UndoLoadBanner.vue'
import { StripId, stripTwoRows, type StripButton } from './phoneMenu'
import { isEmptySlot } from './slots'
import { FocusMark, QuickMenuView, openerMark } from './view'

const props = defineProps<{
  /** The phone on its side: the side panel's column. */
  side: boolean
  /** The quick slot (Rust's current slot) and what it holds (undefined: not listed). */
  quickSlot: number
  quickInfo: SlotInfo | undefined
  /** Why loading is off for this game (Trello #93): «Carica rapido» disabled, the reason under. */
  loadBlock: StateLoadBlock | null
  strip: StripButton[]
  /** A save made here, while its «Annulla» is offered. */
  saved: { slot: number; thumb: string | null } | null
  /** A load that can be undone («Caricato lo slot 2»), else null. */
  undoLoadText: string | null
}>()
const emit = defineEmits<{
  quickSave: []
  quickLoad: []
  resume: []
  undoSave: []
  undoLoad: []
  strip: [StripId, Event]
  /** «Riavvolgi» pressed (true) and let go (false). */
  rewind: [boolean]
}>()

const { t } = useI18n()

const empty = computed(() => isEmptySlot(props.quickInfo))
const loadOff = computed(() => empty.value || props.loadBlock !== null)
const loadLine = computed(() =>
  empty.value
    ? t('qm.emptySlot', { slot: props.quickSlot })
    : t('qm.slotWhen', {
        slot: props.quickSlot,
        when: slotTimeLabel(props.quickInfo?.savedAt ?? 0),
      }),
)
/** Five on its side: three above, two below (a grid of six columns). */
const twoRows = computed(() => stripTwoRows(props.side, props.strip.length))
const TOP_ROW = 3
/** The menu opens on «Salva rapido» (and comes back to it from «Altro» without an opener). */
const saveMark = { 'data-nav-initial': '', [FocusMark.ChosenSlot]: '' }
const moreMark = openerMark(QuickMenuView.More)

// «Riavvolgi» rewinds while held: a finger or the mouse holds it; the controller's A and Enter
// (a click with no pointer) rewind for a moment.
const TAP_REWIND_MS = 1000
let held = false
let tapTimer: ReturnType<typeof setTimeout> | null = null
function press() {
  if (held) return
  held = true
  emit('rewind', true)
}
function release() {
  if (tapTimer) clearTimeout(tapTimer)
  tapTimer = null
  if (!held) return
  held = false
  emit('rewind', false)
}
function pointerDown(e: PointerEvent) {
  if (e.currentTarget instanceof HTMLElement) e.currentTarget.setPointerCapture?.(e.pointerId)
  press()
}
function stripClick(b: StripButton, e: MouseEvent) {
  if (b.id !== StripId.Rewind) return emit('strip', b.id, e)
  if (e.detail !== 0) return
  press()
  tapTimer = setTimeout(release, TAP_REWIND_MS)
}
onBeforeUnmount(release)
</script>

<template>
  <UndoLoadBanner v-if="undoLoadText" :text="undoLoadText" @undo="emit('undoLoad')" />
  <SavedLine v-else-if="saved" :saved="saved.slot" :thumb="saved.thumb" @undo="emit('undoSave')" />

  <!-- Upright: the row of three. -->
  <div v-if="!side" class="grid grid-cols-3 gap-8">
    <Button
      v-bind="saveMark"
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.QuickTile"
      :icon="Save"
      @click="emit('quickSave')"
    >
      <span class="flex min-w-0 max-w-full flex-col">
        <span class="truncate font-semibold">{{ t('qm.quickSave') }}</span>
        <span class="truncate text-fine text-ink-muted">{{
          t('qm.slot', { slot: quickSlot })
        }}</span>
      </span>
    </Button>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.QuickTile"
      :icon="RotateCcw"
      :disabled="loadOff"
      @click="emit('quickLoad')"
    >
      <span class="flex min-w-0 max-w-full flex-col">
        <span class="truncate font-semibold">{{ t('qm.quickLoad') }}</span>
        <span class="truncate text-fine text-ink-muted tabular-nums">{{ loadLine }}</span>
      </span>
    </Button>
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.QuickTile"
      :icon="Play"
      icon-filled
      @click="emit('resume')"
    >
      <span class="truncate">{{ t('qm.resumeShort') }}</span>
    </Button>
  </div>

  <!-- On its side: a row, a growing tile. -->
  <template v-else>
    <Button
      v-bind="saveMark"
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.QuickRow"
      :icon="Save"
      @click="emit('quickSave')"
    >
      <span class="truncate font-semibold">{{ t('qm.quickSave') }}</span>
      <span class="ml-auto truncate text-fine text-ink-muted">
        {{ t('qm.slot', { slot: quickSlot }) }}
      </span>
    </Button>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.QuickTileGrow"
      :disabled="loadOff"
      @click="emit('quickLoad')"
    >
      <SaveThumb
        :src="quickInfo?.thumbUrl"
        :class="
          cn(
            'aspect-4/3 h-full max-w-[50%] shrink rounded-badge',
            empty && 'border border-dashed border-line-strong bg-surface-raised',
          )
        "
      />
      <span class="flex min-w-0 flex-1 flex-col justify-between">
        <RotateCcw :size="20" :stroke-width="1.75" aria-hidden="true" />
        <span class="flex min-w-0 flex-col">
          <span class="truncate font-semibold">{{ t('qm.quickLoad') }}</span>
          <span class="truncate text-fine text-ink-muted tabular-nums">{{ loadLine }}</span>
          <span v-if="!loadOff" class="truncate text-fine text-ink-faint">
            {{ t('qm.quickLoadHint') }}
          </span>
        </span>
      </span>
    </Button>
  </template>

  <nav :class="cn('flex gap-4', twoRows && 'grid grid-cols-6')" :aria-label="t('qm.toolsTitle')">
    <Button
      v-for="(b, i) in strip"
      :key="b.id"
      v-bind="b.id === StripId.More ? moreMark : {}"
      :variant="b.danger ? ButtonVariant.StripDanger : ButtonVariant.Strip"
      :size="ButtonSize.Strip"
      :icon="b.icon"
      :aria-pressed="b.pressed"
      :aria-label="b.name ? t(b.name) : undefined"
      :class="
        cn(
          b.id === StripId.Rewind && 'touch-none',
          twoRows && (i < TOP_ROW ? 'col-span-2' : 'col-span-3'),
        )
      "
      @pointerdown="b.id === StripId.Rewind ? pointerDown($event) : undefined"
      @pointerup="b.id === StripId.Rewind ? release() : undefined"
      @pointercancel="b.id === StripId.Rewind ? release() : undefined"
      @contextmenu.prevent
      @click="stripClick(b, $event)"
    >
      <span class="max-w-full truncate">{{ t(b.label) }}</span>
    </Button>
  </nav>

  <Button
    v-if="side"
    :variant="ButtonVariant.Primary"
    :size="ButtonSize.MenuMain"
    :icon="Play"
    icon-filled
    @click="emit('resume')"
  >
    {{ t('qm.resumeShort') }}
  </Button>

  <span v-if="loadBlock !== null" class="text-caption text-ink-muted">
    {{ stateLoadBlockText(loadBlock) }}
  </span>
</template>
