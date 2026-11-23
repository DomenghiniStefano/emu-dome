<script setup lang="ts">
// C04 «Stick e grilletti» (board «Controlli»): the `pad.*` rows of sticks, triggers and menus,
// as settings rows at «Tutti i giochi» (the same rows as the Settings pages), and the live
// stick: the dashed circle is the dead zone, the dot the stick's position now.
// They take effect at once: the controller thread reloads them on `settings://changed`.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { type PadState, type SettingEntry, type SettingRow, type SettingsSchema } from '@/emu'
import { isShown } from '@/lib/settingsModel'
import type { CaptureDevice } from '@/lib/controls'
import { Card, CardVariant } from '@/design-system/components/card'
import RowColumns from '@/components/settings/RowColumns.vue'
import SettingRowView from '@/components/settings/SettingRow.vue'
import LiveStick from './LiveStick.vue'
import { axisUnit } from './stick'
import { useErrorSink } from '@/composables/useToasts'
import { useGlobalSettings } from './useGlobalSettings'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

const props = defineProps<{
  device: CaptureDevice
  states: PadState[]
  schema: SettingsSchema
  globalRows: Record<string, SettingRow>
}>()
const { t } = useI18n()
const settings = useGlobalSettings(useErrorSink())

const GROUPS: { id: string; keys: string[] }[] = [
  { id: 'sticks', keys: ['pad.dead', 'pad.stickdpad', 'pad.trig', 'pad.rumble'] },
  { id: 'menus', keys: ['pad.confirm', 'pad.repeat'] },
]
const groups = computed(() =>
  GROUPS.map(g => ({
    id: g.id,
    entries: g.keys
      .map(k => props.schema.entries.find(e => e.key === k))
      .filter((e): e is SettingEntry => !!e && isShown(e, props.schema)),
  })),
)

// ---- Live stick ----
const dead = computed(() => Number(props.globalRows['pad.dead']?.value ?? 15))
const axes = computed(() => {
  const id = props.device.kind === 'pad' ? props.device.id : null
  return props.states.find(s => s.id === id)?.axes
})
</script>

<template>
  <div class="flex flex-col gap-12">
    <section v-for="g in groups" :key="g.id" class="flex flex-col gap-8">
      <Heading as="h2" :variant="HeadingVariant.Overline">{{
        t(`settings.sections.pad.${g.id}`)
      }}</Heading>
      <Card :variant="CardVariant.List">
        <RowColumns>
          <SettingRowView
            v-for="e in g.entries"
            :key="e.key"
            :entry="e"
            :row="globalRows[e.key]"
            :schema="schema"
            level="global"
            :level-name="() => t('settingsUi.scope.global')"
            :scoped="false"
            @set="v => settings.set(e.key, v)"
            @reset="settings.reset(e.key)"
          />
        </RowColumns>
      </Card>
    </section>

    <LiveStick :x="axisUnit(axes?.[0])" :y="axisUnit(axes?.[1])" :dead="dead" />
  </div>
</template>
