// The startup splash (Trello #101): index.html draws it from the first paint, before any script
// runs, and its own CSS shows the late bar after 2 s. Once the app is ready (App.vue: the first
// run or the shell decided and rendered; the Kit page once mounted) it fades out and is removed.

/** How long the fade may take before the splash is removed anyway (no `transitionend`: a
 *  hidden window, or a style that did not load). Longer than its 250 ms fade. */
export const SPLASH_REMOVE_AFTER_MS = 600

/** Fades the splash out over the app and removes it; again, or with no splash, does nothing. */
export function hideSplash(doc: Document = document): void {
  const splash = doc.getElementById('splash')
  if (!splash || splash.dataset.state === 'out') return
  splash.dataset.state = 'out'
  const remove = () => splash.remove()
  splash.addEventListener('transitionend', remove, { once: true })
  setTimeout(remove, SPLASH_REMOVE_AFTER_MS)
}
