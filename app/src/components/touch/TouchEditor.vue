<script setup lang="ts">
// C06 «Comandi touch» (Trello #29, option A of the owner's mockup, 2026-11-22): the touch
// controls and the game screens (#134) over the game at their real place, in the whole window,
// as the C09 screens editor. Tap a control or a screen to pick it (accent ring), drag it to move
// it (8 px snap, not drawn; while a screen moves, accent guides show the window's centre lines it
// meets), pinch a control or pull a screen's accent corner to resize it; the glass toolbar at
// the top (TouchToolbar) has undo / redo, add, size, opacity, behaviour and the rest (a screen:
// size and «Mantieni le proporzioni» only), «Riduci» folds it into a pill (TouchEditorPill) and
// «Salva» stores the layout on this device for the console and the way the phone is held. It
// fades while something is dragged. A dashed line marks the system's safe edges (camera, bars).
// Opened from the quick menu's «Controlli» (the game paused behind it, `live`: the game is placed
// as the layout being edited places it, `placement` and `inset` sent as a pair at each step) and
// from Settings › Controlli touch (no game: the theme's flat ground, the screens as placeholders).
// Controller: L1 / R1 pick a screen or a control, the d-pad moves it, the right stick resizes
// it, X steps a control's opacity, Select folds the toolbar, A saves, B closes the open tool
// panel first (size, opacity, behaviour, add; #132), then leaves without saving. Keyboard: PageUp
// / PageDown or [ ], the arrows, Shift + arrows, Ctrl + Z / Ctrl + Y, Enter, Esc. The state and
// the steps are useTouchEditor's, the geometry touchLayout.ts's and editorScreens.ts's.
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ConsoleId, CustomScreen, ScreenId } from '@/emu'
import type { InsetPx } from '@/components/game/inset'
import EditorHints from '@/components/screen-editor/EditorHints.vue'
import ScreenFrame from '@/components/screen-editor/ScreenFrame.vue'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { useImmersive } from '@/composables/useImmersive'
import { useErrorSink } from '@/composables/useToasts'
import { useNavScope } from '@/nav'
import { centreGuides, otherScreen } from './editorScreens'
import TouchControl from './TouchControl.vue'
import TouchEditorCaption from './TouchEditorCaption.vue'
import TouchEditorPill from './TouchEditorPill.vue'
import TouchToolPanel from './TouchToolPanel.vue'
import TouchToolbar from './TouchToolbar.vue'
import { useTouchEditor, type ToolPanel } from './useTouchEditor'

const props = defineProps<{
  console: ConsoleId
  /** A game is open behind (the quick menu); else Settings, over the theme's flat ground. */
  live: boolean
}>()
const emit = defineEmits<{
  close: []
  /** Over a game (`live`): the edges the layout being edited keeps clear of it, as TouchLayer
   * keeps them in game, so the game is placed where it plays (#131); each step sends it again. */
  inset: [InsetPx]
  /** Over a game (`live`): where the layout being edited puts the screens (empty: none of its
   * own), sent before `inset` at each step; `null` once the editor closes. */
  placement: [CustomScreen[] | null]
}>()

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
/** A phone, upright or on its side: the hint to turn it shows only there. */
const { touchDensity } = useDeviceLayout()
const editor = useTouchEditor({
  console: props.console,
  live: props.live,
  root,
  onError: useErrorSink(),
  close: () => emit('close'),
  phone: () => touchDensity.value,
})
const {
  draft,
  selected,
  current,
  currentScreen,
  collapsed,
  panel,
  gesturing,
  busy,
  prefs,
  limits,
} = editor
// Over a game: the game itself laid out for the layout being edited (once it is loaded, so the
// game never jumps to the whole window first): the placement, then the inset it is fitted in.
watch(
  [editor.placement, editor.placementInset, editor.loaded],
  ([placement, inset, loaded]) => {
    if (!props.live || !loaded) return
    emit('placement', placement)
    emit('inset', inset)
  },
  { immediate: true },
)
// Closed: the touch layer's placement comes back.
onBeforeUnmount(() => {
  if (props.live) emit('placement', null)
})
// From Settings: the whole screen, as a game has it (Android's bars hidden, #131), so the
// controls and the screens are where they will be in game.
useImmersive(!props.live)
const toolbar = ref<{ $el: HTMLElement } | null>(null)
/**
 * Back (B, Esc, Android's back): an open tool panel closes first, the focus back on its button
 * in the toolbar (or on «…», when the tool lives in its menu) if it was in the panel; then the
 * editor is left without saving.
 */
function back() {
  if (!panel.value) return editor.discard()
  const bar = toolbar.value?.$el
  const opener =
    bar?.querySelector<HTMLElement>('[aria-pressed="true"]') ??
    bar?.querySelector<HTMLElement>('[aria-expanded]')
  panel.value = null
  void nextTick(() => {
    const active = document.activeElement
    if (!active || active === document.body || !active.isConnected) opener?.focus()
  })
}
useNavScope(root, {
  back,
  tab: dir => editor.pickNext(dir),
  own: editor.own,
  autofocus: true,
  // Outside the game Esc is a scope's own; in game it reaches the menu as «back» already.
  escape: !props.live,
})

/** The picture a frame holds: the other screen's while the game shows them swapped. */
function pictureOf(id: ScreenId): ScreenId {
  return editor.swapped.value ? otherScreen(id) : id
}
const screenName = (id: ScreenId) => t(`touch.editor.screen.${pictureOf(id)}`)
const name = computed(() => {
  if (currentScreen.value) return screenName(currentScreen.value.id)
  return current.value ? t(`touch.editor.control.${current.value.id}`) : t('touch.editor.none')
})
const where = computed(() =>
  t(`touch.editor.where.${editor.orientation.value}`, {
    console: t(`console.${props.console}`),
  }),
)
/** The window's centre lines the screen being dragged meets. */
const guides = computed(() =>
  gesturing.value && currentScreen.value
    ? centreGuides(currentScreen.value, editor.area.value)
    : { across: false, down: false },
)
function togglePanel(p: ToolPanel) {
  panel.value = panel.value === p ? null : p
}
/** Out of the way while a control or a screen is dragged or pinched. */
const fade = computed(() => (gesturing.value ? 'pointer-events-none opacity-0' : ''))
</script>

<template>
  <section
    ref="root"
    class="fixed inset-0 z-dialog touch-none overflow-hidden select-none"
    :class="live ? '' : 'bg-bg'"
    :aria-label="t('touch.editor.title')"
    @pointerdown="editor.pointerDown"
  >
    <!-- The system's safe edges (camera, bars): a screen may go past them, the line says where. -->
    <div
      class="pointer-events-none absolute top-safe-top right-safe-right bottom-safe-bottom left-safe-left border border-dashed border-line-strong"
      aria-hidden="true"
    ></div>

    <ScreenFrame
      v-for="s in editor.shownScreens.value"
      :key="s.id"
      :screen="s"
      :label="screenName(s.id)"
      :selected="s.id === selected"
      :placeholder="!live"
      @press="(e, corner) => editor.pressScreen(s.id, e, corner)"
      @focus="editor.select(s.id)"
    />

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
      v-if="guides.across"
      class="pointer-events-none absolute inset-y-0 left-1/2 border-l border-accent"
      aria-hidden="true"
    ></div>
    <div
      v-if="guides.down"
      class="pointer-events-none absolute inset-x-0 top-1/2 border-t border-accent"
      aria-hidden="true"
    ></div>

    <div
      class="absolute inset-x-16 top-16 flex flex-col items-center gap-8 transition-opacity duration-fast"
      :class="fade"
    >
      <template v-if="!collapsed">
        <TouchEditorCaption>{{ where }}</TouchEditorCaption>
        <!-- Until the other orientation was shown: its layout is saved with this one. -->
        <TouchEditorCaption v-if="editor.turnHint.value">
          {{ t(`touch.editor.turnHint.${editor.turnHint.value}`) }}
        </TouchEditorCaption>
        <TouchToolbar
          ref="toolbar"
          class="w-full"
          :can-undo="editor.canUndo.value"
          :can-redo="editor.canRedo.value"
          :picked="!!current || !!currentScreen"
          :screen-picked="!!currentScreen"
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
          :screen="currentScreen"
          :screen-name="currentScreen ? screenName(currentScreen.id) : undefined"
          :keep="editor.keep.value"
          @add="editor.addControl"
          @size="editor.setSize"
          @opacity="editor.setOpacity"
          @behaviour="editor.setBehaviour"
          @screen-size="editor.setScreenSize"
          @keep="editor.setKeep"
        />
        <!-- Why opacity, behaviour and remove are off for a screen (X05). -->
        <TouchEditorCaption v-else-if="currentScreen">
          {{ t('touch.editor.screenOnly') }}
        </TouchEditorCaption>
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
