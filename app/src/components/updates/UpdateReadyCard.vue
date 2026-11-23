<script setup lang="ts">
// X06 «Aggiornamento disponibile» (owner, 2026-11-22, Trello #32 / #44): when a new version is
// ready, a small glass card at the bottom right of the shell, never blocking: «EmuDome <versione>
// è pronto · Riavvia per aggiornare. I giochi e i salvataggi non cambiano.», «Più tardi» and the
// main action «Riavvia e aggiorna». It lives in the shell only, so it never shows in game (it
// waits until the library is back). Controller: inside the shell's nav scope, reached by the
// normal spatial steps (nav.ts), no focus taken when it appears. In TV mode it sits over the TV
// button bar (shell/TvButtonBar.vue), lined up with the bar's edge.
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Clock, RefreshCw } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { useUpdates } from '@/composables/useUpdates'
import { installing, stateVersion } from '@/lib/updates'

const { t } = useI18n()
const { status, later, install } = useUpdates()

/** «Riavvia e aggiorna» was pressed: the card stays while the update downloads or installs. */
const pressed = ref(false)
const state = computed(() => status.value?.state ?? null)
const busy = computed(() => !!state.value && installing(state.value))
const version = computed(() => (state.value ? stateVersion(state.value) : null))
const shown = computed(
  () => !!version.value && (!!status.value?.prompt || (pressed.value && busy.value)),
)
// A failure (or a newer offer) ends the wait: the card shows the offer again or goes.
watch(busy, b => {
  if (!b) pressed.value = false
})

async function restart() {
  pressed.value = true
  await install()
}
</script>

<template>
  <section
    v-if="shown"
    class="pointer-events-auto fixed right-12 bottom-12 z-toast tv:right-tv-edge tv:bottom-tv-over-bar flex max-w-toast flex-col gap-8 rounded-card p-card-pad glass-thick"
    :aria-label="t('updates.title')"
    aria-live="polite"
  >
    <div class="flex flex-col gap-4">
      <span class="font-semibold">{{ t('updates.ready', { version }) }}</span>
      <span class="text-caption text-ink-muted">{{ t('updates.note') }}</span>
    </div>
    <div class="flex items-center justify-end gap-8">
      <Button
        :variant="ButtonVariant.GhostRegular"
        :size="ButtonSize.Small"
        :icon="Clock"
        :disabled="busy"
        @click="later"
      >
        {{ t('updates.later') }}
      </Button>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Small"
        :icon="RefreshCw"
        :disabled="busy"
        @click="restart"
      >
        {{ busy ? t('updates.installing') : t('updates.restart') }}
      </Button>
    </div>
  </section>
</template>
