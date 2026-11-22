<script setup lang="ts">
// TV mode's controller glyphs (#30): the button that carries out each action, from the routing
// and `pad.confirm`, drawn for the connected controller's family. With no controller connected
// (as here) the generic Nintendo letters; the d-pad's ◀ ▶ as arrows.
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { KbdVariant } from '@/components/ui/kbd'
import TvGlyph from '@/components/shell/TvGlyph.vue'
import type { PadAction } from '@/lib/padGlyphs'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'

const ACTIONS: PadAction[] = ['confirm', 'back', 'more', 'search', 'prev', 'next', 'left', 'right']
</script>

<template>
  <KitSection title="TV glyph" :group="KitGroup.Game">
    <KitState label="Every action: face buttons round, shoulders and d-pad squared">
      <div class="flex items-center gap-12">
        <TvGlyph v-for="a in ACTIONS" :key="a" :action="a" />
      </div>
    </KitState>
    <KitState label="On the main action's accent">
      <Button :variant="ButtonVariant.Primary" :size="ButtonSize.Tv">
        <TvGlyph action="confirm" :variant="KbdVariant.OnAccent" />
        Gioca
      </Button>
    </KitState>
  </KitSection>
</template>
