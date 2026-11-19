<script setup lang="ts">
// V05, importing a save: 1. the file (recognised format), 2. the game it goes to (found from the
// file name, or chosen), the warning when it replaces an in-game save, «Annulla» and «Importa».
// The state lives in useSaveImport; this panel shows it and asks for the steps.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { GameSaves, ImportCandidate, LibraryGame } from '../../emu'
import { saveWhenLabel } from '../../i18n/labels'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import ImportStep from './ImportStep.vue'
import SaveWarning from './SaveWarning.vue'
import { Heading } from '@/components/ui/heading'

const props = defineProps<{
  candidate: ImportCandidate | null
  target: LibraryGame | null
  /** The game was found from the file name, not chosen. */
  targetFound: boolean
  targetSaves: GameSaves | null
  importing: boolean
}>()
const emit = defineEmits<{ pickFile: []; chooseGame: []; cancel: []; import: [] }>()

const { t } = useI18n()

const fileLine = computed(() =>
  props.candidate
    ? t(`saves.importer.format.${props.candidate.format}`, { file: props.candidate.fileName })
    : t('saves.importer.noFile'),
)
const gameLine = computed(() =>
  props.target
    ? t(props.targetFound ? 'saves.importer.gameFound' : 'saves.importer.gameChosen', {
        title: props.target.title,
        console: t(`console.${props.target.console}`),
      })
    : t('saves.importer.gameNone'),
)
/** The in-game save the import would replace, when the target has one. */
const replaced = computed(() => (props.target ? (props.targetSaves?.inGame ?? null) : null))
</script>

<template>
  <section class="flex flex-col gap-18">
    <div class="flex flex-col gap-4">
      <Heading>{{ t('saves.importer.title') }}</Heading>
      <p class="m-0 text-ink-muted">{{ t('saves.importer.intro') }}</p>
    </div>
    <Card class="gap-14">
      <ImportStep :n="1" :title="t('saves.importer.file')" :detail="fileLine" :done="!!candidate">
        <Button data-nav-initial @click="emit('pickFile')">
          {{ candidate ? t('saves.importer.change') : t('saves.importer.pick') }}
        </Button>
      </ImportStep>
      <ImportStep :n="2" :title="t('saves.importer.game')" :detail="gameLine" :done="!!target">
        <Button :disabled="!candidate" @click="emit('chooseGame')">
          {{ target ? t('saves.importer.otherGame') : t('saves.importer.chooseGame') }}
        </Button>
      </ImportStep>
      <SaveWarning v-if="replaced">
        {{
          t('saves.importer.replaceWarn', { when: saveWhenLabel(replaced.modified ?? Date.now()) })
        }}
      </SaveWarning>
      <div class="flex justify-end gap-8">
        <Button @click="emit('cancel')">{{ t('saves.importer.cancel') }}</Button>
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Large"
          :disabled="!candidate || !target || importing"
          @click="emit('import')"
        >
          {{ targetSaves?.inGame ? t('saves.importer.replace') : t('saves.importer.go') }}
        </Button>
      </div>
    </Card>
    <p class="m-0 text-caption text-ink-muted">{{ t('saves.importer.formats') }}</p>
  </section>
</template>
