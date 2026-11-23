<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@/design-system/components/input'
import { Segmented, SegmentedItem } from '@/design-system/components/segmented'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/design-system/components/select'
import { Slider } from '@/design-system/components/slider'
import { Switch } from '@/design-system/components/switch'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'

const on = ref(true)
const off = ref(false)
</script>

<template>
  <KitSection title="Switch">
    <KitState label="On">
      <Switch v-model="on" aria-label="On" />
    </KitState>
    <KitState label="Off">
      <Switch v-model="off" aria-label="Off" />
    </KitState>
    <KitState label="On · disabled">
      <Switch :model-value="true" disabled aria-label="On, disabled" />
    </KitState>
    <KitState label="Off · disabled">
      <Switch :model-value="false" disabled aria-label="Off, disabled" />
    </KitState>
    <template #review>
      <KitIssue
        title="Disabilitati uguali agli attivi: Switch, Segmented, Slider, Select, Input"
        :status="IssueStatus.Proposed"
      >
        Questi cinque primitivi da disabilitati cambiano solo il cursore. Nelle impostazioni una
        riga bloccata (livello non accettato, altro dispositivo, «in arrivo») sbiadisce solo il nome
        (SettingRowText): l'interruttore, i segmenti, lo slider, la tendina e il campo di testo
        restano pieni e sembrano usabili. Nella stessa pagina invece DateTimeField sbiadisce da sé
        (0,4) e Chip, card di scelta e campioni di colore sbiadiscono col loro contenitore (0,55).
        <template #current>
          <div class="flex flex-col items-start gap-8">
            <Switch :model-value="true" disabled aria-label="Example, disabled" />
            <Segmented model-value="a" disabled aria-label="Example segments">
              <SegmentedItem value="a">Normal</SegmentedItem>
              <SegmentedItem value="b">Fast</SegmentedItem>
            </Segmented>
            <Slider :model-value="40" disabled label="Example slider" />
            <Select model-value="en" disabled>
              <SelectTrigger aria-label="Example list"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
            <Input model-value="wfc.example" aria-label="Example field" disabled />
          </div>
        </template>
        <template #proposed>
          <div class="flex flex-col items-start gap-8">
            <Switch
              :model-value="true"
              disabled
              aria-label="Example, disabled"
              class="disabled:opacity-disabled"
            />
            <Segmented
              model-value="a"
              disabled
              aria-label="Example segments"
              class="opacity-disabled"
            >
              <SegmentedItem value="a">Normal</SegmentedItem>
              <SegmentedItem value="b">Fast</SegmentedItem>
            </Segmented>
            <Slider
              :model-value="40"
              disabled
              label="Example slider"
              class="data-[disabled]:opacity-disabled"
            />
            <Select model-value="en" disabled>
              <SelectTrigger aria-label="Example list" class="disabled:opacity-disabled">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
              </SelectContent>
            </Select>
            <Input
              model-value="wfc.example"
              aria-label="Example field"
              disabled
              class="disabled:opacity-disabled"
            />
          </div>
        </template>
        <template #solution>
          Lo sbiadito da disabilitato nel primitivo, come Button e DateTimeField: switch/Switch.vue,
          segmented/variants.ts (il gruppo, e il singolo segmento disabilitato), slider/Slider.vue,
          select/variants.ts, input/variants.ts (Field e Bare). Le righe delle impostazioni smettono
          di sbiadire Chip, card e campioni da fuori. Il valore è quello unico proposto in Button
          («Tre opacità per disabilitato»).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
