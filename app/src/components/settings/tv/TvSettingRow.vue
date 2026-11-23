<script setup lang="ts">
// One setting in TV mode (#30): a large row with the name on the left, the value on the right,
// the description under the name while the row has the focus. The same row model as the desk's
// (useSettingRow, settingRowModel.ts): the level's state, the lock with its reason (X05), the
// options this device offers. How it changes is tvSettingRow.ts's:
// - ◀ ▶ (the `tv-step` event TvSettings sends the focused row) step a short choice, a switch, a
//   slider or a colour in place; a locked row takes them and does nothing;
// - A toggles a switch, opens the sheet of a choice (TvChoiceSheet), or runs the row's action as
//   the desk's button does (a file's picker, the Controls section, the screens editor);
// - X (a right click) resets a value changed at this level, as the desk's «Ripristina».
// A locked row stays focusable, faded, so its description can say why.
import { computed, nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import type {
  ConsoleId,
  SettingEntry,
  SettingLevel,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import { cn } from '@/design-system/lib/cn'
import { msg } from '@/lib/settingsModel'
import { chosenList, swatchKey, toggleChosen } from '@/components/settings/settingRowModel'
import { useRowChoose } from '@/components/settings/useRowChoose'
import { useSettingRow } from '@/components/settings/useSettingRow'
import TvChoiceSheet, { type SheetOption } from './TvChoiceSheet.vue'
import { opensSheet, stepOption, stepSlider, toggled, tvEdit } from './tvSettingRow'

const props = withDefaults(
  defineProps<{
    entry: SettingEntry
    row: SettingRow | undefined
    schema: SettingsSchema
    level: SettingLevel
    levelName: (level: SettingLevel) => string
    scoped: boolean
    console?: ConsoleId | null
    /** The page says once why its «Tutti i giochi» rows are grey (useSettingRow). */
    lockNote?: boolean
  }>(),
  { console: null, lockNote: true },
)
const emit = defineEmits<{
  set: [SettingValue]
  reset: []
  /** A binding list: open the Controls section. */
  open: []
  /** A row the page carries out (`video.surroundImg`'s picker, `about.lic`): useRowChoose. */
  choose: []
  /** A screen layout: open the C09 screens editor. */
  edit: []
  /** The Google account (`cloud.acct`): open the Cloud section. */
  openCloud: []
}>()

const { t, te } = useI18n()
const { key, value, state, name, desc, noFile, fileName, buttonText, options } =
  useSettingRow(props)
const kind = computed(() => props.entry.kind)
const edit = computed(() => tvEdit(props.entry))
const disabled = computed(() => state.value.disabled)
/** A screen layout is edited for one two-screen console (or one of its games) at a time. */
const twoScreens = computed(() => props.console === 'nds' || props.console === '3ds')

function set(v: SettingValue) {
  if (!disabled.value) emit('set', v)
}

// ---- The words of the value

function swatchName(hex: string): string {
  const k = swatchKey(key.value, hex)
  return te(k) ? t(k) : hex
}
const swatches = computed(() => (kind.value.type === 'colour' ? kind.value.swatches : []))
const chosenSwatch = computed(() =>
  typeof value.value === 'string' && value.value ? value.value : (swatches.value[0] ?? ''),
)
const optionText = (id: string) => options.value.find(o => o.id === id)?.text ?? id

const valueText = computed(() => {
  const k = kind.value
  const v = value.value
  switch (k.type) {
    case 'toggle':
      return t(v === true ? 'tv.value.on' : 'tv.value.off')
    case 'choice':
      return optionText(String(v ?? ''))
    case 'slider':
      return t(`settingsUi.unit.${k.unit}`, { v: Number(v ?? k.min) })
    case 'colour':
      return swatchName(chosenSwatch.value)
    case 'multi': {
      const list = chosenList(v)
      return list.length
        ? list.map(id => t(msg.option(key.value, id))).join(', ')
        : t('settingsUi.none')
    }
    case 'file':
      return fileName.value || noFile.value
    case 'bindings':
      return t('settingsUi.openControls')
    default:
      return buttonText.value
  }
})
/** The action of a file or action row beside its file's name («Scegli un’immagine…»). */
const actionText = computed(() =>
  kind.value.type === 'file' && buttonText.value !== valueText.value ? buttonText.value : '',
)
/** ◀ ▶ change the value in place (tvSettingRow.ts). */
const inPlace = computed(() => edit.value === 'toggle' || edit.value === 'step')
/** What A does, said by the arrow at the end: a sheet or an action follows (not on a locked
 * row, an action with no button, or a screen layout away from a two-screen console). */
const leadsOn = computed(() => {
  if (disabled.value) return false
  if (edit.value === 'sheet') return true
  if (edit.value !== 'run') return false
  if (kind.value.type === 'screenLayout') return twoScreens.value
  return kind.value.type === 'bindings' || buttonText.value !== ''
})

// ---- ◀ ▶, A, X

function step(e: CustomEvent<-1 | 1>) {
  if (edit.value !== 'toggle' && edit.value !== 'step') return
  e.preventDefault()
  if (disabled.value) return
  const dir = e.detail
  const k = kind.value
  const v = value.value
  if (k.type === 'toggle') return set(toggled(v))
  if (k.type === 'slider') {
    const next = stepSlider(Number(v ?? k.min), k, dir)
    if (next !== null) set(next)
    return
  }
  const list =
    k.type === 'colour' ? swatches.value.map(id => ({ id, notYet: false })) : options.value
  const current = k.type === 'colour' ? chosenSwatch.value : String(v ?? '')
  const next = stepOption(list, current, dir)
  if (next !== null) set(next)
}

const sheet = ref(false)
const self = ref<{ $el: HTMLElement } | null>(null)
function closeSheet() {
  sheet.value = false
  void nextTick(() => self.value?.$el.focus())
}

async function confirm() {
  if (disabled.value) return
  const k = kind.value
  if (k.type === 'toggle') return set(toggled(value.value))
  if (opensSheet(props.entry)) {
    sheet.value = true
    return
  }
  if (k.type === 'bindings') return emit('open')
  if (k.type === 'screenLayout') return twoScreens.value ? emit('edit') : undefined
  // A file or action row: as the desk's button does (useRowChoose).
  if (k.type === 'file' || k.type === 'action') await choose()
}
const choose = useRowChoose(
  () => key.value,
  () => disabled.value,
  { openCloud: () => emit('openCloud'), choose: () => emit('choose') },
)

function more() {
  if (state.value.changedHere) emit('reset')
}

// ---- The sheet

const sheetOptions = computed<SheetOption[]>(() => {
  if (kind.value.type === 'colour')
    return swatches.value.map(hex => ({ id: hex, text: swatchName(hex), swatch: hex }))
  const apply = kind.value.type === 'apply'
  return options.value.map(o => ({
    id: o.id,
    text: o.text,
    desc: o.notYet || apply ? `${o.desc} ${t('settingsUi.notYet')}`.trim() : o.desc,
    // Presets are not carried out yet (the desk's cards are never picked either).
    disabled: o.notYet || apply,
  }))
})
const sheetChosen = computed(() => {
  const k = kind.value
  if (k.type === 'multi') return chosenList(value.value)
  if (k.type === 'colour') return [chosenSwatch.value]
  if (k.type === 'apply') return []
  return [String(value.value ?? '')]
})
function pickInSheet(id: string) {
  if (kind.value.type !== 'multi') return set(id)
  set(
    toggleChosen(
      chosenList(value.value),
      options.value.map(o => o.id),
      id,
    ),
  )
}

/** Under the name while focused: the description, then where an inherited value comes from. */
const note = computed(() =>
  state.value.inheritedFrom && !state.value.changedHere
    ? `${desc.value} ${t('settingsUi.sameAs', { level: props.levelName(state.value.inheritedFrom) })}`.trim()
    : desc.value,
)
</script>

<template>
  <Button
    ref="self"
    :variant="ButtonVariant.TvRow"
    :size="ButtonSize.TvRow"
    :fade="ButtonFade.None"
    class="group"
    :data-tv-step="inPlace ? '' : undefined"
    :aria-label="`${name}: ${valueText}`"
    :aria-description="note"
    @click="confirm"
    @contextmenu.prevent="more"
    @tv-step="step"
  >
    <span
      class="flex min-w-0 flex-1 flex-col gap-4"
      :class="{ 'opacity-locked': disabled }"
      aria-hidden="true"
    >
      <span class="flex flex-wrap items-center gap-12">
        <span
          v-if="state.changedHere"
          class="size-8 flex-none rounded-full bg-accent-text"
          aria-hidden="true"
        ></span>
        <span class="font-medium" :class="{ 'text-ink-muted': disabled }">{{ name }}</span>
        <span v-if="state.changedHere" class="text-tv-hint text-accent-text">
          {{ t('settingsUi.changed') }}
        </span>
        <!-- The desk's badge words, in the TV hint's type (the badge's own is under 18 px). -->
        <span v-if="entry.perDevice" class="text-tv-hint text-ink-muted">
          {{ t(`settingsUi.onlyHere.${schema.device}`) }}
        </span>
      </span>
      <span class="hidden text-tv-hint text-ink-muted group-focus:block">{{ note }}</span>
    </span>

    <span
      class="flex max-w-[50%] flex-none items-center gap-12 text-ink-muted"
      :class="{ 'opacity-locked': disabled }"
      aria-hidden="true"
    >
      <!-- ◀ ▶ around the value while focused, where they change it (kept in place, hidden, on a
           locked row, so its value does not move). -->
      <ChevronLeft
        v-if="inPlace"
        :class="cn('invisible size-tv-glyph flex-none', !disabled && 'group-focus:visible')"
        :stroke-width="2.25"
      />
      <span
        v-if="kind.type === 'colour'"
        class="size-tv-glyph flex-none rounded-full border border-line"
        :style="{ background: chosenSwatch }"
      ></span>
      <span class="truncate text-ink group-focus:font-semibold">{{ valueText }}</span>
      <span v-if="actionText" class="truncate">{{ actionText }}</span>
      <ChevronRight
        v-if="inPlace"
        :class="cn('invisible size-tv-glyph flex-none', !disabled && 'group-focus:visible')"
        :stroke-width="2.25"
      />
      <ChevronRight v-else-if="leadsOn" class="size-tv-glyph flex-none" :stroke-width="2.25" />
    </span>
  </Button>

  <TvChoiceSheet
    v-if="sheet"
    :title="name"
    :desc="desc"
    :options="sheetOptions"
    :chosen="sheetChosen"
    :multi="kind.type === 'multi'"
    @pick="pickInSheet"
    @close="closeSheet"
  />
</template>
