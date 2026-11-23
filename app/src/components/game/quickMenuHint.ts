// G06: which keys the hint over the game names (Trello #118). Pure.
//
// A computer names both, Esc and Home, as it always did. A phone names only what is there: Esc
// once a physical keyboard has been used, Home while a controller is connected; with neither the
// quick menu opens from the touch controls' own menu button, which is on screen, and the hint
// does not show at all.

/** The keys the hint names, or null when it does not show. */
export interface HintKeys {
  esc: boolean
  home: boolean
}

export function quickMenuHintKeys(
  phone: boolean,
  keyboardSeen: boolean,
  controller: boolean,
): HintKeys | null {
  if (!phone) return { esc: true, home: true }
  if (!keyboardSeen && !controller) return null
  return { esc: keyboardSeen, home: controller }
}
