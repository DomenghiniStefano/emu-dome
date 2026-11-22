<script setup lang="ts">
// Kbd: a key or a controller button named in the interface, in every size and variant the app
// uses (G06's hint, the top bar's Ctrl K, the search palette's foot, the quick menu's Back and
// Resume, TV mode's glyphs).
import { ChevronLeft, Play } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdSize, KbdVariant } from '@/components/ui/kbd'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const SINGLE = ['K', 'A', 'I', 'W', '↑', '↓', '↵']
</script>

<template>
  <KitSection title="Kbd">
    <KitState label="Default: words (Esc, Home, Ctrl K)">
      <div class="flex items-center gap-6">
        <Kbd>Esc</Kbd>
        <Kbd>Home</Kbd>
        <Kbd>Ctrl</Kbd>
        <Kbd>K</Kbd>
      </div>
    </KitState>
    <KitState label="Default: one character each (search foot, C01)">
      <div class="flex items-center gap-6">
        <Kbd v-for="k in SINGLE" :key="k">{{ k }}</Kbd>
      </div>
    </KitState>
    <KitState label="In a line of text, on glass (G06 hint)">
      <div class="glass-thick flex items-center gap-6 rounded-card px-10 py-6 text-ink-muted">
        <Kbd>Esc</Kbd>
        <span>or</span>
        <Kbd>Home</Kbd>
        <span>for the quick menu</span>
      </div>
    </KitState>
    <KitState label="Small: beside a Ghost · Back label (quick menu)">
      <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Back">
        <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
        Quick menu
        <Kbd :size="KbdSize.Small" class="ml-4">B</Kbd>
      </Button>
    </KitState>
    <KitState label="OnAccent · Small: inside Primary · MenuMain">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.MenuMain">
        <Play :size="18" fill="currentColor" :stroke-width="0" aria-hidden="true" />
        Resume game
        <Kbd :variant="KbdVariant.OnAccent" :size="KbdSize.Small" class="ml-4">Esc</Kbd>
      </Button>
    </KitState>
    <KitState label="Small: one character and a word">
      <div class="flex items-center gap-6">
        <Kbd :size="KbdSize.Small">B</Kbd>
        <Kbd :size="KbdSize.Small">Esc</Kbd>
      </div>
    </KitState>
    <KitState label="TvFace · TvShoulder (TV glyphs), Default and OnAccent">
      <div class="flex items-center gap-8">
        <Kbd :size="KbdSize.TvFace">A</Kbd>
        <Kbd :size="KbdSize.TvFace">Y</Kbd>
        <Kbd :size="KbdSize.TvShoulder">LB</Kbd>
        <Kbd :size="KbdSize.TvShoulder">R1</Kbd>
        <span class="inline-flex items-center rounded-control bg-accent px-8 py-6">
          <Kbd :variant="KbdVariant.OnAccent" :size="KbdSize.TvFace">A</Kbd>
        </span>
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Il suggerimento è più forte dell'azione" :status="IssueStatus.Proposed">
        Nel pulsante «‹ Menu rapido» il testo è grigio (Ghost, `ink-muted`) ma il tasto «B» accanto
        è in inchiostro pieno con il bordo forte: l'occhio va prima al suggerimento che all'azione.
        Lo stesso nel piede del menu rapido («Ctrl K Cerca»: tasti bianchi, testo grigio). Sul
        pulsante principale invece (OnAccent) il tasto è giustamente più tenue del testo.
        <template #current>
          <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Back">
            <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
            Quick menu
            <Kbd :size="KbdSize.Small" class="ml-4">B</Kbd>
          </Button>
          <span class="inline-flex items-center gap-6 text-caption text-ink-muted">
            <Kbd>Ctrl</Kbd>
            <Kbd>K</Kbd>
            Search
          </span>
        </template>
        <template #proposed>
          <Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Back">
            <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
            Quick menu
            <Kbd :size="KbdSize.Small" class="ml-4 border-line text-ink-muted">B</Kbd>
          </Button>
          <span class="inline-flex items-center gap-6 text-caption text-ink-muted">
            <Kbd class="border-line text-ink-muted">Ctrl</Kbd>
            <Kbd class="border-line text-ink-muted">K</Kbd>
            Search
          </span>
        </template>
        <template #solution>
          una variante `KbdVariant.Hint` (`border-line text-ink-muted`) in
          components/ui/kbd/variants.ts, usata dove il tasto accompagna un testo grigio (BackButton,
          MenuFooter, AppTopBar, QuickMenuHint); Default resta per i tasti che sono il contenuto
          (C01, la ricerca).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
