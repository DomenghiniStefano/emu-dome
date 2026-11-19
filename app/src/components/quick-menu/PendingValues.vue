<script setup lang="ts">
// The quick menu's values that no design-system token holds yet (literal colours and shadows),
// named once here as `--qm-*` variables around the menu and read by its parts through `var()`.
// Awaiting Cowork (spec Decision 5, «Handed back to design»): when they become tokens this file
// goes, and with it the quick menu's last entry on the scanner's LEGACY list. No box of its own.
</script>

<template>
  <div class="pending-values"><slot /></div>
</template>

<style scoped>
.pending-values {
  display: contents;
  /* The paused game behind the menu. */
  --qm-scrim: #00000099;
  /* The panel's lit top edge and its deep shadow (the design system's float shadow is lighter). */
  --qm-panel-shadow: inset 0 1px 0 var(--glass-edge), 0 32px 96px #000000a6;
  /* The phone's sheet, lifted off the game above it. */
  --qm-sheet-shadow: 0 -24px 64px #0000008c;
  /* The sheet's grip. */
  --qm-grip: #ffffff33;
  /* A key hint's edge on the accent fill of the main action. */
  --qm-hint-edge-on-accent: #ffffff80;
  /* A G09 card changed «in prova». */
  --qm-trial-fill: color-mix(in srgb, var(--violet) 10%, transparent);
}
</style>
