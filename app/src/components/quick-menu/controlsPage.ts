// What G10 «Controlli» says about the input (#150, the rule of #118: only the input that is
// there). Pure.

/** The page's lines that depend on the input in use. */
export interface ControlsPageInput {
  /** The «Controller in uso» card's line with no controller (a message key). */
  noController: 'qm.noControllerTouch' | 'qm.noController'
  /** The controller shortcuts' card: only with a controller, or on a computer. */
  shortcuts: boolean
}

/**
 * `touch`: a touch screen is the main pointer; `controller`: a controller is connected. On a
 * touch screen with no controller the touch controls are what plays the game: the card says so
 * and the controller shortcuts wait for a controller. A computer keeps the keyboard's line and
 * the shortcuts (a controller can be plugged in at any time and the page explains them).
 */
export function controlsPageInput(touch: boolean, controller: boolean): ControlsPageInput {
  const touchOnly = touch && !controller
  return {
    noController: touchOnly ? 'qm.noControllerTouch' : 'qm.noController',
    shortcuts: !touchOnly,
  }
}
