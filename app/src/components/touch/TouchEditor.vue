<script setup lang="ts">
// C06 «Comandi touch» (Trello #29, option A of the owner's mockup, 2026-11-22): the touch
// controls over the game at their real place, in the whole window, as the C09 screens editor.
// Tap a control to pick it (accent ring), drag it to move it (8 px grid), pinch to resize it; the
// glass toolbar at the top (TouchToolbar) has undo / redo, add, size, opacity, behaviour and the
// rest, «Riduci» folds it into a pill (TouchEditorPill) and «Salva» stores the layout on this
// device for the console and the way the phone is held. It fades while a control is dragged.
// Opened from the quick menu's «Controlli» (the game paused behind it, `live`, placed for the
// layout being edited as TouchLayer places it in game: `inset`) and from Settings
// › Controlli touch (no game: a dark ground with the console's screens drawn where the game would
// show them, TouchPreviewScreens, Trello #111).
// Controller: L1 / R1 pick a control, the d-pad moves it, the right stick resizes it, X steps its
// opacity, Select folds the toolbar, A saves, B leaves without saving. Keyboard: PageUp /
// PageDown or [ ], the arrows, Shift + arrows, Ctrl + Z / Ctrl + Y, Enter, Esc. The state and the
// steps are useTouchEditor's, the geometry touchLayout.ts's.
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ConsoleId, ScreenPlacement } from '@/emu'
import type { InsetPx } from '@/components/game/inset'
import EditorHints from '@/components/screen-editor/EditorHints.vue'
import { useImmersive } from '@/composables/useImmersive'
import { useErrorSink } from '@/composables/useToasts'
import { useNavScope } from '@/nav'
import TouchControl from './TouchControl.vue'
import TouchEditorPill from './TouchEditorPill.vue'
import TouchPreviewScreens from './TouchPreviewScreens.vue'
import TouchToolPanel from './TouchToolPanel.vue'
import TouchToolbar from './TouchToolbar.vue'
import { useScreenPreview } from './useScreenPreview'
import { useTouchEditor, type ToolPanel } from './useTouchEditor'

const props = defineProps<{
  console: ConsoleId
  /** A game is open behind (the quick menu); else Settings, over a dark ground. */
  live: boolean
}>()
const emit = defineEmits<{
  close: []
  /** Over a game (`live`): the edges the layout being edited keeps clear of it, as TouchLayer
   * keeps them in game, so the game is placed where it plays (#131); each step sends it again. */
  inset: [InsetPx]
}>()

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
const editor = useTouchEditor({
  console: props.console,
  live: props.live,
  root,
  onError: useErrorSink(),
  close: () => emit('close'),
})
const { draft, selected, current, collapsed, panel, gesturing, busy, prefs, limits } = editor
/** No game behind (Settings): the console's screens where the game would show them. */
const previewScreens = props.live
  ? ref<ScreenPlacement[]>([])
  : useScreenPreview(props.console, editor.area, editor.inset, useErrorSink())
// Over a game: the game itself laid out for the layout being edited (once it is loaded, so the
// game never jumps to the whole window first).
watch(
  [editor.inset, editor.loaded],
  ([inset, loaded]) => {
    if (props.live && loaded) emit('inset', inset)
  },
  { immediate: true },
)
// From Settings: the whole screen, as a game has it (Android's bars hidden, #131), so the
// controls and the screens are where they will be in game.
useImmersive(!props.live)
useNavScope(root, {
  back: editor.discard,
  tab: dir => editor.pickNext(dir),
  own: editor.own,
  autofocus: true,
  // Outside the game Esc is a scope's own; in game it reaches the menu as «back» already.
  escape: !props.live,
})

const name = computed(() =>
  current.value ? t(`touch.editor.control.${current.value.id}`) : t('touch.editor.none'),
)
const where = computed(() =>
  t(`touch.editor.where.${editor.orientation.value}`, {
    console: t(`console.${props.console}`),
  }),
)
function togglePanel(p: ToolPanel) {
  panel.value = panel.value === p ? null : p
}
/** Out of the way while a control is dragged or pinched. */
const fade = computed(() => (gesturing.value ? 'pointer-events-none opacity-0' : ''))
</script>

<template>
  <section
    ref="root"
    class="fixed inset-0 z-dialog touch-none overflow-hidden select-none"
    :class="live ? '' : 'bg-letterbox'"
    :aria-label="t('touch.editor.title')"
    @pointerdown="editor.pointerDown"
  >
    <TouchPreviewScreens v-if="!live" :console="props.console" :placements="previewScreens" />
    <div class="pointer-events-none absolute inset-0 bg-touch-grid" aria-hidden="true"></div>

    <TouchControl
      v-for="c in draft.controls"
      :key="c.id"
      :control="c"
      :skin="prefs.skin"
      :opacity="prefs.opacity"
      :interactive="false"
      :selected="c.id === selected"
      tabindex="0"
      :aria-label="t(`touch.editor.control.${c.id}`)"
      :aria-current="c.id === selected ? 'true' : undefined"
      class="focus-ring-none"
      @focus="editor.select(c.id)"
    />

    <div
      class="absolute inset-x-16 top-16 flex flex-col items-center gap-8 transition-opacity duration-fast"
      :class="fade"
    >
      <template v-if="!collapsed">
        <span class="rounded-full border border-line px-12 py-4 text-caption glass-thick-flat">
          {{ where }}
        </span>
        <TouchToolbar
          class="w-full"
          :can-undo="editor.canUndo.value"
          :can-redo="editor.canRedo.value"
          :picked="!!current"
          :can-remove="!!current && current.id !== 'menu'"
          :can-add="editor.addable.value.length > 0"
          :panel="panel"
          :busy="busy"
          :can-save="editor.canSave.value"
          @undo="editor.undo"
          @redo="editor.redo"
          @panel="togglePanel"
          @remove="editor.removeSelected"
          @reset="editor.resetLayout"
          @discard="editor.discard"
          @collapse="editor.toggleCollapsed"
          @save="editor.save"
        />
        <TouchToolPanel
          v-if="panel"
          :panel="panel"
          :control="current"
          :button="!!current && editor.isButton(current.id)"
          :addable="editor.addable.value"
          :size-min="current ? editor.minSizeOf(current.id) : limits.sizeMin"
          :size-max="limits.sizeMax"
          :opacity-min="limits.opacityMin"
          :opacity-max="limits.opacityMax"
          @add="editor.addControl"
          @size="editor.setSize"
          @opacity="editor.setOpacity"
          @behaviour="editor.setBehaviour"
        />
      </template>
      <TouchEditorPill
        v-else
        :name="name"
        :busy="busy"
        :can-save="editor.canSave.value"
        @expand="editor.toggleCollapsed"
        @save="editor.save"
      />
    </div>

    <!-- Only words: a control under it stays reachable. -->
    <div
      class="pointer-events-none absolute bottom-16 left-1/2 -translate-x-1/2 transition-opacity duration-fast"
      :class="fade"
    >
      <EditorHints :pick="t('touch.editor.hint.pick')" />
    </div>
  </section>
</template>
