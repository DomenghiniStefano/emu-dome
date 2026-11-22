<script setup lang="ts">
// SettingRow's states at the page's level: default, changed here (Global, Console, Game),
// inherited, locked by the level, by the console's core, not wired yet, per device, the quick
// menu's compact row, a long name and description.
import SettingSegments from '@/components/settings/SettingSegments.vue'
import SettingSlider from '@/components/settings/SettingSlider.vue'
import { Button, ButtonFade, ButtonVariant } from '@/components/ui/button'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import SettingsRowProposal from '@/kit/proposals/SettingsRowProposal.vue'
import KitSettingRow from './KitSettingRow.vue'
import RowCard from './RowCard.vue'
import { E, options } from './mock'

const THEME = options(
  ['system', 'Come il sistema'],
  ['dark', 'Scuro'],
  ['black', 'Nero'],
  ['light', 'Chiaro'],
)
</script>

<template>
  <KitSection title="Setting row · states" :group="KitGroup.Settings">
    <KitState class="w-full" label="Default value, page with one level (no scope bar)">
      <RowCard><KitSettingRow :entry="E.scale" /></RowCard>
    </KitState>
    <KitState class="w-full" label="Changed at Tutti i giochi: dot, «Modificato», «Ripristina»">
      <RowCard><KitSettingRow :entry="E.scale" value="integer" source="global" scoped /></RowCard>
    </KitState>
    <KitState class="w-full" label="Console level, inherited from Tutti i giochi: «Come in …»">
      <RowCard>
        <KitSettingRow
          :entry="E.scale"
          value="integer"
          source="global"
          level="console"
          console-id="gba"
          scoped
        />
      </RowCard>
    </KitState>
    <KitState class="w-full" label="Changed at Console: «Usa il valore di Tutti i giochi»">
      <RowCard>
        <KitSettingRow
          :entry="E.scale"
          value="fill"
          source="console"
          level="console"
          console-id="gba"
          scoped
        />
      </RowCard>
    </KitState>
    <KitState class="w-full" label="Changed at Game (slider)">
      <RowCard>
        <KitSettingRow
          :entry="E.volume"
          :value="45"
          source="game"
          level="game"
          console-id="gba"
          scoped
        />
      </RowCard>
    </KitState>
    <KitState
      class="w-full"
      label="Locked: Tutti i giochi only, page at Console (reason in the description)"
    >
      <RowCard>
        <KitSettingRow :entry="E.theme" level="console" console-id="gba" scoped />
      </RowCard>
    </KitState>
    <KitState
      class="w-full"
      label="Locked, the page says it once (lockNote): description unchanged"
    >
      <RowCard>
        <KitSettingRow :entry="E.theme" level="console" console-id="gba" scoped lock-note />
      </RowCard>
    </KitState>
    <KitState class="w-full" label="Locked by the console's core (perf.cpuclock on GBA)">
      <RowCard>
        <KitSettingRow :entry="E.cpuclock" level="console" console-id="gba" scoped />
      </RowCard>
    </KitState>
    <KitState class="w-full" label="Not wired yet: «Arriva con un prossimo aggiornamento»">
      <RowCard><KitSettingRow :entry="E.fps" /></RowCard>
    </KitState>
    <KitState class="w-full" label="Per device: «Solo su questo computer», changed here">
      <RowCard><KitSettingRow :entry="E.latency" value="low" source="global" /></RowCard>
    </KitState>
    <KitState
      class="w-full"
      label="Compact (quick menu, session «in prova»): no reset, no «Come in»"
    >
      <RowCard>
        <KitSettingRow
          :entry="E.scale"
          value="fill"
          source="session"
          level="session"
          console-id="gba"
          compact
        />
      </RowCard>
    </KitState>
    <KitState class="w-full" label="Long name and description, changed, per device, list control">
      <RowCard>
        <KitSettingRow :entry="E.frameskip" value="2" source="global" scoped />
      </RowCard>
    </KitState>
    <template #review>
      <KitIssue
        title="Righe bloccate: il controllo sembra attivo, il motivo è sbiadito"
        :status="IssueStatus.Proposed"
      >
        Una riga bloccata (livello non accettato, console che non ha l’opzione, non ancora
        collegata) sbiadisce nome e descrizione, ma il controllo resta pieno: il segmento scelto e
        lo slider in ambra, il pulsante «Importa un salvataggio…» identico a uno attivo
        (ButtonFade.None). Sembra cliccabile, e la frase che spiega perché non lo è risulta la parte
        meno leggibile: nel tema Chiaro nome e descrizione sono quasi invisibili.
        <template #current>
          <RowCard>
            <KitSettingRow :entry="E.theme" level="console" console-id="gba" scoped />
            <KitSettingRow :entry="E.cpuclock" level="console" console-id="gba" scoped />
            <KitSettingRow :entry="E.importSave" />
          </RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal
              name="Tema"
              desc="Vale sempre per tutti i giochi. Per cambiarla scegli «Tutti i giochi» in alto. Nero spegne i pixel sugli schermi OLED."
              locked
            >
              <SettingSegments
                label="Tema"
                value="system"
                :options="THEME"
                :disabled="true"
                @set="() => {}"
              />
            </SettingsRowProposal>
            <SettingsRowProposal
              name="Velocità del processore emulato"
              desc="Non disponibile su questa console: il suo emulatore non ha questa opzione. Più alta toglie i rallentamenti di alcuni giochi, ma pesa di più."
              locked
            >
              <SettingSlider
                label="Velocità del processore emulato"
                :value="100"
                :min="50"
                :max="400"
                :step="25"
                value-text="100 %"
                :disabled="true"
                @set="() => {}"
              />
            </SettingsRowProposal>
            <SettingsRowProposal
              name="Importa salvataggi da altri emulatori"
              desc="DeSmuME (.dsv), backup Action Replay (.duc) e file .sav. Arriva con un prossimo aggiornamento."
              locked
            >
              <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.None" disabled>
                Importa un salvataggio…
              </Button>
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingRow.vue `opacity-locked` passa dalla colonna del testo (SettingRowText.vue) a
          quella del controllo, per ogni tipo (anche i blocchi sotto il testo); il nome resta
          `text-ink-muted`, la descrizione a piena leggibilità. Da controllare nei tre temi.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
