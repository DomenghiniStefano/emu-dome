<script setup lang="ts">
// DropdownMenu: a game's «…» (CoverMenu.vue, from a cover and from the game details). The live
// one opens from its own button as the app does (the screen holds `open`, the menu hangs from
// the button); the others draw the open menu's look in place (dropdownMenuVariants on a div, the
// items as the app's Menu · MenuItem Buttons), since Reka's items need an open menu around them.
import { Ellipsis, EyeOff, ImageIcon, Star } from '@lucide/vue'
import { ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  dropdownMenuVariants,
} from '@/design-system/components/dropdown-menu'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { IssueStatus } from '@/design-system/kit/review'

const open = ref(false)
const trigger = ref<InstanceType<typeof Button> | null>(null)
const anchor = () => (trigger.value?.$el as HTMLElement | undefined) ?? null
const menuLook = dropdownMenuVariants()
</script>

<template>
  <KitSection title="DropdownMenu">
    <KitState label="Live: Soft · Tall «…» opens it (A, Enter or a click; B / Esc close)">
      <div class="relative">
        <Button
          ref="trigger"
          :size="ButtonSize.Tall"
          aria-label="More"
          aria-haspopup="menu"
          :aria-expanded="open"
          @click="open = !open"
        >
          <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Button>
        <DropdownMenu v-if="open" :open="true" @update:open="o => o || (open = false)">
          <DropdownMenuContent :anchor="anchor()" aria-label="More for Kit Game">
            <DropdownMenuItem @select="open = false">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem" data-nav-initial>
                <ImageIcon :size="16" :stroke-width="1.75" aria-hidden="true" />
                Choose the cover…
              </Button>
            </DropdownMenuItem>
            <DropdownMenuItem @select="open = false">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
                <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
                Add to favourites
              </Button>
            </DropdownMenuItem>
            <DropdownMenuItem @select="open = false">
              <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
                <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
                Hide from the library
              </Button>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </KitState>
    <KitState label="Open look · Actions: three items (game details)">
      <div :class="menuLook">
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <ImageIcon :size="16" :stroke-width="1.75" aria-hidden="true" />
          Choose the cover…
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
          Hide from the library
        </Button>
      </div>
    </KitState>
    <KitState label="Open look · two items (a cover's «…»), one aria-disabled">
      <div :class="menuLook">
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem" aria-disabled="true">
          <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
          Hide from the library
        </Button>
      </div>
    </KitState>
    <KitState label="Open look · long Italian item (nowrap: the menu grows)">
      <div :class="menuLook">
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
          Mostra di nuovo nella libreria di questo dispositivo
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
          Togli dai preferiti
        </Button>
      </div>
    </KitState>
    <KitState label="Open look on the phone (touch density)">
      <div data-density="touch" :class="menuLook">
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <ImageIcon :size="16" :stroke-width="1.75" aria-hidden="true" />
          Choose the cover…
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <Star :size="16" :stroke-width="1.75" aria-hidden="true" />
          Add to favourites
        </Button>
        <Button :variant="ButtonVariant.Menu" :size="ButtonSize.MenuItem">
          <EyeOff :size="16" :stroke-width="1.75" aria-hidden="true" />
          Hide from the library
        </Button>
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Nessuna larghezza massima" :status="IssueStatus.Open">
        Il menu ha una larghezza minima (`menu-min`, 220) ma non una massima, e le voci non vanno a
        capo (il Button è `whitespace-nowrap`). Oggi le voci italiane sono corte, ma una traduzione
        più lunga allarga il menu senza limite: sul telefono, aperto dal «…» in alto a destra,
        uscirebbe dallo schermo a sinistra. Da decidere: un massimo (es. `panel` o la larghezza
        dello schermo meno i margini) con le voci che vanno a capo, oppure il testo troncato.
      </KitIssue>
    </template>
  </KitSection>
</template>
