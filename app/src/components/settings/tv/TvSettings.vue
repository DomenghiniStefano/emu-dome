<script setup lang="ts">
// The Settings section in TV mode (#30, the owner's option B of 2026-11-22: the desk's structure,
// only larger): the same screen as the desk's (SettingsView holds the schema, the level, the
// values and the writes) with the TV sizes. The pages on the left (TvCategoryList), the page on
// the right: its summary, the level as its first row (TvScopeRow, the desk's scope bar), and its
// sections of large rows (TvSettingRow). Kinds the TV layout does not draw keep the desk's row,
// scaled up (tvSettingRow.ts `tvEdit`).
//
// Controller: the focus moves across the two columns as on the desk; L1 / R1 change the level
// (SettingsView's own `tab`, as on the desk). ◀ ▶ go to the focused row first (`tv-step`, nav.ts
// `useNavSection`'s `own`), which takes them when it changes in place; elsewhere they move the
// focus (from a row that does not take them, left reaches the pages). B from a row goes back to
// the page shown in the list, from the list to the section tabs (the shell).
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SettingKey, SettingsSchema, SettingValue } from '@/emu'
import type { GameFocus } from '@/lib/gameDetails'
import { msg } from '@/lib/settingsModel'
import { focusInitial, useNavSection } from '@/nav'
import SettingRowView from '@/components/settings/SettingRow.vue'
import {
  INTROS,
  OVERVIEW,
  RESULTS,
  changedCount,
  type EditLevel,
  type PageValues,
  type ShownSection,
} from '@/components/settings/settingsPage'
import type { useSettingsScope } from '@/components/settings/useSettingsScope'
import TvCategoryList from './TvCategoryList.vue'
import TvScopeRow from './TvScopeRow.vue'
import TvSettingRow from './TvSettingRow.vue'
import { tvEdit } from './tvSettingRow'

const props = defineProps<{
  schema: SettingsSchema
  page: string
  sections: ShownSection[]
  values: Record<string, PageValues>
  scopeBar: ReturnType<typeof useSettingsScope>
  gameFocus: GameFocus | null
  lockedNote: string
  /** X01: the row the search opened this section for, lit until its animation ends. */
  lit: SettingKey | null
}>()
const emit = defineEmits<{
  pick: [page: string]
  set: [key: SettingKey, value: SettingValue]
  reset: [key: SettingKey]
  openControls: []
  /** The Google account's row: open the Cloud section. */
  openCloud: []
  /** A row the page carries out, by its key (the desk's `choose` / `editScreens`). */
  choose: [key: SettingKey]
  edit: [key: SettingKey]
  unlit: []
  /** A level chosen in the scope row's sheet (the desk's scope bar: `level`). */
  level: [EditLevel]
}>()
const { t } = useI18n()

const scoped = computed(() => props.scopeBar.scoped.value)
const effective = computed(() => props.scopeBar.effective.value)
const scope = computed(() => props.scopeBar.scope.value)
/** The console of the scope being edited (none at «Tutti i giochi»). */
const scopeConsole = computed(() => ('console' in scope.value ? scope.value.console : null))
const levelName = props.scopeBar.levelName
const changedOn = (p: string) => changedCount(props.schema, p, props.values[p], effective.value)

const root = ref<HTMLElement | null>(null)
const list = ref<HTMLElement | null>(null)
const pages = ref<{ $el: HTMLElement } | null>(null)

/** ◀ ▶ to the focused row, which takes them (`preventDefault`) when it changes in place. */
function own(action: string) {
  if (action !== 'left' && action !== 'right') return false
  const row = document.activeElement?.closest('[data-tv-step]')
  if (!row) return false
  const e = new CustomEvent('tv-step', { detail: action === 'left' ? -1 : 1, cancelable: true })
  return !row.dispatchEvent(e)
}
/** B from a row: back to the page shown in the list; from the list the shell takes it. */
function back() {
  const active = document.activeElement
  if (!active || !list.value?.contains(active)) return false
  pages.value?.$el.querySelector<HTMLElement>('[aria-current="page"]')?.focus()
}
useNavSection(root, { own, back })

function pick(p: string) {
  emit('pick', p)
  void nextTick(() => list.value?.scrollTo({ top: 0 }))
}

onMounted(() => {
  // T01 «In primo piano» is the desk's first screen; from the sofa the first page is.
  const first = props.schema.groups[0]?.pages[0]
  if (props.page === OVERVIEW && first) emit('pick', first)
  void nextTick(() => {
    if (root.value) focusInitial(root.value, { unlessMoved: true })
  })
})
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1 gap-tv-row-x px-tv-edge">
    <TvCategoryList
      ref="pages"
      :schema="schema"
      :page="page"
      :changed-on="changedOn"
      @pick="pick"
    />

    <!-- The page, the room around the rows for the focus ring (4 + 4) the scrolling column would
         cut. -->
    <div
      ref="list"
      class="flex min-h-0 min-w-0 flex-1 flex-col gap-tv-shelf-gap overflow-y-auto pb-tv-shelf-gap [scrollbar-width:none]"
    >
      <div class="flex flex-col gap-8 px-tv-row-x pt-8">
        <p class="m-0 text-tv-lead text-ink-muted">{{ t(msg.summary(page)) }}</p>
        <p v-if="INTROS.has(page)" class="m-0 text-tv-hint text-ink-muted">
          {{ t(`settingsUi.intro.${page}`) }}
        </p>
      </div>

      <div class="flex flex-col gap-tv-row-gap p-8">
        <TvScopeRow
          v-if="scoped"
          :levels="scopeBar.levels.value"
          :effective="effective"
          :context-console="scopeBar.contextConsole.value"
          :console-options="scopeBar.consoleOptions.value"
          :game-focus="gameFocus"
          :level-name="levelName"
          :locked-note="lockedNote"
          @level="l => emit('level', l)"
          @pick-console="scopeBar.pickConsole"
          @step="scopeBar.tab"
        />
        <p v-else class="m-0 px-tv-row-x text-tv-hint text-ink-muted">
          {{ t('settingsUi.appWide') }}
        </p>
      </div>

      <section
        v-for="sec in sections"
        :key="`${sec.page}.${sec.id}`"
        class="flex flex-col gap-12"
        :aria-label="sec.id === RESULTS ? t(msg.page(sec.page)) : t(msg.section(sec.page, sec.id))"
      >
        <h2 class="m-0 px-tv-row-x text-tv-overline text-ink-muted uppercase">
          {{ sec.id === RESULTS ? t(msg.page(sec.page)) : t(msg.section(sec.page, sec.id)) }}
        </h2>
        <div class="flex flex-col gap-tv-row-gap p-8">
          <div
            v-for="e in sec.entries"
            :key="e.key"
            class="flex flex-col rounded-card"
            :data-setting-key="e.key"
            :class="lit === e.key ? 'animate-row-highlight' : undefined"
            @animationend="emit('unlit')"
          >
            <TvSettingRow
              v-if="tvEdit(e) !== 'desk'"
              :entry="e"
              :row="values[sec.page]?.[e.key]"
              :schema="schema"
              :level="effective"
              :level-name="levelName"
              :scoped="scoped"
              :console="scopeConsole"
              @set="v => emit('set', e.key, v)"
              @reset="emit('reset', e.key)"
              @open="emit('openControls')"
              @open-cloud="emit('openCloud')"
              @choose="emit('choose', e.key)"
              @edit="emit('edit', e.key)"
            />
            <!-- A kind the TV layout does not draw: the desk's row, scaled up. -->
            <SettingRowView
              v-else
              class="tv:zoom-tv"
              :entry="e"
              :row="values[sec.page]?.[e.key]"
              :schema="schema"
              :level="effective"
              :level-name="levelName"
              :scoped="scoped"
              :console="scopeConsole"
              lock-note
              @set="v => emit('set', e.key, v)"
              @reset="emit('reset', e.key)"
              @open="emit('openControls')"
              @open-cloud="emit('openCloud')"
              @choose="emit('choose', e.key)"
              @edit="emit('edit', e.key)"
            />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
