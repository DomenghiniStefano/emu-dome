<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectTriggerVariant,
  SelectValue,
} from '@/components/ui/select'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const options = [
  { value: 'system', label: 'Like the system' },
  { value: 'en', label: 'English' },
  { value: 'it', label: 'Italiano' },
  { value: 'later', label: 'Not yet', disabled: true },
]
const language = ref('system')
// `audio.out`: the devices come from the system, with names as long as the driver makes them.
const devices = [
  { value: 'default', label: 'Like the system' },
  { value: 'realtek', label: 'Speakers (Realtek(R) High Definition Audio with a long name)' },
]
const device = ref('realtek')
const deviceNow = ref('realtek')
const deviceProposed = ref('realtek')
</script>

<template>
  <KitSection title="Select">
    <KitState label="Closed (A / Enter opens; B, Escape or a click outside close)">
      <Select v-model="language">
        <SelectTrigger aria-label="Language"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem v-for="o in options" :key="o.value" :value="o.value" :disabled="o.disabled">
            {{ o.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </KitState>
    <KitState label="Disabled">
      <Select model-value="en" disabled>
        <SelectTrigger aria-label="Language"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem v-for="o in options" :key="o.value" :value="o.value">{{
            o.label
          }}</SelectItem>
        </SelectContent>
      </Select>
    </KitState>
    <KitState label="InTitle: inside a title, in its colour and weight">
      <span class="text-accent-text">
        <Select v-model="language">
          <SelectTrigger :variant="SelectTriggerVariant.InTitle" aria-label="Language">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="o in options" :key="o.value" :value="o.value" :disabled="o.disabled">
              {{ o.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </span>
    </KitState>
    <KitState label="A long value (an audio device's name, audio.out) in a settings row's room">
      <div class="flex w-field">
        <Select v-model="device">
          <SelectTrigger aria-label="Audio output"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem v-for="o in devices" :key="o.value" :value="o.value">
              {{ o.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
    </KitState>
    <template #review>
      <KitIssue title="Due colori di bordo per i campi" :status="IssueStatus.Proposed">
        Select e DateTimeField hanno il bordo line (bianco al 7% in Scuro, 1,47:1 contro lo sfondo);
        Input e SearchField il bordo line-strong (4,2:1). Nella stessa pagina delle impostazioni una
        tendina e un campo di testo uno sotto l'altro hanno contorni diversi, e il contorno della
        tendina è sotto il 3:1 chiesto per il bordo di un controllo.
        <template #current>
          <div class="flex flex-col items-start gap-8">
            <Select model-value="en">
              <SelectTrigger aria-label="Edge, now"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
            <Input model-value="wfc.example" aria-label="Edge, a text field" />
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col items-start gap-8">
            <Select model-value="en">
              <SelectTrigger aria-label="Edge, proposed" class="border-line-strong">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
            <Input model-value="wfc.example" aria-label="Edge, a text field too" />
          </div>
        </template>
        <template #solution>
          Field di select/variants.ts e di date-time-field/variants.ts passa a border-line-strong,
          come Input e SearchField: un solo bordo per «campo».
        </template>
      </KitIssue>
      <KitIssue title="Un valore lungo va a capo e esce dal campo" :status="IssueStatus.Proposed">
        Il campo è alto 32 fisso ma il testo può andare a capo: il nome di un dispositivo audio
        (audio.out) o di un'opzione lunga in una riga stretta va su due righe e la seconda esce dal
        bordo (stato «A long value» qui sopra). Su telefono succede con quasi ogni dispositivo.
        <template #current>
          <div class="flex w-field">
            <Select v-model="deviceNow">
              <SelectTrigger aria-label="Long value, now"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in devices" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-field">
            <Select v-model="deviceProposed">
              <SelectTrigger
                aria-label="Long value, proposed"
                class="min-w-0 flex-1 [&>span]:min-w-0 [&>span]:truncate"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in devices" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </template>
        <template #solution>
          In SelectTrigger.vue il testo sta su una riga con i puntini (truncate, min-w-0) e il campo
          può restringersi nel suo contenitore; il nome intero si legge aprendo la lista, che resta
          larga quanto serve.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
