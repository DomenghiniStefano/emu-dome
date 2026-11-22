<script setup lang="ts">
// G15 «Coperchio chiuso» (Trello #28, option A of the owner's mockup): while the DS lid is closed,
// a thick-glass card in the middle of the darkened game. The game decides what a closed lid does
// (it usually sleeps): EmuDome does not pause it. «Apri il coperchio» opens it again, and so do
// the controller's shortcut (L3 + Select by default, read from the bindings) and the quick menu.
// The game keeps the controller while the card shows, so the card's button is for the mouse and
// the keyboard; the hint names the shortcut.
import { onMounted, ref } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import { Laptop } from '@lucide/vue'
import { emu, type Pos } from '@/emu'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { Kbd } from '@/components/ui/kbd'
import { posName } from '@/lib/controls'

const emit = defineEmits<{ open: [] }>()
const { t } = useI18n()

const button = ref<{ $el: HTMLElement } | null>(null)
/** The lid's shortcut, null while unknown or unbound (no hint then). */
const shortcut = ref<{ modifier: Pos; pos: Pos } | null>(null)

onMounted(async () => {
  button.value?.$el.focus()
  try {
    const b = await emu.controls.bindings('generic', { level: 'session' })
    const pos = b.combos.find(c => c.hotkey === 'lid')?.pos ?? null
    shortcut.value = pos ? { modifier: b.modifier, pos } : null
  } catch (e) {
    // Deliberate: only the hint is missing; the button and the quick menu still open the lid.
    console.warn('lid shortcut not read', e)
  }
})
</script>

<template>
  <div
    class="pointer-events-none fixed inset-0 z-overlay flex items-center justify-center bg-game-scrim"
    role="status"
  >
    <div
      class="pointer-events-auto flex w-lid-card max-w-[calc(100vw-var(--spacing-window-margin))] flex-col items-center gap-8 p-dialog-pad text-center glass-thick"
    >
      <Laptop class="text-ink-muted" :size="24" :stroke-width="1.75" aria-hidden="true" />
      <Heading as="h2" :variant="HeadingVariant.PageTitle">{{ t('flow.lid.title') }}</Heading>
      <p class="m-0 text-ink-muted">{{ t('flow.lid.text') }}</p>
      <Button ref="button" :variant="ButtonVariant.Soft" @click="emit('open')">
        {{ t('flow.lid.open') }}
      </Button>
      <I18nT
        v-if="shortcut"
        keypath="flow.lid.hint"
        tag="span"
        class="text-caption text-ink-muted"
        scope="global"
      >
        <template #modifier>
          <Kbd>{{ posName('generic', shortcut.modifier, false) }}</Kbd>
        </template>
        <template #button>
          <Kbd>{{ posName('generic', shortcut.pos, false) }}</Kbd>
        </template>
      </I18nT>
    </div>
  </div>
</template>
