<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { ChevronDown, Ellipsis, Star, EyeOff } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import { Popover, PopoverContent } from '@/components/ui/popover'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

// The screen holds `open` and its own button, which takes the focus back on closing.
const panelOpen = ref(false)
const panelButton = ref<InstanceType<typeof Button> | null>(null)
const menuOpen = ref(false)
const menuButton = ref<InstanceType<typeof Button> | null>(null)
const el = (b: InstanceType<typeof Button> | null) => (b?.$el as HTMLElement | undefined) ?? null
function closePanel() {
  panelOpen.value = false
  void nextTick(() => el(panelButton.value)?.focus())
}
function closeMenu() {
  menuOpen.value = false
  void nextTick(() => el(menuButton.value)?.focus())
}
const favourites = ref(true)
</script>

<template>
  <KitSection title="Popover, DropdownMenu, Checkbox">
    <KitState label="Popover: a panel under its button (L03)">
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
    <KitState label="DropdownMenu: a game's «…»">
      <DropdownMenu :open="menuOpen" @update:open="o => o || closeMenu()">
        <Button
          ref="menuButton"
          :variant="ButtonVariant.Soft"
          :size="ButtonSize.Tall"
          aria-label="More"
          aria-haspopup="menu"
          :aria-expanded="menuOpen"
          @click="menuOpen ? closeMenu() : (menuOpen = true)"
        >
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <DropdownMenuContent :anchor="el(menuButton)" aria-label="More">
          <DropdownMenuItem @select="closeMenu">
            <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
              <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
              Add to favourites
            </Button>
          </DropdownMenuItem>
          <DropdownMenuItem @select="closeMenu">
            <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
              <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
              Hide from the library
            </Button>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </KitState>
  </KitSection>
</template>
