<script setup lang="ts">
// The shape a touch button shows, drawn inside its box (the Pad sizes of ui/button: the box is
// what takes the touch, the shape may be smaller). It has no colour of its own: the Pad variant
// of the button around it sets `fill` and `stroke` (inherited), so the style (`touch.tskin`) and
// the pressed look are the variant's. One opaque edge per shape; the menu's lines in the
// letters' colour. Viewboxes in the boxes' CSS pixels (spacing.css `pad-*`).
// - face: A B X Y, a circle as large as the box;
// - shoulderL / shoulderR: L and R, 26 tall in the 36 strip, the outer top corner rounded;
// - pill: Select / Start, 20 tall in the 32 box;
// - menu: a small square with three lines.
export type PadShapeKind = 'face' | 'shoulderL' | 'shoulderR' | 'pill' | 'menu'

defineProps<{ kind: PadShapeKind }>()

/** The edge's width, and the inset that keeps it inside the box. */
const EDGE = 1.5
const IN = EDGE / 2

/** A shoulder 84 × 26 centred in its 84 × 36 box: outer top corner 12, the others 3 (L's outer
 * side is its left; R is the same path mirrored). */
const SHOULDER = (() => {
  const [x0, x1, y0, y1, R, r] = [IN, 84 - IN, 5, 31, 12, 3]
  return [
    `M${x0 + R} ${y0}`,
    `H${x1 - r} Q${x1} ${y0} ${x1} ${y0 + r}`,
    `V${y1 - r} Q${x1} ${y1} ${x1 - r} ${y1}`,
    `H${x0 + r} Q${x0} ${y1} ${x0} ${y1 - r}`,
    `V${y0 + R} Q${x0} ${y0} ${x0 + R} ${y0} Z`,
  ].join(' ')
})()
</script>

<template>
  <svg
    v-if="kind === 'face'"
    class="pointer-events-none absolute inset-0 size-full overflow-visible"
    viewBox="0 0 48 48"
    aria-hidden="true"
  >
    <circle cx="24" cy="24" :r="24 - IN" :stroke-width="EDGE" />
  </svg>
  <svg
    v-else-if="kind === 'shoulderL' || kind === 'shoulderR'"
    class="pointer-events-none absolute inset-0 size-full overflow-visible"
    viewBox="0 0 84 36"
    aria-hidden="true"
  >
    <path
      :d="SHOULDER"
      :stroke-width="EDGE"
      stroke-linejoin="round"
      :transform="kind === 'shoulderR' ? 'matrix(-1 0 0 1 84 0)' : undefined"
    />
  </svg>
  <svg
    v-else-if="kind === 'pill'"
    class="pointer-events-none absolute inset-0 size-full overflow-visible"
    viewBox="0 0 60 32"
    aria-hidden="true"
  >
    <rect :x="IN" y="6" :width="60 - EDGE" height="20" rx="10" :stroke-width="EDGE" />
  </svg>
  <svg
    v-else
    class="pointer-events-none absolute inset-0 size-full overflow-visible"
    viewBox="0 0 36 36"
    aria-hidden="true"
  >
    <rect x="3" y="5" width="30" height="26" rx="4" :stroke-width="EDGE" />
    <g fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
      <line x1="11" y1="13" x2="25" y2="13" />
      <line x1="11" y1="18" x2="25" y2="18" />
      <line x1="11" y1="23" x2="25" y2="23" />
    </g>
  </svg>
</template>
