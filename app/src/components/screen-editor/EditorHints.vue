<script setup lang="ts">
// The C09 editor's keys, in a glass pill at the bottom: the controller's with a controller in
// use, the keyboard's otherwise, none on touch (as the quick menu's hints). The C06 touch
// controls editor shows the same keys, with its own words for what picking does (`pick`).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Kbd } from '@/design-system/components/kbd'
import { posName } from '@/lib/controls'
import { inputMode } from '@/lib/inputMode'

const props = withDefaults(defineProps<{ pick?: string }>(), { pick: '' })
const { t } = useI18n()
const said = (does: string) =>
  does === 'pick' && props.pick ? props.pick : t(`screenEditor.hint.${does}`)

interface Hint {
  keys: string[]
  /** What they do (message key). */
  does: string
}

const hints = computed<Hint[]>(() =>
  inputMode.value === 'controller'
    ? [
        {
          keys: [posName('generic', 'l1', false), posName('generic', 'r1', false)],
          does: 'pick',
        },
        { keys: [t('screenEditor.key.dpad')], does: 'move' },
        { keys: [t('screenEditor.key.rightStick')], does: 'resize' },
        { keys: [t('controls.button.a')], does: 'save' },
        { keys: [t('controls.button.b')], does: 'cancel' },
      ]
    : [
        { keys: [t('screenEditor.key.pageUp'), t('screenEditor.key.pageDown')], does: 'pick' },
        { keys: [t('screenEditor.key.arrows')], does: 'move' },
        { keys: [t('screenEditor.key.shiftArrows')], does: 'resize' },
        { keys: [t('controls.key.Enter')], does: 'save' },
        { keys: [t('controls.key.Escape')], does: 'cancel' },
      ],
)
</script>

<template>
  <ul
    v-if="inputMode !== 'touch'"
    class="m-0 flex list-none flex-wrap items-center gap-12 rounded-panel border border-line px-12 py-6 text-caption text-ink-muted shadow-glass glass-thick-flat"
    :aria-label="t('screenEditor.hintsLabel')"
  >
    <li v-for="h in hints" :key="h.does" class="flex items-center gap-6">
      <Kbd v-for="k in h.keys" :key="k">{{ k }}</Kbd>
      {{ said(h.does) }}
    </li>
  </ul>
</template>
