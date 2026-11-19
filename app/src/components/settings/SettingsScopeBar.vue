<script setup lang="ts">
// «Stai cambiando le impostazioni di:» (ScopeSelector), fixed at the top of a page while it
// scrolls: the levels as cards («Tutti i giochi», «Giochi <console> ▾» with its console menu,
// «Solo questo gioco» when opened for a game), «Mostra solo le impostazioni modificate», and the
// sentence saying what a change does at the level chosen.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Disc3, Gamepad2, Layers } from '@lucide/vue'
import { Card, CardVariant } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import type { ConsoleId, SettingLevel } from '../../emu'
import type { GameFocus } from '../../gameDetails'
import SettingSelect from '../SettingSelect.vue'
import type { EditLevel } from './settingsPage'

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
const onlyChanged = defineModel<boolean>('onlyChanged', { required: true })
const emit = defineEmits<{ level: [EditLevel]; pickConsole: [string] }>()
const { t } = useI18n()

// One column per level, as Tailwind's own classes (repeat(n, minmax(0, 1fr))).
const COLUMNS: Record<number, string> = { 1: 'grid-cols-1', 2: 'grid-cols-2', 3: 'grid-cols-3' }
const columns = computed(() => COLUMNS[props.levels.length] ?? 'grid-cols-1')

/** What a change does at the level chosen. */
const line = computed(() => {
  if (props.effective === 'game' && props.gameFocus)
    return t('settingsUi.scope.gameLine', { title: props.gameFocus.title })
  if (props.effective === 'console' && props.contextConsole)
    return t('settingsUi.scope.consoleLine', { console: t(`console.${props.contextConsole}`) })
  return t('settingsUi.scope.globalLine')
})
</script>

<template>
  <!-- Thick glass with radius 12 (pending Cowork): the glass's material, edge and shadow, its
       own corners. Sticks over the rows that scroll under it. -->
  <div
    class="glass-thick-flat sticky top-0 z-sticky flex flex-col gap-10 rounded-12 border border-line px-16 py-14 shadow-glass"
    role="region"
    :aria-label="t('settingsUi.scopeRegion')"
  >
    <div class="flex flex-wrap items-center justify-between gap-12">
      <div class="text-heading">{{ t('settingsUi.scopeTitle') }}</div>
      <!-- 6 / 1 padding: what the browser gave the button this used to be -->
      <label class="flex cursor-pointer items-center gap-8 px-6 py-px font-medium text-ink-muted">
        {{ t('settingsUi.onlyChanged') }}
        <!-- no glow when on, unlike a setting's switch (as the bar's own switch was) -->
        <Switch v-model="onlyChanged" class="data-[state=checked]:shadow-none" />
      </label>
    </div>
    <div
      class="grid gap-8"
      :class="columns"
      role="radiogroup"
      :aria-label="t('settingsUi.scopeTitle')"
    >
      <Card
        as="button"
        :variant="CardVariant.RaisedAccent"
        :selected="effective === 'global'"
        class="min-h-level flex-row items-start gap-10 px-12"
        role="radio"
        :aria-checked="effective === 'global'"
        @click="emit('level', 'global')"
      >
        <Layers :size="20" :stroke-width="1.75" aria-hidden="true" />
        <span class="flex flex-col gap-2">
          <span class="font-semibold">{{ levelName('global') }}</span>
          <span class="text-caption text-ink-muted">{{ t('settingsUi.scope.globalSub') }}</span>
        </span>
      </Card>
      <!-- «Giochi <console> ▾»: picking a console also selects this level -->
      <Card
        v-if="levels.includes('console') && contextConsole"
        :variant="CardVariant.RaisedAccent"
        :selected="effective === 'console'"
        class="min-h-level flex-row items-start gap-10 px-12"
        role="radio"
        :aria-checked="effective === 'console'"
        @click="emit('level', 'console')"
      >
        <Gamepad2 :size="20" :stroke-width="1.75" aria-hidden="true" />
        <span class="flex flex-col gap-2">
          <SettingSelect
            in-title
            :label="t('settingsUi.scope.consolePick')"
            :value="contextConsole"
            :options="consoleOptions"
            @pick="id => emit('pickConsole', id)"
            @click.stop
          />
          <span class="text-caption text-ink-muted">
            {{ t('settingsUi.scope.consoleSub', { short: t(`consoleShort.${contextConsole}`) }) }}
          </span>
        </span>
      </Card>
      <!-- «Solo questo gioco»: only when opened from a game's details (L06) -->
      <Card
        v-if="levels.includes('game') && gameFocus"
        as="button"
        :variant="CardVariant.RaisedAccent"
        :selected="effective === 'game'"
        class="min-h-level flex-row items-start gap-10 px-12"
        role="radio"
        :aria-checked="effective === 'game'"
        @click="emit('level', 'game')"
      >
        <Disc3 :size="20" :stroke-width="1.75" aria-hidden="true" />
        <span class="flex flex-col gap-2">
          <span class="font-semibold">{{ levelName('game') }}</span>
          <span class="text-caption text-ink-muted">{{ gameFocus.title }}</span>
        </span>
      </Card>
    </div>
    <div class="text-ink-muted">{{ line }} {{ lockedNote }}</div>
  </div>
</template>
