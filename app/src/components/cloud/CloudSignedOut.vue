<script setup lang="ts">
// The Cloud section before the sign-in (Trello #31, mockup section 1): what the sync does in
// three points and «Accedi con Google». A build without the Google client (`configured` false)
// or a system that cannot sign in yet (`supported` false, Android) keeps the button disabled and
// says why with the existing `cloudNotConfigured` / `cloudSignInUnsupported` texts.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, LogIn } from '@lucide/vue'
import type { CloudStatus, ErrorCode } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Heading, HeadingVariant } from '@/design-system/components/heading'
import { errorText } from '@/i18n/labels'

const props = defineProps<{ status: CloudStatus; signingIn: boolean }>()
const emit = defineEmits<{ signIn: [] }>()
const { t } = useI18n()

/** Why the button cannot be used, else null. */
const blocked = computed<ErrorCode | null>(() => {
  if (!props.status.configured) return { code: 'cloudNotConfigured' }
  if (!props.status.supported) return { code: 'cloudSignInUnsupported' }
  return null
})
/** The last failure (an expired access, say), when the button can be used. */
const failure = computed(() => (blocked.value ? null : props.status.error))
const POINTS = ['saves', 'settings', 'games'] as const
</script>

<template>
  <div class="grid flex-1 place-items-center">
    <section class="flex w-full max-w-lead flex-col gap-12">
      <Heading as="h1" :variant="HeadingVariant.Display">{{ t('cloud.signedOut.title') }}</Heading>
      <p class="m-0 text-lead text-ink-muted">{{ t('cloud.signedOut.lead') }}</p>
      <ul class="m-0 flex list-none flex-col gap-8 p-0 text-lead">
        <li v-for="p in POINTS" :key="p" class="flex gap-8">
          <!-- Down to sit on the text's first line. -->
          <Check
            class="mt-4 flex-none text-success"
            :size="16"
            :stroke-width="2"
            aria-hidden="true"
          />
          <span>{{ t(`cloud.signedOut.${p}`) }}</span>
        </li>
      </ul>
      <div class="mt-4 flex flex-wrap items-center gap-8">
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Large"
          :disabled="blocked !== null || signingIn"
          :icon="LogIn"
          data-nav-initial
          @click="emit('signIn')"
        >
          {{ t('cloud.signedOut.signIn') }}
        </Button>
        <span class="text-caption text-ink-muted">{{
          signingIn ? t('cloud.signedOut.signingIn') : t('cloud.signedOut.privacy')
        }}</span>
      </div>
      <p v-if="blocked" class="m-0 text-ink-muted">{{ errorText(blocked) }}</p>
      <p v-else-if="failure" class="m-0 text-danger" role="alert">{{ errorText(failure) }}</p>
    </section>
  </div>
</template>
