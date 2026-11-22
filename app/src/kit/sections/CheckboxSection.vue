<script setup lang="ts">
// Checkbox: behaviour only (Reka), `as-child` on a Menu · MenuRow Button that draws its own box,
// as L03 «Mostra» (LibraryFilters.vue) and the TV question's «Non chiederlo più»
// (TvModeDialog.vue) do today.
import { reactive, ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import CheckboxBoxProposal from '@/kit/proposals/CheckboxBoxProposal.vue'
import { IssueStatus } from '@/kit/review'

const LABELS = {
  favourites: 'Favourites only',
  unplayed: 'Never played only',
  hidden: 'Hidden games too',
} as const
type ShowKey = keyof typeof LABELS
const KEYS: ShowKey[] = ['favourites', 'unplayed', 'hidden']
const TWO: ShowKey[] = ['favourites', 'unplayed']

const show = reactive<Record<ShowKey, boolean>>({
  favourites: true,
  unplayed: false,
  hidden: false,
})
const proposed = reactive<Record<ShowKey, boolean>>({
  favourites: true,
  unplayed: false,
  hidden: false,
})
const remember = ref(false)
</script>

<template>
  <KitSection title="Checkbox">
    <KitState label="Checked / not (L03 «Show», a popover's rows)">
      <div class="flex w-field flex-col gap-2 px-6">
        <Checkbox v-for="k in KEYS" :key="k" v-model="show[k]">
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
            <span
              class="size-14 shrink-0 rounded-4"
              :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
              aria-hidden="true"
            ></span>
            {{ LABELS[k] }}
          </Button>
        </Checkbox>
      </div>
    </KitState>
    <KitState label="Disabled, checked and not">
      <div class="flex w-field flex-col gap-2 px-6">
        <Checkbox :model-value="true" disabled>
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
            <span class="size-14 shrink-0 rounded-4 bg-accent-text" aria-hidden="true"></span>
            On this device only
          </Button>
        </Checkbox>
        <Checkbox :model-value="false" disabled>
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
            <span
              class="size-14 shrink-0 rounded-4 border border-line-strong"
              aria-hidden="true"
            ></span>
            Hidden games too
          </Button>
        </Checkbox>
      </div>
    </KitState>
    <KitState label="Alone in a dialog, self-start (TV ask «Don't ask again»)">
      <div class="flex w-field flex-col gap-8 px-6">
        <Checkbox v-model="remember">
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6 self-start">
            <span
              class="size-14 shrink-0 rounded-4"
              :class="remember ? 'bg-accent-text' : 'border border-line-strong'"
              aria-hidden="true"
            ></span>
            Don't ask again
          </Button>
        </Checkbox>
      </div>
    </KitState>
    <KitState label="Long label in a narrow popover (the row does not wrap)">
      <div class="flex w-field flex-col gap-2 overflow-hidden border border-line px-6">
        <Checkbox :model-value="false">
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
            <span
              class="size-14 shrink-0 rounded-4 border border-line-strong"
              aria-hidden="true"
            ></span>
            Solo quelli su questo dispositivo
          </Button>
        </Checkbox>
      </div>
    </KitState>
    <KitState label="On the phone (touch density)">
      <div data-density="touch" class="flex w-field flex-col gap-2 px-6">
        <Checkbox v-for="k in KEYS" :key="k" v-model="show[k]">
          <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
            <span
              class="size-14 shrink-0 rounded-4"
              :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
              aria-hidden="true"
            ></span>
            {{ LABELS[k] }}
          </Button>
        </Checkbox>
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Spuntata senza spunta" :status="IssueStatus.Proposed">
        Quando è spuntata, la casella diventa un quadrato pieno color accento, senza il segno di
        spunta. Da sola sembra un campione di colore o un tasto «stop»; nel pannello L03, sotto i
        pallini dell'ordinamento, non si capisce subito che è una scelta multipla. E il colore è
        l'unico segno: chi distingue male l'ambra vede solo «pieno» e «vuoto».
        <template #current>
          <div class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-for="k in TWO" :key="k" v-model="show[k]">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
                <span
                  class="size-14 shrink-0 rounded-4"
                  :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
                  aria-hidden="true"
                ></span>
                {{ LABELS[k] }}
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-for="k in TWO" :key="k" v-model="proposed[k]">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
                <CheckboxBoxProposal :checked="proposed[k]" />
                {{ LABELS[k] }}
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #solution>
          una spunta Lucide «Check» opaca dentro il quadrato accento, nel colore dello sfondo pagina
          (`stroke-bg`): si legge in Scuro, Nero e Chiaro (in Chiaro l'accento testo è ambra scura,
          quindi `on-accent`, scuro, non basterebbe). Il colore va sul tratto dell'icona: il Button
          Menu tinge di grigio ogni svg che contiene. Dove metterla: la proposta qui sotto.
        </template>
      </KitIssue>

      <KitIssue
        title="Il riquadro è scritto a mano in ogni schermata"
        :status="IssueStatus.Proposed"
      >
        La primitiva non disegna nulla: la casella (span 14 × 14, bordo, riempimento) è copiata
        uguale in LibraryFilters.vue e in TvModeDialog.vue, e ogni copia legge lo stato dalla
        variabile della schermata, non da Reka. Una terza schermata la copierebbe di nuovo, e ogni
        cambio di aspetto (la spunta qui sopra) va fatto in ogni copia.
        <template #current>
          <div class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-model="remember">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
                <span
                  class="size-14 shrink-0 rounded-4"
                  :class="remember ? 'bg-accent-text' : 'border border-line-strong'"
                  aria-hidden="true"
                ></span>
                Don't ask again
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-model="remember">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
                <CheckboxBoxProposal :checked="remember" />
                Don't ask again
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #solution>
          una parte `CheckboxBox.vue` in components/ui/checkbox (su `CheckboxIndicator` di Reka, lo
          stato da `data-state`), messa dentro il Button della schermata; LibraryFilters e
          TvModeDialog tolgono il loro span. Il Button resta della schermata, come oggi.
        </template>
      </KitIssue>

      <KitIssue title="Riga troppo bassa al tocco" :status="IssueStatus.Proposed">
        Le righe sono Button Menu · MenuRow: 28 px anche sul telefono, sotto il minimo touch di 44
        (token `touch`). Il pannello L03 si apre anche dalla libreria del telefono, e le caselle
        «Mostra» stanno a 2 px l'una dall'altra: è facile toccare quella sbagliata.
        <template #current>
          <div data-density="touch" class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-for="k in TWO" :key="k" v-model="show[k]">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" class="-mx-6">
                <span
                  class="size-14 shrink-0 rounded-4"
                  :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
                  aria-hidden="true"
                ></span>
                {{ LABELS[k] }}
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #proposed>
          <div data-density="touch" class="flex w-field flex-col gap-2 px-6">
            <Checkbox v-for="k in TWO" :key="k" v-model="show[k]">
              <Button
                :variant="ButtonVariant.Menu"
                :size="ButtonSize.MenuRow"
                class="-mx-6 touch:min-h-touch"
              >
                <span
                  class="size-14 shrink-0 rounded-4"
                  :class="show[k] ? 'bg-accent-text' : 'border border-line-strong'"
                  aria-hidden="true"
                ></span>
                {{ LABELS[k] }}
              </Button>
            </Checkbox>
          </div>
        </template>
        <template #solution>
          `touch:min-h-touch` in ButtonSize.MenuRow (components/ui/button/variants.ts). Vale anche
          per l'ordinamento (RadioGroup) dello stesso pannello, che usa la stessa misura.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
