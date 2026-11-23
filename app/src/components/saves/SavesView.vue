<script setup lang="ts">
// The Saves section (board «Salvataggi», V01–V05; «Errori» E03): on the left the games that have
// saves (search, «Importa…»), on the right the chosen game (V02: in-game save, quick saves,
// backups with «Ripristina») or the import (V05). A quick save opens its details (V03).
// No saves at all: E03. The in-game save set aside as unreadable (E04) shows as a note over the
// backups. V06 (two versions) waits for Google Drive sync: saves/ConflictTwoUp.vue is ready for it.
//
// Saves are named by each library game's `GameId`: games opened from a file outside the library
// have saves too, but are not listed here (the list is the library's).
//
// The parts and their logic live in saves/: this view loads, decides what is shown and passes
// the events on. Controller / keyboard: the shell's nav scope covers this view; the dialogs open
// their own.
//
// The phone, upright or on its side (#127): one column, as Settings does upright: the list of
// games first; picking one (or «Importa…») opens its panel over the list, with ‹ back to it
// (B / Android's back too). Opened for a game (`arrival`), the panel shows first.
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft } from '@lucide/vue'
import { emu, type GameId, type GameSaves, type LibraryGame } from '@/emu'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { library } from '@/lib/library'
import QuickSaveDialog from './QuickSaveDialog.vue'
import { loadGameSaves } from '@/components/details/gameSaves'
import PickGameDialog from './PickGameDialog.vue'
import SavesEmpty from './SavesEmpty.vue'
import SavesGameList from './SavesGameList.vue'
import SavesGamePanel from './SavesGamePanel.vue'
import SavesImport from './SavesImport.vue'
import { emptyTextKey, filterByTitle, sortByTitle } from './savesModel'
import { useFocusWhenLoaded } from './useFocusWhenLoaded'
import { useQuickSaveDetails } from './useQuickSaveDetails'
import { useSaveImport } from './useSaveImport'
import { useSavesOverview } from './useSavesOverview'
import { useSaveShortcut } from './useSaveShortcut'
import { useSavesSettings } from './useSavesSettings'
import { SectionPage } from '@/design-system/components/section-page'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { focusInitial, useNavSection } from '@/nav'

const props = defineProps<{
  /**
   * Opened for one game (the shell's route, read on mount): «Salvataggi» on a «Continua a
   * giocare» card (L01), «Mostra i backup» (L05) show it; `importing`, «Importa…» (L05), opens
   * its import (V05); Settings' `saves.import` (#112) the import with no game (`game` null).
   */
  arrival: { game: GameId | null; importing: boolean } | null
}>()
const emit = defineEmits<{
  openGame: [game: LibraryGame, slot: number]
  /** «Dettagli del gioco» (L04). */
  openDetails: [LibraryGame]
  openLibrary: []
}>()

const { t } = useI18n()
const toasts = useToasts()
const fail = useErrorSink()

const settings = useSavesSettings(fail)
const shortcut = useSaveShortcut(fail)
const overview = useSavesOverview(settings.slotCount, fail)
const { loaded, withSaves } = overview

// ---- V01: the list and its search ----

const query = ref('')
const rows = computed(() =>
  filterByTitle(withSaves.value, query.value).map(game => ({
    game,
    detail: overview.detailOf(game),
    broken: overview.isBroken(game),
  })),
)

// ---- V02: one game ----

const mode = ref<'game' | 'import'>('game')
// Read once: the shell drops the arrival after this section has mounted.
const arrival = props.arrival
const selectedId = ref<GameId | null>(arrival?.game ?? null)
const selected = computed(
  () => withSaves.value.find(g => g.id === selectedId.value) ?? withSaves.value[0] ?? null,
)
const details = ref<GameSaves | null>(null)

async function loadDetails() {
  const g = selected.value
  if (!g) {
    details.value = null
    return
  }
  try {
    const saves = await loadGameSaves(g, settings.slotCount.value)
    // Rule 7: another game chosen meanwhile, its own load shows.
    if (selected.value?.id === g.id) details.value = saves
  } catch (e) {
    if (selected.value?.id === g.id) fail(e)
  }
}
watch(selected, () => void loadDetails())

async function refresh() {
  await overview.load()
  await loadDetails()
}

// ---- The phone: one column ----

const layout = useDeviceLayout()
const oneColumn = layout.touchDensity
/** One column: the list shows, not the panel. Opened for a game, its panel first. */
const listOpen = ref(arrival === null)
const listShown = computed(() => !oneColumn.value || listOpen.value)
const panelShown = computed(() => !oneColumn.value || !listOpen.value)

/** One column: the panel over the list, its first control focused. */
function openPanel() {
  listOpen.value = false
  void nextTick(() => {
    if (oneColumn.value && root.value) focusInitial(root.value)
  })
}
/** One column: back to the list, on the game that was shown. */
function showList() {
  listOpen.value = true
  void nextTick(() =>
    root.value?.querySelector<HTMLElement>('aside [aria-current="true"], aside button')?.focus(),
  )
}

function pick(g: LibraryGame) {
  selectedId.value = g.id
  mode.value = 'game'
  openPanel()
}

/** «Annulla» of the import: the game shown before, or the list in one column. */
function cancelImport() {
  mode.value = 'game'
  if (oneColumn.value) showList()
}

/** «Gioca da qui»: the chosen game, on that slot. */
function playSlot(slot: number) {
  const g = selected.value
  if (g) emit('openGame', g, slot)
}

async function exportInGame() {
  const g = selected.value
  if (!g) return
  try {
    if (await emu.saves.export(g.id, { kind: 'inGame' })) toasts.notify(t('saves.exported'))
  } catch (e) {
    fail(e)
  }
}

async function restore(name: string) {
  const g = selected.value
  if (!g) return
  try {
    await emu.saves.restore(g.id, name)
    toasts.notify(t('saves.restored'))
  } catch (e) {
    fail(e)
  }
  await refresh()
}

// ---- V03: a quick save's details ----

const quickDetails = useQuickSaveDetails()
const { quick } = quickDetails

function playFromDetails(slot: number) {
  quick.value = null
  playSlot(slot)
}

// ---- V05: import ----

const importer = useSaveImport(settings.slotCount, fail, async g => {
  toasts.notify(t('saves.imported'))
  mode.value = 'game'
  selectedId.value = g.id
  await refresh()
})
const { candidate, target, targetFound, targetSaves, choosingGame, importing } = importer
const gamesByTitle = computed(() => sortByTitle(library.games))

function startImport() {
  mode.value = 'import'
  importer.reset()
  openPanel()
}

// ---- E03 ----

const emptyText = computed(() =>
  t(emptyTextKey(settings.autoq.value), { combo: shortcut.text.value }),
)

// ---- Loading ----

const root = ref<HTMLElement | null>(null)
useFocusWhenLoaded(root, loaded)
// One column on a panel: B / Android's back return to the list; anywhere else the shell's back.
useNavSection(root, { back: () => (oneColumn.value && !listOpen.value ? showList() : false) })
watch(
  () => library.games,
  () => void refresh(),
)

onMounted(async () => {
  await settings.load()
  if (arrival?.importing) {
    mode.value = 'import'
    if (arrival.game) await importer.resetFor(arrival.game)
    else importer.reset()
  }
  await refresh()
  void shortcut.load()
})
useSettingsChanged(['saves.slots', 'saves.autoq'], () => void settings.load().then(() => refresh()))
useSettingsChanged(['pad.combos', 'pad.mod'], () => void shortcut.load())
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1">
    <!-- E03: nothing saved yet. -->
    <SavesEmpty
      v-if="loaded && withSaves.length === 0 && mode === 'game'"
      :text="emptyText"
      @open-library="emit('openLibrary')"
      @import="startImport"
    />

    <template v-else-if="loaded">
      <SavesGameList
        v-if="listShown"
        v-model:query="query"
        :rows="rows"
        :current="mode === 'game' ? (selected?.id ?? null) : null"
        :full="oneColumn"
        @pick="pick"
        @import="startImport"
      />

      <SectionPage v-if="panelShown" as="main">
        <!-- One column: the way back to the list, at the panel's start. -->
        <div v-if="oneColumn" class="flex">
          <Button
            :variant="ButtonVariant.Ghost"
            :size="ButtonSize.TextLine"
            class="-ml-6"
            :icon="ChevronLeft"
            data-nav-initial
            @click="showList"
          >
            {{ t('shell.saves') }}
          </Button>
        </div>
        <SavesImport
          v-if="mode === 'import'"
          :candidate="candidate"
          :target="target"
          :target-found="targetFound"
          :target-saves="targetSaves"
          :importing="importing"
          @pick-file="importer.pickFile"
          @choose-game="choosingGame = true"
          @cancel="cancelImport"
          @import="importer.run"
        />

        <SavesGamePanel
          v-else-if="selected && details"
          :game="selected"
          :saves="details"
          :detail="overview.detailOf(selected)"
          @open-details="emit('openDetails', selected)"
          @export-in-game="exportInGame"
          @play="playSlot"
          @more="quickDetails.open"
          @restore="restore"
        />
      </SectionPage>
    </template>

    <!-- V03: a quick save -->
    <QuickSaveDialog
      v-if="quick && selected"
      :game="selected"
      :quick="quick"
      @close="quickDetails.close"
      @play="playFromDetails"
      @changed="refresh"
    />

    <!-- V05: «Scegli un altro gioco…» -->
    <PickGameDialog
      v-if="choosingGame"
      :games="gamesByTitle"
      :chosen="target"
      @choose="importer.chooseTarget"
      @cancel="choosingGame = false"
    />
  </div>
</template>
