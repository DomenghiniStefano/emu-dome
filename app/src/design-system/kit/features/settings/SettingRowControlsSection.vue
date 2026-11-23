<script setup lang="ts">
// SettingRow's controls beside the text, one row per kind on one card, as a page mixes them:
// toggle, segments, list, slider, text, date and time, file (none / chosen), action, link,
// bindings, screen layout.
import { ChevronRight } from '@lucide/vue'
import SettingTextField from '@/components/settings/SettingTextField.vue'
import { Button, ButtonFade, ButtonVariant } from '@/design-system/components/button'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/design-system/kit/review'
import SettingsRowProposal from '@/design-system/kit/proposals/SettingsRowProposal.vue'
import KitSettingRow from './KitSettingRow.vue'
import RowCard from './RowCard.vue'
import { E } from './mock'
</script>

<template>
  <KitSection title="Setting row · controls beside the text" :group="KitGroup.Settings">
    <KitState class="w-full" label="Every inline kind on one card (toggle … screen layout)">
      <RowCard>
        <KitSettingRow :entry="E.toggle" />
        <KitSettingRow :entry="E.autoq" />
        <KitSettingRow :entry="E.lang" />
        <KitSettingRow :entry="E.volume" />
        <KitSettingRow :entry="E.backups" />
        <KitSettingRow :entry="E.dsname" level="console" console-id="nds" />
        <KitSettingRow :entry="E.rtcdate" />
        <KitSettingRow :entry="E.surroundImg" />
        <KitSettingRow :entry="E.fxfile" value="C:/Shader/crt-royale-kurozumi.slangp" />
        <KitSettingRow :entry="E.importSave" />
        <KitSettingRow :entry="E.licences" />
        <KitSettingRow :entry="E.turbo" />
        <KitSettingRow :entry="E.remap" />
        <KitSettingRow :entry="E.layout" level="console" console-id="nds" />
      </RowCard>
    </KitState>

    <template #review>
      <KitIssue
        title="Le righe che aprono un’altra schermata non hanno niente da premere"
        :status="IssueStatus.Proposed"
      >
        Una riga di tipo link senza testo del pulsante («Pressione ripetuta (turbo)», `pad.turbo`)
        mostra solo nome e descrizione: a destra non c’è nulla, quindi né il mouse né il controller
        hanno un punto su cui fermarsi, e non si capisce che porta altrove. «Licenze open source»
        invece ha «Mostra»: due righe dello stesso tipo, due aspetti.
        <template #current>
          <RowCard>
            <KitSettingRow :entry="E.turbo" />
            <KitSettingRow :entry="E.licences" />
          </RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal
              name="Pressione ripetuta (turbo)"
              desc="Pulsanti che si ripetono da soli finché li tieni premuti. Arriva con un prossimo aggiornamento."
              locked
            >
              <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.None" disabled>
                Apri
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </Button>
            </SettingsRowProposal>
            <SettingsRowProposal name="Licenze open source" desc="EmuDome e i suoi componenti.">
              <Button :variant="ButtonVariant.Soft" :fade="ButtonFade.None">
                Mostra
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </Button>
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingRow.vue un’azione `style: 'link'` mostra sempre un pulsante Soft con la freccia
          (ChevronRight), con «Apri» quando la voce non ha un testo suo (`settingsUi.open` nei due
          file di lingua). Le azioni `button` restano senza freccia.
        </template>
      </KitIssue>
      <KitIssue title="Campo di testo vuoto senza indicazione" :status="IssueStatus.Proposed">
        «Nome utente» (`consoles.dsname`) parte vuoto: vuoto vuol dire «il nome utente del sistema»,
        ma la riga mostra solo un riquadro bianco, che sembra un campo da riempire per forza.
        `online.wfc` ha già il suo segnaposto; questo no.
        <template #current>
          <RowCard>
            <KitSettingRow :entry="E.dsname" level="console" console-id="nds" />
          </RowCard>
        </template>
        <template #proposed>
          <RowCard>
            <SettingsRowProposal name="Nome utente" desc="Il nome che i giochi vedono.">
              <SettingTextField
                label="Nome utente"
                value=""
                placeholder="Come il nome del computer"
                :disabled="false"
                @set="() => {}"
              />
            </SettingsRowProposal>
          </RowCard>
        </template>
        <template #solution>
          in SettingRow.vue il segnaposto di `consoles.dsname` (come già per `online.wfc`), testo
          nuovo `settings.consoles.dsname.value.none` in en.json e it.json, da rivedere con te.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
