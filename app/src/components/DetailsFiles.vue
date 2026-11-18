<script setup lang="ts">
// Game details › File e informazioni (L10, board «Dettagli» desktop): two cards.
// - Come appare: «Nome mostrato» (`library.setName`; blank goes back to the ROM's title, shown as
//   the field's placeholder), the cover with «Cambia…» (L12), «Nascondi dalla libreria».
// - File del gioco: where the file is on this device and its size, then region, version, game
//   code, the identity (SHA-1, or the title ID of a 3DS game) and the emulation engine.
// Not built yet (no backend): the board's Google Drive line and «Rimuovi dalla libreria…».
// Phone in portrait: the same cards, one per row.
//
// Controller / keyboard: the shell's nav scope; L12 opens its own.
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type LibraryGame, type SettingDevice } from '../emu'
import { errorText, sizeLabel } from '../i18n/labels'
import GameCover from './GameCover.vue'

const props = defineProps<{ game: LibraryGame; phone: boolean; device?: SettingDevice | null }>()
const emit = defineEmits<{
  /** «Cambia…»: L12. */
  cover: []
  error: [string]
}>()

const { t } = useI18n()

// ---- «Nome mostrato» ----
const name = ref(props.game.renamed ? props.game.title : '')
watch(
  () => [props.game.title, props.game.renamed] as const,
  ([title, renamed]) => (name.value = renamed ? title : ''),
)
async function saveName() {
  const now = name.value.trim()
  const before = props.game.renamed ? props.game.title : ''
  if (now === before) return
  try {
    await emu.library.setName(props.game.id, now)
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function setHidden(hidden: boolean) {
  try {
    await emu.library.setFlags(props.game.id, { hidden })
  } catch (e) {
    emit('error', errorText(e))
  }
}

const coverLine = computed(() => {
  switch (props.game.cover) {
    case 'generated':
      return t('library.cover.generatedDesc')
    case 'icon':
      return t('library.cover.icon')
    case 'user':
      return t('library.cover.user')
  }
  return ''
})

// ---- File del gioco ----
const onDevice = computed(() =>
  t(props.device === 'phone' ? 'details.files.onPhone' : 'details.files.onComputer'),
)
/** The hex part of the id: a SHA-1 (GB, GBC, GBA, NDS), or the 3DS title ID and version. */
const idHex = computed(() => props.game.id.slice(props.game.id.indexOf(':') + 1))
const is3ds = computed(() => props.game.console === '3ds')

interface Fact {
  label: string
  value: string
  mono?: boolean
}
const facts = computed<Fact[]>(() => {
  const g = props.game
  const out: Fact[] = []
  if (g.region)
    out.push({ label: t('details.files.region'), value: t(`details.region.${g.region}`) })
  if (g.version !== null)
    out.push({
      label: t('details.files.version'),
      // Header revision: 0 is the first release (1.0). The 3DS remaster version is a number.
      value: is3ds.value ? String(g.version) : `1.${g.version}`,
    })
  if (g.code) out.push({ label: t('details.files.code'), value: g.code })
  out.push({
    label: t(is3ds.value ? 'details.files.titleId' : 'details.files.hash'),
    value: is3ds.value ? idHex.value.slice(0, 16).toUpperCase() : idHex.value,
    mono: true,
  })
  out.push({ label: t('details.files.engine'), value: t(`details.files.core.${g.core}`) })
  return out
})
</script>

<template>
  <div class="files" :class="{ phone }">
    <div class="card">
      <h2 class="overline">{{ t('details.files.looksTitle') }}</h2>
      <label class="field">
        <span class="strong">{{ t('details.files.name') }}</span>
        <input
          v-model="name"
          class="input"
          type="text"
          maxlength="128"
          :placeholder="game.romTitle"
          @change="saveName"
          @keydown.enter="saveName"
        />
      </label>
      <div class="line">
        <GameCover class="mini-cover" :game="game" :badge="false" />
        <div class="grow">
          <div class="strong">{{ t('details.files.cover') }}</div>
          <div class="muted small">{{ coverLine }}</div>
        </div>
        <button class="btn" @click="emit('cover')">{{ t('details.files.coverChange') }}</button>
      </div>
      <div class="line">
        <span class="grow strong">{{ t('library.menu.hide') }}</span>
        <button
          class="switch"
          :class="{ on: game.hidden }"
          role="switch"
          :aria-checked="game.hidden"
          :aria-label="t('library.menu.hide')"
          @click="setHidden(!game.hidden)"
        >
          <span class="knob"></span>
        </button>
      </div>
    </div>

    <div class="card">
      <h2 class="overline">{{ t('details.files.fileTitle') }}</h2>
      <div class="stack">
        <span class="strong">{{ onDevice }}</span>
        <span class="muted small path">{{ game.path }} · {{ sizeLabel(game.size) }}</span>
      </div>
      <dl class="facts small">
        <template v-for="f in facts" :key="f.label">
          <dt>{{ f.label }}</dt>
          <dd :class="{ mono: f.mono }">{{ f.value }}</dd>
        </template>
      </dl>
    </div>
  </div>
</template>

<style scoped>
.files {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
}
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.overline {
  margin: 0;
}
.strong {
  font-weight: 500;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.input {
  height: 32px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  border: 1px solid var(--line-strong);
  background: var(--surface-raised);
  color: var(--ink);
  font: inherit;
}
.input::placeholder {
  color: var(--ink-muted);
}
.line {
  display: flex;
  align-items: center;
  gap: 12px;
}
.grow {
  flex: 1;
  min-width: 0;
}
.mini-cover {
  flex: 0 0 auto;
  width: 48px;
}
.stack {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.path {
  word-break: break-all;
}
.facts {
  margin: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 4px 16px;
  color: var(--ink-muted);
}
.facts dd {
  margin: 0;
}
.mono {
  font-family: ui-monospace, monospace;
  word-break: break-all;
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
  box-shadow: 0 0 12px #7c5cff73;
}
.knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fff;
}

/* Phone in portrait: one card per row. */
.files.phone {
  grid-template-columns: minmax(0, 1fr);
}
.phone .card {
  border-radius: var(--radius-16);
  padding: 14px;
}
.phone .input,
.phone .btn {
  min-height: 44px;
}
.phone .btn {
  padding: 0 14px;
  border-radius: var(--radius-10);
}
</style>
