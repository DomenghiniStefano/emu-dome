<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { Bell, ChevronDown } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Popover, PopoverContent, PopoverVariant } from '@/components/ui/popover'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

// The screen holds `open` and its own button, which takes the focus back on closing.
const panelOpen = ref(false)
const panelButton = ref<InstanceType<typeof Button> | null>(null)
const bellOpen = ref(false)
const bellButton = ref<InstanceType<typeof Button> | null>(null)
const el = (b: InstanceType<typeof Button> | null) => (b?.$el as HTMLElement | undefined) ?? null
function closePanel() {
  panelOpen.value = false
  void nextTick(() => el(panelButton.value)?.focus())
}
function closeBell() {
  bellOpen.value = false
  void nextTick(() => el(bellButton.value)?.focus())
}
const favourites = ref(true)
</script>

<template>
  <KitSection title="Popover">
    <KitState label="Panel: a panel under its button (L03), with a Checkbox row">
      <Popover :open="panelOpen" @update:open="o => o || closePanel()">
        <Button
          ref="panelButton"
          :variant="ButtonVariant.Quiet"
          :aria-expanded="panelOpen"
          aria-haspopup="dialog"
          @click="panelOpen ? closePanel() : (panelOpen = true)"
        >
          Sort by: recent
          <ChevronDown :size="14" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <PopoverContent :anchor="el(panelButton)" aria-label="Sort and filter">
          <Checkbox :model-value="favourites" @update:model-value="favourites = !favourites">
            <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuRow" data-nav-initial>
              <span
                class="size-14 shrink-0 rounded-4"
                :class="favourites ? 'bg-accent-text' : 'border border-line-strong'"
                aria-hidden="true"
              ></span>
              Favourites only
            </Button>
          </Checkbox>
          <Button @click="closePanel">Reset</Button>
        </PopoverContent>
      </Popover>
    </KitState>
    <KitState label="Notifications: the bell's panel (X03), its list scrolling when short">
      <Popover :open="bellOpen" @update:open="o => o || closeBell()">
        <Button
          ref="bellButton"
          :variant="ButtonVariant.BarIcon"
          :size="ButtonSize.Icon"
          aria-label="Notifications"
          aria-haspopup="dialog"
          :aria-expanded="bellOpen"
          @click="bellOpen ? closeBell() : (bellOpen = true)"
        >
          <Bell :size="20" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <PopoverContent
          :anchor="el(bellButton)"
          :variant="PopoverVariant.Notifications"
          aria-label="Notifications"
        >
          <Button
            v-for="n in 2"
            :key="n"
            :variant="ButtonVariant.Result"
            :size="ButtonSize.Notice"
            :data-nav-initial="n === 1 ? '' : undefined"
          >
            <span class="flex min-w-0 flex-1 flex-col gap-2">
              <span class="font-medium">Controller connected</span>
              <span class="text-caption text-ink-muted">Xbox Wireless Controller</span>
            </span>
            <span class="flex-none text-caption text-ink-muted">{{ n * 5 }} minutes ago</span>
          </Button>
        </PopoverContent>
      </Popover>
    </KitState>
  </KitSection>
</template>
