<script setup lang="ts">
// A warning about a save on the warning's soft fill (E04 the in-game save set aside, V05 an
// import that replaces it).
import SaveWarning from '@/components/saves/SaveWarning.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'
</script>

<template>
  <KitSection title="Save warning" :group="KitGroup.Settings">
    <KitState label="E04: the in-game save was set aside (Saves panel width)">
      <div class="w-panel max-w-full">
        <SaveWarning>
          Il salvataggio del gioco non si leggeva ed è stato messo da parte
          (pocket-quest.sav.broken-2026-11-22). Ripristina un backup qui sotto.
        </SaveWarning>
      </div>
    </KitState>
    <KitState label="V05: an import replaces the in-game save (import step width)">
      <div class="w-step max-w-full">
        <SaveWarning>
          Questo gioco ha già un salvataggio (oggi, 18:32). Verrà sostituito; la versione attuale
          resta tra i backup.
        </SaveWarning>
      </div>
    </KitState>
  </KitSection>
</template>
