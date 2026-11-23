<script setup lang="ts">
// «…» of a game (board L01 «Altre azioni» on a cover, #59 item 2; L04 in the game details): the
// game's actions that have a backend today, those listed in `items`. «Scegli la copertina…»
// opens L12 and is offered from the game details only (design, #40); favourite and hide go
// through `library.setFlags`.
// On the DropdownMenu primitive, hanging from the parent's «…» button (`anchor`); open while
// mounted (the parent shows it with `v-if`, so a grid of covers holds no closed menus), closed by
// the parent on `close` or after an action (each action's own closing, not Reka's, so the focus
// goes where the parent puts it).
// Controller / keyboard: the menu's own nav scope while open, the first item focused; B / Esc / a
// click outside close it.
import { useI18n } from 'vue-i18n'
import { EyeOff, Eye, ImageIcon, Star, StarOff } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import type { LibraryGame } from '@/emu'

const props = defineProps<{
  game: LibraryGame
  items: Array<'cover' | 'favourite' | 'hide'>
  /** The «…» button the menu hangs from. */
  anchor: HTMLElement | null
}>()
const emit = defineEmits<{
  cover: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
  close: []
}>()

const { t } = useI18n()
</script>

<template>
  <DropdownMenu :open="true" @update:open="o => o || emit('close')">
    <DropdownMenuContent
      :anchor="anchor"
      :aria-label="t('library.moreFor', { title: props.game.title })"
    >
      <DropdownMenuItem v-if="props.items.includes('cover')" @select.prevent="emit('cover')">
        <Button
          :variant="ButtonVariant.Menu"
          :size="ButtonSize.MenuItem"
          :icon="ImageIcon"
          data-nav-initial
        >
          {{ t('library.menu.cover') }}
        </Button>
      </DropdownMenuItem>
      <DropdownMenuItem
        v-if="props.items.includes('favourite')"
        @select.prevent="emit('flags', { favourite: !props.game.favourite })"
      >
        <Button
          :variant="ButtonVariant.Menu"
          :size="ButtonSize.MenuItem"
          :icon="props.game.favourite ? StarOff : Star"
        >
          {{ t(props.game.favourite ? 'library.menu.unfavourite' : 'library.menu.favourite') }}
        </Button>
      </DropdownMenuItem>
      <DropdownMenuItem
        v-if="props.items.includes('hide')"
        @select.prevent="emit('flags', { hidden: !props.game.hidden })"
      >
        <Button
          :variant="ButtonVariant.Menu"
          :size="ButtonSize.MenuItem"
          :icon="props.game.hidden ? Eye : EyeOff"
        >
          {{ t(props.game.hidden ? 'library.menu.unhide' : 'library.menu.hide') }}
        </Button>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
