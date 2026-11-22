<script setup lang="ts">
// The rows of a list card (setting rows, link rows, shortcut rows) in as many columns of at least
// `panel` as the card holds: one on a small window, two or more on a wide one, so a row's name and
// its control never sit a whole window apart and no band is left empty. The rows keep their
// order, down the first column, then the next; a line between the columns. A row that needs the
// whole width (choice cards) takes `[column-span:all]`.
// The rows draw their own line on top (none on the first): the column's top row would draw it
// under the card's edge, so the columns start 1px up, under the card's clip, and the first row,
// which has no line, comes 1px down to stay level with the others. The rule between the columns
// reads the theme's own `--line` (`--color-line` is a build-time name, colors.css).
</script>

<template>
  <div
    class="-mt-px columns-[var(--spacing-panel)] gap-x-0 [column-rule:var(--default-border-width)_solid_var(--line)] *:break-inside-avoid [&>:first-child]:mt-px"
  >
    <slot />
  </div>
</template>
