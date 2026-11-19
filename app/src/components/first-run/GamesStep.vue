<script setup lang="ts">
// S02 Games: the library folders (`library.folders`) and the scan in the same step, with the
// games found per console.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Folder } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { emu } from '../../emu'
import { errorText } from '../../i18n/labels'
import { consoleTiles } from '../../lib/first-run/library'
import { chooseFolder, library } from '../../library'
import ScanBar from './ScanBar.vue'
import StepLayout from './StepLayout.vue'
import StepTitle from './StepTitle.vue'

const props = defineProps<{ phone: boolean }>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

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

const tiles = computed(() => consoleTiles(library.games))
const found = computed(() => library.progress?.found ?? 0)
const addSize = computed(() => (props.phone ? ButtonSize.TallLead : ButtonSize.Tall))
</script>

<template>
  <StepLayout wide :phone="phone">
    <StepTitle :phone="phone">{{ t('firstRun.games.title') }}</StepTitle>
    <p class="m-0 text-lead text-ink-muted">{{ t('firstRun.games.lead') }}</p>
    <ul
      v-if="library.folders.length"
      class="m-0 list-none divide-y divide-line overflow-hidden rounded-10 border border-line bg-surface p-0"
      :aria-label="t('firstRun.games.folders')"
    >
      <li
        v-for="f in library.folders"
        :key="f.path"
        class="flex items-center gap-10 px-16 py-12 text-ink-muted"
      >
        <Folder :size="16" :stroke-width="1.75" aria-hidden="true" />
        <span class="min-w-0 flex-1 truncate text-ink">{{ f.path }}</span>
        <Button
          :variant="ButtonVariant.GhostBody"
          :size="phone ? ButtonSize.SmallLead : ButtonSize.Small"
          :aria-label="t('firstRun.games.removeFolder', { path: f.path })"
          @click="removeFolder(f.path)"
        >
          {{ t('firstRun.games.remove') }}
        </Button>
      </li>
    </ul>
    <Button
      :size="addSize"
      class="self-start"
      :data-nav-initial="library.folders.length ? undefined : ''"
      @click="addFolder"
    >
      {{ t('firstRun.games.add') }}
    </Button>
    <template v-if="library.folders.length">
      <ScanBar :running="library.scanning" />
      <p v-if="library.scanning" class="m-0 text-ink-muted tabular-nums">
        {{ t('library.scan.found', { found }, found) }}
      </p>
      <div class="grid gap-10" :class="phone ? 'grid-cols-2' : 'grid-cols-4'">
        <div
          v-for="tile in tiles"
          :key="tile.id"
          class="rounded-10 border border-line bg-surface p-12"
        >
          <!-- 24 / 30 waits for Cowork (spec Decision 5). -->
          <div class="text-[24px]/[30px] font-semibold tabular-nums">{{ tile.count }}</div>
          <div class="text-caption text-ink-muted">{{ t(tile.label) }}</div>
        </div>
      </div>
      <p v-if="library.scanning" class="m-0 text-ink-muted">
        {{ t('firstRun.games.background') }}
      </p>
    </template>
  </StepLayout>
</template>
