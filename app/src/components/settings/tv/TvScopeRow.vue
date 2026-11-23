<script setup lang="ts">
// «Stai cambiando le impostazioni di:» in TV mode (#30): the desk's scope bar as the page's first
// row. ◀ ▶ step the level as the desk's L1 / R1 do («Tutti i giochi», the console's games, «Solo
// questo gioco»); A opens the sheet with every level and every console of the library, as the
// desk's console menu. Under it while focused: what a change does at the level chosen, and the
// page's note on the rows only «Tutti i giochi» changes.
import { computed, nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import type { ConsoleId, SettingLevel } from '@/emu'
import type { GameFocus } from '@/lib/gameDetails'
import type { EditLevel } from '@/components/settings/settingsPage'
import TvChoiceSheet, { type SheetOption } from './TvChoiceSheet.vue'
import { scopeChoiceAt, scopeChoices } from './tvSettingRow'

const props = defineProps<{
  levels: EditLevel[]
  effective: EditLevel
  contextConsole: ConsoleId | null
  consoleOptions: { id: string; text: string }[]
  gameFocus: GameFocus | null
  levelName: (level: SettingLevel) => string
  /** «Le voci in grigio valgono per tutti i giochi…», when the page has such rows. */
  lockedNote: string
}>()
const emit = defineEmits<{ level: [EditLevel]; pickConsole: [string]; step: [dir: -1 | 1] }>()
const { t } = useI18n()

/** What a change does at the level chosen (the desk's scope bar says the same). */
const line = computed(() => {
  if (props.effective === 'game' && props.gameFocus)
    return t('settingsUi.scope.gameLine', { title: props.gameFocus.title })
  if (props.effective === 'console' && props.contextConsole)
    return t('settingsUi.scope.consoleLine', { console: t(`console.${props.contextConsole}`) })
  return t('settingsUi.scope.globalLine')
})
const note = computed(() => `${line.value} ${props.lockedNote}`.trim())

const choices = computed(() =>
  scopeChoices(
    props.levels,
    props.consoleOptions.map(c => c.id),
  ),
)
const sheetOptions = computed<SheetOption[]>(() =>
  choices.value.map(c => {
    if (c.level === 'console') {
      const text = props.consoleOptions.find(o => o.id === c.id)?.text ?? c.id
      const short = t(`consoleShort.${c.id}`)
      return { id: c.id, text, desc: t('settingsUi.scope.consoleSub', { short }) }
    }
    if (c.level === 'game')
      return { id: c.id, text: props.levelName('game'), desc: props.gameFocus?.title ?? '' }
    return { id: c.id, text: props.levelName('global'), desc: t('settingsUi.scope.globalSub') }
  }),
)
const chosen = computed(() => [scopeChoiceAt(props.effective, props.contextConsole)])

function pick(id: string) {
  const c = choices.value.find(x => x.id === id)
  if (!c) return
  if (c.level === 'console') emit('pickConsole', c.id)
  else emit('level', c.level)
}

function step(e: CustomEvent<-1 | 1>) {
  e.preventDefault()
  emit('step', e.detail)
}

const sheet = ref(false)
const self = ref<{ $el: HTMLElement } | null>(null)
function closeSheet() {
  sheet.value = false
  void nextTick(() => self.value?.$el.focus())
}
</script>

<template>
  <Button
    ref="self"
    :variant="ButtonVariant.TvRow"
    :size="ButtonSize.TvRow"
    :fade="ButtonFade.None"
    class="group flex-none"
    data-tv-step
    :aria-label="`${t('settingsUi.scopeTitle')} ${levelName(effective)}`"
    :aria-description="note"
    @click="sheet = true"
    @tv-step="step"
  >
    <span class="flex min-w-0 flex-1 flex-col gap-4" aria-hidden="true">
      <span class="font-medium">{{ t('settingsUi.scopeTitle') }}</span>
      <span class="hidden text-tv-hint text-ink-muted group-focus:block">{{ note }}</span>
    </span>
    <span class="flex flex-none items-center gap-12 text-ink-muted" aria-hidden="true">
      <ChevronLeft
        v-if="levels.length > 1"
        class="invisible size-tv-glyph flex-none group-focus:visible"
        :stroke-width="2.25"
      />
      <span class="font-semibold text-ink">{{ levelName(effective) }}</span>
      <ChevronRight class="size-tv-glyph flex-none" :stroke-width="2.25" />
    </span>
  </Button>

  <TvChoiceSheet
    v-if="sheet"
    :title="t('settingsUi.scopeTitle')"
    :desc="line"
    :options="sheetOptions"
    :chosen="chosen"
    @pick="pick"
    @close="closeSheet"
  />
</template>
