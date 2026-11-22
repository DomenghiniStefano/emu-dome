<script setup lang="ts">
// S06: an import of saves from a folder (DeSmuME's, or any other). Pick a folder, see what it
// holds, then import the saves that match library games.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderOpen, Import } from '@lucide/vue'
import { Button, ButtonFade } from '@/components/ui/button'
import { canImport, importLine, type ImportState } from './imports'
import ReadyRow from './ReadyRow.vue'

const props = defineProps<{
  name: string
  /** What the line says before a folder is picked. */
  hint: string
  state: ImportState
  busy: boolean
}>()
const emit = defineEmits<{ pick: []; import: [] }>()
const { t } = useI18n()

const line = computed(() => {
  const l = importLine(props.state)
  switch (l.kind) {
    case 'hint':
      return props.hint
    case 'imported':
      return t('firstRun.ready.imported', l.n)
    case 'found': {
      const where = t('firstRun.ready.found', { n: l.n, folder: l.folder }, l.n)
      return l.matched === null
        ? where
        : `${where} · ${t('firstRun.ready.matched', { n: l.matched }, l.matched)}`
    }
  }
  return props.hint
})
const matches = computed(() => props.state.found?.matches.length ?? 0)
</script>

<template>
  <ReadyRow :name="name">
    <div class="text-caption text-ink-muted wrap-anywhere">{{ line }}</div>
    <template #action>
      <Button
        v-if="canImport(state)"
        :fade="ButtonFade.Dim"
        :icon="Import"
        :disabled="busy"
        @click="emit('import')"
      >
        {{ t('firstRun.ready.import', { n: matches }, matches) }}
      </Button>
      <Button
        v-else
        :fade="ButtonFade.Dim"
        :icon="FolderOpen"
        :disabled="busy"
        @click="emit('pick')"
      >
        {{ t('library.chooseFolder') }}
      </Button>
    </template>
  </ReadyRow>
</template>
