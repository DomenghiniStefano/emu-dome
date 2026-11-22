<script setup lang="ts">
// LabelledBlock: a block under its overline label, with the asides the app puts beside it (a
// link in the quick menu, a note in Settings and Controls), and without a head.
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardPad, CardVariant } from '@/components/ui/card'
import { Kbd } from '@/components/ui/kbd'
import { LabelledBlock } from '@/components/ui/labelled-block'
import { LinkRow } from '@/components/ui/link-row'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import LabelledBlockProposal from '@/kit/proposals/LabelledBlockProposal.vue'
import { IssueStatus } from '@/kit/review'
</script>

<template>
  <KitSection title="LabelledBlock">
    <KitState label="Title only (Saves «In-game save», title-as h3)">
      <LabelledBlock title="In-game save" title-as="h3" class="w-menu max-w-full">
        <Card :pad="CardPad.Snug" class="flex-row items-center gap-8">
          <span class="min-w-0 flex-1 tabular-nums">Saved 21 Nov, 18:42 · 32 KB</span>
          <Button>Export</Button>
        </Card>
      </LabelledBlock>
    </KitState>
    <KitState label="Aside · a Link · LabelLink (quick menu «More slots»)">
      <LabelledBlock title="Quick saves" class="w-menu max-w-full">
        <template #aside>
          <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink">
            More slots and saves
          </Button>
        </template>
        <Card :pad="CardPad.Snug">Slots</Card>
      </LabelledBlock>
    </KitState>
    <KitState label="Aside · a note, text-caption (SettingsSection)">
      <LabelledBlock title="Most used" title-as="h3" class="w-menu max-w-full">
        <template #aside>
          <span class="text-caption text-ink-muted">Apply to every game</span>
        </template>
        <Card :variant="CardVariant.List">
          <LinkRow name="Screen effect" sub="Screen effects" value="CRT TV" />
        </Card>
      </LabelledBlock>
    </KitState>
    <KitState label="Aside · a note, text-caption-tall (Controls «Shortcuts to remember»)">
      <LabelledBlock title="Shortcuts to remember" class="w-menu max-w-full">
        <template #aside>
          <span class="text-caption-tall text-ink-muted">Hold L3 and press a button</span>
        </template>
        <Card :variant="CardVariant.List">
          <div class="flex min-h-control-lg items-center gap-8 px-12 py-4">
            <Kbd>R1</Kbd>
            <span>Quick save</span>
          </div>
        </Card>
      </LabelledBlock>
    </KitState>
    <KitState label="No head (quick menu's saves page, «all»)">
      <LabelledBlock class="w-menu max-w-full" aria-label="Quick saves">
        <Card :pad="CardPad.Snug">Every slot, no label above</Card>
      </LabelledBlock>
    </KitState>
    <KitState label="Long title and note in a narrow column (wraps)">
      <LabelledBlock title="Stick, grilletti e vibrazione del controller" class="w-field">
        <template #aside>
          <span class="text-caption text-ink-muted">Valgono per tutti i giochi</span>
        </template>
        <Card :pad="CardPad.Snug">Block</Card>
      </LabelledBlock>
    </KitState>

    <template #review>
      <KitIssue
        title="La testata cambia altezza con quello che ha accanto"
        :status="IssueStatus.Proposed"
      >
        L'altezza della testata dipende da cosa c'è a destra del titolo: titolo solo 14 px, nota
        `text-caption` 16 (Impostazioni), nota `text-caption-tall` o link 18 (Controlli, menu
        rapido). Il contenuto parte quindi a 24, 26 o 28 px dall'etichetta: lo spazio cambia da una
        pagina all'altra, e due blocchi messi in griglia non si allineano. La nota la scrive chi usa
        il blocco, scegliendo ogni volta il tipo.
        <template #current>
          <div class="grid w-panel max-w-full grid-cols-3 gap-8">
            <LabelledBlock title="In-game save" title-as="h3">
              <Card :pad="CardPad.Snug">24</Card>
            </LabelledBlock>
            <LabelledBlock title="Most used" title-as="h3">
              <template #aside>
                <span class="text-caption text-ink-muted">Every game</span>
              </template>
              <Card :pad="CardPad.Snug">26</Card>
            </LabelledBlock>
            <LabelledBlock title="Quick saves" title-as="h3">
              <template #aside>
                <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink">
                  More slots
                </Button>
              </template>
              <Card :pad="CardPad.Snug">28</Card>
            </LabelledBlock>
          </div>
        </template>
        <template #proposed>
          <div class="grid w-panel max-w-full grid-cols-3 gap-8">
            <LabelledBlockProposal title="In-game save">
              <Card :pad="CardPad.Snug">28</Card>
            </LabelledBlockProposal>
            <LabelledBlockProposal title="Most used" note="Every game">
              <Card :pad="CardPad.Snug">28</Card>
            </LabelledBlockProposal>
            <LabelledBlockProposal title="Quick saves">
              <template #aside>
                <Button :variant="ButtonVariant.Link" :size="ButtonSize.LabelLink">
                  More slots
                </Button>
              </template>
              <Card :pad="CardPad.Snug">28</Card>
            </LabelledBlockProposal>
          </div>
        </template>
        <template #solution>
          in components/ui/labelled-block/LabelledBlock.vue la testata `min-h-18` e una prop `note`
          disegnata dal blocco in `text-caption-tall text-ink-muted`; lo slot `#aside` resta per i
          link («Altri slot», «Imposta le velocità»). SettingsSection e ShortcutsToRemember passano
          `note` e tolgono il loro span.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
