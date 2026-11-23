<script setup lang="ts">
// «Altri dati › Screenshot › Apri» (S3): each game's screenshots as a grid (the cover picker's
// look, newest first), a shot picked with A or a click; «Elimina» asks first and deletes only the
// shots picked. A cover made from a screenshot is a copy of its own: it stays.
// Controller: the page's nav section; each shot is a box (A toggles it).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Trash2 } from '@lucide/vue'
import { emu, type GameId, type GameScreenshot, type StorageScreenshotCount } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { Checkbox } from '@/design-system/components/checkbox'
import { EmptyState } from '@/design-system/components/empty-state'
import { Heading, HeadingVariant } from '@/design-system/components/heading'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { dateTimeLabel } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { library } from '@/lib/library'
import StorageConfirm from './StorageConfirm.vue'
import { pickKey } from './storage'

const props = defineProps<{ games: StorageScreenshotCount[] }>()
const emit = defineEmits<{ changed: [] }>()
const { t } = useI18n()
const toasts = useToasts()
const failWith = useErrorSink()

interface GameShots {
  id: GameId
  shots: GameScreenshot[]
}
const groups = ref<GameShots[] | null>(null)
const picked = ref<Set<string>>(new Set())
const asking = ref(false)
const busy = ref(false)
const loads = latest()

async function load() {
  const fresh = loads.next()
  try {
    const all = await Promise.all(
      props.games.map(async g => ({ id: g.id, shots: await emu.library.screenshots(g.id) })),
    )
    if (fresh()) groups.value = all.filter(g => g.shots.length > 0)
  } catch (e) {
    if (fresh()) failWith(e)
  }
}
onMounted(load)

const titleOf = (id: GameId) =>
  library.games.find(x => x.id === id)?.title ?? t('storage.quick.unknownGame')
const count = computed(() => picked.value.size)

function toggle(key: string) {
  const next = new Set(picked.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  picked.value = next
}

async function remove() {
  asking.value = false
  busy.value = true
  let n = 0
  try {
    for (const g of groups.value ?? []) {
      const names = g.shots.filter(s => picked.value.has(pickKey(g.id, s.name))).map(s => s.name)
      if (names.length > 0) n += await emu.storage.deleteScreenshots(g.id, names)
    }
    toasts.notify(t('storage.shots.deleted', { n }, n))
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
  <div class="flex flex-col gap-12">
    <EmptyState v-if="groups && groups.length === 0" :title="t('storage.shots.none')" />
    <section v-for="(g, gi) in groups ?? []" :key="g.id" class="flex flex-col gap-6">
      <Heading as="h3" :variant="HeadingVariant.Overline">{{ titleOf(g.id) }}</Heading>
      <!-- As many columns as fit a small cover's width: three on a phone held upright. -->
      <div class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-cover-small),1fr))] gap-6">
        <Checkbox
          v-for="(s, si) in g.shots"
          :key="s.name"
          :model-value="picked.has(pickKey(g.id, s.name))"
          @update:model-value="toggle(pickKey(g.id, s.name))"
        >
          <Card
            as="button"
            :variant="CardVariant.PictureChoice"
            :selected="picked.has(pickKey(g.id, s.name))"
            :aria-label="t('library.cover.shots.taken', { date: dateTimeLabel(s.takenAt) })"
            :data-nav-initial="gi === 0 && si === 0 ? '' : undefined"
          >
            <img
              class="aspect-3/2 w-full rounded-badge bg-letterbox object-contain"
              :src="s.url"
              alt=""
              loading="lazy"
            />
          </Card>
        </Checkbox>
      </div>
    </section>
    <div v-if="groups && groups.length > 0" class="flex justify-end">
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :icon="Trash2"
        :disabled="count === 0 || busy"
        @click="asking = true"
      >
        {{ count === 0 ? t('storage.shots.deleteNone') : t('storage.shots.delete', { n: count }) }}
      </Button>
    </div>
    <StorageConfirm
      v-if="asking"
      :title="t('storage.shots.askTitle', { n: count }, count)"
      :text="t('storage.shots.askText')"
      :confirm="t('storage.shots.askConfirm')"
      :icon="Trash2"
      @confirm="remove"
      @cancel="asking = false"
    />
  </div>
</template>
