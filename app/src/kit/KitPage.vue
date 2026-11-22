<script setup lang="ts">
// The Kit page (development only, `#kit`, main.ts): every primitive of src/components/ui and the
// main feature components (kit/features/) in every state side by side, each with its review issues
// (KitIssue: problem, now / proposal, solution), in each of the three themes. Fully usable with the arrows / Enter (and the
// controller inside the app): it is a nav scope like any screen, which is how each primitive's
// navigation is checked by hand. Its labels are plain text, not i18n: the page never ships
// (scripts/check exempts src/kit/ from the visible-text gate, and from that only).
import { ref, watch } from 'vue'
import { ACCENT_SWATCHES, accentTokens, DEFAULT_ACCENT } from '@/lib/accent'
import { Segmented, SegmentedItem } from '@/components/ui/segmented'
import { dispatchNav, keyToNav, useNavScope } from '@/nav'
import BadgeSection from '@/kit/sections/BadgeSection.vue'
import ButtonSection from '@/kit/sections/ButtonSection.vue'
import CaptureFieldSection from '@/kit/sections/CaptureFieldSection.vue'
import CardSection from '@/kit/sections/CardSection.vue'
import ChipSection from '@/kit/sections/ChipSection.vue'
import DateTimeFieldSection from '@/kit/sections/DateTimeFieldSection.vue'
import ClosingStripSection from '@/kit/sections/ClosingStripSection.vue'
import DialogSection from '@/kit/sections/DialogSection.vue'
import EmptyStateSection from '@/kit/sections/EmptyStateSection.vue'
import HeadingSection from '@/kit/sections/HeadingSection.vue'
import PopoverSection from '@/kit/sections/PopoverSection.vue'
import ProgressSection from '@/kit/sections/ProgressSection.vue'
import RadioGroupSection from '@/kit/sections/RadioGroupSection.vue'
import SearchFieldSection from '@/kit/sections/SearchFieldSection.vue'
import InputSection from '@/kit/sections/InputSection.vue'
import SectionPageSection from '@/kit/sections/SectionPageSection.vue'
import SegmentedSection from '@/kit/sections/SegmentedSection.vue'
import SelectSection from '@/kit/sections/SelectSection.vue'
import SliderSection from '@/kit/sections/SliderSection.vue'
import SwitchSection from '@/kit/sections/SwitchSection.vue'
import TabsSection from '@/kit/sections/TabsSection.vue'
import CheckboxSection from '@/kit/sections/CheckboxSection.vue'
import DropdownMenuSection from '@/kit/sections/DropdownMenuSection.vue'
import KbdSection from '@/kit/sections/KbdSection.vue'
import LabelledBlockSection from '@/kit/sections/LabelledBlockSection.vue'
import LinkRowSection from '@/kit/sections/LinkRowSection.vue'
import StatTileSection from '@/kit/sections/StatTileSection.vue'
import GameGroup from '@/kit/features/game/GameGroup.vue'
import LibraryGroup from '@/kit/features/library/LibraryGroup.vue'
import SettingsGroup from '@/kit/features/settings/SettingsGroup.vue'
import KitIndex from '@/kit/KitIndex.vue'
import { KIT_GROUP_TITLES, KitGroup, provideKitBoard } from '@/kit/review'

const theme = ref(document.documentElement.dataset.theme ?? 'dark')

watch(theme, t => (document.documentElement.dataset.theme = t), { immediate: true })

// The swatches of `general.accent` (settings catalog), written the way appearance.ts writes the
// user's accent, so every primitive is seen with the run-time accent tokens (none for the
// default, which colors.css holds).
const SWATCHES = ACCENT_SWATCHES
const accent = ref(DEFAULT_ACCENT)
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

// The review board: sections and issues register into it (review.ts, KitIndex.vue).
const board = provideKitBoard()
const FEATURES = [
  { group: KitGroup.Library, component: LibraryGroup },
  { group: KitGroup.Settings, component: SettingsGroup },
  { group: KitGroup.Game, component: GameGroup },
]

const root = ref<HTMLElement | null>(null)
useNavScope(root, { autofocus: true })
</script>

<template>
  <main ref="root" class="h-full overflow-y-auto bg-bg font-sans text-body text-ink">
    <div class="flex flex-col gap-16 p-16">
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
      <KitIndex v-model:only-open="board.onlyOpen.value" :sections="board.sections" />
      <h2 class="m-0 text-title text-ink">{{ KIT_GROUP_TITLES[KitGroup.Primitives] }}</h2>
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
      <DateTimeFieldSection />
      <InputSection />
      <DialogSection />
      <ClosingStripSection />
      <EmptyStateSection />
      <SearchFieldSection />
      <CaptureFieldSection />
      <SectionPageSection />
      <RadioGroupSection />
      <PopoverSection />
      <CheckboxSection />
      <DropdownMenuSection />
      <KbdSection />
      <LabelledBlockSection />
      <LinkRowSection />
      <StatTileSection />
      <template v-for="f in FEATURES" :key="f.group">
        <h2 class="m-0 text-title text-ink">{{ KIT_GROUP_TITLES[f.group] }}</h2>
        <component :is="f.component" />
      </template>
    </div>
  </main>
</template>
