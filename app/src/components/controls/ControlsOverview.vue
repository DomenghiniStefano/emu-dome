<script setup lang="ts">
// C01 «Controlli» (board «C01 · Desktop · Controlli», Controlli2, 18/11): what you use most
// first. The devices as cards (connected, battery; «In uso» on the controller the section starts
// on), selectable: «Scorciatoie da ricordare» and «Prova i pulsanti» follow the chosen one. The
// controller's shortcuts come from the bindings in force (`DEFAULT_COMBOS` with `pad.combos`),
// the keyboard's from its one table (keyboard.ts): never a list here. The tester is the
// 12-button grid of C02. Then the rows down to C02 «Rimappa i pulsanti», C04 «Stick, grilletti e
// vibrazione» and the touch controls (customised on the phone).
//
// TODO(Cowork, C01): the board also lists controllers used before and now disconnected
// («Non collegato · ricordato»); EmuDome does not remember controllers yet.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Gamepad2, Keyboard } from '@lucide/vue'
import {
  emu,
  type ComboRow,
  type PadInfo,
  type PadState,
  type Pos,
  type RemapRow,
  type SettingRow,
} from '@/emu'
import { errorText, gamepadName } from '@/i18n/labels'
import { buttonName, keyName, posName, testerCells } from '@/lib/controls'
import { GLOBAL } from '@/lib/settingsModel'
import { Card, CardVariant } from '@/components/ui/card'
import DeviceCard from './DeviceCard.vue'
import TesterCard from './TesterCard.vue'
import ShortcutsToRemember from './ShortcutsToRemember.vue'
import { connectionText, familyOf, isSameDevice, type Device, type DeviceId } from './devices'
import { keyboardRememberLines, padRememberLines } from './shortcutLines'
import { ControlsTab } from './tabs'
import { buttonsUsing } from './tester'
import { useButtonTester } from './useButtonTester'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { LinkRow } from '@/components/ui/link-row'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'

const props = defineProps<{
  pads: PadInfo[]
  /** The device chosen in this section. */
  device: Device
  states: PadState[]
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{
  pick: [DeviceId]
  /** Go one step down: C02 remap, C03 shortcuts, C04 sticks. */
  open: [ControlsTab]
  error: [string]
}>()
const { t } = useI18n()

const family = computed(() => familyOf(props.device))
const isKeyboard = computed(() => props.device === 'keyboard')

// ---- Bindings in force at «Tutti i giochi»: the shortcuts and what a press does in the game ----
const combos = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
const remap = ref<RemapRow[]>([])
async function load() {
  try {
    const own = await emu.controls.bindings(family.value, GLOBAL)
    remap.value = own.remap
    combos.value = own.combos
    modifier.value = own.modifier
  } catch (e) {
    emit('error', errorText(e))
  }
}
watch(family, () => void load())
watch(
  () => [props.globalRows['pad.combos']?.value, props.globalRows['pad.remap']?.value],
  () => void load(),
  { deep: true },
)
onMounted(() => void load())

const lines = computed(() =>
  isKeyboard.value
    ? keyboardRememberLines()
    : padRememberLines(family.value, modifier.value, combos.value),
)
const shortcutsHint = computed(() =>
  isKeyboard.value
    ? t('controls.overview.keyboardKeys')
    : t('controls.overview.shortcutsHint', {
        modifier: posName(family.value, modifier.value, false),
      }),
)

// ---- Tester: what is held lights up; the last press says what it does in the game ----
const cells = computed(() => testerCells(family.value, remap.value))
function sourceText(source: string): string {
  return family.value === 'keyboard' ? keyName(source) : posName(family.value, source as Pos)
}
function describe(source: string): string {
  const users = buttonsUsing(remap.value, source).map(buttonName)
  return users.length
    ? t('controls.overview.lastPressed', { source: sourceText(source), button: users.join(', ') })
    : t('controls.overview.lastUnused', { source: sourceText(source) })
}
const tester = useButtonTester({
  padId: () => (props.device === 'keyboard' ? null : props.device.id),
  describe,
})
watch(() => props.states, tester.onPads)
watch(() => props.device, tester.reset)

// ---- Rows down to C02 / C04 ----
const sticksSub = computed(() =>
  t('controls.overview.sticksSub', {
    dead: Number(props.globalRows['pad.dead']?.value ?? 15),
    trig: Number(props.globalRows['pad.trig']?.value ?? 50),
  }),
)
</script>

<template>
  <SectionPage :width="SectionWidth.Wide">
    <Heading as="h1" :variant="HeadingVariant.Title">{{ t('shell.controls') }}</Heading>

    <div
      class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-tile-min),1fr))] gap-12"
      role="radiogroup"
      :aria-label="t('controls.devices')"
    >
      <DeviceCard
        v-for="(p, i) in pads"
        :key="p.id"
        :icon="Gamepad2"
        :name="gamepadName(p)"
        :detail="connectionText(p)"
        :chosen="isSameDevice(device, p)"
        :badge="i === 0 ? t('controls.overview.inUse') : undefined"
        :data-nav-initial="i === 0 ? '' : undefined"
        @click="emit('pick', p.id)"
      />
      <DeviceCard
        :icon="Keyboard"
        :name="t('controls.keyboard')"
        :detail="t('controls.keyboardDetail')"
        :chosen="isSameDevice(device, 'keyboard')"
        :data-nav-initial="pads.length === 0 ? '' : undefined"
        @click="emit('pick', 'keyboard')"
      />
    </div>

    <div
      class="grid grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] items-start gap-20 max-controls-stack:grid-cols-[minmax(0,1fr)]"
    >
      <ShortcutsToRemember
        :lines="lines"
        :hint="shortcutsHint"
        @edit="emit('open', ControlsTab.Shortcuts)"
      />

      <section class="flex flex-col gap-8">
        <TesterCard
          :cells="cells"
          :held="tester.held.value"
          :last="tester.last.value"
          :idle="t('controls.overview.press')"
        />

        <Card :variant="CardVariant.List">
          <LinkRow
            :name="t('controls.overview.remap')"
            :sub="t('controls.overview.remapSub')"
            @click="emit('open', ControlsTab.Remap)"
          />
          <LinkRow
            :name="t('controls.overview.sticks')"
            :sub="sticksSub"
            :disabled="isKeyboard"
            @click="emit('open', ControlsTab.Sticks)"
          />
          <LinkRow
            as="div"
            :name="t('controls.overview.touch')"
            :sub="t('controls.overview.touchSub')"
          />
        </Card>
      </section>
    </div>
  </SectionPage>
</template>
