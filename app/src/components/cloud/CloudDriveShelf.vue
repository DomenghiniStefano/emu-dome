<script setup lang="ts">
// The library's games that are only on Google Drive (Trello #31, mockup «Solo su Google Drive,
// nella libreria»): their covers faded with a dashed edge (GameCover `missing`), the one chosen
// with «Scarica e gioca» and the size it brings down. Shown while signed in and Drive has such
// games, under the grid (or alone, on a device with no games yet).
// Controller: the covers and the button are in the shell's scope; A on a cover chooses it.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Download } from '@lucide/vue'
import type { GameId, LibraryGame } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { LabelledBlock } from '@/components/ui/labelled-block'
import { Progress } from '@/components/ui/progress'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import { sizeLabel } from '@/i18n/labels'
import { coverOf, driveOnly, progressOf, transferShare } from './cloud'
import { cloud, failCloud, loadGames } from './cloudState'
import { useDriveDownload } from './useDriveDownload'

const emit = defineEmits<{ play: [LibraryGame] }>()
const { t } = useI18n()

const signedIn = computed(() => cloud.status?.signedIn ?? false)
const games = computed(() => (signedIn.value ? driveOnly(cloud.games) : []))

const picked = ref<GameId | null>(null)
const chosen = computed(() => games.value.find(g => g.id === picked.value) ?? games.value[0])

const { downloading, downloadAndPlay } = useDriveDownload(g => emit('play', g))
const transfer = computed(() =>
  chosen.value ? progressOf(cloud.progress, 'downloading', chosen.value.id) : null,
)

async function reload() {
  if (!signedIn.value) return
  try {
    await loadGames()
  } catch (e) {
    failCloud(e)
  }
}
onMounted(reload)
watch(() => [signedIn.value, cloud.status?.lastSync], reload)
</script>

<template>
  <LabelledBlock v-if="chosen" :title="t('cloud.drive.title')">
    <div class="flex flex-wrap items-end gap-12">
      <div class="flex min-w-0 flex-wrap gap-12">
        <Button
          v-for="g in games"
          :key="g.id"
          :variant="ButtonVariant.Pick"
          :size="ButtonSize.Area"
          class="rounded-card p-4"
          :aria-pressed="g.id === chosen.id"
          :aria-label="t('cloud.drive.pick', { title: g.title })"
          @click="picked = g.id"
        >
          <GameCover :game="coverOf(g, undefined)" :size="CoverSize.Small" missing />
        </Button>
      </div>
      <div class="flex min-w-0 flex-col gap-8">
        <span class="font-medium wrap-anywhere">{{ chosen.title }}</span>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Large"
          :disabled="downloading !== null"
          @click="downloadAndPlay(chosen)"
        >
          <Download :size="16" :stroke-width="2" aria-hidden="true" />
          {{ t('cloud.drive.play') }}
        </Button>
        <Progress
          v-if="downloading === chosen.id"
          class="w-progress max-w-full"
          :value="transferShare(transfer)"
          :label="t('cloud.drive.downloading', { title: chosen.title })"
        />
        <span class="text-caption text-ink-muted">{{
          downloading === chosen.id
            ? t('cloud.drive.downloading', { title: chosen.title })
            : t('cloud.drive.size', { size: sizeLabel(chosen.size) })
        }}</span>
      </div>
    </div>
  </LabelledBlock>
</template>
