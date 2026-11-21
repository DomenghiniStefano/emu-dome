<script setup lang="ts">
// The Library section (board L01 · Desktop · Libreria, Libreria2 17/11): «Continua a giocare»
// with up to four games played last («Riprendi» opens one and loads its newest quick save,
// «Salvataggi» shows its saves), then «I tuoi giochi»: the console chips, the order
// (`library.sort`, Global) and the L03 panel, the cover grid. Around it: E01 banners for folders
// that are not reachable (their games stay, faded), the scan progress, L11 «N giochi aggiunti»
// after a scan. A cover's click opens its game details (L04, #34); back from the details focuses
// the cover again (`detailsReturn`). No games at all: L02 (LibraryEmpty).
//
// Controller / keyboard: the shell's nav scope covers this view; L03 and «…» open their own.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { TriangleAlert } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import type { LibraryGame } from '@/emu'
import { chooseFolder, library, replaceFolder } from '@/lib/library'
import ContinuePlaying from './ContinuePlaying.vue'
import CoverGrid from './CoverGrid.vue'
import LibraryToolbar from './LibraryToolbar.vue'
import { recentlyPlayed, unreachableFolders } from './libraryView'
import ScanProgress from './ScanProgress.vue'
import ScanSummary from './ScanSummary.vue'
import UnreachableBanner from './UnreachableBanner.vue'
import { useCoverMenu } from './useCoverMenu'
import { useForegroundGame } from './useForegroundGame'
import { useLibraryFilters } from './useLibraryFilters'
import { useLibraryFocus } from './useLibraryFocus'
import { useLibrarySort } from './useLibrarySort'
import { useResumePoints } from './useResumePoints'
import LibraryEmpty from './LibraryEmpty.vue'

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
useLibraryFocus(root)

const { sort, setSort, resetSort } = useLibrarySort(failWith)
const { show, consoleFilter, chips, games, filtering, toggleShow, clearShow, clearFilters } =
  useLibraryFilters(sort)
/** L03 «Ripristina»: the switches off and the order back to its default. */
async function resetFilters() {
  clearShow()
  await resetSort()
}

const recent = computed(() => recentlyPlayed(library.games))
const resume = useResumePoints(recent, failWith)
const foreground = useForegroundGame(recent, games)
const { menuFor, toggleMenu, closeMenu, setFlags } = useCoverMenu(failWith)

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
</script>

<template>
  <div ref="root" class="flex flex-1 flex-col gap-24">
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
      <CoverGrid
        :games="games"
        :menu-for="menuFor"
        @details="g => emit('details', g)"
        @play="g => open(g)"
        @toggle-menu="toggleMenu"
        @close-menu="closeMenu"
        @flags="setFlags"
        @foreground="foreground"
      />
      <div v-if="games.length === 0" class="flex flex-wrap items-center gap-12">
        <p class="m-0 text-ink-muted">{{ t('library.noneShown') }}</p>
        <Button v-if="filtering" @click="clearFilters">{{ t('library.clearFilters') }}</Button>
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
  </div>
</template>
