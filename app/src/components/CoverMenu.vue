<script setup lang="ts">
// «…» of a game (board L01 «Altre azioni» on a cover, #59 item 2; L04 in the game details): the
// game's actions that have a backend today, those listed in `items`. «Scegli la copertina…»
// opens L12 and is offered from the game details only (Cowork, #40); favourite and hide go
// through `library.setFlags`.
// Controller / keyboard: its own nav scope while open; B / Esc / a click outside close it.
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { EyeOff, Eye, ImageIcon, Star, StarOff } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useNavScope } from '../nav'
import type { LibraryGame } from '../emu'
import { useDismiss } from './library/useDismiss'

const props = defineProps<{ game: LibraryGame; items: Array<'cover' | 'favourite' | 'hide'> }>()
const emit = defineEmits<{
  cover: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
  close: []
}>()

const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, escape: true, back: () => emit('close') })
// The «…» button next to it toggles it itself.
useDismiss(root, () => emit('close'))
</script>

<template>
  <!-- 220px is the menu's own least width. -->
  <div
    ref="root"
    class="glass-thick absolute top-[calc(100%+6px)] right-0 z-raised flex min-w-[220px] flex-col gap-2 p-6"
    role="menu"
    :aria-label="t('library.moreFor', { title: props.game.title })"
  >
    <Button
      v-if="props.items.includes('cover')"
      :variant="ButtonVariant.Menu"
      :size="ButtonSize.MenuItem"
      role="menuitem"
      data-nav-initial
      @click="emit('cover')"
    >
      <ImageIcon :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('library.menu.cover') }}
    </Button>
    <Button
      v-if="props.items.includes('favourite')"
      :variant="ButtonVariant.Menu"
      :size="ButtonSize.MenuItem"
      role="menuitem"
      @click="emit('flags', { favourite: !props.game.favourite })"
    >
      <component
        :is="props.game.favourite ? StarOff : Star"
        :size="16"
        :stroke-width="1.75"
        aria-hidden="true"
      />
      {{ t(props.game.favourite ? 'library.menu.unfavourite' : 'library.menu.favourite') }}
    </Button>
    <Button
      v-if="props.items.includes('hide')"
      :variant="ButtonVariant.Menu"
      :size="ButtonSize.MenuItem"
      role="menuitem"
      @click="emit('flags', { hidden: !props.game.hidden })"
    >
      <component
        :is="props.game.hidden ? Eye : EyeOff"
        :size="16"
        :stroke-width="1.75"
        aria-hidden="true"
      />
      {{ t(props.game.hidden ? 'library.menu.unhide' : 'library.menu.hide') }}
    </Button>
  </div>
</template>
