<script setup lang="ts">
// The tab «Giochi su Google Drive» (Trello #31): every game of this library or of Drive, with
// where its file is and what to do with it (useCloudGames).
import { useI18n } from 'vue-i18n'
import { Card, CardVariant } from '@/components/ui/card'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import CloudGameRow from './CloudGameRow.vue'
import { cloud } from './cloudState'
import { useCloudGames } from './useCloudGames'

const { t } = useI18n()
const { busy, act } = useCloudGames()
</script>

<template>
  <template v-if="cloud.gamesLoaded">
    <Card v-if="cloud.games.length" :variant="CardVariant.List">
      <CloudGameRow
        v-for="g in cloud.games"
        :key="g.id"
        :game="g"
        :busy="busy.has(g.id)"
        @act="a => act(g, a)"
      />
    </Card>
    <EmptyState v-else as="div" :variant="EmptyStateVariant.Card" :text="t('cloud.games.empty')" />
  </template>
</template>
