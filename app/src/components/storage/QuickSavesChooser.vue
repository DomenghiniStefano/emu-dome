<script setup lang="ts">
// «Altri dati › Salvataggi rapidi › Scegli» (S3, the owner's answer A): every game's quick saves,
// the heaviest games first, each slot with its box and its size; a game's own box picks all its
// slots. «Elimina» asks first and deletes only the quick saves picked: never a game's in-game
// save. The open game's are refused by Rust (`saveGameOpen`).
// Controller: the page's nav section; each box is a button (A toggles it).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Trash2 } from '@lucide/vue'
import { emu, type StorageQuickGame } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { Checkbox } from '@/design-system/components/checkbox'
import { EmptyState } from '@/design-system/components/empty-state'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { saveWhenLabel, sizeLabel } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { library } from '@/lib/library'
import { useSlotName } from '@/components/saves/useSlotName'
import StorageConfirm from './StorageConfirm.vue'
import { byWeight, gamePicked, pickKey, pickedBytes, pickedSlots, toggleGame } from './storage'

const emit = defineEmits<{ changed: [] }>()
const { t } = useI18n()
const toasts = useToasts()
const failWith = useErrorSink()
const slotName = useSlotName()

const games = ref<StorageQuickGame[] | null>(null)
const picked = ref<Set<string>>(new Set())
const asking = ref(false)
const busy = ref(false)
const loads = latest()

async function load() {
  const fresh = loads.next()
  try {
    const list = await emu.storage.quickSaves()
    if (fresh()) games.value = byWeight(list)
  } catch (e) {
    if (fresh()) failWith(e)
  }
}
onMounted(load)

const titleOf = (g: StorageQuickGame) =>
  library.games.find(x => x.id === g.id)?.title ?? t('storage.quick.unknownGame')
const chosen = computed(() => pickedSlots(games.value ?? [], picked.value))
const chosenBytes = computed(() => pickedBytes(games.value ?? [], picked.value))

function toggle(key: string) {
  const next = new Set(picked.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  picked.value = next
}

async function remove() {
  asking.value = false
  busy.value = true
  try {
    const n = await emu.storage.deleteQuick(chosen.value)
    toasts.notify(t('storage.quick.deleted', { n }, n))
  } catch (e) {
    failWith(e)
  } finally {
    busy.value = false
    picked.value = new Set()
    emit('changed')
    await load()
  }
}
</script>

<template>
  <div class="flex flex-col gap-8">
    <p class="m-0 text-ink-muted">{{ t('storage.quick.intro') }}</p>
    <EmptyState v-if="games && games.length === 0" :title="t('storage.quick.none')" />
    <Card
      v-for="(g, gi) in games ?? []"
      :key="g.id"
      :variant="CardVariant.List"
      as="ul"
      :aria-label="titleOf(g)"
    >
      <li>
        <Checkbox
          :model-value="gamePicked(g, picked)"
          @update:model-value="picked = toggleGame(g, picked)"
        >
          <Button
            :variant="ButtonVariant.Menu"
            :size="ButtonSize.MenuRow"
            class="w-full px-10 font-medium"
            :data-nav-initial="gi === 0 ? '' : undefined"
          >
            <span
              class="size-14 shrink-0 rounded-badge"
              :class="
                gamePicked(g, picked) === true ? 'bg-accent-text' : 'border border-line-strong'
              "
              aria-hidden="true"
            ></span>
            <span class="min-w-0 flex-1 truncate">{{ titleOf(g) }}</span>
          </Button>
        </Checkbox>
      </li>
      <li v-for="s in g.slots" :key="s.slot" class="border-t border-line">
        <Checkbox
          :model-value="picked.has(pickKey(g.id, s.slot))"
          @update:model-value="toggle(pickKey(g.id, s.slot))"
        >
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="w-full px-10">
            <span
              class="size-14 shrink-0 rounded-badge"
              :class="
                picked.has(pickKey(g.id, s.slot)) ? 'bg-accent-text' : 'border border-line-strong'
              "
              aria-hidden="true"
            ></span>
            <span class="min-w-0 flex-1 truncate">{{ slotName(s.slot) }}</span>
            <span class="flex-none text-caption text-ink-muted tabular-nums">
              {{
                s.savedAt !== null
                  ? t('storage.quick.slotLine', {
                      when: saveWhenLabel(s.savedAt),
                      size: sizeLabel(s.size),
                    })
                  : sizeLabel(s.size)
              }}
            </span>
          </Button>
        </Checkbox>
      </li>
    </Card>
    <div v-if="games && games.length > 0" class="flex justify-end">
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :icon="Trash2"
        :disabled="chosen.length === 0 || busy"
        @click="asking = true"
      >
        {{
          chosen.length === 0
            ? t('storage.quick.deleteNone')
            : t('storage.quick.delete', { n: chosen.length, size: sizeLabel(chosenBytes) })
        }}
      </Button>
    </div>
    <StorageConfirm
      v-if="asking"
      :title="t('storage.quick.askTitle', { n: chosen.length }, chosen.length)"
      :text="t('storage.quick.askText')"
      :confirm="t('storage.quick.askConfirm')"
      :icon="Trash2"
      @confirm="remove"
      @cancel="asking = false"
    />
  </div>
</template>
