<script setup lang="ts">
// X01 / X02, the search everywhere (mockup «Ricerca e notifiche», approved 2026-11-21): a thick
// glass panel near the top of the window, over a scrim; the field on top, the results grouped
// Azioni / Impostazioni / Giochi, the hints at the bottom. Nothing found: «Nessun risultato per
// «…»» with «Forse cercavi:» from the catalogue's synonyms (search.ts).
//
// Choosing: a setting opens its page with the row lit (`openSetting`, the shell's route); an
// action runs (outside a game only fullscreen can; the game's shortcuts say why not, X05); a game
// opens its details, Ctrl + Enter starts it. Enter in the field chooses the first result that
// can be chosen.
//
// Controller and keyboard: the Dialog's nav scope; the field takes the first focus, ↑ ↓ (the
// d-pad) move between the field and the results, A / Enter choose, B / Esc close. Typing while a
// result has the focus goes on in the field. Before anything is typed, the games played last.
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Search } from '@lucide/vue'
import { emu, type LibraryGame, type SettingKey } from '@/emu'
import { report } from '@/lib/errors'
import { Dialog, DialogContent, DialogSize, DialogTitle } from '@/components/ui/dialog'
import { Input, InputVariant } from '@/components/ui/input'
import { Kbd } from '@/components/ui/kbd'
import SearchResult from './SearchResult.vue'
import { useSearchSources, type SearchItem } from './useSearchSources'

const open = defineModel<boolean>('open', { required: true })
const emit = defineEmits<{
  openSetting: [key: SettingKey]
  details: [game: LibraryGame]
  play: [game: LibraryGame]
}>()
const { t } = useI18n()

const query = ref('')
const sources = useSearchSources()
watch(open, now => {
  if (!now) return
  query.value = ''
  void sources.load()
})

const typed = computed(() => query.value.trim() !== '')
const results = computed(() => sources.resultsFor(query.value))

/** The groups shown, in order; before anything is typed, the games played last. */
const groups = computed(() => {
  if (!typed.value)
    return sources.recentGames.value.length
      ? [{ id: 'games', items: sources.recentGames.value as SearchItem[] }]
      : []
  const r = results.value
  return [
    { id: 'actions', items: r.actions as SearchItem[] },
    { id: 'settings', items: r.settings as SearchItem[] },
    { id: 'games', items: r.games as SearchItem[] },
  ].filter(g => g.items.length > 0)
})
const nothing = computed(() => typed.value && groups.value.length === 0)
const suggestions = computed(() => results.value.suggestions)

function choose(item: SearchItem, start: boolean) {
  switch (item.kind) {
    case 'action':
      if (item.reason !== null) return
      open.value = false
      // Outside a game only the keyboard's fullscreen works (the others carry a reason).
      if (item.id === 'fullscreen') report(emu.toggleFullscreen())
      return
    case 'setting':
      open.value = false
      emit('openSetting', item.entry.key)
      return
    case 'game':
      open.value = false
      if (start) emit('play', item.game)
      else emit('details', item.game)
      return
  }
}

/** Enter in the field: the first result that can be chosen (a suggestion when nothing matched). */
function chooseFirst(start: boolean) {
  const shown = nothing.value ? suggestions.value : groups.value.flatMap(g => g.items)
  const first = shown.find(i => i.kind !== 'action' || i.reason === null)
  if (first) choose(first, start)
}

const field = ref<{ $el: HTMLInputElement } | null>(null)
/** A letter typed while a result has the focus goes on in the field. */
function typeInField(e: KeyboardEvent) {
  const input = field.value?.$el
  if (!input || e.target === input || e.ctrlKey || e.metaKey || e.altKey) return
  if (e.key.length !== 1 || e.key === ' ') return
  e.preventDefault()
  query.value += e.key
  void nextTick(() => input.focus())
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent :size="DialogSize.Palette" :aria-describedby="undefined" @keydown="typeInField">
      <DialogTitle class="sr-only">{{ t('search.title') }}</DialogTitle>
      <div class="flex h-52 flex-none items-center gap-10 border-b border-line px-16">
        <Search
          class="flex-none text-ink-muted"
          :size="20"
          :stroke-width="1.75"
          aria-hidden="true"
        />
        <Input
          ref="field"
          v-model="query"
          :variant="InputVariant.Palette"
          type="search"
          data-nav-initial
          :placeholder="t('search.field')"
          :aria-label="t('search.field')"
          @keydown.enter.exact.stop.prevent="chooseFirst(false)"
          @keydown.enter.ctrl.exact.stop.prevent="chooseFirst(true)"
        />
        <Kbd class="touch:hidden">{{ t('search.key.esc') }}</Kbd>
      </div>

      <div v-if="groups.length || nothing" class="min-h-0 flex-1 overflow-auto px-8 pt-4 pb-8">
        <template v-for="g in groups" :key="g.id">
          <div class="px-8 pt-8 pb-2 text-overline text-ink-muted uppercase">
            {{ t(`search.groups.${g.id}`) }}
          </div>
          <SearchResult
            v-for="item in g.items"
            :key="`${item.kind}:${item.kind === 'game' ? item.game.id : item.kind === 'setting' ? item.entry.key : item.id}`"
            :item="item"
            @choose="start => choose(item, start)"
          />
        </template>

        <!-- X02 -->
        <div v-if="nothing" class="flex flex-col gap-10 px-8 py-12">
          <span class="text-heading">{{ t('search.noResults', { query: query.trim() }) }}</span>
          <template v-if="suggestions.length">
            <span class="text-ink-muted">{{ t('search.didYouMean') }}</span>
            <div class="flex flex-col">
              <SearchResult
                v-for="item in suggestions"
                :key="item.entry.key"
                :item="item"
                @choose="start => choose(item, start)"
              />
            </div>
          </template>
        </div>
      </div>

      <footer
        class="flex flex-none flex-wrap items-center gap-16 border-t border-line px-16 py-10 text-caption text-ink-muted touch:hidden"
      >
        <span class="inline-flex items-center gap-6">
          <Kbd>{{ t('search.key.up') }}</Kbd>
          <Kbd>{{ t('search.key.down') }}</Kbd>
          {{ t('search.hint.choose') }}
        </span>
        <span class="inline-flex items-center gap-6">
          <Kbd>{{ t('search.key.enter') }}</Kbd>
          {{ t('search.hint.open') }}
        </span>
        <span class="inline-flex items-center gap-6">
          <Kbd>{{ t('search.key.esc') }}</Kbd>
          {{ t('search.hint.close') }}
        </span>
        <span class="ml-auto">{{ t('search.hint.controller') }}</span>
      </footer>
    </DialogContent>
  </Dialog>
</template>
