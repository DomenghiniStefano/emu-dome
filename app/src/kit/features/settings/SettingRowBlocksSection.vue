<script setup lang="ts">
// SettingRow's controls under the text: the multiple choice's chips, the colour swatches, the
// choice cards, the presets (`apply`) and the game folders.
import { ref } from 'vue'
import SettingSwatches from '@/components/settings/SettingSwatches.vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Chip, ChipVariant } from '@/components/ui/chip'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import SettingsCardsProposal from '@/kit/proposals/SettingsCardsProposal.vue'
import SettingsRowProposal from '@/kit/proposals/SettingsRowProposal.vue'
import KitSettingRow from './KitSettingRow.vue'
import RowCard from './RowCard.vue'
import { E, options } from './mock'

const SURROUND = options(
  ['black', 'Nero', 'Niente distrazioni'],
  ['cover', 'Colore della copertina', 'Una sfumatura presa dalla copertina'],
  ['blur', 'Gioco sfocato', 'Il gioco ingrandito e sfocato. Solo sul computer.'],
  ['builtin', 'Sfondo pronto', 'Sfumature e motivi di EmuDome'],
  ['image', 'Immagine tua', 'Una foto o un disegno dai tuoi file'],
)
const surroundSoon = SURROUND.map(o =>
  o.id === 'blur'
    ? { ...o, desc: `${o.desc} Arriva con un prossimo aggiornamento.`, notYet: true }
    : o,
)
const CYCLE = ['Verticale', 'Affiancati', 'Grande e piccolo', 'Solo lo schermo superiore']
const accent = ref('#2f7ae5')
</script>

<template>
  <KitSection title="Setting row · controls under the text" :group="KitGroup.Settings">
    <KitState label="Multiple choice (chips): the list, «Modifica l’elenco» to edit">
      <RowCard><KitSettingRow :entry="E.cycle" level="console" console-id="nds" /></RowCard>
    </KitState>
    <KitState label="Colour swatches, changed">
      <RowCard><KitSettingRow :entry="E.accent" value="#2f7ae5" source="global" /></RowCard>
    </KitState>
    <KitState label="Choice cards with a preview">
      <RowCard><KitSettingRow :entry="E.surround" value="cover" source="global" /></RowCard>
    </KitState>
    <KitState label="Presets (apply), not wired yet">
      <RowCard><KitSettingRow :entry="E.preset" /></RowCard>
    </KitState>
    <KitState label="Game folders, per device (a long path is cut)">
      <RowCard>
        <KitSettingRow
          :entry="E.folders"
          :value="[
            'D:/Giochi/GBA',
            'C:/Users/utente/Documents/Emulazione/Nintendo DS/Giochi europei e giapponesi/Collezione completa',
          ]"
          source="global"
        />
      </RowCard>
    </KitState>
    <KitState label="Game folders, none yet">
      <RowCard><KitSettingRow :entry="E.folders" /></RowCard>
    </KitState>

    <template #review>
      <KitIssue
        title="Cinque schede su due righe, una sola nella seconda"
        :status="IssueStatus.Proposed"
      >
        «Sfondo ai lati del gioco» ha cinque opzioni: alla larghezza della pagina ne stanno quattro
        per riga (minimo `card-min`, 180), così «Immagine tua» resta sola sotto e accanto resta una
        banda vuota larga tre schede. La riga diventa alta il doppio per una scheda.
        <template #current>
          <RowCard><KitSettingRow :entry="E.surround" value="cover" source="global" /></RowCard>
        </template>
        <template #proposed>
          <div class="w-page max-w-full">
            <SettingsCardsProposal
              setting-key="video.surround"
              :options="surroundSoon"
              value="cover"
            />
          </div>
        </template>
        <template #solution>
          in SettingCards.vue la griglia usa `auto-fit` (le colonne riempiono la riga) con un minimo
          più stretto: un token nuovo `card-min-compact` (160) in theme/spacing.css. Con sei opzioni
          (`video.surroundSet`) vanno a tre per riga su due righe piene.
        </template>
      </KitIssue>
      <KitIssue
        title="«Arriva con un prossimo aggiornamento» attaccato senza punto"
        :status="IssueStatus.Proposed"
      >
        Sulla scheda «Gioco sfocato» si legge «… Solo sul computer Arriva con un prossimo
        aggiornamento.»: la descrizione dell’opzione non finisce con il punto e la frase che
        SettingCards aggiunge le si attacca.
        <template #current>
          <span class="w-card-min text-caption text-ink-muted">
            Il gioco ingrandito e sfocato. Solo sul computer Arriva con un prossimo aggiornamento.
          </span>
        </template>
        <template #proposed>
          <div class="w-page max-w-full">
            <SettingsCardsProposal
              setting-key="video.surround"
              :options="surroundSoon"
              value="cover"
            />
          </div>
        </template>
        <template #solution>
          il punto finale in `settings.video.surround.optionDesc.blur` (it.json e en.json), e le
          descrizioni delle opzioni scritte sempre come frasi complete.
        </template>
      </KitIssue>
      <KitIssue
        title="L’azione della riga in tre stili diversi, uno in ambra"
        :status="IssueStatus.Proposed"
      >
        Sotto il testo, «Modifica l’elenco» (scelta multipla) è un pulsante fantasma color ambra,
        «Aggiungi una cartella…» è un pulsante Soft, «Ripristina» fantasma neutro. L’ambra va solo
        sull’azione principale: qui è un’azione secondaria e attira l’occhio più del valore scelto.
        <template #current>
          <RowCard><KitSettingRow :entry="E.cycle" level="console" console-id="nds" /></RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal
              name="Disposizioni da alternare"
              desc="Quelle che scorri con il pulsante dedicato o dal menu rapido, in quest’ordine."
            >
              <template #below>
                <div class="flex flex-wrap items-center gap-8">
                  <Chip
                    v-for="(name, i) in CYCLE"
                    :key="name"
                    as="span"
                    :variant="ChipVariant.Option"
                  >
                    <span class="text-ink-muted tabular-nums">{{ i + 1 }}</span>
                    {{ name }}
                  </Chip>
                  <Button
                    :variant="ButtonVariant.Soft"
                    :fade="ButtonFade.None"
                    :size="ButtonSize.Small"
                  >
                    Modifica l’elenco
                  </Button>
                </div>
              </template>
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingChips.vue i due pulsanti («Modifica l’elenco», «Fatto») passano da
          `ButtonVariant.GhostAccent` a `ButtonVariant.Soft`, come le altre azioni dentro una riga.
        </template>
      </KitIssue>
      <KitIssue
        title="Colore d’accento: i pallini vanno a capo e lasciano vuota la destra"
        :status="IssueStatus.Proposed"
      >
        I cinque pallini stanno su una riga loro, sotto il testo, mentre a destra resta solo
        «Ripristina»: la riga è alta il doppio di un interruttore e metà è vuota. Cinque pallini
        occupano meno spazio dei segmenti di «Tema», che stanno accanto al testo.
        <template #current>
          <RowCard><KitSettingRow :entry="E.accent" value="#2f7ae5" source="global" /></RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal
              name="Colore d’accento"
              desc="Il colore dei pulsanti principali e della selezione."
              changed
              reset-text="Ripristina"
            >
              <SettingSwatches
                setting-key="general.accent"
                label="Colore d’accento"
                :swatches="['#faa40f', '#7152f7', '#2f7ae5', '#1f9d8a', '#c2417a']"
                :value="accent"
                :disabled="false"
                @set="c => (accent = c)"
              />
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingRow.vue SettingSwatches passa nella colonna di destra, con i controlli in linea
          (resta sotto il testo solo quando la riga è stretta, come gli altri).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
