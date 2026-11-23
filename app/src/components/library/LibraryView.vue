<script setup lang="ts">
// The Library section (board L01 · Desktop · Libreria, Libreria2 17/11): «Continua a giocare»
// with up to four games played last («Riprendi» opens one and loads its newest quick save,
// «Salvataggi» shows its saves), then «I tuoi giochi»: the console chips, the order
// (`library.sort`, Global) and the L03 panel, the cover grid. Around it: E01 banners for folders
// that are not reachable (their games stay, faded), the scan progress, L11 «N giochi aggiunti»
// after a scan. A cover's click opens its game details (L04, #34); back from the details focuses
// the cover again (`returnTo`, from the shell's route). No games at all: L02 (LibraryEmpty).
//
// Controller / keyboard: the shell's nav scope covers this view; L03 and «…» open their own.
//
// TV mode (#30): with games, the same library as shelves (tv/TvLibrary.vue), the same open and
// details; without, or until loaded, this view as on the desk.
//
// The phone (#29, owner 2026-11-22, mockup «Libreria sul telefono»): no «Continua a giocare»
// and no heading; its own toolbar (PhoneLibraryToolbar: short console names, the order, Lista /
// Griglia, a square «+»); the games as a list (GameList, the phone's default) or a compact grid
// (CoverGrid `compact`), as `library.view` says for this device. The game played last carries
// «Riprendi» on its row or cover (`resumeTarget`), the others «Gioca» (the list) or nothing (the
// grid). The desktop shows the list too when the user picks it in Settings, with «Continua a
// giocare» kept above it and «Gioca» on every row.
//
// L11 says every scan since it was closed (lib/library.ts `mergeSummary`) and is seen once: it
// closes when this view goes (another section, a game's details, a game; #127).
import { computed, onBeforeUnmount, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { FunnelX, TriangleAlert } from '@lucide/vue'
import CloudDriveShelf from '@/components/cloud/CloudDriveShelf.vue'
import TvLibrary from '@/components/library/tv/TvLibrary.vue'
import { Button } from '@/design-system/components/button'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { uiMode } from '@/composables/useUiMode'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import type { GameId, LibraryGame } from '@/emu'
import { chooseFolder, library, replaceFolder } from '@/lib/library'
import ContinuePlaying from './ContinuePlaying.vue'
import CoverGrid from './CoverGrid.vue'
import GameList from './GameList.vue'
import LibraryToolbar from './LibraryToolbar.vue'
import PhoneLibraryToolbar from './PhoneLibraryToolbar.vue'
import { recentlyPlayed, resumeTarget, unreachableFolders } from './libraryView'
import ScanProgress from './ScanProgress.vue'
import ScanSummary from './ScanSummary.vue'
import UnreachableBanner from './UnreachableBanner.vue'
import { useCoverMenu } from './useCoverMenu'
import { useForegroundGame } from './useForegroundGame'
import { useLibraryFilters } from './useLibraryFilters'
import { useLibraryFocus } from './useLibraryFocus'
import { useLibrarySort } from './useLibrarySort'
import { useGamesView } from './useGamesView'
import { useResumePoints } from './useResumePoints'
import LibraryEmpty from './LibraryEmpty.vue'

const props = defineProps<{
  /** Back from this game's details: its cover takes the first focus (read on mount). */
  returnTo: GameId | null
  /** Settings' «Giochi nascosti» (#112): L03's «Nascosti» switch on (read on mount); in TV
   * mode the desk's grid, whose switch shows them (the shelves leave hidden games out). */
  showHidden: boolean
}>()
const emit = defineEmits<{
  /** `slot`: the quick save to load once the game runs («Riprendi»). */
  open: [game: LibraryGame, slot?: number]
  /** «Salvataggi» on a «Continua a giocare» card: the Saves section on that game. */
  openSaves: [LibraryGame]
  /** L04: the game's details. */
  details: [LibraryGame]
  openFile: []
}>()

const { t } = useI18n()
const toasts = useToasts()
const failWith = useErrorSink()

const root = ref<HTMLElement | null>(null)

const { sort, setSort, resetSort } = useLibrarySort(failWith)
const { show, consoleFilter, chips, games, filtering, toggleShow, clearShow, clearFilters } =
  useLibraryFilters(sort)
const showingHidden = props.showHidden
if (showingHidden) show.hidden = true
/** L03 «Ripristina»: the switches off and the order back to its default. */
async function resetFilters() {
  clearShow()
  await resetSort()
}

const recent = computed(() => recentlyPlayed(library.games))
const resume = useResumePoints(recent, failWith)
const foreground = useForegroundGame(recent, games)
const { menuFor, toggleMenu, closeMenu, setFlags } = useCoverMenu(failWith)

/** The phone (upright or on its side): its toolbar, no «Continua a giocare», «Riprendi» in place. */
const { touchDensity: phone } = useDeviceLayout()
const { view, setView } = useGamesView(() => (phone.value ? 'list' : 'grid'), failWith)
/** The game whose row or cover carries «Riprendi» on the phone. */
const resumeId = computed(() => (phone.value ? resumeTarget(recent.value, resume.value) : null))
// The way back from a game's details is read once, here: the shell's route lets go of it while
// `library.view` is still being read, before the games mount.
const anchor = props.returnTo
useLibraryFocus(root, anchor, () => view.value !== null)

/** Opens a game, or says why not when its folder is not reachable. */
function open(g: LibraryGame, slot?: number) {
  if (!g.available) {
    failWith({ code: 'libraryGameUnavailable', params: { id: g.id } })
    return
  }
  emit('open', g, slot)
}
/** «Riprendi»: from where `useResumePoints` found the newest save. */
const resumeGame = (g: LibraryGame) => open(g, resume.value[g.id]?.slot ?? undefined)

// ---- Folders: L02 / «Aggiungi giochi», E01 ----

async function addFolder() {
  const failed = await chooseFolder()
  if (failed) toasts.fail(failed)
}
async function replace(path: string) {
  const failed = await replaceFolder(path)
  if (failed) toasts.fail(failed)
}
const unreachable = computed(() => unreachableFolders(library.folders, library.games))

// L11 seen: leaving the view closes it, as its close button does.
onBeforeUnmount(() => (library.summary = null))

/** TV mode with games: the shelves. */
const shelves = computed(
  () => uiMode.tv && !showingHidden && library.loaded && library.games.length > 0,
)
</script>

<template>
  <TvLibrary
    v-if="shelves"
    :games="library.games"
    :return-to="returnTo"
    @open="open"
    @details="g => emit('details', g)"
  />
  <!-- In TV mode the shell leaves the library flush (its shelves reach the edge): the desk's
       view keeps its own room. -->
  <div v-else ref="root" class="flex flex-1 flex-col gap-16 tv:p-32">
    <UnreachableBanner
      v-for="f in unreachable"
      :key="f.path"
      :folder="f"
      @replace="replace(f.path)"
    />
    <ScanProgress v-if="library.scanning" :progress="library.progress" />
    <ScanSummary
      v-if="library.summary"
      :summary="library.summary"
      @close="library.summary = null"
    />

    <LibraryEmpty
      v-if="library.loaded && library.games.length === 0"
      :error="library.error"
      @choose-folder="addFolder"
      @open-file="emit('openFile')"
    />

    <template v-else-if="library.loaded">
      <PhoneLibraryToolbar
        v-if="phone && view"
        v-model:console="consoleFilter"
        :view="view"
        :chips="chips"
        :sort="sort"
        :show="show"
        @update:view="setView"
        @sort="setSort"
        @toggle="toggleShow"
        @reset="resetFilters"
        @add-folder="addFolder"
      />
      <template v-else-if="view">
        <ContinuePlaying
          v-if="recent.length"
          :games="recent"
          :resume="resume"
          @resume="resumeGame"
          @open-saves="g => emit('openSaves', g)"
          @foreground="foreground"
        />
        <LibraryToolbar
          v-model:console="consoleFilter"
          :chips="chips"
          :initial-chip="!recent.length"
          :sort="sort"
          :show="show"
          @sort="setSort"
          @toggle="toggleShow"
          @reset="resetFilters"
          @add-folder="addFolder"
        />
      </template>
      <!-- Until `library.view` is read (after the device, both from the settings schema), no
           toolbar and no games: never one layout and then the other. -->
      <GameList
        v-if="view === 'list'"
        :games="games"
        :resume-id="resumeId"
        :anchor="anchor"
        @details="g => emit('details', g)"
        @play="g => open(g)"
        @resume="resumeGame"
        @foreground="foreground"
      />
      <CoverGrid
        v-else-if="view === 'grid'"
        :games="games"
        :menu-for="menuFor"
        :anchor="anchor"
        :compact="phone"
        :resume-id="resumeId"
        @details="g => emit('details', g)"
        @play="g => open(g)"
        @resume="resumeGame"
        @toggle-menu="toggleMenu"
        @close-menu="closeMenu"
        @flags="setFlags"
        @foreground="foreground"
      />
      <div v-if="games.length === 0" class="flex flex-wrap items-center gap-8">
        <p class="m-0 text-ink-muted">{{ t('library.noneShown') }}</p>
        <Button v-if="filtering" :icon="FunnelX" @click="clearFilters">
          {{ t('library.clearFilters') }}
        </Button>
      </div>
      <p v-if="unreachable.length" class="m-0 text-ink-muted">
        <TriangleAlert
          class="flex-none align-icon text-warning"
          :size="14"
          :stroke-width="1.75"
          aria-hidden="true"
        />
        {{ t('library.unreachable.hint') }}
      </p>
    </template>
    <!-- #31: the games only on Google Drive, «Scarica e gioca». -->
    <CloudDriveShelf v-if="library.loaded" @play="g => open(g)" />
  </div>
</template>
