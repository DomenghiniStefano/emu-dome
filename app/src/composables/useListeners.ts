// The backend's events a component follows while it is mounted. Subscribing is asynchronous (the
// bridge answers later), so a listener kept in a variable assigned after the `await` is never
// stopped when the component unmounts before the answer. Here every listener goes into one set,
// stopped with the component; a listener that arrives after that is stopped as it arrives.
import { onScopeDispose } from 'vue'

/** Stops one listener (Tauri's `UnlistenFn`). */
export type Unlisten = () => void

export interface ListenerSet {
  /** Keeps each listener once it is subscribed; resolves when all of them are. */
  add: (...subscriptions: Array<Unlisten | Promise<Unlisten>>) => Promise<void>
  /** Stops every listener, and from now on each one that arrives late. */
  stop: () => void
}

/** A set of listeners stopped together (no Vue: the composable below binds it to a component). */
export function listenerSet(): ListenerSet {
  let stopped = false
  const live: Unlisten[] = []
  async function keep(subscription: Unlisten | Promise<Unlisten>) {
    const off = await subscription
    if (stopped) off()
    else live.push(off)
  }
  return {
    add: async (...subscriptions) => {
      await Promise.all(subscriptions.map(keep))
    },
    stop: () => {
      stopped = true
      live.splice(0).forEach(off => off())
    },
  }
}

/** The listener set of the calling component (or effect scope), stopped when it goes. */
export function useListeners(): ListenerSet {
  const set = listenerSet()
  onScopeDispose(set.stop)
  return set
}
