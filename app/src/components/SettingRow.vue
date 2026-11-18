<script setup lang="ts">
// One setting on a settings page (Settings board, SettingsParts + InheritedRow): name, description,
// the control for its kind, and its state at the page's level:
// - changed here: dot + «Modificato» + «Ripristina» (Tutti i giochi) or «Usa il valore di …»;
// - inherited: «Come in «…»» under the control, which stays usable (changing it sets this level);
// - locked (level not accepted, other device, no backend yet): faded, the description says why;
//   with `lockNote` a row only «Tutti i giochi» changes keeps its own description (the page's
//   note after the scope sentence says it once).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Folder, Undo2 } from '@lucide/vue'
import {
  emu,
  type SettingEntry,
  type SettingLevel,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
} from '../emu'
import { LOCALE } from '../i18n'
import { errorText } from '../i18n/labels'
import { lockReason, msg, notYetWired, optionNotYet } from '../settingsModel'
import { BUILTIN_BACKGROUNDS, SURROUND_CARDS } from '../surround'
import SettingSelect from './SettingSelect.vue'

const props = defineProps<{
  entry: SettingEntry
  row: SettingRow | undefined
  schema: SettingsSchema
  level: SettingLevel
  /** «Tutti i giochi», «Giochi Nintendo DS»…: the words of the scope bar for a level. */
  levelName: (level: SettingLevel) => string
  /** The page offers more than one level (scope bar shown). */
  scoped: boolean
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
  error: [string]
  /** A binding list (`pad.remap`, `pad.combos`): open the Controls section, where it is edited. */
  open: []
  /** A file row with a backend (`video.surroundImg`): open its picker at the page's scope. */
  choose: []
}>()

const { t, te } = useI18n()
const key = computed(() => props.entry.key)
const kind = computed(() => props.entry.kind)
const value = computed(() => props.row?.value ?? props.entry.default)

const version = ref('')
onMounted(async () => {
  if (key.value === 'about.ver') version.value = await emu.appVersion()
})

const reason = computed(() => lockReason(props.entry, props.schema, props.level))
const notYet = computed(() => notYetWired(props.entry))
const locked = computed(() => reason.value !== null)
const disabled = computed(() => locked.value || notYet.value)

/** Changed at this page's level (not merely different from the default). */
const changedHere = computed(() => !locked.value && props.row?.source === props.level)
const inheritedFrom = computed(() => {
  const s = props.row?.source
  if (!props.scoped || !s || s === 'default' || s === props.level) return null
  return props.levelName(s)
})
/** The level a reset falls back to: the next weaker one this entry accepts. */
const fallback = computed(() => {
  if (props.level === 'global') return null
  return props.entry.levels.includes('global') ? props.levelName('global') : null
})

const name = computed(() => {
  if (key.value === 'pad.dev1') return t('settings.pad.dev1.name', { controllerName: '' })
  return t(msg.name(key.value))
})
const desc = computed(() => {
  const r = reason.value
  if (!r || (props.lockNote && r === 'settingsUi.lockedGlobal')) return baseDesc.value
  return `${t(r)} ${baseDesc.value}`.trim()
})
/** A file row with no file chosen says so beside its button («Nessuna immagine scelta»). */
const noFile = computed(() =>
  kind.value.type === 'file' && !value.value && te(`settings.${key.value}.value.none`)
    ? t(`settings.${key.value}.value.none`)
    : '',
)
// Rows whose description shows live data the app does not have yet get their neutral form.
const baseDesc = computed(() => {
  const k = key.value
  if (k.startsWith('bios.') && te(`settings.${k}.status.builtin`))
    return t(`settings.${k}.status.builtin`)
  switch (k) {
    case 'cloud.acct':
      return t('settings.cloud.acct.value.signedOut')
    case 'cloud.space':
    case 'about.upd':
      return t('settingsUi.notYet')
    case 'about.ver':
      return t('settings.about.ver.desc', { version: version.value })
    default:
      return t(msg.desc(k))
  }
})
const buttonText = computed(() => {
  const k = key.value
  if (k === 'cloud.acct') return t('settings.cloud.acct.value.signIn')
  if (k.startsWith('bios.') && te(`settings.${k}.buttons.use`))
    return t(`settings.${k}.buttons.use`)
  return te(msg.button(k)) ? t(msg.button(k)) : ''
})

/** The options this device offers (a choice may have some for another device only). */
const options = computed(() => {
  const k = kind.value
  if (k.type !== 'choice' && k.type !== 'multi' && k.type !== 'apply') return []
  const hidden = props.entry.hiddenOptions
  return k.options
    .filter(id => !hidden.includes(id))
    .map(id => ({
      id,
      // `{language}`: «Come il sistema (Italiano)» of `general.lang`; other options ignore it.
      text: t(msg.option(key.value, id), { language: t(`settings.general.lang.option.${LOCALE}`) }),
      desc: te(msg.optionDesc(key.value, id)) ? t(msg.optionDesc(key.value, id)) : '',
      /** Shown, not pickable yet (X05). */
      notYet: optionNotYet(props.entry, id),
    }))
})

function set(v: SettingValue) {
  if (!disabled.value) emit('set', v)
}

// ---- Multi (chips): the chosen ones in order; «Modifica l’elenco» opens every option as a toggle
const editingList = ref(false)
const chosen = computed(() => (Array.isArray(value.value) ? value.value : []))
function toggleChip(id: string) {
  const list = chosen.value.includes(id)
    ? chosen.value.filter(x => x !== id)
    : options.value.map(o => o.id).filter(x => x === id || chosen.value.includes(x))
  set(list)
}

// ---- Slider
const sliderText = computed(() => {
  const k = kind.value
  if (k.type !== 'slider') return ''
  return t(`settingsUi.unit.${k.unit}`, { v: Number(value.value ?? 0) })
})
const sliderPct = computed(() => {
  const k = kind.value
  if (k.type !== 'slider') return 0
  return ((Number(value.value ?? k.min) - k.min) / (k.max - k.min)) * 100
})

// ---- Folders (`library.folders`): through the library, which rescans and updates the setting
async function removeFolder(path: string) {
  try {
    await emu.library.removeFolder(path)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function addFolder() {
  try {
    await emu.library.chooseFolder()
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- Cards: preview swatches from the board (surround.ts holds the background ones)
const CARD_BG: Record<string, string> = {
  ...Object.fromEntries(
    Object.entries(SURROUND_CARDS).map(([id, bg]) => [`video.surround.${id}`, bg]),
  ),
  ...Object.fromEntries(
    Object.entries(BUILTIN_BACKGROUNDS).map(([id, bg]) => [`video.surroundSet.${id}`, bg]),
  ),
}
const HUES = ['#2f5d8a', '#4f6b2e', '#3d6b3a', '#8a3b2f', '#5b3f8c', '#245f7a']
function cardBg(id: string, i: number): string {
  return CARD_BG[`${key.value}.${id}`] ?? HUES[i % HUES.length]
}

const swatches = computed(() => (kind.value.type === 'colour' ? kind.value.swatches : []))
function swatchName(hex: string): string {
  const k = `settings.${key.value}.swatch.${hex.replace('#', '')}`
  return te(k) ? t(k) : hex
}
</script>

<template>
  <div class="row" :class="{ locked: disabled }">
    <div class="head">
      <div class="text">
        <div class="title">
          <span v-if="changedHere" class="dot" aria-hidden="true"></span>
          <span class="name">{{ name }}</span>
          <span v-if="changedHere" class="changed">
            {{ t('settingsUi.changed') }}
            <span class="sr">{{ t('settingsUi.changedSr') }}</span>
          </span>
          <span v-if="entry.perDevice" class="tag">
            {{ t(`settingsUi.onlyHere.${schema.device}`) }}
          </span>
        </div>
        <div class="desc">
          {{ sub ?? (notYet && !reason ? `${desc} ${t('settingsUi.notYet')}` : desc) }}
        </div>
      </div>

      <div class="control">
        <span v-if="inheritedFrom && !changedHere && !compact" class="from">
          {{ t('settingsUi.sameAs', { level: inheritedFrom }) }}
        </span>
        <button v-if="changedHere && !compact" class="ghost" @click="emit('reset')">
          <Undo2 :size="14" :stroke-width="1.75" aria-hidden="true" />
          {{ fallback ? t('settingsUi.useValueOf', { level: fallback }) : t('settingsUi.reset') }}
        </button>

        <!-- toggle -->
        <button
          v-if="kind.type === 'toggle'"
          class="switch"
          :class="{ on: value === true }"
          role="switch"
          :aria-checked="value === true"
          :aria-label="name"
          :disabled="disabled"
          @click="set(value !== true)"
        >
          <span class="knob"></span>
        </button>

        <!-- choice: segments -->
        <div
          v-else-if="kind.type === 'choice' && kind.style === 'segments'"
          class="segments"
          role="radiogroup"
          :aria-label="name"
        >
          <button
            v-for="o in options"
            :key="o.id"
            class="seg"
            :class="{ on: o.id === value }"
            role="radio"
            :aria-checked="o.id === value"
            :disabled="disabled"
            @click="set(o.id)"
          >
            {{ o.text }}
          </button>
        </div>

        <!-- choice: list -->
        <SettingSelect
          v-else-if="kind.type === 'choice' && kind.style === 'list'"
          :label="name"
          :value="String(value ?? '')"
          :options="options"
          :disabled="disabled"
          @pick="set"
        />

        <!-- slider -->
        <div v-else-if="kind.type === 'slider'" class="slider">
          <input
            type="range"
            :min="kind.min"
            :max="kind.max"
            :step="kind.step"
            :value="Number(value ?? kind.min)"
            :aria-label="name"
            :aria-valuetext="sliderText"
            :disabled="disabled"
            :style="{ '--pct': `${sliderPct}%` }"
            @change="set(Number(($event.target as HTMLInputElement).value))"
          />
          <span class="value num">{{ sliderText }}</span>
        </div>

        <!-- text -->
        <input
          v-else-if="kind.type === 'text'"
          class="textfield"
          :aria-label="name"
          :value="String(value ?? '')"
          :placeholder="key === 'online.wfc' ? t('settings.online.wfc.value.none') : ''"
          :disabled="disabled"
          @change="set(($event.target as HTMLInputElement).value)"
        />

        <!-- binding list: edited in the Controls section -->
        <button
          v-else-if="kind.type === 'bindings'"
          class="btn"
          :disabled="disabled || compact"
          @click="emit('open')"
        >
          {{ t('settingsUi.openControls') }}
        </button>

        <!-- file, action -->
        <template v-else-if="kind.type === 'file' || kind.type === 'action'">
          <span v-if="noFile" class="file-state">{{ noFile }}</span>
          <button
            v-if="buttonText && !notYet"
            class="btn"
            :disabled="disabled"
            @click="emit('choose')"
          >
            {{ buttonText }}
          </button>
          <button v-else-if="buttonText" class="btn" disabled :title="t('settingsUi.notYet')">
            {{ buttonText }}
          </button>
        </template>
      </div>
    </div>

    <!-- multi (chips) -->
    <div v-if="kind.type === 'multi'" class="chips">
      <template v-if="!editingList">
        <span v-for="(id, i) in chosen" :key="id" class="chip">
          <span class="n num">{{ i + 1 }}</span>
          {{ t(msg.option(key, id)) }}
        </span>
        <button class="ghost accent" :disabled="disabled" @click="editingList = true">
          {{ buttonText }}
        </button>
      </template>
      <template v-else>
        <button
          v-for="o in options"
          :key="o.id"
          class="chip toggle"
          :class="{ on: chosen.includes(o.id) }"
          role="checkbox"
          :aria-checked="chosen.includes(o.id)"
          :disabled="disabled"
          @click="toggleChip(o.id)"
        >
          {{ o.text }}
        </button>
        <button class="ghost accent" @click="editingList = false">
          {{ t('settingsUi.done') }}
        </button>
      </template>
    </div>

    <!-- folders -->
    <div v-if="kind.type === 'folders'" class="folders">
      <div v-for="path in chosen" :key="path" class="folder">
        <Folder :size="16" :stroke-width="1.75" aria-hidden="true" />
        <span class="path">{{ path }}</span>
        <button class="ghost" :disabled="disabled" @click="removeFolder(path)">
          {{ t('settings.library.folders.value.remove') }}
        </button>
      </div>
      <button class="btn add" :disabled="disabled" @click="addFolder">{{ buttonText }}</button>
    </div>

    <!-- colour -->
    <div v-if="kind.type === 'colour'" class="swatches" role="radiogroup" :aria-label="name">
      <button
        v-for="c in swatches"
        :key="c"
        class="swatch"
        :class="{ on: (value || swatches[0]) === c }"
        role="radio"
        :aria-checked="(value || swatches[0]) === c"
        :aria-label="swatchName(c)"
        :title="swatchName(c)"
        :style="{ background: c }"
        :disabled="disabled"
        @click="set(c)"
      ></button>
    </div>

    <!-- choice: cards -->
    <div
      v-if="kind.type === 'choice' && kind.style === 'cards'"
      class="cards"
      role="radiogroup"
      :aria-label="name"
    >
      <button
        v-for="(o, i) in options"
        :key="o.id"
        class="card"
        :class="{ on: o.id === value, 'not-yet': o.notYet }"
        role="radio"
        :aria-checked="o.id === value"
        :disabled="disabled || o.notYet"
        :title="o.notYet ? t('settingsUi.notYet') : undefined"
        @click="set(o.id)"
      >
        <span class="thumb" aria-hidden="true" :style="{ background: cardBg(o.id, i) }"></span>
        <span class="card-title">{{ o.text }}</span>
        <span v-if="o.desc || o.notYet" class="card-desc">
          {{ o.notYet ? `${o.desc} ${t('settingsUi.notYet')}`.trim() : o.desc }}
        </span>
      </button>
    </div>

    <!-- apply (presets): cards that apply a bundle, nothing is ever selected -->
    <div v-if="kind.type === 'apply'" class="cards" role="group" :aria-label="name">
      <button
        v-for="(o, i) in options"
        :key="o.id"
        class="card"
        :disabled="disabled"
        :title="notYet ? t('settingsUi.notYet') : undefined"
      >
        <span class="thumb" aria-hidden="true" :style="{ background: cardBg(o.id, i) }"></span>
        <span class="card-title">{{ o.text }}</span>
        <span v-if="o.desc" class="card-desc">{{ o.desc }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.file-state {
  font-size: 12px;
  line-height: 20px;
  padding: 0 8px;
  border-radius: 999px;
  background: var(--surface-hover);
  color: var(--ink-muted);
  white-space: nowrap;
}
.row {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 16px;
  border-top: 1px solid var(--line);
}
.row:first-child {
  border-top: 0;
}
.row.locked .text,
.row.locked .chips,
.row.locked .cards,
.row.locked .swatches,
.card.not-yet {
  opacity: 0.55;
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}
.text {
  flex: 1 1 300px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.title {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.name {
  font-weight: 500;
}
.row.locked .name {
  color: var(--ink-muted);
}
.dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--accent-text);
}
.changed {
  font-size: 12px;
  line-height: 16px;
  color: var(--accent-text);
}
.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}
.tag {
  font-size: 11px;
  line-height: 16px;
  padding: 0 6px;
  border-radius: 4px;
  background: var(--surface-raised);
  color: var(--ink-muted);
}
.desc {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.control {
  flex: 0 1 auto;
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.from {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.ghost {
  height: 28px;
  padding: 0 8px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink-muted);
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.ghost:hover:not(:disabled) {
  background: var(--surface-hover);
}
.ghost.accent {
  color: var(--accent-text);
  font-weight: 500;
}
.switch {
  flex: 0 0 auto;
  width: 36px;
  height: 20px;
  border-radius: 999px;
  padding: 2px;
  display: flex;
  justify-content: flex-start;
  border: 1px solid var(--line-strong);
  background: var(--surface-raised);
}
.switch.on {
  justify-content: flex-end;
  border: 0;
  background: linear-gradient(var(--accent-crown), var(--accent-deep));
  box-shadow: 0 0 12px color-mix(in srgb, var(--violet) 45%, transparent);
}
.knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fff;
}
.segments {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.seg {
  height: 28px;
  padding: 0 10px;
  border: 0;
  border-radius: 4px;
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
}
.seg.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  font-weight: 600;
}
.slider {
  display: flex;
  align-items: center;
  gap: 12px;
}
.slider input {
  width: 200px;
  height: 20px;
  margin: 0;
  appearance: none;
  background: none;
}
.slider input::-webkit-slider-runnable-track {
  height: 4px;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: linear-gradient(
    to right,
    var(--accent-text) var(--pct),
    var(--surface-raised) var(--pct)
  );
}
.slider input::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  margin-top: -7px;
  border-radius: 999px;
  background: #fff;
  box-shadow: 0 1px 3px #00000080;
}
.value {
  min-width: 64px;
  text-align: right;
  font-weight: 600;
}
.textfield {
  width: 200px;
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  border: 1px solid var(--line-strong);
  background: var(--surface-raised);
  color: var(--ink);
  font: inherit;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.chip {
  height: 28px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
}
.chip .n {
  color: var(--ink-muted);
}
.chip.toggle.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
  color: var(--accent-text);
}
.folders {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.folder {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
}
.path {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.add {
  align-self: flex-start;
}
.swatches {
  display: flex;
  gap: 10px;
}
.swatch {
  width: 28px;
  height: 28px;
  border-radius: 999px;
  border: 0;
}
.swatch.on {
  outline: 2px solid var(--ink);
  outline-offset: 3px;
}
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}
.card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
  border-radius: var(--radius-10);
  text-align: left;
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.card.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
}
.thumb {
  display: block;
  height: 56px;
  border-radius: var(--radius-6);
  border: 1px solid var(--line);
}
.card-title {
  font-weight: 600;
}
.card-desc {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
button:disabled {
  cursor: not-allowed;
}
</style>
