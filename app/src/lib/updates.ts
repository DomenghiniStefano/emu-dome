// What the settings' pure rules need to know about EmuDome's own update (Trello #44), without
// Vue components: whether this build can update itself. Set by composables/useUpdates.ts from
// the backend's status; false until known, so nothing is locked by a guess.
import { ref } from 'vue'
import type { ErrorCode, UpdateState, UpdateStatus } from '@/emu'
import type { Message } from '@/i18n/message'

/** This build cannot update itself (no update key, or Android): `general.updates` is locked. */
export const updateUnavailable = ref(false)

/** The rows that only make sense where EmuDome updates itself. */
export const UPDATE_ROWS: readonly string[] = ['general.updates', 'about.upd']

/** The version a state is about, when it names one. */
export function stateVersion(s: UpdateState): string | null {
  switch (s.state) {
    case 'available':
    case 'downloading':
    case 'ready':
    case 'installing':
      return s.version
    case 'unavailable':
    case 'idle':
    case 'checking':
    case 'upToDate':
    case 'failed':
      return null
  }
}

/** The update is on its way to being installed (X06's main button waits). */
export function installing(s: UpdateState): boolean {
  return s.state === 'downloading' || s.state === 'installing'
}

/**
 * `about.upd`'s line: where the update stands (`settings.about.upd.status.<state>`). `when` says
 * the last check's time; a failure is the error's own text (`error`, from labels.ts).
 */
export function updateLine(
  s: UpdateStatus,
  when: (ms: number) => string,
): Message | { error: ErrorCode } {
  const key = (state: string) => `settings.about.upd.status.${state}`
  const st = s.state
  switch (st.state) {
    case 'failed':
      return { error: st.error }
    case 'upToDate':
      return { key: key('upToDate'), params: { when: s.checkedAt ? when(s.checkedAt) : '' } }
    case 'available':
    case 'downloading':
    case 'ready':
    case 'installing':
      return { key: key(st.state), params: { version: st.version } }
    case 'unavailable':
    case 'idle':
    case 'checking':
      return { key: key(st.state) }
  }
}
