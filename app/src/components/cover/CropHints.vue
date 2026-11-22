<script setup lang="ts">
// «Adatta la copertina»'s keys (the mockup's row under the zoom): the controller's with a
// controller in use, the keyboard's otherwise, none on touch (as the C09 editor's hints).
// L1 / R1 zoom: LT / RT do not reach the interface as steps.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Kbd } from '@/components/ui/kbd'
import { posName } from '@/lib/controls'
import { inputMode } from '@/lib/inputMode'

const { t } = useI18n()

interface Hint {
  keys: string[]
  /** What they do (message key under `library.cover.crop.hint`). */
  does: string
}

const hints = computed<Hint[]>(() =>
  inputMode.value === 'controller'
    ? [
        { keys: [t('screenEditor.key.dpad')], does: 'move' },
        {
          keys: [posName('generic', 'l1', false), posName('generic', 'r1', false)],
          does: 'zoom',
        },
        { keys: [t('controls.button.x')], does: 'mode' },
        { keys: [t('controls.button.a')], does: 'use' },
        { keys: [t('controls.button.b')], does: 'back' },
      ]
    : [
        { keys: [t('screenEditor.key.arrows')], does: 'move' },
        {
          keys: [t('library.cover.crop.keyZoomOut'), t('library.cover.crop.keyZoomIn')],
          does: 'zoom',
        },
        { keys: [t('controls.key.Enter')], does: 'use' },
        { keys: [t('controls.key.Escape')], does: 'back' },
      ],
)
</script>

<template>
  <ul
    v-if="inputMode !== 'touch'"
    class="m-0 flex list-none flex-wrap items-center gap-14 p-0 text-caption text-ink-muted"
    :aria-label="t('library.cover.crop.hintsLabel')"
  >
    <li v-for="h in hints" :key="h.does" class="flex items-center gap-6">
      <Kbd v-for="k in h.keys" :key="k">{{ k }}</Kbd>
      {{ t(`library.cover.crop.hint.${h.does}`) }}
    </li>
  </ul>
</template>
