<script setup lang="ts">
// First run (Trello #35, boards PrimoAvvio and Tel-PrimoAvvio): four steps, shown once per
// device (App.vue asks `emu.firstRun.show`).
// - S01 Welcome: the language. No «Indietro».
// - S02 Games: the library folders with the scan in the same step.
// - S04 Controller: only while a controller is connected (the steps come from Rust,
//   `first_run::steps`); A/B layout and a press test. The only step with «Salta».
// - S06 Ready: game count, covers, optional imports, the quick menu hint.
// Controller: one nav scope; B goes back (not on S01, and not on S04, where B is a button to
// try); Esc does the same on the keyboard.
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type PadInfo } from '../emu'
import { errorText } from '../i18n/labels'
import { useNavScope } from '../nav'
import { useFirstRunSteps } from '../composables/useFirstRunSteps'
import { useLanguageChoice } from '../composables/useLanguageChoice'
import { usePadTester } from '../composables/usePadTester'
import { usePhoneLayout } from '../composables/usePhoneLayout'
import { useSaveImports } from '../composables/useSaveImports'
import ControllerStep from './first-run/ControllerStep.vue'
import FirstRunFooter from './first-run/FirstRunFooter.vue'
import GamesStep from './first-run/GamesStep.vue'
import ReadyStep from './first-run/ReadyStep.vue'
import StepDots from './first-run/StepDots.vue'
import WelcomeStep from './first-run/WelcomeStep.vue'

const emit = defineEmits<{ done: []; error: [string] }>()
const { t } = useI18n()
const fail = (text: string) => emit('error', text)

const pads = ref<PadInfo[]>([])
const { steps, step, index, mainKey, refresh, next, back } = useFirstRunSteps(pads, {
  error: fail,
  done: () => emit('done'),
})
const { phone, detect: detectPhone } = usePhoneLayout()
const language = useLanguageChoice(fail)
const tester = usePadTester(pads, () => step.value === 'controller', fail)
const { imports, busy, pickFolder, importAll } = useSaveImports(fail)

// ---- Navigation ----
const root = ref<HTMLElement | null>(null)
useNavScope(root, {
  autofocus: true,
  back: () => {
    if (step.value !== 'controller') back()
  },
})
function onKey(e: KeyboardEvent) {
  if (e.code !== 'Escape' || index.value === 0) return
  e.preventDefault()
  back()
}
/** Each step focuses its own first element (`data-nav-initial`), else the main button. */
watch(step, async () => {
  await nextTick()
  const el =
    root.value?.querySelector<HTMLElement>('main [data-nav-initial]') ??
    root.value?.querySelector<HTMLElement>('footer [data-nav-initial]')
  el?.focus({ preventScroll: true })
})
watch(step, s => tester.listen(s === 'controller'))
watch(pads, () => void refresh())

// ---- Lifecycle ----
const unlisten: Array<() => void> = []
onMounted(async () => {
  window.addEventListener('keydown', onKey)
  try {
    await detectPhone()
    await language.load()
    pads.value = await emu.controls.pads()
  } catch (e) {
    fail(errorText(e))
  }
  unlisten.push(
    await emu.onGamepadConnected(p => {
      if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
    }),
    await emu.onGamepadDisconnected(p => {
      pads.value = pads.value.filter(x => x.id !== p.id)
    }),
    await emu.controls.onPadState(tester.onPadState),
    await emu.settings.onChanged(e => {
      if (e.key === 'pad.ab' || e.key === 'pad.remap') void tester.load()
    }),
  )
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
  unlisten.forEach(u => u())
  if (step.value === 'controller') void emu.controls.watch(false)
})
</script>

<template>
  <div
    ref="root"
    class="fixed inset-0 flex flex-col overflow-hidden text-ink"
    :class="phone ? 'bg-welcome-glow-phone text-lead' : 'bg-welcome-glow'"
  >
    <StepDots :steps="steps" :index="index" :phone="phone" />

    <main
      class="grid min-h-0 flex-1 place-items-center overflow-y-auto"
      :class="phone ? 'px-20' : 'px-16'"
    >
      <WelcomeStep
        v-if="step === 'welcome'"
        :phone="phone"
        :lang="language.lang.value"
        :lang-options="language.options.value"
        @lang="language.choose"
      />
      <GamesStep v-else-if="step === 'games'" :phone="phone" @error="fail" />
      <ControllerStep
        v-else-if="step === 'controller' && tester.pad.value"
        :phone="phone"
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
      @next="next"
      @back="back"
    />
  </div>
</template>
