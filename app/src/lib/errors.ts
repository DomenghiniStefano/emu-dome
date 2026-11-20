// Where a failure goes when the screen that caused it does not show it (CLAUDE.md, engineering
// rule 6: errors never vanish). Every bridge call either handles its error where it is made
// (try / catch, a dialog, a row's message) or says here where it goes:
//   report(p)        the user sees it: logged, and shown as a toast;
//   ignore(p, why)   a deliberate ignore: logged only, `why` says why the user need not know;
//   settle(p)        like report, awaited: a sequence goes on after a step that failed.
// What escapes all of them (a rejected promise nobody awaited, an error in a component) reaches
// `catchEverything`'s handlers, which report it the same way. A bare `void emu.x()` is refused by
// ESLint (app/eslint.config.js).
import type { App } from 'vue'

type Sink = (e: unknown) => void

let sink: Sink | null = null

/** App.vue hands its toasts over once mounted; until then a failure is logged only. */
export function setErrorSink(to: Sink | null) {
  sink = to
}

/** A failure the user must see: logged, and shown as a toast once the toasts exist. */
export function reportError(e: unknown): void {
  console.error(e)
  sink?.(e)
}

/** Runs `p` in the background; its failure is reported. */
export function report(p: Promise<unknown>): void {
  p.catch(reportError)
}

/** Waits for `p`; its failure is reported, never thrown, so what follows still runs. */
export async function settle(p: Promise<unknown>): Promise<void> {
  try {
    await p
  } catch (e) {
    reportError(e)
  }
}

/** Runs `p` in the background and only logs its failure: `why` the user need not know. */
export function ignore(p: Promise<unknown>, why: string): void {
  p.catch(e => console.warn(why, e))
}

/**
 * The last net: an error thrown in a component (render, watcher, a handler, a promise an event
 * handler returned) and a rejected promise nobody awaited are reported instead of lost.
 */
export function catchEverything(app: App, target: Window = window) {
  app.config.errorHandler = err => reportError(err)
  target.addEventListener('unhandledrejection', e => {
    e.preventDefault()
    reportError(e.reason)
  })
}
