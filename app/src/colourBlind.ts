// `a11y.cblind` («Filtro per il daltonismo sul gioco»): a correction of the game's colours, not a
// simulation. Daltonization (Fidaner, Lin, Ozguven 2005): what a viewer with the deficiency
// cannot see (original − simulated) is moved onto the channels they can see. The simulation is
// Machado, Oliveira, Fernandes 2009 at full severity. Everything folds into one 3×3 matrix,
// applied as an SVG feColorMatrix on the webview's canvas. Pure.

export type ColourBlindMode = 'protan' | 'deutan' | 'tritan'
type M3 = number[][]

/** Machado et al. 2009, severity 1.0. */
const SIMULATION: Record<ColourBlindMode, M3> = {
  protan: [
    [0.152286, 1.052583, -0.204868],
    [0.114503, 0.786281, 0.099216],
    [-0.003882, -0.048116, 1.051998],
  ],
  deutan: [
    [0.367322, 0.860646, -0.227968],
    [0.280085, 0.672501, 0.047413],
    [-0.01182, 0.04294, 0.968881],
  ],
  tritan: [
    [1.255528, -0.076749, -0.178779],
    [-0.078411, 0.930809, 0.147602],
    [0.004733, 0.691367, 0.3039],
  ],
}

/** Where the lost information goes: red–green losses to green and blue, blue losses to red and
 * green (the daltonize shift matrices). */
const SHIFT: Record<ColourBlindMode, M3> = {
  protan: [
    [0, 0, 0],
    [0.7, 1, 0],
    [0.7, 0, 1],
  ],
  deutan: [
    [0, 0, 0],
    [0.7, 1, 0],
    [0.7, 0, 1],
  ],
  tritan: [
    [1, 0, 0.7],
    [0, 1, 0.7],
    [0, 0, 0],
  ],
}

const mul = (a: M3, b: M3): M3 =>
  a.map((row, i) => b[0].map((_, j) => row.reduce((s, _v, k) => s + a[i][k] * b[k][j], 0)))

/** The correction matrix: I + Shift · (I − Simulation). */
export function correction(mode: ColourBlindMode): M3 {
  const s = SIMULATION[mode]
  const lost = s.map((row, i) => row.map((v, j) => (i === j ? 1 : 0) - v))
  const e = mul(SHIFT[mode], lost)
  return e.map((row, i) => row.map((v, j) => v + (i === j ? 1 : 0)))
}

/** The `values` of an `<feColorMatrix type="matrix">` (4×5, alpha unchanged). */
export function feColorMatrix(mode: ColourBlindMode): string {
  const m = correction(mode)
  const rows = m.map(r => [...r.map(v => v.toFixed(5)), '0'].join(' '))
  return [...rows, '0 0 0 1 0'].join(' ')
}

export function isColourBlindMode(v: unknown): v is ColourBlindMode {
  return v === 'protan' || v === 'deutan' || v === 'tritan'
}
