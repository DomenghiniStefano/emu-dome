<script setup lang="ts">
// Game details (#34, boards «Dettagli» L04–L10 desktop and «Tel-Dettagli» phone portrait), inside
// the Library section. L04 on top (DetailsHero); under it six tabs: L05 Salvataggi, L06
// Impostazioni del gioco, L07 Controlli, L08 Codici trucco and L09 Patch e traduzioni (not built
// yet: the reason, X05), L10 File e informazioni. «…» offers L12 «Scegli la copertina…», which is
// reachable from here only (Cowork #40), as does «Cambia…» in L10.
// Phone in portrait (device phone, as the settings schema tells it): the tabs as scrolling pills.
//
// Controller / keyboard: the shell's nav scope covers it; L1 / R1 change tab (exposed `tab`);
// back (B, Esc) returns to the library on this game's cover (the shell does it).
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type GameId, type LibraryGame } from '../emu'
import { errorText } from '../i18n/labels'
import { library } from '../library'
import { Tabs, TabsList, TabsSize, TabsTrigger } from '@/components/ui/tabs'
import CoverPicker from './CoverPicker.vue'
import DetailsControls from './DetailsControls.vue'
import DetailsFiles from './DetailsFiles.vue'
import DetailsSaves from './DetailsSaves.vue'
import DetailsSettings from './DetailsSettings.vue'
import DetailsHero from './details/DetailsHero.vue'
import NotBuiltPanel from './details/NotBuiltPanel.vue'
import { DETAILS_TABS, isDetailsTab, stepTab, tabLabelKey, type DetailsTab } from './details/tabs'
import { useEscapeBack } from './details/useEscapeBack'
import { usePhoneLayout } from './details/usePhoneLayout'
import { useResumePoint } from './details/useResumePoint'

const props = defineProps<{ id: GameId }>()
const emit = defineEmits<{
  back: []
  /** `slot`: the quick save to load once the game runs. */
  play: [game: LibraryGame, slot?: number]
  /** L05 «Mostra i backup»: the Saves section on this game. */
  backups: [LibraryGame]
  /** L05 «Importa…»: the Saves section's import, for this game. */
  import: [LibraryGame]
  /** L06: the Settings section at «Solo questo gioco». */
  settings: [LibraryGame]
  /** L07: button remapping (C02) at «Solo questo gioco». */
  remap: [LibraryGame]
  error: [string]
  notice: [string]
}>()

const { t } = useI18n()

const game = computed(() => library.games.find(g => g.id === props.id) ?? null)
// The game left the library (its folder removed): back to the grid.
watch(game, g => {
  if (!g && library.loaded) emit('back')
})

const { device, phone, loadDevice } = usePhoneLayout()
const { resume, reload: reloadResume } = useResumePoint(game)

// ---- L04: «Riprendi» / «Gioca», the favourite, «…» ----
function play() {
  const g = game.value
  if (!g) return
  if (!g.available) {
    emit('error', errorText({ code: 'libraryGameUnavailable', params: { id: g.id } }))
    return
  }
  emit('play', g, resume.value?.slot ?? undefined)
}

async function setFlags(flags: { favourite?: boolean; hidden?: boolean }) {
  const g = game.value
  if (!g) return
  try {
    await emu.library.setFlags(g.id, flags)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// L12, from «…» or from L10 «Cambia…»; closing it puts the focus on «…».
const hero = ref<InstanceType<typeof DetailsHero> | null>(null)
const pickingCover = ref(false)
function closeCoverPicker() {
  pickingCover.value = false
  void nextTick(() => hero.value?.focusMore())
}

// ---- Tabs ----
const current = ref<DetailsTab>('saves')
function selectTab(value: unknown) {
  if (isDetailsTab(value)) current.value = value
}
/** L1 / R1: the previous / next tab; the focus follows when it was on the tabs. */
function step(dir: -1 | 1) {
  current.value = stepTab(current.value, dir)
  void nextTick(() => {
    const list = root.value?.querySelector<HTMLElement>('[role="tablist"]')
    if (!list) return
    const on = list.querySelector<HTMLElement>('[aria-selected="true"]')
    // Sideways the default is already the nearest edge, which scrolls the tab row itself.
    on?.scrollIntoView({ block: 'nearest' })
    if (list.contains(document.activeElement)) on?.focus()
  })
}
defineExpose({ tab: step })

// ---- Mount: the focus on «Riprendi» (or the way back), Esc goes back ----
const root = ref<HTMLElement | null>(null)
useEscapeBack(root, () => emit('back'))
onMounted(async () => {
  await nextTick()
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
  await loadDevice()
  await reloadResume()
})
</script>

<template>
  <div
    v-if="game"
    ref="root"
    class="flex min-h-0 flex-1 flex-col overflow-y-auto"
    :class="phone && 'text-lead'"
  >
    <DetailsHero
      ref="hero"
      :game="game"
      :phone="phone"
      :resume="resume"
      @back="emit('back')"
      @play="play"
      @cover="pickingCover = true"
      @flags="setFlags"
    />

    <!-- `contents`: the tab row stays an item of this column, as the panel does. -->
    <Tabs
      class="contents"
      :model-value="current"
      :size="phone ? TabsSize.Pill : TabsSize.Large"
      @update:model-value="selectTab"
    >
      <TabsList
        class="overflow-x-auto"
        :class="phone ? 'gap-8 border-b-0 px-16 pt-16' : 'px-32'"
        :aria-label="t('details.tabsLabel')"
      >
        <TabsTrigger v-for="tab in DETAILS_TABS" :key="tab" :value="tab">
          {{ t(tabLabelKey(tab, phone)) }}
        </TabsTrigger>
      </TabsList>
    </Tabs>

    <div
      class="flex max-w-[1040px] flex-col"
      :class="phone ? 'gap-16 p-16' : 'gap-20 px-32 pt-24 pb-48'"
      role="tabpanel"
      :aria-label="t(tabLabelKey(current, phone))"
    >
      <DetailsSaves
        v-if="current === 'saves'"
        :game="game"
        :phone="phone"
        @play="slot => emit('play', game!, slot)"
        @backups="emit('backups', game!)"
        @import="emit('import', game!)"
        @changed="reloadResume"
        @error="text => emit('error', text)"
        @notice="text => emit('notice', text)"
      />
      <DetailsSettings
        v-else-if="current === 'settings'"
        :game="game"
        :phone="phone"
        @open="emit('settings', game!)"
        @error="text => emit('error', text)"
      />
      <DetailsControls
        v-else-if="current === 'controls'"
        :game="game"
        :phone="phone"
        @remap="emit('remap', game!)"
        @error="text => emit('error', text)"
      />
      <DetailsFiles
        v-else-if="current === 'files'"
        :game="game"
        :phone="phone"
        :device="device"
        @cover="pickingCover = true"
        @error="text => emit('error', text)"
      />
      <NotBuiltPanel v-else :tab="current" />
    </div>

    <CoverPicker v-if="pickingCover" :game="game" @close="closeCoverPicker" />
  </div>
</template>
