<script setup lang="ts">
// The touch controls over the game (Trello #29, C05 / C08): glass controls (the Pad variants, the
// skin of `touch.tskin`) where the layout of this console and orientation puts them, the default
// or the user's own (`touch.tport` / `touch.tland`, C06), faded by `touch.topac` and each
// control's opacity. Each control keeps its own pointer, so several are held at once; the layer
// lets touches through except on the controls.
// - Hidden under the quick menu, and while a controller is connected with `touch.tpad`; faded
//   after `touch.thide` without a touch (still pressable, so the touch that wakes them counts).
// - The layout keeps its part of the window clear of the game (`inset`, App.vue's
//   `session.setInset`, as the quick menu does): upright the band at the bottom, the game right
//   above it; on its side the two columns, the game between them (#29).
// - E14: the last controller gone while playing (ControllerLostDialog, useControllerLoss).
// App.vue mounts it on a touch screen while a game is open.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ConsoleId, TouchControl as Control } from '@/emu'
import { flow } from '@/components/game/gameFlow'
import { NO_INSET, type InsetPx } from '@/components/game/inset'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import ControllerLostDialog from './ControllerLostDialog.vue'
import TouchControl from './TouchControl.vue'
import { gameButton } from './touchControls'
import { activeLayout, isDualScreen, orientationOf, reservedInset } from './touchLayout'
import { hideAfterMs, touchShown } from './touchVisibility'
import { useControllerLoss } from './useControllerLoss'
import { useTouchPlay } from './useTouchPlay'
import { useTouchSetup } from './useTouchSetup'
import { useArea, useControlBoxes, useIdleFade } from './useTouchView'

const props = defineProps<{ console: ConsoleId; menuOpen: boolean }>()
const emit = defineEmits<{
  menu: []
  /** The edges the controls keep clear of the game, CSS px (none while hidden). */
  inset: [InsetPx]
  /** E14 paused the game, or let it go on. */
  paused: [boolean]
}>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
const { setup, prefs } = useTouchSetup(() => props.console)
const area = useArea()
const orientation = computed(() => orientationOf(area.value))
const layout = computed(() =>
  setup.value
    ? activeLayout(
        props.console,
        setup.value,
        orientation.value,
        prefs.value[orientation.value],
        area.value,
      )
    : { controls: [] },
)
const { boxes, boxOf } = useControlBoxes(root, () => layout.value)

const device = useDeviceLayout().device
/** Something of the game flow holds the game (a question, the game closing). */
const busy = () =>
  flow.ending ||
  flow.resumeAsk ||
  !!flow.closing ||
  !!flow.crashed ||
  !!flow.startError ||
  !!flow.saveBroken
const loss = useControllerLoss({
  device: () => device.value,
  menuOpen: () => props.menuOpen,
  busy,
  choice: () => prefs.value.lost,
  paused: on => emit('paused', on),
})
onMounted(() => void loss.listen())

const shown = computed(() =>
  touchShown({
    touch: true,
    inGame: true,
    menuOpen: props.menuOpen,
    editing: false,
    hideWithPad: prefs.value.hideWithPad,
    controllers: loss.pads.value.length,
  }),
)
const faded = useIdleFade(() => hideAfterMs(prefs.value.hide))
const play = useTouchPlay(() => prefs.value.haptics)

const behaviour = (c: Control) => c.behaviour
const latched = (c: Control) => {
  const b = gameButton(c.id)
  return b !== null && play.latched.value.has(b)
}

// Nothing stays pressed when the controls go away (the menu, a controller, the game closing).
watch(shown, on => on || play.releaseAll())

// The controls keep their part of the window; the quick menu keeps its own while open.
watch(
  [shown, () => props.menuOpen, layout, area, boxes],
  () => {
    if (props.menuOpen) return
    emit(
      'inset',
      shown.value
        ? reservedInset(layout.value, area.value, boxOf, isDualScreen(props.console))
        : NO_INSET,
    )
  },
  { flush: 'post', immediate: true },
)
onBeforeUnmount(() => {
  play.releaseAll()
  emit('inset', NO_INSET)
})
</script>

<template>
  <div
    ref="root"
    class="pointer-events-none fixed inset-0 touch-none select-none"
    :aria-label="t('touch.label')"
  >
    <div
      v-if="shown"
      class="absolute inset-0 transition-opacity duration-scene"
      :class="faded ? 'opacity-0' : ''"
    >
      <TouchControl
        v-for="c in layout.controls"
        :key="c.id"
        :control="c"
        :skin="prefs.skin"
        :opacity="prefs.opacity"
        :latched="latched(c)"
        interactive
        @down="b => play.down(b, behaviour(c))"
        @up="b => play.up(b, behaviour(c))"
        @dirs="play.setDirs"
        @stick="play.stick"
        @menu="emit('menu')"
      />
    </div>
    <ControllerLostDialog
      v-if="loss.asking.value"
      @wait="loss.wait"
      @touch="loss.continueWithTouch"
    />
  </div>
</template>
