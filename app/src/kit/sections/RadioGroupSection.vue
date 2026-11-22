<script setup lang="ts">
import { ref } from 'vue'
import { Layers, Gamepad2, Disc3 } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const level = ref('global')
const order = ref('recent')
const LEVELS = [
  { id: 'global', name: 'All games', icon: Layers },
  { id: 'console', name: 'Nintendo DS games', icon: Gamepad2 },
  { id: 'game', name: 'Only this game', icon: Disc3 },
]
</script>

<template>
  <KitSection title="RadioGroup">
    <KitState label="Cards in a row (the settings scope bar): arrows along the row choose">
      <RadioGroup v-model="level" class="flex gap-6" aria-label="Level">
        <RadioGroupItem v-for="l in LEVELS" :key="l.id" :value="l.id">
          <Card
            as="button"
            :variant="CardVariant.RaisedAccent"
            :selected="level === l.id"
            class="flex-row items-center gap-6"
          >
            <component :is="l.icon" :size="16" :stroke-width="1.75" aria-hidden="true" />
            <span class="font-semibold">{{ l.name }}</span>
          </Card>
        </RadioGroupItem>
      </RadioGroup>
    </KitState>
    <KitState label="Menu rows in a column (the library's order)">
      <RadioGroup
        v-model="order"
        orientation="vertical"
        class="flex w-field flex-col gap-px"
        aria-label="Order"
      >
        <RadioGroupItem v-for="o in ['recent', 'title', 'console']" :key="o" :value="o">
          <Button
            :variant="ButtonVariant.Menu"
            :size="ButtonSize.MenuRow"
            class="group"
            :data-selected="order === o ? '' : undefined"
          >
            <span
              class="size-14 shrink-0 rounded-full border border-line-strong group-data-selected:border-check group-data-selected:border-accent-text"
              aria-hidden="true"
            ></span>
            {{ o }}
          </Button>
        </RadioGroupItem>
      </RadioGroup>
    </KitState>
    <template #review>
      <KitIssue title="Il pallino del radio lo disegna chi lo usa">
        Il cerchio delle righe in colonna (bordo line-strong, più spesso e in accento quando è
        scelto) non è nel primitivo: LibraryFilters.vue lo scrive a mano con le sue classi, e il Kit
        le copia. Una seconda lista di radio dovrebbe copiarle ancora, e il pallino non ha il punto
        pieno che di solito dice «scelto». Lo stesso per il quadrato di Checkbox nello stesso
        pannello (pieno, senza segno di spunta: è della sezione Checkbox).
        <template #solution>
          Una parte RadioGroupIndicator in components/ui/radio-group (cerchio con un punto pieno in
          accento quando è scelto), usata da LibraryFilters.vue e dal Kit; nessuna classe del
          pallino fuori da ui/.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
