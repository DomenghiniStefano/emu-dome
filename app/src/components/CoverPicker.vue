<script setup lang="ts">
// «Scegli la copertina» (board L12, Stati): generated, the ROM icon (only when the ROM has one),
// or the user's image (the backend opens the image picker; cancelling it keeps this open).
// Controller / keyboard: its own nav scope, B / Esc cancel.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus } from '@lucide/vue'
import { emu, type CoverKind, type LibraryGame } from '../emu'
import { errorText } from '../i18n/labels'
import { useNavScope } from '../nav'
import GameCover from './GameCover.vue'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

const kinds = computed<CoverKind[]>(() =>
  props.game.hasIcon ? ['generated', 'icon', 'user'] : ['generated', 'user'],
)
const choice = ref<CoverKind>(props.game.cover)
const busy = ref(false)
const error = ref<string | null>(null)

// Previews: the generated cover always, the others when the game already shows them.
const generated = computed<LibraryGame>(() => ({
  ...props.game,
  cover: 'generated',
  coverUrl: null,
}))
const iconUrl = computed(() => (props.game.cover === 'icon' ? props.game.coverUrl : null))
const userUrl = computed(() => (props.game.cover === 'user' ? props.game.coverUrl : null))

async function apply() {
  if (busy.value) return
  busy.value = true
  error.value = null
  try {
    if (await emu.library.setCover(props.game.id, choice.value)) emit('close')
  } catch (e) {
    error.value = errorText(e)
  } finally {
    busy.value = false
  }
}

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true, back: () => emit('close') })

function onKey(e: KeyboardEvent) {
  if (e.code === 'Escape') emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="scrim">
    <div
      ref="root"
      class="dialog glass-thick"
      role="dialog"
      aria-modal="true"
      :aria-label="t('library.cover.title', { title: game.title })"
    >
      <div class="heading">{{ t('library.cover.title', { title: game.title }) }}</div>
      <div
        class="choices"
        role="radiogroup"
        :aria-label="t('library.cover.title', { title: game.title })"
      >
        <button
          v-for="k in kinds"
          :key="k"
          class="choice"
          :class="{ on: choice === k, add: k === 'user' && !userUrl }"
          role="radio"
          :aria-checked="choice === k"
          :data-nav-initial="choice === k ? '' : undefined"
          @click="choice = k"
        >
          <GameCover v-if="k === 'generated'" class="preview" :game="generated" :badge="false" />
          <span v-else-if="k === 'icon'" class="preview plain">
            <img v-if="iconUrl" class="icon" :src="iconUrl" alt="" />
            <span v-else class="swatch" :style="{ background: game.colour }"></span>
          </span>
          <span v-else class="preview plain">
            <img v-if="userUrl" class="art" :src="userUrl" alt="" />
            <Plus v-else :size="20" :stroke-width="1.75" aria-hidden="true" />
          </span>
          <span class="name">{{ t(`library.cover.${k}`) }}</span>
          <span class="desc">{{ t(`library.cover.${k}Desc`) }}</span>
        </button>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <div class="actions">
        <button class="btn" @click="emit('close')">{{ t('library.cover.cancel') }}</button>
        <button class="btn btn-primary" :disabled="busy" @click="apply">
          {{ t('library.cover.use') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scrim {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: grid;
  place-items: center;
  padding: 16px;
  background: var(--scrim);
  backdrop-filter: blur(8px);
}
.dialog {
  width: min(640px, 100%);
  max-height: 100%;
  overflow: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.heading {
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.choices {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}
.choice {
  padding: 10px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  gap: 8px;
  text-align: left;
}
.choice.add {
  border-style: dashed;
  border-color: var(--line-strong);
}
.choice.on {
  background: var(--accent-soft);
  border: 1px solid var(--accent-text);
}
.preview {
  border-radius: var(--radius-6);
}
.plain {
  position: relative;
  aspect-ratio: 2 / 3;
  overflow: hidden;
  display: grid;
  place-items: center;
  background: var(--surface-raised);
  color: var(--ink-muted);
}
.choice.add .plain {
  background: none;
}
.swatch {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-6);
}
.icon {
  width: 48px;
  height: 48px;
  image-rendering: pixelated;
}
.art {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.name {
  font-weight: 500;
}
.desc {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.error {
  margin: 0;
  color: var(--danger);
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.actions .btn-primary {
  height: 36px;
  padding: 0 16px;
  font-size: 13px;
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
