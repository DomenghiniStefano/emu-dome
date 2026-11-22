<script setup lang="ts">
// S01 Welcome: the mark, what EmuDome is in four points, and the language (`general.lang`,
// «Tutti i giochi»). No «Indietro».
import { Folder, Gamepad, Save, Smartphone } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import mark from '@/assets/brand/emudome-mark.svg'
import SettingSelect from '@/components/settings/SettingSelect.vue'
import StepLayout from './StepLayout.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'

defineProps<{ lang: string; langOptions: { id: string; text: string }[] }>()
const emit = defineEmits<{ lang: [id: string] }>()
const { t } = useI18n()

// The points under the title (#101), each with its icon.
const points = [
  { key: 'firstRun.welcome.points.consoles', icon: Gamepad },
  { key: 'firstRun.welcome.points.devices', icon: Smartphone },
  { key: 'firstRun.welcome.points.play', icon: Save },
  { key: 'firstRun.welcome.points.games', icon: Folder },
] as const
</script>

<template>
  <StepLayout centred>
    <!-- The mark draws its own amber glow; the title beside it says the name. -->
    <img :src="mark" alt="" aria-hidden="true" class="block h-auto w-mark" />
    <Heading as="h1" :variant="HeadingVariant.Display">{{ t('firstRun.welcome.title') }}</Heading>
    <ul class="m-0 flex max-w-lead list-none flex-col gap-8 p-0 text-left">
      <li
        v-for="point in points"
        :key="point.key"
        class="flex items-start gap-10 text-lead text-ink"
      >
        <component
          :is="point.icon"
          :size="18"
          :stroke-width="1.75"
          class="mt-2 flex-none text-accent-text"
          aria-hidden="true"
        />
        <span class="text-pretty">{{ t(point.key) }}</span>
      </li>
    </ul>
    <div class="flex items-center gap-8 text-ink-muted">
      <span>{{ t('firstRun.welcome.language') }}</span>
      <SettingSelect
        :label="t('settings.general.lang.name')"
        :value="lang"
        :options="langOptions"
        @pick="id => emit('lang', id)"
      />
    </div>
  </StepLayout>
</template>
