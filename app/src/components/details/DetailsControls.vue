<script setup lang="ts">
// Game details › Controlli (L07, board «Dettagli» desktop): cards for this game's controls.
// - Controller e tastiera: whether `pad.remap` has a value of its own at «Solo questo gioco», and
//   «Rimappa i pulsanti per questo gioco» (C02 at the Game level, `controlsFocus`).
// - Pressione ripetuta (turbo): not built yet (`pad.turbo` has no backend), X05 with the reason.
// - Controlli touch (telefono): customised on the phone (Phase 4).
// - Tocco sullo schermo inferiore: Nintendo DS and 3DS games only, `pad.pointer` for this game.
// Two per row, an odd last card the whole row; phone in portrait: one per row.
//
// Controller / keyboard: the shell's nav scope.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame } from '@/emu'
import { settingsUnder, useSettingsChanged } from '@/composables/useSettingsChanged'
import { gameScopeOf } from '@/lib/gameDetails'
import { useErrorSink } from '@/composables/useToasts'
import DetailsAction from './DetailsAction.vue'
import DetailsCard from './DetailsCard.vue'
import { hasOwnRemap, hasTouchScreen, pointerLabel, type PadRows } from './gameControls'
import { Heading, HeadingVariant } from '@/components/ui/heading'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{
  /** «Rimappa i pulsanti per questo gioco». */
  remap: []
}>()

const { t, te } = useI18n()
const failWith = useErrorSink()

const scope = computed(() => gameScopeOf(props.game))
const consoleName = computed(() => t(`console.${props.game.console}`))

const rows = ref<PadRows>({})
async function load() {
  const s = scope.value
  if (!s) return
  try {
    const page = await emu.settings.page('pad', s)
    // Rule 7: the details show another game now; its own load shows.
    if (scope.value.game !== s.game) return
    rows.value = Object.fromEntries(page.map(r => [r.key, r]))
  } catch (e) {
    if (scope.value.game === s.game) failWith(e)
  }
}
watch(scope, () => void load())

const ownRemap = computed(() => hasOwnRemap(rows.value))
const pointer = computed(() => pointerLabel(rows.value, { t, te }))

onMounted(() => void load())
useSettingsChanged(settingsUnder('pad.'), () => void load())
</script>

<template>
  <!-- A card of this tab: the odd one out at the end takes the whole row, no empty cell beside it. -->
  <div class="grid grid-cols-2 gap-16 touch:grid-cols-1">
    <DetailsCard class="gap-10 last:odd:col-span-full touch:gap-6">
      <Heading as="h2" :variant="HeadingVariant.Overline">{{
        t('details.controls.padTitle')
      }}</Heading>
      <div class="font-medium">
        {{
          ownRemap
            ? t('details.controls.remapOwn')
            : t('details.controls.remapInherited', { console: consoleName })
        }}
      </div>
      <div class="text-ink-muted">{{ t('details.controls.remapLead') }}</div>
      <DetailsAction class="self-start" :disabled="!scope" @click="emit('remap')">
        {{ t('details.controls.remap') }}
      </DetailsAction>
    </DetailsCard>
    <DetailsCard class="gap-10 last:odd:col-span-full touch:gap-6">
      <Heading as="h2" :variant="HeadingVariant.Overline">{{
        t('details.controls.turboTitle')
      }}</Heading>
      <div class="text-ink-muted">{{ t('details.notYet') }}</div>
    </DetailsCard>
    <DetailsCard class="gap-10 last:odd:col-span-full touch:gap-6">
      <Heading as="h2" :variant="HeadingVariant.Overline">{{
        t('details.controls.touchTitle')
      }}</Heading>
      <div class="text-ink-muted">{{ t('details.controls.touchLead') }}</div>
    </DetailsCard>
    <DetailsCard
      v-if="hasTouchScreen(game.console)"
      class="gap-10 last:odd:col-span-full touch:gap-6"
    >
      <Heading as="h2" :variant="HeadingVariant.Overline">{{
        t('details.controls.pointerTitle')
      }}</Heading>
      <div class="font-medium">{{ pointer }}</div>
      <div class="text-ink-muted">{{ t('details.controls.pointerLead') }}</div>
    </DetailsCard>
  </div>
</template>
