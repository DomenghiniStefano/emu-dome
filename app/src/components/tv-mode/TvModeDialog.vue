<script setup lang="ts">
// The TV question (#30, owner 2026-11-22): one dialog for both moments, on the game flow's
// dialog (FlowDialog: X04's look, centred over the scrim, its own nav scope, B is the safe exit).
// - `connect`: a controller connected at Automatica outside TV: «Passare alla modalità TV?»;
// - `disconnect`: the last controller left TV mode at Automatica: «Controller scollegato», in place
//   of the shell's «controller disconnected» toast (never both); it goes away by itself when a
//   controller connects again (useUiMode), and TV stays.
// The first button is the main action (accent, first focus); the other, B and Esc keep things as
// they are. «Non chiederlo più» saves the answer as `general.ui` (useUiMode `answerTvAsk`).
// In game the game pauses while it shows, and the controller drives the dialog (the interface
// has the input, as with the quick menu); both are given back when it closes.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import FlowDialog from '@/components/game/FlowDialog.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { answerTvAsk } from '@/composables/useUiMode'
import { emu } from '@/emu'
import { report } from '@/lib/errors'
import type { TvAsk, UiLayout } from '@/lib/uiMode'

const props = defineProps<{
  ask: TvAsk
  /** A game is open: it pauses while the dialog shows. */
  inGame: boolean
  /** The quick menu is open (the game is already paused, the interface has the input). */
  menuOpen: boolean
}>()
const { t } = useI18n()

const remember = ref(false)
/** The main action's layout: TV on connect, the desktop on disconnect. */
const main: UiLayout = props.ask === 'connect' ? 'tv' : 'desktop'
const other: UiLayout = props.ask === 'connect' ? 'desktop' : 'tv'
const answer = (layout: UiLayout) => answerTvAsk(layout, remember.value)

// In game: hold the game and give the controller to the interface while the question shows.
let held = false
onMounted(() => {
  if (!props.inGame || props.menuOpen) return
  held = true
  report(emu.setMenuOpen(true))
  report(emu.setPaused(true))
})
onBeforeUnmount(() => {
  if (!held) return
  report(emu.setMenuOpen(false))
  report(emu.setPaused(false))
})
</script>

<template>
  <FlowDialog :title="t(`tv.ask.${ask}.title`)" :escape-closes="!inGame" @back="answer(other)">
    <p class="m-0">{{ t(`tv.ask.${ask}.text`) }}</p>
    <Checkbox v-model="remember">
      <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6 self-start">
        <span
          class="size-14 shrink-0 rounded-badge"
          :class="remember ? 'bg-accent-text' : 'border border-line-strong'"
          aria-hidden="true"
        ></span>
        {{ t('tv.ask.remember') }}
      </Button>
    </Checkbox>
    <p v-if="ask === 'connect'" class="m-0 text-caption">{{ t('tv.ask.where') }}</p>
    <template #actions>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="answer(main)"
      >
        {{ t(`tv.ask.${ask}.main`) }}
      </Button>
      <Button :size="ButtonSize.Medium" @click="answer(other)">
        {{ t(`tv.ask.${ask}.other`) }}
      </Button>
    </template>
  </FlowDialog>
</template>
