<script setup lang="ts">
// E05: no controller connected. The keyboard can be chosen meanwhile; on the phone with no
// physical keyboard used yet (#118) the touch controls are the way to play instead.
//
// Where the touch controls can be edited here (`touchEditable`: a touch device, Settings'
// `touch.tedit` row open; #132) E05 is one card at the top of the page and C01's «Controlli
// touch» row leads to the C06 editor under it, so the editor is reached with no controller too.
// Elsewhere E05 fills the section, centred.
import { useI18n } from 'vue-i18n'
import { Gamepad2, Keyboard } from '@lucide/vue'
import { useKeyboardKeys } from '@/composables/useKeyboardKeys'
import { useShellJump } from '@/components/shell/useShellJump'
import { Button, ButtonSize } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { EmptyState, EmptyStateVariant } from '@/design-system/components/empty-state'
import { Heading } from '@/design-system/components/heading'
import { LinkRow } from '@/design-system/components/link-row'
import { SectionPage } from '@/design-system/components/section-page'

defineProps<{
  /** The touch controls can be edited on this device: E05 becomes a card above their row. */
  touchEditable: boolean
}>()
const emit = defineEmits<{ keyboard: [] }>()
const { t } = useI18n()
const keyboardKeys = useKeyboardKeys()
const jump = useShellJump()
</script>

<template>
  <SectionPage as="section">
    <Heading as="h1">{{ t('shell.controls') }}</Heading>
    <EmptyState
      as="div"
      :variant="touchEditable ? EmptyStateVariant.Card : EmptyStateVariant.Page"
      :icon="Gamepad2"
      :title="t('controls.e05.title')"
      :text="keyboardKeys ? t('controls.e05.lead') : t('controls.e05.leadTouch')"
    >
      <Button
        v-if="keyboardKeys"
        :size="ButtonSize.Medium"
        :icon="Keyboard"
        data-nav-initial
        @click="emit('keyboard')"
      >
        {{ t('controls.e05.keyboard') }}
      </Button>
    </EmptyState>
    <Card v-if="touchEditable && jump" :variant="CardVariant.List">
      <LinkRow
        :name="t('controls.overview.touch')"
        :sub="t('controls.overview.touchEdit')"
        :data-nav-initial="keyboardKeys ? undefined : ''"
        @click="jump('touchEditor')"
      />
    </Card>
  </SectionPage>
</template>
