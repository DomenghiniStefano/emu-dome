<script setup lang="ts">
// First run (Trello #35, boards PrimoAvvio and Tel-PrimoAvvio): four steps, shown once per
// device (App.vue asks `emu.firstRun.show`).
// - S01 Welcome: the language (`general.lang`, «Tutti i giochi»). No «Indietro».
// - S02 Games: the library folders (`library.folders`) with the scan in the same step.
// - S04 Controller: only while a controller is connected (the steps come from Rust,
//   `first_run::steps`); A/B layout (`pad.ab`) and a press test. The only step with «Salta».
// - S06 Ready: game count, up to 3 covers, optional imports (DeSmuME, Google Drive not built yet,
//   another folder or emulator), the quick menu hint.
// Controller: one nav scope; B goes back (not on S01, and not on S04, where B is a button to
// try); Esc does the same on the keyboard.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Folder } from '@lucide/vue'
import {
  emu,
  type ConsoleId,
  type FirstRunStep,
  type FolderImport,
  type FolderSaves,
  type PadInfo,
  type PadState,
  type Pos,
  type RemapRow,
} from '../emu'
import { library, chooseFolder } from '../library'
import { LOCALE } from '../i18n'
import { errorText, gamepadName } from '../i18n/labels'
import { buttonName, posName } from '../controls'
import { loadSchema } from '../settingsModel'
import { useNavScope } from '../nav'
import SettingSelect from './SettingSelect.vue'
import GameCover from './GameCover.vue'

const emit = defineEmits<{ done: []; error: [string] }>()
const { t } = useI18n()

// ---- Steps ----
const steps = ref<FirstRunStep[]>(['welcome', 'games', 'ready'])
const step = ref<FirstRunStep>('welcome')
const index = computed(() => Math.max(0, steps.value.indexOf(step.value)))
const isLast = computed(() => index.value === steps.value.length - 1)

/** The steps for the controllers connected now; a step that went away moves to the one now
 * in its place (the controller unplugged on S04 → S06). */
async function refreshSteps() {
  try {
    const next = await emu.firstRun.steps(pads.value.length > 0)
    if (!next.includes(step.value))
      step.value = next[Math.min(index.value, next.length - 1)] ?? 'ready'
    steps.value = next
  } catch (e) {
    emit('error', errorText(e))
  }
}

async function finish() {
  try {
    await emu.firstRun.done()
  } catch (e) {
    emit('error', errorText(e))
  }
  emit('done')
}

function next() {
  if (isLast.value) return void finish()
  step.value = steps.value[index.value + 1] ?? step.value
}
function back() {
  if (index.value > 0) step.value = steps.value[index.value - 1] ?? step.value
}

const nextLabel = computed(() =>
  index.value === 0 ? t('firstRun.start') : isLast.value ? t('firstRun.finish') : t('firstRun.next'),
)

// ---- Layout: phone portrait (Tel-PrimoAvvio), like the quick menu decides it ----
const phoneDevice = ref(false)
const portraitQuery = window.matchMedia('(orientation: portrait)')
const portrait = ref(portraitQuery.matches)
const onPortrait = (e: MediaQueryListEvent) => (portrait.value = e.matches)
const phone = computed(() => phoneDevice.value && portrait.value)

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
    root.value?.querySelector<HTMLElement>('.body [data-nav-initial]') ??
    root.value?.querySelector<HTMLElement>('.btn-primary')
  el?.focus({ preventScroll: true })
})

// ---- S01: language ----
const lang = ref('system')
const langOptions = computed(() =>
  ['system', 'it', 'en'].map(id => ({
    id,
    text: t(`settings.general.lang.option.${id}`, {
      language: t(`settings.general.lang.option.${LOCALE}`),
    }),
  })),
)
async function setLang(id: string) {
  lang.value = id
  try {
    await emu.settings.set('general.lang', { level: 'global' }, id)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- S02: folders and scan ----
async function addFolder() {
  const failed = await chooseFolder()
  if (failed) emit('error', failed)
}
async function removeFolder(path: string) {
  try {
    await emu.library.removeFolder(path)
  } catch (e) {
    emit('error', errorText(e))
  }
}
/** The board's four tiles: Game Boy and Game Boy Color share one. */
const TILES: { id: string; consoles: ConsoleId[]; label: string }[] = [
  { id: 'gba', consoles: ['gba'], label: 'console.gba' },
  { id: 'nds', consoles: ['nds'], label: 'console.nds' },
  { id: '3ds', consoles: ['3ds'], label: 'console.3ds' },
  { id: 'gb', consoles: ['gb', 'gbc'], label: 'library.added.gbAndColor' },
]
const tiles = computed(() =>
  TILES.map(tile => ({
    ...tile,
    count: library.games.filter(g => tile.consoles.includes(g.console)).length,
  })),
)

// ---- S04: controller ----
const pads = ref<PadInfo[]>([])
const pad = computed(() => pads.value[0] ?? null)
const ab = ref<'nintendo' | 'printed'>('nintendo')
const remap = ref<RemapRow[]>([])
const FACES: Pos[] = ['south', 'east', 'west', 'north']
/** The last press: what it is on the controller and what the game gets. */
const pressed = ref<{ which: string; button: string | null } | null>(null)
let prevHeld = new Set<Pos>()

async function loadPad() {
  try {
    const rows = await emu.settings.page('pad', { level: 'global' })
    ab.value = rows.find(r => r.key === 'pad.ab')?.value === 'printed' ? 'printed' : 'nintendo'
    if (pad.value)
      remap.value = (await emu.controls.bindings(pad.value.family, { level: 'global' })).remap
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function setAb(id: 'nintendo' | 'printed') {
  ab.value = id
  try {
    await emu.settings.set('pad.ab', { level: 'global' }, id)
  } catch (e) {
    emit('error', errorText(e))
  }
}
function onPadState(states: PadState[]) {
  const p = pad.value
  if (!p || step.value !== 'controller') return
  const held = new Set<Pos>(states.find(s => s.id === p.id)?.held ?? [])
  for (const pos of held) {
    if (prevHeld.has(pos)) continue
    const button = remap.value.find(r => r.source === pos)?.button
    pressed.value = {
      which: FACES.includes(pos)
        ? t(`firstRun.controller.which.${pos}`)
        : posName(p.family, pos, false),
      button: button ? buttonName(button) : null,
    }
  }
  prevHeld = held
}
watch(step, s => {
  pressed.value = null
  prevHeld = new Set()
  void emu.controls.watch(s === 'controller')
  if (s === 'controller') void loadPad()
})
watch(pads, () => void refreshSteps())

// ---- S06: ready ----
const covers = computed(() =>
  [...library.games]
    .filter(g => !g.hidden)
    .sort((a, b) => a.title.localeCompare(b.title))
    .slice(0, 3),
)
const imports = ref<Record<FolderImport, FolderSaves | null>>({ desmume: null, any: null })
const importedCount = ref<Record<FolderImport, number | null>>({ desmume: null, any: null })
const busy = ref(false)
async function pickFolder(what: FolderImport) {
  busy.value = true
  try {
    const found = await emu.saves.importFolder(
      what,
      library.games.map(g => g.path),
    )
    if (found) {
      imports.value = { ...imports.value, [what]: found }
      importedCount.value = { ...importedCount.value, [what]: null }
    }
  } catch (e) {
    emit('error', errorText(e))
  } finally {
    busy.value = false
  }
}
async function importAll(what: FolderImport) {
  const found = imports.value[what]
  if (!found) return
  busy.value = true
  try {
    const n = await emu.saves.importMany(found.matches)
    importedCount.value = { ...importedCount.value, [what]: n }
  } catch (e) {
    emit('error', errorText(e))
  } finally {
    busy.value = false
  }
}
function importLine(what: FolderImport, fallback: string): string {
  const found = imports.value[what]
  const done = importedCount.value[what]
  if (done !== null) return t('firstRun.ready.imported', done)
  if (!found) return fallback
  const where = t('firstRun.ready.found', { n: found.found, folder: found.folder }, found.found)
  if (found.found === 0) return where
  const matched = found.matches.length
  return `${where} · ${t('firstRun.ready.matched', { n: matched }, matched)}`
}
/** The row's button imports once a folder offered saves for library games, else picks one. */
const canImport = (what: FolderImport) =>
  importedCount.value[what] === null && (imports.value[what]?.matches.length ?? 0) > 0

// ---- Lifecycle ----
const unlisten: Array<() => void> = []
onMounted(async () => {
  window.addEventListener('keydown', onKey)
  portraitQuery.addEventListener('change', onPortrait)
  try {
    phoneDevice.value = (await loadSchema()).device === 'phone'
    const rows = await emu.settings.page('general', { level: 'global' })
    const v = rows.find(r => r.key === 'general.lang')?.value
    if (typeof v === 'string') lang.value = v
    pads.value = await emu.controls.pads()
  } catch (e) {
    emit('error', errorText(e))
  }
  unlisten.push(
    await emu.onGamepadConnected(p => {
      if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
    }),
    await emu.onGamepadDisconnected(p => {
      pads.value = pads.value.filter(x => x.id !== p.id)
    }),
    await emu.controls.onPadState(onPadState),
    await emu.settings.onChanged(e => {
      if (e.key === 'pad.ab' || e.key === 'pad.remap') void loadPad()
    }),
  )
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
  portraitQuery.removeEventListener('change', onPortrait)
  unlisten.forEach(u => u())
  if (step.value === 'controller') void emu.controls.watch(false)
})
</script>

<template>
  <div ref="root" class="first-run" :class="{ phone }">
    <ol class="dots" :aria-label="t('firstRun.steps')">
      <li
        v-for="(s, i) in steps"
        :key="s"
        class="dot"
        :class="{ current: i === index, past: i < index }"
        :aria-label="t(`firstRun.step.${s}`)"
        :aria-current="i === index ? 'step' : undefined"
      ></li>
    </ol>

    <main class="body">
      <!-- S01 -->
      <section v-if="step === 'welcome'" class="step center">
        <div class="logo" aria-hidden="true"></div>
        <h1 class="title">{{ t('firstRun.welcome.title') }}</h1>
        <p class="lead narrow">{{ t('firstRun.welcome.lead') }}</p>
        <div class="lang">
          <span>{{ t('firstRun.welcome.language') }}</span>
          <SettingSelect
            :label="t('settings.general.lang.name')"
            :value="lang"
            :options="langOptions"
            @pick="setLang"
          />
        </div>
      </section>

      <!-- S02 -->
      <section v-else-if="step === 'games'" class="step wide">
        <h1 class="title">{{ t('firstRun.games.title') }}</h1>
        <p class="lead">{{ t('firstRun.games.lead') }}</p>
        <ul v-if="library.folders.length" class="list" :aria-label="t('firstRun.games.folders')">
          <li v-for="f in library.folders" :key="f.path" class="folder">
            <Folder :size="16" :stroke-width="1.75" aria-hidden="true" />
            <span class="path">{{ f.path }}</span>
            <button
              class="ghost"
              :aria-label="t('firstRun.games.removeFolder', { path: f.path })"
              @click="removeFolder(f.path)"
            >
              {{ t('firstRun.games.remove') }}
            </button>
          </li>
        </ul>
        <button
          class="btn add"
          :data-nav-initial="library.folders.length ? undefined : ''"
          @click="addFolder"
        >
          {{ t('firstRun.games.add') }}
        </button>
        <template v-if="library.folders.length">
          <div
            class="bar"
            :class="{ running: library.scanning }"
            role="progressbar"
            :aria-label="t('firstRun.games.progress')"
            :aria-valuenow="library.scanning ? undefined : 100"
          >
            <span></span>
          </div>
          <p v-if="library.scanning" class="muted num">
            {{ t('library.scan.found', { found: library.progress?.found ?? 0 }, library.progress?.found ?? 0) }}
          </p>
          <div class="tiles">
            <div v-for="tile in tiles" :key="tile.id" class="tile">
              <div class="tile-n num">{{ tile.count }}</div>
              <div class="tile-label">{{ t(tile.label) }}</div>
            </div>
          </div>
          <p v-if="library.scanning" class="muted">{{ t('firstRun.games.background') }}</p>
        </template>
      </section>

      <!-- S04 -->
      <section v-else-if="step === 'controller' && pad" class="step wide">
        <h1 class="title">{{ t('firstRun.controller.title') }}</h1>
        <p class="lead">{{ t('firstRun.controller.lead', { name: gamepadName(pad) }) }}</p>
        <div class="cards" role="radiogroup" :aria-label="t('settings.pad.ab.name')">
          <button
            v-for="id in ['nintendo', 'printed'] as const"
            :key="id"
            class="card"
            :class="{ on: ab === id }"
            role="radio"
            :aria-checked="ab === id"
            :data-nav-initial="id === 'nintendo' ? '' : undefined"
            @click="setAb(id)"
          >
            <span class="card-name">{{ t(`settings.pad.ab.option.${id}`) }}</span>
            <span class="card-desc">{{ t(`firstRun.controller.${id}`) }}</span>
          </button>
        </div>
        <div class="tester" aria-live="polite">
          <span v-if="pressed?.button" class="glyph">{{ pressed.button }}</span>
          <span class="muted">
            <template v-if="!pressed">{{ t('firstRun.controller.press') }}</template>
            <i18n-t v-else-if="pressed.button" keypath="firstRun.controller.pressed" scope="global">
              <template #which>{{ pressed.which }}</template>
              <template #button>
                <strong class="ink">{{ pressed.button }}</strong>
              </template>
            </i18n-t>
            <template v-else>
              {{ t('firstRun.controller.unused', { which: pressed.which }) }}
            </template>
          </span>
        </div>
      </section>

      <!-- S06 -->
      <section v-else-if="step === 'ready'" class="step center ready">
        <h1 class="title">{{ t('firstRun.ready.title') }}</h1>
        <p class="lead num">
          {{ t('firstRun.ready.count', { n: library.games.length }, library.games.length) }}
        </p>
        <div v-if="covers.length" class="covers" aria-hidden="true">
          <GameCover v-for="g in covers" :key="g.id" class="cover" :game="g" :badge="false" />
        </div>
        <div class="optional overline">{{ t('firstRun.ready.optional') }}</div>
        <div class="list rows">
          <div class="row">
            <div class="row-text">
              <div class="row-name">{{ t('firstRun.ready.desmume') }}</div>
              <div class="row-desc">
                {{ importLine('desmume', t('firstRun.ready.desmumeHint')) }}
              </div>
            </div>
            <button
              v-if="canImport('desmume')"
              class="btn"
              :disabled="busy"
              @click="importAll('desmume')"
            >
              {{
                t(
                  'firstRun.ready.import',
                  { n: imports.desmume?.matches.length ?? 0 },
                  imports.desmume?.matches.length ?? 0,
                )
              }}
            </button>
            <button
              v-else
              class="btn"
              :disabled="busy"
              @click="pickFolder('desmume')"
            >
              {{ t('library.chooseFolder') }}
            </button>
          </div>
          <div class="row">
            <div class="row-text">
              <div class="row-name">{{ t('firstRun.ready.drive') }}</div>
              <div class="row-desc">{{ t('firstRun.ready.driveDesc') }}</div>
              <div class="row-desc faint">{{ t('settingsUi.notYet') }}</div>
            </div>
            <button class="btn" disabled :title="t('settingsUi.notYet')">
              {{ t('firstRun.ready.driveSignIn') }}
            </button>
          </div>
          <div class="row">
            <div class="row-text">
              <div class="row-name">{{ t('firstRun.ready.other') }}</div>
              <div class="row-desc">{{ importLine('any', t('firstRun.ready.otherDesc')) }}</div>
            </div>
            <button
              v-if="canImport('any')"
              class="btn"
              :disabled="busy"
              @click="importAll('any')"
            >
              {{
                t(
                  'firstRun.ready.import',
                  { n: imports.any?.matches.length ?? 0 },
                  imports.any?.matches.length ?? 0,
                )
              }}
            </button>
            <button v-else class="btn" :disabled="busy" @click="pickFolder('any')">
              {{ t('library.chooseFolder') }}
            </button>
          </div>
        </div>
        <p class="muted">
          {{ phone ? t('firstRun.ready.hintPhone') : t('firstRun.ready.hintDesktop') }}
        </p>
      </section>
    </main>

    <footer class="foot">
      <button v-if="phone" class="btn btn-primary main" data-nav-initial @click="next">{{ nextLabel }}</button>
      <div class="side">
        <button v-if="index > 0" class="btn big" @click="back">{{ t('firstRun.back') }}</button>
        <span v-else-if="!phone"></span>
        <div class="right">
          <button v-if="step === 'controller'" class="btn big" @click="next">
            {{ t('firstRun.skip') }}
          </button>
          <button v-if="!phone" class="btn btn-primary" data-nav-initial @click="next">{{ nextLabel }}</button>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.first-run {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: var(--ink);
  background:
    radial-gradient(60% 50% at 50% 0%, #7c5cff40 0%, transparent 70%),
    var(--bg);
}
.first-run.phone {
  background:
    radial-gradient(90% 40% at 50% 0%, #7c5cff40 0%, transparent 70%),
    var(--bg);
  font-size: 15px;
  line-height: 22px;
}

/* Dots: the current one long, the past ones half accent. */
.dots {
  height: 72px;
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex: none;
}
.phone .dots {
  height: 88px;
  align-items: flex-end;
  padding-bottom: 12px;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #ffffff33;
}
.dot.past {
  background: #a593ff88;
}
.dot.current {
  width: 24px;
  background: var(--accent-text);
}

.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  place-items: center;
  padding: 0 16px;
}
.step {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin: 16px 0;
}
.step.center {
  align-items: center;
  text-align: center;
}
.step.wide {
  width: min(560px, 100%);
}
.step.ready {
  width: min(560px, 100%);
}
.phone .body {
  padding: 0 20px;
}
.phone .step.wide,
.phone .step.ready {
  width: 100%;
}
.logo {
  width: 72px;
  height: 72px;
  border-radius: 20px;
  background: radial-gradient(circle at 50% 30%, #a996ff, #664bd1);
  box-shadow: 0 0 60px #7c5cff59;
}
.title {
  margin: 0;
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.phone .title {
  font-size: 26px;
  line-height: 32px;
}
.lead {
  margin: 0;
  font-size: 15px;
  line-height: 22px;
  color: var(--ink-muted);
}
.lead.narrow {
  max-width: 520px;
}
.lang {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--ink-muted);
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}
.faint {
  color: var(--ink-faint);
}
.ink {
  color: var(--ink);
}

/* S02 */
.list {
  margin: 0;
  padding: 0;
  list-style: none;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.folder {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  color: var(--ink-muted);
}
.folder + .folder {
  border-top: 1px solid var(--line);
}
.path {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ink);
}
.ghost {
  height: 28px;
  padding: 0 8px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink-muted);
}
.ghost:hover {
  background: var(--surface-hover);
}
.btn.add,
.btn.big {
  height: 40px;
  padding: 0 16px;
}
.btn.add {
  align-self: flex-start;
}
.bar {
  height: 6px;
  border-radius: 999px;
  background: var(--surface-raised);
  overflow: hidden;
}
.bar span {
  display: block;
  width: 100%;
  height: 100%;
  background: var(--accent-text);
}
/* No total is known while walking the folders: an honest moving band, full when done. */
.bar.running span {
  width: 30%;
  animation: sweep 1.4s var(--ease-out) infinite;
}
@keyframes sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(340%);
  }
}
@media (prefers-reduced-motion: reduce) {
  .bar.running span {
    animation: none;
    width: 100%;
    opacity: 0.5;
  }
}
.tiles {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}
.phone .tiles {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.tile {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 12px;
}
.tile-n {
  font-size: 24px;
  line-height: 30px;
  font-weight: 600;
}
.tile-label {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}

/* S04 */
.cards {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}
.phone .cards {
  grid-template-columns: minmax(0, 1fr);
}
.card {
  padding: 14px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  gap: 6px;
  text-align: left;
}
.card.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
}
.card-name {
  font-weight: 600;
}
.card-desc {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.tester {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 14px;
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 70px;
}
.glyph {
  flex: none;
  width: 40px;
  height: 40px;
  border-radius: 999px;
  background: var(--accent-soft);
  display: grid;
  place-items: center;
  font-weight: 600;
  color: var(--accent-text);
}

/* S06 */
.covers {
  display: flex;
  gap: 14px;
}
.cover {
  width: 96px;
  font-size: 11px;
  line-height: 14px;
  padding: 8px;
}
.phone .cover {
  width: 80px;
}
.optional {
  align-self: stretch;
  text-align: left;
  margin-top: 8px;
}
.rows {
  align-self: stretch;
  text-align: left;
}
.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
}
.row + .row {
  border-top: 1px solid var(--line);
}
.row-text {
  flex: 1;
  min-width: 0;
}
.row-name {
  font-weight: 500;
}
.row-desc {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
  overflow-wrap: anywhere;
}
.row .btn {
  flex: none;
}
.row .btn:disabled {
  opacity: 0.5;
  cursor: default;
}

/* Footer: back on the left, skip and the main action on the right; on the phone the main
   action first, full width, then back and skip side by side. */
.foot {
  flex: none;
  height: 96px;
  padding: 0 48px;
  display: flex;
  align-items: center;
}
.side {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.right {
  display: flex;
  gap: 8px;
}
.phone .foot {
  height: auto;
  padding: 12px 20px 32px;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
}
.phone .main {
  height: 52px;
}
.phone .right {
  display: contents;
}
.phone .side .btn {
  flex: 1;
  height: 48px;
}
</style>
