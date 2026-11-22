<script setup lang="ts">
// The Cloud section (Trello #31, approved mockup «Cloud con Google Drive»). Before the sign-in:
// what the sync does and «Accedi con Google» (CloudSignedOut). Signed in (option A): the head
// with the account, the status card, then the tabs.
// The status is the app's (cloudState.ts, kept up to date by `cloud://status`); it is read again
// when the section opens.
// Controller: the shell's nav scope; its first control takes the first focus (`data-nav-initial`).
import { computed, onMounted } from 'vue'
import { SectionPage, SectionWidth } from '@/components/ui/section-page'
import CloudHeader from './CloudHeader.vue'
import CloudSignedOut from './CloudSignedOut.vue'
import CloudStatusCard from './CloudStatusCard.vue'
import { cloud, failCloud, gameTitle, loadStatus } from './cloudState'
import { useCloudAccount } from './useCloudAccount'
import { useCloudSync } from './useCloudSync'

const status = computed(() => cloud.status)
const account = useCloudAccount()
const sync = useCloudSync()

onMounted(async () => {
  try {
    await loadStatus()
  } catch (e) {
    failCloud(e)
  }
})
</script>

<template>
  <SectionPage v-if="status" as="main" :width="SectionWidth.Page">
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
    </template>
  </SectionPage>
</template>
