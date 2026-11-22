<script setup lang="ts">
// Game details › «Versioni» (#102, mockup «Versioni regionali in libreria», owner 2026-11-22):
// shown when the game is one of several regional versions. One row per version (gameVersions.ts):
// its region's tag, its own title, region · play time · last played. The version on the
// library's tile says «In libreria»; the others offer «Gioca questa» (starts that version) and
// «Mostra questa in libreria» (pins it: the tile shows it whichever was played last).
//
// Controller / keyboard: the rows' buttons are plain buttons in the details' nav section.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { LabelledBlock } from '@/components/ui/labelled-block'
import { useErrorSink } from '@/composables/useToasts'
import { emu, type LibraryGame } from '@/emu'
import { lastPlayedLabel, playTimeLabel } from '@/i18n/labels'
import { library, libraryFamilies } from '@/lib/library'
import { regionTagKey, versionRows } from './gameVersions'

const props = defineProps<{ game: LibraryGame }>()
const emit = defineEmits<{
  /** «Gioca questa»: start that version. */
  play: [LibraryGame]
}>()

const { t } = useI18n()
const failWith = useErrorSink()

const rows = computed(() =>
  versionRows(libraryFamilies.value.byGame.get(props.game.id), library.games),
)

function meta(g: LibraryGame): string {
  const region = g.region ? t(`details.region.${g.region}`) : t('details.versions.noRegion')
  return g.lastPlayed === null
    ? t('details.versions.metaNew', { region })
    : t('details.versions.meta', {
        region,
        time: playTimeLabel(g.playTimeSecs),
        when: lastPlayedLabel(g.lastPlayed),
      })
}

/** «Mostra questa in libreria»: the families come back as `library://changed`. */
async function pin(g: LibraryGame) {
  try {
    await emu.library.pinVersion(g.id, true)
  } catch (e) {
    failWith(e)
  }
}
</script>

<template>
  <LabelledBlock v-if="rows.length > 1" :title="t('details.versions.title')">
    <Card :variant="CardVariant.List">
      <div
        v-for="r in rows"
        :key="r.game.id"
        class="flex flex-wrap items-center gap-12 border-t border-line px-16 py-8 first:border-t-0"
        :class="{ 'bg-accent-soft': r.inLibrary }"
      >
        <Badge :variant="BadgeVariant.Neutral" :size="BadgeSize.Tag">
          {{ t(regionTagKey(r.game.region)) }}
        </Badge>
        <div class="flex min-w-0 flex-1 flex-col">
          <span class="font-semibold wrap-anywhere">{{ r.game.title }}</span>
          <span class="text-caption text-ink-muted">{{ meta(r.game) }}</span>
        </div>
        <Badge v-if="r.inLibrary" :variant="BadgeVariant.Success" :size="BadgeSize.Pill">
          {{ t('details.versions.inLibrary') }}
        </Badge>
        <template v-else>
          <Button
            :variant="ButtonVariant.Soft"
            :size="ButtonSize.Small"
            :aria-disabled="!r.game.available || undefined"
            @click="emit('play', r.game)"
          >
            {{ t('details.versions.play') }}
          </Button>
          <Button
            :variant="ButtonVariant.GhostRegular"
            :size="ButtonSize.Small"
            @click="pin(r.game)"
          >
            {{ t('details.versions.pin') }}
          </Button>
        </template>
      </div>
    </Card>
  </LabelledBlock>
</template>
