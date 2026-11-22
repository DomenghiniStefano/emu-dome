<script setup lang="ts">
// L04 «…»: the game's other actions (CoverMenu) under their button. A tall Soft button beside
// «Riprendi» on the desktop, an icon alone on the phone's bar. Closing the menu puts the focus
// back on the button; `focus()` does the same for the cover picker it can open.
import { nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Ellipsis, EllipsisVertical } from '@lucide/vue'
import type { LibraryGame } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import CoverMenu from '@/components/cover/CoverMenu.vue'

defineProps<{
  game: LibraryGame
  phone: boolean
  items: Array<'cover' | 'favourite' | 'hide'>
}>()
const emit = defineEmits<{
  /** «Scegli la copertina…»: L12. */
  cover: []
  flags: [{ favourite?: boolean; hidden?: boolean }]
}>()

const { t } = useI18n()

const open = ref(false)
const trigger = ref<InstanceType<typeof Button> | null>(null)

function focus() {
  ;(trigger.value?.$el as HTMLElement | undefined)?.focus()
}
function close() {
  open.value = false
  void nextTick(focus)
}
function setFlags(flags: { favourite?: boolean; hidden?: boolean }) {
  close()
  emit('flags', flags)
}
function chooseCover() {
  open.value = false
  emit('cover')
}
defineExpose({ focus })
</script>

<template>
  <div class="relative">
    <Button
      ref="trigger"
      :variant="phone ? ButtonVariant.Plain : ButtonVariant.Soft"
      :size="phone ? ButtonSize.IconTouch : ButtonSize.Tall"
      :label="t('library.more')"
      aria-haspopup="menu"
      :aria-expanded="open"
      @click="open = !open"
    >
      <EllipsisVertical v-if="phone" :size="22" :stroke-width="1.75" aria-hidden="true" />
      <Ellipsis v-else :size="16" :stroke-width="1.75" aria-hidden="true" />
    </Button>
    <CoverMenu
      v-if="open"
      :game="game"
      :items="items"
      :anchor="(trigger?.$el as HTMLElement | undefined) ?? null"
      @cover="chooseCover"
      @flags="setFlags"
      @close="close"
    />
  </div>
</template>
