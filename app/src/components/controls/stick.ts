// C04's live stick: where the dot is drawn for the stick's position, and how wide the dead zone's
// circle is, in a square pad of `size` px.

/** An SDL axis value (-32768..32767) as -1..1. */
export const axisUnit = (value: number | undefined): number => (value ?? 0) / 32767

/** The dot's top-left corner, so that it sits centred at rest and on the edge at full tilt. */
export function stickDot(x: number, y: number, size: number, dot: number) {
  const r = (size - dot) / 2
  return { left: r + x * r, top: r + y * r }
}

/** The dead zone's diameter: `dead` per cent of the pad. */
export const deadZoneDiameter = (size: number, dead: number): number => (size * dead) / 100
