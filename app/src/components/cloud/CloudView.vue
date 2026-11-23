<script setup lang="ts">
// The Cloud section (Trello #31, approved mockup «Cloud con Google Drive»). Before the sign-in:
// what the sync does and «Accedi con Google» (CloudSignedOut). Signed in (option A): the head
// with the account, the status card, then the tabs: Giochi su Google Drive, Cosa sincronizzare,
// I tuoi dispositivi, Attività recenti, Come funziona.
// The status is the app's (cloudState.ts, kept up to date by `cloud://status`); it is read again
// when the section opens.
// Controller: the shell's nav scope; its first control takes the first focus (`data-nav-initial`);
// L1 / R1 change tab (this section's own `tab`, nav.ts `useNavSection`).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { cycle } from '@/lib/cycle'
import { useNavSection } from '@/nav'
import { SectionPage, SectionWidth } from '@/design-system/components/section-page'
import { Tabs, TabsList, TabsTrigger } from '@/design-system/components/tabs'
import CloudActivity from './CloudActivity.vue'
import CloudDevices from './CloudDevices.vue'
import CloudGames from './CloudGames.vue'
import CloudHeader from './CloudHeader.vue'
import CloudHow from './CloudHow.vue'
import CloudSignedOut from './CloudSignedOut.vue'
import CloudStatusCard from './CloudStatusCard.vue'
import CloudSyncSettings from './CloudSyncSettings.vue'
import { cloud, failCloud, gameTitle, loadStatus } from './cloudState'
import { useCloudAccount } from './useCloudAccount'
import { useCloudSync } from './useCloudSync'

const { t } = useI18n()
const status = computed(() => cloud.status)
const account = useCloudAccount()
const sync = useCloudSync()

const TABS = ['games', 'sync', 'devices', 'activity', 'how'] as const
type Tab = (typeof TABS)[number]
const tab = ref<Tab>('games')
/** Reka hands the tab chosen with a click, Enter or A. */
function chooseTab(value: string | number) {
  tab.value = TABS.find(x => x === value) ?? 'games'
}

const root = ref<HTMLElement | null>(null)
useNavSection(root, {
  tab: dir => {
    if (!status.value?.signedIn) return false
    tab.value = cycle(TABS, tab.value, dir)
  },
})

onMounted(async () => {
  try {
    await loadStatus()
  } catch (e) {
    failCloud(e)
  }
})
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1">
    <!-- Signed in, the card and the lists grow with the window (no empty band on a large one). -->
    <SectionPage
      v-if="status"
      as="main"
      class="flex-1"
      :width="status.signedIn ? SectionWidth.Full : SectionWidth.Page"
    >
      <CloudSignedOut
        v-if="!status.signedIn"
        :status="status"
        :signing-in="account.signingIn.value"
        @sign-in="account.signIn"
      />
      <template v-else>
        <CloudHeader :account="status.account" @sign-out="account.signOut" />
        <CloudStatusCard
          :status="status"
          :title-of="gameTitle"
          @sync-now="sync.syncNow"
          @choose="sync.choose"
        />
        <Tabs :model-value="tab" class="contents" @update:model-value="chooseTab">
          <TabsList class="overflow-x-auto" :aria-label="t('cloud.tabs.label')">
            <TabsTrigger v-for="tb in TABS" :key="tb" :value="tb">
              {{ t(`cloud.tabs.${tb}`) }}
            </TabsTrigger>
          </TabsList>
        </Tabs>
        <CloudGames v-if="tab === 'games'" />
        <CloudSyncSettings v-else-if="tab === 'sync'" @open-games="tab = 'games'" />
        <CloudDevices v-else-if="tab === 'devices'" />
        <CloudActivity v-else-if="tab === 'activity'" />
        <CloudHow v-else />
      </template>
    </SectionPage>
  </div>
</template>
