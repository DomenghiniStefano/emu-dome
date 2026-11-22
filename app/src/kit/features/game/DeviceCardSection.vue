<script setup lang="ts">
// C01's devices: icon, name, state, «In uso» on the controller the section starts on; one of a
// radio group (choosing one sets what the rest of C01 shows).
import { ref } from 'vue'
import { Gamepad2, Keyboard } from '@lucide/vue'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Card, CardVariant } from '@/components/ui/card'
import DeviceCard from '@/components/controls/DeviceCard.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'

const chosen = ref('pad')
</script>

<template>
  <KitSection title="Device card" :group="KitGroup.Game">
    <KitState label="Controller in use and chosen · keyboard (click to choose)">
      <div class="grid w-page max-w-full grid-cols-3 gap-12">
        <DeviceCard
          :icon="Gamepad2"
          name="8BitDo Ultimate 2C Wireless Controller"
          detail="USB · batteria 80 %"
          :chosen="chosen === 'pad'"
          badge="In uso"
          @click="chosen = 'pad'"
        />
        <DeviceCard
          :icon="Gamepad2"
          name="Steam Controller"
          detail="Bluetooth"
          :chosen="chosen === 'steam'"
          @click="chosen = 'steam'"
        />
        <DeviceCard
          :icon="Keyboard"
          name="Tastiera"
          detail="Sempre disponibile"
          :chosen="chosen === 'keyboard'"
          @click="chosen = 'keyboard'"
        />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="L’icona è sempre verde" :status="IssueStatus.Proposed">
        L’icona di ogni dispositivo è in <code>text-success</code>, anche per la tastiera e per un
        controller non scelto: non dice niente (in elenco ci sono solo dispositivi collegati) e
        accanto al badge verde «In uso» e al bordo ambra della card scelta fa tre colori in una
        card. Il design system vuole un grafite neutro, il colore solo dove significa qualcosa.
        <template #current>
          <div class="w-tile-min">
            <DeviceCard
              class="w-full"
              :icon="Gamepad2"
              name="Steam Controller"
              detail="Bluetooth"
              :chosen="false"
            />
          </div>
          <div class="w-tile-min">
            <DeviceCard
              class="w-full"
              :icon="Gamepad2"
              name="8BitDo Ultimate 2C"
              detail="USB"
              chosen
              badge="In uso"
            />
          </div>
        </template>
        <template #proposed>
          <div class="w-tile-min">
            <Card as="div" :variant="CardVariant.Choice" class="flex-row items-center gap-12">
              <span
                class="grid size-control-lg flex-none place-items-center rounded-10 bg-surface-raised text-ink-muted"
              >
                <Gamepad2 :size="22" :stroke-width="1.75" aria-hidden="true" />
              </span>
              <span class="flex min-w-0 flex-1 flex-col">
                <span class="truncate font-semibold">Steam Controller</span>
                <span class="text-caption text-ink-muted">Bluetooth</span>
              </span>
            </Card>
          </div>
          <div class="w-tile-min">
            <Card
              as="div"
              :variant="CardVariant.Choice"
              selected
              class="flex-row items-center gap-12"
            >
              <span
                class="grid size-control-lg flex-none place-items-center rounded-10 bg-surface-raised text-accent-text"
              >
                <Gamepad2 :size="22" :stroke-width="1.75" aria-hidden="true" />
              </span>
              <span class="flex min-w-0 flex-1 flex-col">
                <span class="truncate font-semibold">8BitDo Ultimate 2C</span>
                <span class="text-caption text-ink-muted">USB</span>
              </span>
              <Badge :variant="BadgeVariant.Success" :size="BadgeSize.Pill">In uso</Badge>
            </Card>
          </div>
        </template>
        <template #solution>
          in DeviceCard.vue l’icona passa a <code>text-ink-muted</code>, e a
          <code>text-accent-text</code> sulla card scelta (come il testo delle altre scelte accese).
          Il verde resta solo al badge «In uso».
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
