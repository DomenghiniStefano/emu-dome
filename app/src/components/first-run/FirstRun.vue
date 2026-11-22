<script setup lang="ts">
// First run (Trello #35, boards PrimoAvvio and Tel-PrimoAvvio): four steps, shown once per
// device (App.vue asks `emu.firstRun.show`).
// - S01 Welcome: the language. No «Indietro».
// - S02 Games: the library folders with the scan in the same step.
// - S04 Controller: only while a controller is connected (the steps come from Rust,
//   `first_run::steps`); A/B layout and a press test. The only step with «Salta».
// - S06 Ready: game count, covers, optional imports, the quick menu hint.
// Controller: one nav scope; B goes back (not on S01, and not on S04, where B is a button to
// try). Esc is that same `back` on the keyboard (nav.ts `escape`), so it stays on S04 too.
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type PadInfo } from '@/emu'
import { report } from '@/lib/errors'
import { useErrorSink } from '@/composables/useToasts'
import { focusInitial, useNavScope } from '@/nav'
import { useFirstRunSteps } from './useFirstRunSteps'
import { useListeners } from '@/composables/useListeners'
import { onSettingsChanged } from '@/composables/useSettingsChanged'
import { useLanguageChoice } from './useLanguageChoice'
import { usePadTester } from './usePadTester'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { useFirstRunImports } from './useFirstRunImports'
import ControllerStep from './ControllerStep.vue'
import FirstRunFooter from './FirstRunFooter.vue'
import GamesStep from './GamesStep.vue'
import ReadyStep from './ReadyStep.vue'
import StepDots from './StepDots.vue'
import WelcomeStep from './WelcomeStep.vue'

const emit = defineEmits<{ done: [] }>()
const { t } = useI18n()
const failWith = useErrorSink()

const pads = ref<PadInfo[]>([])
const { steps, step, index, mainKey, refresh, next, back } = useFirstRunSteps(pads, {
  error: failWith,
  done: () => emit('done'),
})
const { phone } = useDeviceLayout()
const language = useLanguageChoice(failWith)
const tester = usePadTester(pads, () => step.value === 'controller', failWith)
const { imports, busy, pickFolder, importAll } = useFirstRunImports(failWith)

// ---- Navigation ----
const root = ref<HTMLElement | null>(null)
useNavScope(root, {
  autofocus: true,
  escape: true,
  back: () => {
    if (step.value !== 'controller') back()
  },
})
/** Each step focuses its own first element (`data-nav-initial`), else the main button. */
watch(step, async () => {
  await nextTick()
  if (root.value) focusInitial(root.value)
})
watch(step, s => tester.listen(s === 'controller'))
watch(pads, () => void refresh())

// ---- Lifecycle ----
const listeners = useListeners()
onMounted(async () => {
  try {
    await language.load()
    pads.value = await emu.controls.pads()
  } catch (e) {
    failWith(e)
  }
  await listeners.add(
    emu.onGamepadConnected(p => {
      if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
    }),
    emu.onGamepadDisconnected(p => {
      pads.value = pads.value.filter(x => x.id !== p.id)
    }),
    emu.controls.onPadState(tester.onPadState),
    onSettingsChanged(['pad.ab', 'pad.remap'], () => void tester.load()),
  )
})
onBeforeUnmount(() => {
  if (step.value === 'controller') report(emu.controls.watch(false))
})
</script>

<template>
  <div
    ref="root"
    class="fixed inset-0 flex flex-col overflow-hidden bg-welcome-glow text-ink touch:bg-welcome-glow-phone touch:text-lead"
  >
    <StepDots :steps="steps" :index="index" />

    <main class="grid min-h-0 flex-1 place-items-center overflow-y-auto px-16 touch:px-20">
      <WelcomeStep
        v-if="step === 'welcome'"
        :lang="language.lang.value"
        :lang-options="language.options.value"
        @lang="language.choose"
      />
      <GamesStep v-else-if="step === 'games'" />
      <ControllerStep
        v-else-if="step === 'controller' && tester.pad.value"
        :pad="tester.pad.value"
        :ab="tester.ab.value"
        :pressed="tester.pressed.value"
        @ab="tester.setAb"
      />
      <ReadyStep
        v-else-if="step === 'ready'"
        :phone="phone"
        :imports="imports"
        :busy="busy"
        @pick="pickFolder"
        @import="importAll"
      />
    </main>

    <FirstRunFooter
      :phone="phone"
      :can-go-back="index > 0"
      :skippable="step === 'controller'"
      :main-label="t(mainKey)"
      :last="index === steps.length - 1"
      @next="next"
      @back="back"
    />
  </div>
</template>
