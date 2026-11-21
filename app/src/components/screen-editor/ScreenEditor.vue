<script setup lang="ts">
// C09 «Disposizione degli schermi» (Trello #28, option A of the owner's mockup, 2026-11-21): the
// two screens over the game at their real size, in the whole window. Drag a screen to move it,
// its accent corner to resize it; the side panel (EditorPanel) has the numbers and the choices
// and folds into a pill (EditorPill) to leave the window free; it fades while a screen is dragged
// and stands on the side away from the selected screen. The keys are in the pill at the bottom
// (EditorHints).
// Opened from the quick menu's G09 (a game open: every change shows on the game at once, «in
// prova») and from Settings › Schermi di DS e 3DS (no game: the screens are placeholders, saved
// to the page's scope).
// Controller: L1 / R1 pick a screen, the d-pad moves it, the right stick resizes it, Select folds
// the panel, A saves, B cancels. Keyboard: PageUp / PageDown or [ ], the arrows, Shift + arrows,
// Enter, Esc. The state and the steps are useScreenEditor's, the geometry screenEditor.ts's.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ConsoleId, ScreenStatus, SettingsScope } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { useNavScope } from '@/nav'
import EditorHints from './EditorHints.vue'
import EditorPanel from './EditorPanel.vue'
import EditorPill from './EditorPill.vue'
import ScreenFrame from './ScreenFrame.vue'
import { front, other, panelSide, percent, type EditorScreen } from './screenEditor'
import { useScreenEditor } from './useScreenEditor'

const props = defineProps<{
  console: ConsoleId
  /** Settings: the scope it saves to (no game open). Null: the open game, previewed live. */
  scope: SettingsScope | null
  /** The open game's screens (null from Settings). */
  status: ScreenStatus | null
}>()
const emit = defineEmits<{ close: [] }>()

const { t, locale } = useI18n()
const root = ref<HTMLElement | null>(null)
const editor = useScreenEditor({
  console: props.console,
  scope: props.scope,
  status: props.status,
  onError: useErrorSink(),
  close: () => emit('close'),
})
const {
  live,
  layout,
  selected,
  current,
  keep,
  snap,
  collapsed,
  dragging,
  target,
  busy,
  canSave,
  swapped,
} = editor
useNavScope(root, {
  back: () => void editor.cancel(),
  tab: dir => editor.pickNext(dir),
  own: editor.own,
  autofocus: true,
  // Outside the game Esc is a scope's own; in game it reaches the menu as «back» already.
  escape: !live,
})

/** The picture a rectangle holds: the other screen's while they are swapped. */
const holds = (id: EditorScreen) => (swapped ? other(id) : id)
const names = computed<Record<EditorScreen, string>>(() => ({
  top: t(`screenEditor.short.${holds('top')}`),
  bottom: t(`screenEditor.short.${holds('bottom')}`),
}))
const screenName = computed(() => t(`screenEditor.screen.${holds(selected.value)}`))
const frontScreen = computed(() => front(layout.value))
const side = computed(() => panelSide(current.value))
const consoleScope = computed(() =>
  t('settingsUi.scope.console', { console: t(`console.${props.console}`) }),
)
const scopeName = computed(() =>
  props.scope?.level === 'game' ? t('screenEditor.thisGame') : consoleScope.value,
)
const widthText = computed(() =>
  new Intl.NumberFormat(locale.value, { maximumFractionDigits: 1 }).format(
    percent(current.value?.w ?? 0),
  ),
)
/** Out of the way while a screen is dragged. */
const fade = computed(() => (dragging.value ? 'pointer-events-none opacity-0' : ''))
</script>

<template>
  <section
    ref="root"
    class="fixed inset-0 z-dialog overflow-hidden"
    :class="live ? '' : 'bg-letterbox'"
    :aria-label="t('screenEditor.title')"
  >
    <div
      v-if="snap"
      class="pointer-events-none absolute inset-0 bg-editor-grid"
      aria-hidden="true"
    ></div>

    <ScreenFrame
      v-for="s in layout.screens"
      :key="s.id"
      :screen="s"
      :label="names[s.id as EditorScreen]"
      :selected="s.id === selected"
      :placeholder="!live"
      @press="(e, corner) => editor.pointerDown(e, s.id as EditorScreen, corner)"
      @focus="editor.select(s.id as EditorScreen)"
    />

    <div
      v-if="!collapsed"
      class="absolute top-16 transition-opacity duration-fast"
      :class="[side === 'left' ? 'left-16' : 'right-16', fade]"
    >
      <EditorPanel
        v-model:keep="keep"
        v-model:snap="snap"
        v-model:target="target"
        :screen="current"
        :screen-name="screenName"
        :names="names"
        :front="frontScreen"
        :two-screens="layout.screens.length > 1"
        :live="live"
        :console-scope="consoleScope"
        :scope-name="scopeName"
        :busy="busy"
        :can-save="canSave"
        @start="editor.startFrom"
        @field="editor.setField"
        @front="editor.bringToFront"
        @collapse="editor.toggleCollapsed"
        @cancel="editor.cancel"
        @save="editor.save"
      />
    </div>
    <div
      v-else
      class="absolute top-16 left-1/2 -translate-x-1/2 transition-opacity duration-fast"
      :class="fade"
    >
      <EditorPill
        :screen-name="screenName"
        :width="widthText"
        :busy="busy"
        :can-save="canSave"
        @expand="editor.toggleCollapsed"
        @save="editor.save"
      />
    </div>

    <!-- Only words: a screen's corner under it stays reachable. -->
    <div
      class="pointer-events-none absolute bottom-16 left-1/2 -translate-x-1/2 transition-opacity duration-fast"
      :class="fade"
    >
      <EditorHints />
    </div>
  </section>
</template>
