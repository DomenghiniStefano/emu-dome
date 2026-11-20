<script setup lang="ts">
// The Kit page (development only, `#kit`, main.ts): every primitive of src/components/ui in every
// state side by side, in each of the three themes. Fully usable with the arrows / Enter (and the
// controller inside the app): it is a nav scope like any screen, which is how each primitive's
// navigation is checked by hand. Its labels are plain text, not i18n: the page never ships
// (scripts/check exempts src/kit/ from the visible-text gate, and from that only).
import { ref, watch } from 'vue'
import { accentTokens } from '@/lib/accent'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import { dispatchNav, keyToNav, useNavScope } from '@/nav'
import BadgeSection from '@/kit/sections/BadgeSection.vue'
import ButtonSection from '@/kit/sections/ButtonSection.vue'
import CaptureFieldSection from '@/kit/sections/CaptureFieldSection.vue'
import CardSection from '@/kit/sections/CardSection.vue'
import ChipSection from '@/kit/sections/ChipSection.vue'
import ClosingStripSection from '@/kit/sections/ClosingStripSection.vue'
import DialogSection from '@/kit/sections/DialogSection.vue'
import EmptyStateSection from '@/kit/sections/EmptyStateSection.vue'
import HeadingSection from '@/kit/sections/HeadingSection.vue'
import ProgressSection from '@/kit/sections/ProgressSection.vue'
import SearchFieldSection from '@/kit/sections/SearchFieldSection.vue'
import InputSection from '@/kit/sections/InputSection.vue'
import SectionPageSection from '@/kit/sections/SectionPageSection.vue'
import SegmentedSection from '@/kit/sections/SegmentedSection.vue'
import SelectSection from '@/kit/sections/SelectSection.vue'
import SliderSection from '@/kit/sections/SliderSection.vue'
import SwitchSection from '@/kit/sections/SwitchSection.vue'
import TabsSection from '@/kit/sections/TabsSection.vue'

const theme = ref(document.documentElement.dataset.theme ?? 'dark')
watch(theme, t => (document.documentElement.dataset.theme = t), { immediate: true })

// The swatches of `general.accent` (settings catalog), written the way appearance.ts writes the
// user's accent, so every primitive is seen with the run-time accent tokens.
const SWATCHES = ['#7152f7', '#2f7ae5', '#1f9d8a', '#d9622b', '#c2417a']
const accent = ref(SWATCHES[0] ?? '')
watch(accent, swatch => {
  document.getElementById('kit-accent')?.remove()
  const tokens = accentTokens(swatch)
  if (!tokens) return
  const style = document.createElement('style')
  style.id = 'kit-accent'
  const block = (sel: string, t: Record<string, string>) =>
    `${sel}{${Object.entries(t)
      .map(([k, v]) => `${k}:${v}`)
      .join(';')}}`
  style.textContent = [
    block(':root', tokens.dark),
    block(":root[data-theme='dark']", tokens.dark),
    block(":root[data-theme='black']", tokens.black),
    block(":root[data-theme='light']", tokens.light),
  ].join(' ')
  document.head.appendChild(style)
})

// The page stands in for App.vue: arrows and Enter become nav steps, exactly as App.vue's own
// window listener does in the shell.
window.addEventListener('keydown', e => {
  const nav = keyToNav(e)
  if (!nav) return
  e.preventDefault()
  dispatchNav(nav)
})

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true })
</script>

<template>
  <main ref="root" class="h-full overflow-y-auto bg-bg font-sans text-body text-ink">
    <div class="flex flex-col gap-24 p-32">
      <header class="flex flex-wrap items-center justify-between gap-16">
        <h1 class="m-0 text-title text-ink">EmuDome · Kit</h1>
        <div class="flex flex-wrap items-center gap-12">
          <Segmented v-model="accent" aria-label="Accent">
            <SegmentedItem v-for="s in SWATCHES" :key="s" :value="s">{{ s }}</SegmentedItem>
          </Segmented>
          <Segmented v-model="theme" aria-label="Theme">
            <SegmentedItem value="dark">Scuro</SegmentedItem>
            <SegmentedItem value="black">Nero</SegmentedItem>
            <SegmentedItem value="light">Chiaro</SegmentedItem>
          </Segmented>
        </div>
      </header>
      <HeadingSection />
      <ButtonSection />
      <BadgeSection />
      <CardSection />
      <ChipSection />
      <TabsSection />
      <SwitchSection />
      <SegmentedSection />
      <SliderSection />
      <ProgressSection />
      <SelectSection />
      <InputSection />
      <DialogSection />
      <ClosingStripSection />
      <EmptyStateSection />
      <SearchFieldSection />
      <CaptureFieldSection />
      <SectionPageSection />
    </div>
  </main>
</template>
