<script setup lang="ts">
// SettingRow's states at the page's level: default, changed here (Global, Console, Game),
// inherited, locked by the level, by the console's core, not wired yet, per device, the quick
// menu's compact row, a long name and description.
import { ref } from 'vue'
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

const SCALE = options(
  ['fit', 'Adatta allo schermo'],
  ['integer', 'Pixel interi'],
  ['fill', 'Riempi lo schermo'],
)
const THEME = options(
  ['system', 'Come il sistema'],
  ['dark', 'Scuro'],
  ['black', 'Nero'],
  ['light', 'Chiaro'],
)
const SCALE_DESC =
  'Pixel interi mantiene tutti i pixel della stessa grandezza e lascia bordi ai lati.'
const scale = ref('fill')
const inherited = ref('integer')
const volume = ref(45)
</script>

<template>
  <KitSection title="Setting row · states" :group="KitGroup.Settings">
    <KitState label="Default value, page with one level (no scope bar)">
      <RowCard><KitSettingRow :entry="E.scale" /></RowCard>
    </KitState>
    <KitState label="Changed at Tutti i giochi: dot, «Modificato», «Ripristina»">
      <RowCard><KitSettingRow :entry="E.scale" value="integer" source="global" scoped /></RowCard>
    </KitState>
    <KitState label="Console level, inherited from Tutti i giochi: «Come in …»">
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
    <KitState label="Changed at Console: «Usa il valore di Tutti i giochi»">
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
    <KitState label="Changed at Game (slider)">
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
    <KitState label="Locked: Tutti i giochi only, page at Console (reason in the description)">
      <RowCard>
        <KitSettingRow :entry="E.theme" level="console" console-id="gba" scoped />
      </RowCard>
    </KitState>
    <KitState label="Locked, the page says it once (lockNote): description unchanged">
      <RowCard>
        <KitSettingRow :entry="E.theme" level="console" console-id="gba" scoped lock-note />
      </RowCard>
    </KitState>
    <KitState label="Locked by the console's core (perf.cpuclock on GBA)">
      <RowCard>
        <KitSettingRow :entry="E.cpuclock" level="console" console-id="gba" scoped />
      </RowCard>
    </KitState>
    <KitState label="Not wired yet: «Arriva con un prossimo aggiornamento»">
      <RowCard><KitSettingRow :entry="E.fps" /></RowCard>
    </KitState>
    <KitState label="Per device: «Solo su questo computer», changed here">
      <RowCard><KitSettingRow :entry="E.latency" value="low" source="global" /></RowCard>
    </KitState>
    <KitState label="Compact (quick menu, session «in prova»): no reset, no «Come in»">
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
    <KitState label="Long name and description, changed, per device, list control">
      <RowCard>
        <KitSettingRow :entry="E.frameskip" value="2" source="global" scoped />
      </RowCard>
    </KitState>
    <template #review>
      <KitIssue
        title="Il controllo va a capo quando c’è «Usa il valore di…» o «Come in…»"
        :status="IssueStatus.Proposed"
      >
        Ai livelli Console e Gioco il pulsante «Usa il valore di «Tutti i giochi»» o la scritta
        «Come in «…»» stanno nella stessa colonna del controllo: i segmenti vanno a capo sotto il
        testo e si spostano a sinistra, mentre in tutte le altre righe stanno a destra. Con «Come
        in» la descrizione si stringe e va a capo con una parola sola. La stessa impostazione cambia
        forma a seconda del livello.
        <template #current>
          <RowCard>
            <KitSettingRow
              :entry="E.scale"
              value="fill"
              source="console"
              level="console"
              console-id="gba"
              scoped
            />
            <KitSettingRow
              :entry="E.scale"
              value="integer"
              source="global"
              level="console"
              console-id="gba"
              scoped
            />
          </RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal
              name="Dimensione dell’immagine"
              :desc="SCALE_DESC"
              changed
              reset-text="Usa il valore di «Tutti i giochi»"
            >
              <SettingSegments
                label="Dimensione dell’immagine"
                :value="scale"
                :options="SCALE"
                :disabled="false"
                @set="v => (scale = v)"
              />
            </SettingsRowProposal>
            <SettingsRowProposal
              name="Dimensione dell’immagine"
              :desc="SCALE_DESC"
              inherited="Come in «Tutti i giochi»"
            >
              <SettingSegments
                label="Dimensione dell’immagine"
                :value="inherited"
                :options="SCALE"
                :disabled="false"
                @set="v => (inherited = v)"
              />
            </SettingsRowProposal>
            <SettingsRowProposal
              name="Volume"
              desc="Volume dei giochi, indipendente dal volume del sistema."
              changed
              reset-text="Usa il valore di «Tutti i giochi»"
            >
              <SettingSlider
                label="Volume"
                :value="volume"
                :min="0"
                :max="100"
                :step="1"
                :value-text="`${volume} %`"
                :disabled="false"
                @set="v => (volume = v)"
              />
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingRow.vue «Ripristina» / «Usa il valore di…» e «Come in…» passano nella colonna
          del testo (SettingRowText.vue, una riga sotto la descrizione); la colonna di destra tiene
          solo il controllo, sempre allineato a destra. Il quick menu (compact) non cambia.
        </template>
      </KitIssue>
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
