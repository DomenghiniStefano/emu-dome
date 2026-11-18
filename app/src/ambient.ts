// The ambient haze's colours (design system «Colore dinamico»): three colours of the cover in the
// foreground, OKLCH lightness at most 0.72, as `--cover-1/2/3`. The library says which game is
// in the foreground (`ambientGame`: the one under the pointer or the focus, else the first
// «Continua a giocare»); the shell's haze fades to its colours over `duration-ambient`.
//
// A cover image (the user's, or the icon in the ROM) is sampled in a small canvas (the
// `emu://cover` responses allow it: Access-Control-Allow-Origin); a generated cover is one colour,
// so its three spots are that colour at three lightness steps.
import { ref } from 'vue'
import { capLightness, hexToOklch, oklchToHex, toHex } from './colour'
import type { LibraryGame } from './emu'

/** The design system's cap on cover colours. */
export const COVER_MAX_L = 0.72

export type CoverColours = [string, string, string]

/** The game whose cover colours the haze shows; null: the design's fallback spots. */
export const ambientGame = ref<LibraryGame | null>(null)

const cache = new Map<string, Promise<CoverColours>>()

/** One colour at three lightness steps (generated covers, and images with a single colour). */
function steps(hex: string): CoverColours {
  const c = hexToOklch(capLightness(hex, COVER_MAX_L))
  return [
    oklchToHex(c),
    oklchToHex({ ...c, l: Math.max(0, c.l - 0.1) }),
    oklchToHex({ ...c, l: Math.max(0, c.l - 0.2) }),
  ]
}

/**
 * The three most present colourful colours of an image (buckets of 4 bits per channel, weighted
 * by chroma so a grey background does not win), each capped at L 0.72.
 */
function sample(url: string): Promise<CoverColours | null> {
  return new Promise(resolve => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onerror = () => resolve(null)
    img.onload = () => {
      try {
        const size = 48
        const canvas = document.createElement('canvas')
        canvas.width = size
        canvas.height = size
        const ctx = canvas.getContext('2d', { willReadFrequently: true })
        if (!ctx) return resolve(null)
        ctx.drawImage(img, 0, 0, size, size)
        const data = ctx.getImageData(0, 0, size, size).data
        const buckets = new Map<number, { w: number; r: number; g: number; b: number }>()
        for (let i = 0; i < data.length; i += 4) {
          if (data[i + 3] < 128) continue
          const [r, g, b] = [data[i], data[i + 1], data[i + 2]]
          const chroma = (Math.max(r, g, b) - Math.min(r, g, b)) / 255
          const key = ((r >> 4) << 8) | ((g >> 4) << 4) | (b >> 4)
          const e = buckets.get(key) ?? { w: 0, r: 0, g: 0, b: 0 }
          const w = 0.15 + chroma
          e.w += w
          e.r += r * w
          e.g += g * w
          e.b += b * w
          buckets.set(key, e)
        }
        const picked: string[] = []
        for (const e of [...buckets.values()].sort((x, y) => y.w - x.w)) {
          const hex = toHex([e.r / e.w / 255, e.g / e.w / 255, e.b / e.w / 255])
          const c = hexToOklch(hex)
          // Distinct enough from the ones already picked (hue or lightness).
          const far = picked.every(p => {
            const q = hexToOklch(p)
            const dh = Math.min(Math.abs(q.h - c.h), 360 - Math.abs(q.h - c.h))
            return dh > 25 || Math.abs(q.l - c.l) > 0.15
          })
          if (far) picked.push(capLightness(hex, COVER_MAX_L))
          if (picked.length === 3) break
        }
        if (picked.length === 0) return resolve(null)
        if (picked.length < 3) return resolve(steps(picked[0]))
        resolve(picked as CoverColours)
      } catch {
        resolve(null)
      }
    }
    img.src = url
  })
}

/** The cover colours of a game, computed once per cover (its URL changes with the cover). */
export function coverColours(game: LibraryGame): Promise<CoverColours> {
  const key = `${game.id}|${game.cover}|${game.coverUrl ?? game.colour}`
  let found = cache.get(key)
  if (!found) {
    found =
      game.cover !== 'generated' && game.coverUrl
        ? sample(game.coverUrl).then(c => c ?? steps(game.colour))
        : Promise.resolve(steps(game.colour))
    cache.set(key, found)
  }
  return found
}
