<script setup lang="ts">
// S06 Ready: the game count, up to 3 covers, the optional imports (DeSmuME, Google Drive not
// built yet, another folder or emulator), and how to open the quick menu.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { FolderImport } from '../../emu'
import type { ImportState } from '../../lib/first-run/imports'
import { readyCovers } from '../../lib/first-run/library'
import { library } from '../../library'
import GameCover from '../GameCover.vue'
import ImportRow from './ImportRow.vue'
import ReadyRow from './ReadyRow.vue'
import StepLayout from './StepLayout.vue'
import StepTitle from './StepTitle.vue'

const props = defineProps<{
  phone: boolean
  imports: Record<FolderImport, ImportState>
  busy: boolean
}>()
const emit = defineEmits<{ pick: [what: FolderImport]; import: [what: FolderImport] }>()
const { t } = useI18n()

const covers = computed(() => readyCovers(library.games))
const count = computed(() => library.games.length)
const rowSize = computed(() => (props.phone ? ButtonSize.DefaultLead : ButtonSize.Default))
</script>

<template>
  <StepLayout centred wide :phone="phone">
    <StepTitle :phone="phone">{{ t('firstRun.ready.title') }}</StepTitle>
    <p class="m-0 text-lead text-ink-muted tabular-nums">
      {{ t('firstRun.ready.count', { n: count }, count) }}
    </p>
    <div v-if="covers.length" class="flex gap-14" aria-hidden="true">
      <!-- GameCover has no small size yet (Library stream): its own padding and type are
           overridden here, as before, until it has one. -->
      <GameCover
        v-for="g in covers"
        :key="g.id"
        class="p-8! text-[11px]/[14px]!"
        :class="phone ? 'w-[80px]!' : 'w-[96px]!'"
        :game="g"
        :badge="false"
      />
    </div>
    <div class="mt-8 self-stretch text-left text-overline text-ink-muted">
      {{ t('firstRun.ready.optional') }}
    </div>
    <div
      class="divide-y divide-line self-stretch overflow-hidden rounded-10 border border-line bg-surface text-left"
    >
      <ImportRow
        :name="t('firstRun.ready.desmume')"
        :hint="t('firstRun.ready.desmumeHint')"
        :state="imports.desmume"
        :busy="busy"
        :phone="phone"
        @pick="emit('pick', 'desmume')"
        @import="emit('import', 'desmume')"
      />
      <ReadyRow :name="t('firstRun.ready.drive')">
        <div class="text-caption text-ink-muted wrap-anywhere">
          {{ t('firstRun.ready.driveDesc') }}
        </div>
        <!-- Muted, not faint: the old scoped rules let the row's colour win over `.faint`, and
             that is what is on screen (for Cowork). -->
        <div class="text-caption text-ink-muted wrap-anywhere">{{ t('settingsUi.notYet') }}</div>
        <template #action>
          <Button
            :variant="ButtonVariant.SoftRow"
            :size="rowSize"
            disabled
            :title="t('settingsUi.notYet')"
          >
            {{ t('firstRun.ready.driveSignIn') }}
          </Button>
        </template>
      </ReadyRow>
      <ImportRow
        :name="t('firstRun.ready.other')"
        :hint="t('firstRun.ready.otherDesc')"
        :state="imports.any"
        :busy="busy"
        :phone="phone"
        @pick="emit('pick', 'any')"
        @import="emit('import', 'any')"
      />
    </div>
    <p class="m-0 text-ink-muted">
      {{ phone ? t('firstRun.ready.hintPhone') : t('firstRun.ready.hintDesktop') }}
    </p>
  </StepLayout>
</template>
