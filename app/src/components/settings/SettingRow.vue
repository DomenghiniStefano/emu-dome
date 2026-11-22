<script setup lang="ts">
// One setting on a settings page (Settings board, SettingsParts + InheritedRow): name, description,
// the control for its kind, and its state at the page's level:
// - changed here: dot + «Modificato» + «Ripristina» (Tutti i giochi) or «Usa il valore di …»;
// - inherited: «Come in «…»», and the control stays usable (changing it sets this level);
//   both marks sit under the description, so the control keeps its place on the right;
// - locked (level not accepted, other device, no backend yet): faded, the description says why;
//   with `lockNote` a row only «Tutti i giochi» changes keeps its own description (the page's
//   note after the scope sentence says it once).
// Each kind's control is its own component in settings/; the state is settingRowModel.ts's, the
// words useSettingRow's. The quick menu and the Controls section render rows too: these props
// and events are theirs as well.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Undo2 } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type {
  ConsoleId,
  SettingEntry,
  SettingLevel,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '@/emu'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { FILE_PICKERS, ROW_ACTIONS, msg } from '@/lib/settingsModel'
import SettingSelect from './SettingSelect.vue'
import SettingCards from './SettingCards.vue'
import SettingChips from './SettingChips.vue'
import SettingDateTime from './SettingDateTime.vue'
import SettingFileAction from './SettingFileAction.vue'
import SettingFolders from './SettingFolders.vue'
import SettingRowText from './SettingRowText.vue'
import SettingSegments from './SettingSegments.vue'
import SettingSlider from './SettingSlider.vue'
import SettingSwatches from './SettingSwatches.vue'
import SettingTextField from './SettingTextField.vue'
import SettingToggle from './SettingToggle.vue'
import { rowActionIcon } from './rowActionIcon'
import { chosenList } from './settingRowModel'
import { useSettingRow } from './useSettingRow'

const props = defineProps<{
  entry: SettingEntry
  row: SettingRow | undefined
  schema: SettingsSchema
  level: SettingLevel
  /** «Tutti i giochi», «Giochi Nintendo DS»…: the words of the scope bar for a level. */
  levelName: (level: SettingLevel) => string
  /** The page offers more than one level (scope bar shown). */
  scoped: boolean
  /** The console of the scope being edited (none at «Tutti i giochi»). */
  console?: ConsoleId | null
  /** Quick menu: no per-row reset or «Come in …» (the bar at the bottom handles the changes). */
  compact?: boolean
  /** Shown instead of the description (T01 «Usate più spesso»: the page the setting is on). */
  sub?: string
  /** The page says once why its «Tutti i giochi» rows are grey (settings pages, not the menu). */
  lockNote?: boolean
}>()
const emit = defineEmits<{
  set: [SettingValue]
  reset: []
  /** A binding list (`pad.remap`, `pad.combos`): open the Controls section, where it is edited. */
  open: []
  /** A file row with a backend (`video.surroundImg`): open its picker at the page's scope; a
   * link row the page opens (`about.lic`): open it. */
  choose: []
  /** A screen layout (`screens.custom`): open the C09 screens editor at the page's scope; the
   * touch controls (`touch.tedit`): the C06 editor for the page's console. */
  edit: []
  /** The Google account (`cloud.acct`, #31): open the Cloud section, where it is signed in / out. */
  openCloud: []
}>()

const { t } = useI18n()
const failWith = useErrorSink()
const toasts = useToasts()
const { key, value, state, name, desc, noFile, fileName, buttonText, options } =
  useSettingRow(props)
const kind = computed(() => props.entry.kind)
/** The icon on the row's own button, by its kind (#104). */
const actionIcon = computed(() => rowActionIcon(kind.value))
const disabled = computed(() => state.value.disabled)
/** Choice cards and a folder list take the card's whole width when the rows sit in columns
 * (RowColumns). */
const wide = computed(
  () =>
    (kind.value.type === 'choice' && kind.value.style === 'cards') ||
    kind.value.type === 'apply' ||
    kind.value.type === 'folders',
)
/** A screen layout is edited for one two-screen console (or one of its games) at a time. */
const twoScreens = computed(() => props.console === 'nds' || props.console === '3ds')

function set(v: SettingValue) {
  if (!disabled.value) emit('set', v)
}

/** A file row with its own picker (FILE_PICKERS) opens it here, an action row with a backend
 * (ROW_ACTIONS) runs it here, the others let the page do it. */
async function choose() {
  if (key.value === 'cloud.acct') return emit('openCloud')
  const act = ROW_ACTIONS[key.value]
  if (act) {
    if (disabled.value) return
    try {
      const done = await act()
      if (done) toasts.notify(t(done.key, done.params ?? {}))
    } catch (e) {
      failWith(e)
    }
    return
  }
  const pick = FILE_PICKERS[key.value]
  if (!pick) return emit('choose')
  if (disabled.value) return
  try {
    await pick()
  } catch (e) {
    failWith(e)
  }
}
</script>

<template>
  <div
    class="flex flex-col gap-8 border-t border-line px-12 py-6 first:border-t-0"
    :class="{ '[column-span:all]': wide }"
  >
    <div class="flex flex-wrap items-center gap-x-12 gap-y-8">
      <SettingRowText
        :name="name"
        :desc="sub ?? desc"
        :changed-here="state.changedHere"
        :only-on="entry.perDevice ? schema.device : null"
        :locked="disabled"
      >
        <!-- the level marks under the description, so the control keeps its place at every level -->
        <span v-if="state.inheritedFrom && !state.changedHere && !compact" class="text-ink-muted">
          {{ t('settingsUi.sameAs', { level: levelName(state.inheritedFrom) }) }}
        </span>
        <Button
          v-if="state.changedHere && !compact"
          :variant="ButtonVariant.Text"
          :fade="ButtonFade.None"
          :size="ButtonSize.Bare"
          :icon="Undo2"
          @click="emit('reset')"
        >
          {{
            state.fallback
              ? t('settingsUi.useValueOf', { level: levelName(state.fallback) })
              : t('settingsUi.reset')
          }}
        </Button>
      </SettingRowText>

      <div class="flex flex-initial flex-wrap items-center justify-end gap-8">
        <SettingToggle
          v-if="kind.type === 'toggle'"
          :label="name"
          :on="value === true"
          :disabled="disabled"
          @set="set"
        />
        <SettingSegments
          v-else-if="kind.type === 'choice' && kind.style === 'segments'"
          :label="name"
          :value="String(value ?? '')"
          :options="options"
          :disabled="disabled"
          @set="set"
        />
        <SettingSelect
          v-else-if="kind.type === 'choice' && kind.style === 'list'"
          :label="name"
          :value="String(value ?? '')"
          :options="options"
          :disabled="disabled"
          @pick="set"
        />
        <SettingSlider
          v-else-if="kind.type === 'slider'"
          :label="name"
          :value="Number(value ?? kind.min)"
          :min="kind.min"
          :max="kind.max"
          :step="kind.step"
          :value-text="t(`settingsUi.unit.${kind.unit}`, { v: Number(value ?? 0) })"
          :disabled="disabled"
          @set="set"
        />
        <SettingDateTime
          v-else-if="kind.type === 'dateTime'"
          :label="name"
          :value="String(value ?? '')"
          :min-year="kind.minYear"
          :max-year="kind.maxYear"
          :disabled="disabled"
          @set="set"
        />
        <SettingTextField
          v-else-if="kind.type === 'text'"
          :label="name"
          :value="String(value ?? '')"
          :placeholder="key === 'online.wfc' ? t('settings.online.wfc.value.none') : ''"
          :disabled="disabled"
          @set="set"
        />
        <!-- binding list: edited in the Controls section -->
        <Button
          v-else-if="kind.type === 'bindings'"
          :variant="ButtonVariant.Soft"
          :fade="ButtonFade.None"
          :disabled="disabled || compact"
          :icon="actionIcon"
          @click="emit('open')"
        >
          {{ t('settingsUi.openControls') }}
        </Button>
        <!-- screen layout: edited in the C09 screens editor -->
        <Button
          v-else-if="kind.type === 'screenLayout'"
          :variant="ButtonVariant.Soft"
          :fade="ButtonFade.None"
          :disabled="disabled || compact || !twoScreens"
          :icon="actionIcon"
          @click="emit('edit')"
        >
          {{ buttonText }}
        </Button>
        <!-- touch controls: edited in the C06 editor, for one console at a time -->
        <Button
          v-else-if="kind.type === 'touchLayout'"
          :variant="ButtonVariant.Soft"
          :fade="ButtonFade.None"
          :disabled="disabled || compact || !console"
          :icon="actionIcon"
          @click="emit('edit')"
        >
          {{ buttonText }}
        </Button>
        <SettingFileAction
          v-else-if="kind.type === 'file' || kind.type === 'action'"
          :text="buttonText"
          :icon="actionIcon"
          :file-state="fileName || noFile"
          :not-yet="state.notYet"
          :disabled="disabled"
          @choose="choose"
        />
        <SettingSwatches
          v-else-if="kind.type === 'colour'"
          :setting-key="key"
          :label="name"
          :swatches="kind.swatches"
          :value="typeof value === 'string' ? value : ''"
          :disabled="disabled"
          @set="set"
        />
      </div>
    </div>

    <SettingChips
      v-if="kind.type === 'multi'"
      :chosen="chosenList(value)"
      :options="options"
      :option-text="id => t(msg.option(key, id))"
      :edit-text="buttonText"
      :disabled="disabled"
      @set="set"
    />
    <SettingFolders
      v-if="kind.type === 'folders'"
      :folders="chosenList(value)"
      :add-text="buttonText"
      :disabled="disabled"
    />
    <SettingCards
      v-if="(kind.type === 'choice' && kind.style === 'cards') || kind.type === 'apply'"
      :setting-key="key"
      :label="name"
      :options="options"
      :apply="kind.type === 'apply'"
      :value="String(value ?? '')"
      :disabled="disabled"
      :not-yet="state.notYet"
      @set="set"
    />
  </div>
</template>
