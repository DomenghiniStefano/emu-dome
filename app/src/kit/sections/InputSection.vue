<script setup lang="ts">
import { ref } from 'vue'
import { Search } from '@lucide/vue'
import { Input, InputVariant } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const server = ref('')
const query = ref('')
const empty = ref('')
const filled = ref('Pocket Quest')
</script>

<template>
  <KitSection title="Input">
    <KitState label="Field, with a placeholder">
      <Input
        v-model="server"
        :variant="InputVariant.Field"
        aria-label="Server"
        placeholder="None"
      />
    </KitState>
    <KitState label="Field · disabled">
      <Input :variant="InputVariant.Field" model-value="wfc.example" aria-label="Server" disabled />
    </KitState>
    <KitState label="Block · empty, with a placeholder">
      <Input
        v-model="empty"
        :variant="InputVariant.Block"
        class="w-field"
        placeholder="Pocket Quest Advance"
      />
    </KitState>
    <KitState label="Block · filled">
      <Input v-model="filled" :variant="InputVariant.Block" class="w-field" />
    </KitState>
    <KitState label="Block · disabled">
      <Input v-model="filled" :variant="InputVariant.Block" class="w-field" disabled />
    </KitState>
    <KitState label="Bare, inside a box its container draws (a search)">
      <label
        class="flex h-control w-sidebar items-center gap-6 rounded-control border border-line-strong bg-surface-raised px-field-pad text-ink-muted"
      >
        <Search :size="16" :stroke-width="1.75" aria-hidden="true" />
        <Input v-model="query" :variant="InputVariant.Bare" type="search" placeholder="Search" />
      </label>
    </KitState>
    <KitState label="Palette: the search everywhere's field, in its row">
      <div class="flex h-52 w-panel items-center gap-10 border-b border-line px-16">
        <Search class="text-ink-muted" :size="20" :stroke-width="1.75" aria-hidden="true" />
        <Input
          v-model="query"
          :variant="InputVariant.Palette"
          type="search"
          placeholder="Search games, actions and settings"
        />
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Il segnaposto di Field e Bare è del browser" :status="IssueStatus.Proposed">
        Block e Palette colorano il segnaposto in ink-muted; Field e Bare no, e prendono il grigio
        di Chromium, che non segue i temi (in Chiaro e in Scuro non è né ink-muted né ink-faint). Il
        campo «Server» delle impostazioni (Field) e la ricerca delle impostazioni (Bare) hanno
        quindi un segnaposto di un altro colore rispetto al nome mostrato di un gioco (Block).
        <template #current>
          <div class="flex flex-col gap-8">
            <Input aria-label="Placeholder, now" placeholder="None" />
            <Input :variant="InputVariant.Block" class="w-field" placeholder="None (Block)" />
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col gap-8">
            <Input
              aria-label="Placeholder, proposed"
              placeholder="None"
              class="placeholder:text-ink-muted"
            />
            <Input :variant="InputVariant.Block" class="w-field" placeholder="None (Block)" />
          </div>
        </template>
        <template #solution>
          placeholder:text-ink-muted nella base di inputVariants (input/variants.ts) e nell'input di
          SearchField, invece che in due varianti sole.
        </template>
      </KitIssue>
      <KitIssue
        title="Valore regular nel campo, medium nella tendina"
        :status="IssueStatus.Proposed"
      >
        Input scrive il valore in body (peso 400), Select e DateTimeField in label (500). In una
        pagina di impostazioni i valori scelti hanno due pesi a seconda del controllo.
        <template #current>
          <div class="flex flex-col items-start gap-8">
            <Input model-value="wfc.example" aria-label="Weight, a text field" />
            <Select model-value="en">
              <SelectTrigger aria-label="Weight, now"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col items-start gap-8">
            <Input model-value="wfc.example" aria-label="Weight, a text field too" />
            <Select model-value="en">
              <SelectTrigger aria-label="Weight, proposed" class="font-regular">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </template>
        <template #solution>
          Un valore è contenuto, non un'etichetta: Select e DateTimeField scrivono il valore in body
          (select/variants.ts, date-time-field/variants.ts). Oppure tutti in label: lo sceglie il
          proprietario, purché sia uno.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
