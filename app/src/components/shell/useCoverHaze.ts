// The shell's ambient haze takes the colours of the cover in the foreground of the library
// (ambient.ts); elsewhere, and until a cover is known, the design's fallback spots (the
// @property initial values of --cover-1/2/3, tokens/colors.css).
import { computed, ref, watch, type Ref } from 'vue'
import { ambientGame, coverColours, type CoverColours } from '@/lib/ambient'
import type { Section } from './sections'

/** The haze's inline style: the three cover colours, or nothing for the fallback spots. */
export function hazeStyle(colours: CoverColours | null): Record<string, string> {
  return colours
    ? { '--cover-1': colours[0], '--cover-2': colours[1], '--cover-3': colours[2] }
    : {}
}

export function useCoverHaze(section: Ref<Section>) {
  const spots = ref<CoverColours | null>(null)
  const foreground = () => (section.value === 'library' ? ambientGame.value : null)
  watch(
    foreground,
    async game => {
      const colours = game ? await coverColours(game) : null
      // A newer game took the foreground while the colours were read: keep its own.
      if (foreground() === game) spots.value = colours
    },
    { immediate: true },
  )
  return computed(() => hazeStyle(spots.value))
}
