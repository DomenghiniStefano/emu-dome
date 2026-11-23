<script setup lang="ts">
// S06's Google Drive row (#112): «Accedi con Google» signs in as the Cloud section's button does
// (useCloudAccount: the system browser, its waiting line meanwhile, a failure told through
// failCloud); signed in, the account. Where sign-in is not possible (a build without the Google
// client, Android) the button stays disabled with the reason (driveRow.ts). Optional: the first
// run finishes without it.
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, LogIn } from '@lucide/vue'
import { Button, ButtonFade } from '@/design-system/components/button'
import { cloud, failCloud, loadStatus } from '@/components/cloud/cloudState'
import { useCloudAccount } from '@/components/cloud/useCloudAccount'
import { errorText } from '@/i18n/labels'
import { driveRow } from './driveRow'
import ReadyRow from './ReadyRow.vue'

const { t } = useI18n()
const account = useCloudAccount()
const row = computed(() => driveRow(cloud.status, account.signingIn.value))

/** The second line: why not, the browser waiting, or the account. */
const line = computed(() => {
  const r = row.value
  switch (r.kind) {
    case 'blocked':
      return errorText(r.reason)
    case 'signingIn':
      return t('cloud.signedOut.signingIn')
    case 'signedIn':
      return t('cloud.header.account', { name: r.name ?? t('cloud.header.accountUnknown') })
    case 'unknown':
    case 'signedOut':
      return ''
  }
  return ''
})

// The app reads the status at startup (useCloudEvents); asked here too if it has not answered.
onMounted(async () => {
  if (cloud.status) return
  try {
    await loadStatus()
  } catch (e) {
    failCloud(e)
  }
})
</script>

<template>
  <ReadyRow :name="t('firstRun.ready.drive')">
    <div class="text-caption text-ink-muted wrap-anywhere">
      {{ t('firstRun.ready.driveDesc') }}
    </div>
    <div v-if="line" class="text-caption text-ink-muted wrap-anywhere">{{ line }}</div>
    <template #action>
      <span v-if="row.kind === 'signedIn'" class="flex items-center gap-6 text-success">
        <Check :size="16" :stroke-width="2" aria-hidden="true" />
        {{ t('firstRun.ready.driveDone') }}
      </span>
      <Button
        v-else
        :fade="ButtonFade.Dim"
        :icon="LogIn"
        :disabled="row.kind !== 'signedOut'"
        @click="account.signIn"
      >
        {{ t('firstRun.ready.driveSignIn') }}
      </Button>
    </template>
  </ReadyRow>
</template>
