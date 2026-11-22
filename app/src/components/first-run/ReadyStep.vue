<script setup lang="ts">
// S06 Ready: the game count, up to 3 covers, the optional imports (DeSmuME, Google Drive not
// built yet, another folder or emulator), and how to open the quick menu.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { LogIn } from '@lucide/vue'
import { Button, ButtonFade } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import type { FolderImport } from '@/emu'
import type { ImportState } from './imports'
import { readyCovers } from './library'
import { library } from '@/lib/library'
import GameCover from '@/components/cover/GameCover.vue'
import { CoverSize } from '@/components/cover/variants'
import ImportRow from './ImportRow.vue'
import ReadyRow from './ReadyRow.vue'
import StepLayout from './StepLayout.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'

defineProps<{
  phone: boolean
  imports: Record<FolderImport, ImportState>
  busy: boolean
}>()
const emit = defineEmits<{ pick: [what: FolderImport]; import: [what: FolderImport] }>()
const { t } = useI18n()

const covers = computed(() => readyCovers(library.games))
const count = computed(() => library.games.length)
</script>

<template>
  <StepLayout centred wide>
    <Heading as="h1" :variant="HeadingVariant.Display">{{ t('firstRun.ready.title') }}</Heading>
    <p class="m-0 text-lead text-ink-muted tabular-nums">
      {{ t('firstRun.ready.count', { n: count }, count) }}
    </p>
    <div v-if="covers.length" class="flex gap-12" aria-hidden="true">
      <GameCover v-for="g in covers" :key="g.id" :size="CoverSize.Small" :game="g" :badge="false" />
    </div>
    <Heading as="h2" :variant="HeadingVariant.Overline" class="mt-8 self-stretch text-left">
      {{ t('firstRun.ready.optional') }}
    </Heading>
    <Card :variant="CardVariant.List" class="divide-y divide-line self-stretch">
      <ImportRow
        :name="t('firstRun.ready.desmume')"
        :hint="t('firstRun.ready.desmumeHint')"
        :state="imports.desmume"
        :busy="busy"
        @pick="emit('pick', 'desmume')"
        @import="emit('import', 'desmume')"
      />
      <ReadyRow :name="t('firstRun.ready.drive')">
        <div class="text-caption text-ink-muted wrap-anywhere">
          {{ t('firstRun.ready.driveDesc') }}
        </div>
        <!-- Muted like the line above, not faint (Cowork: faint for «not yet»?). -->
        <div class="text-caption text-ink-muted wrap-anywhere">{{ t('settingsUi.notYet') }}</div>
        <template #action>
          <Button :fade="ButtonFade.Dim" :icon="LogIn" disabled :title="t('settingsUi.notYet')">
            {{ t('firstRun.ready.driveSignIn') }}
          </Button>
        </template>
      </ReadyRow>
      <ImportRow
        :name="t('firstRun.ready.other')"
        :hint="t('firstRun.ready.otherDesc')"
        :state="imports.any"
        :busy="busy"
        @pick="emit('pick', 'any')"
        @import="emit('import', 'any')"
      />
    </Card>
    <p class="m-0 text-ink-muted">
      {{ phone ? t('firstRun.ready.hintPhone') : t('firstRun.ready.hintDesktop') }}
    </p>
  </StepLayout>
</template>
