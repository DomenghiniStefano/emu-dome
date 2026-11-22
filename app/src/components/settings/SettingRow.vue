<script setup lang="ts">
// One setting on a settings page (Settings board, SettingsParts + InheritedRow): name, description,
// the control for its kind, and its state at the page's level:
// - changed here: dot + «Modificato» + «Ripristina» (Tutti i giochi) or «Usa il valore di …»;
// - inherited: «Come in «…»» under the control, which stays usable (changing it sets this level);
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
import { useErrorSink } from '@/composables/useToasts'
import { FILE_PICKERS, msg } from '@/lib/settingsModel'
import SettingSelect from './SettingSelect.vue'
import SettingCards from './SettingCards.vue'
import SettingChips from './SettingChips.vue'
import SettingFileAction from './SettingFileAction.vue'
import SettingFolders from './SettingFolders.vue'
import SettingRowText from './SettingRowText.vue'
import SettingSegments from './SettingSegments.vue'
import SettingSlider from './SettingSlider.vue'
import SettingSwatches from './SettingSwatches.vue'
import SettingTextField from './SettingTextField.vue'
import SettingToggle from './SettingToggle.vue'
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
  /** A file row with a backend (`video.surroundImg`): open its picker at the page's scope. */
  choose: []
  /** A screen layout (`screens.custom`): open the C09 screens editor at the page's scope. */
  edit: []
  /** The Google account (`cloud.acct`, #31): open the Cloud section, where it is signed in / out. */
  openCloud: []
}>()

const { t } = useI18n()
const failWith = useErrorSink()
const { key, value, state, name, desc, noFile, fileName, buttonText, options } =
  useSettingRow(props)
const kind = computed(() => props.entry.kind)
const disabled = computed(() => state.value.disabled)
/** A screen layout is edited for one two-screen console (or one of its games) at a time. */
const twoScreens = computed(() => props.console === 'nds' || props.console === '3ds')

function set(v: SettingValue) {
  if (!disabled.value) emit('set', v)
}

/** A file row with its own picker (FILE_PICKERS) opens it here, the others let the page do it. */
async function choose() {
  if (key.value === 'cloud.acct') return emit('openCloud')
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
  <div class="flex flex-col gap-12 border-t border-line px-16 py-12 first:border-t-0">
    <div class="flex flex-wrap items-center gap-12">
      <SettingRowText
        :name="name"
        :desc="sub ?? desc"
        :changed-here="state.changedHere"
        :only-on="entry.perDevice ? schema.device : null"
        :locked="disabled"
      />

      <div class="flex flex-initial flex-wrap items-center justify-end gap-12">
        <span
          v-if="state.inheritedFrom && !state.changedHere && !compact"
          class="text-caption text-ink-muted"
        >
          {{ t('settingsUi.sameAs', { level: levelName(state.inheritedFrom) }) }}
        </span>
        <Button
          v-if="state.changedHere && !compact"
          :variant="ButtonVariant.GhostRegular"
          :fade="ButtonFade.None"
          :size="ButtonSize.Small"
          @click="emit('reset')"
        >
          <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
          {{
            state.fallback
              ? t('settingsUi.useValueOf', { level: levelName(state.fallback) })
              : t('settingsUi.reset')
          }}
        </Button>

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
          @click="emit('edit')"
        >
          {{ buttonText }}
        </Button>
        <SettingFileAction
          v-else-if="kind.type === 'file' || kind.type === 'action'"
          :text="buttonText"
          :file-state="fileName || noFile"
          :not-yet="state.notYet"
          :disabled="disabled"
          @choose="choose"
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
    <SettingSwatches
      v-if="kind.type === 'colour'"
      :setting-key="key"
      :label="name"
      :swatches="kind.swatches"
      :value="typeof value === 'string' ? value : ''"
      :disabled="disabled"
      @set="set"
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
