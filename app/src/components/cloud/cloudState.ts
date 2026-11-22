// Google Drive sync as the interface sees it (Trello #31): one reactive copy of the backend's
// `CloudStatus` (kept up to date by `cloud://status`), the transfer running (`cloud://progress`),
// the games of «Giochi su Google Drive» (also the library's Drive-only ones), the G04 questions
// waiting, and the G05 question asked before a game opens. App.vue starts it once
// (useCloudEvents); the Cloud section, the game flow and the library read `cloud`.
//
// The games list is replaced whole on every change, never changed in place (`markRaw`), like the
// library's.
import { markRaw, reactive } from 'vue'
import {
  emu,
  type CloudGame,
  type CloudGameConflict,
  type CloudProgress,
  type CloudStatus,
  type GameId,
} from '@/emu'
import { isErrorCode } from '@/i18n/labels'
import { reportError } from '@/lib/errors'
import { latest } from '@/lib/latest'
import { errorKey } from './cloud'

/** G05: a game open on another device. `opened`: the game already runs here (it was opened
 * without the check: at startup, «Open with»), so «Annulla» closes it. */
export interface InUseAsk {
  id: GameId
  title: string
  device: string
  /** Unix ms: open there since. */
  since: number
  opened: boolean
}

export const cloud = reactive({
  /** Null until the first `status` answered. */
  status: null as CloudStatus | null,
  /** The transfer running; null when none is (after its `done`). */
  progress: null as CloudProgress | null,
  /** G14: the game whose saves were just copied to Drive (the strip's «Copiati»), for a moment. */
  pushed: null as GameId | null,
  /** «Giochi su Google Drive»: replaced whole, never changed in place (not deeply reactive). */
  games: markRaw([]) as CloudGame[],
  gamesLoaded: false,
  /** G04: the games with two versions waiting for an answer, the one shown first. */
  conflicts: [] as CloudGameConflict[],
  /** G05: the question shown. */
  inUse: null as InUseAsk | null,
})

// ---- The status ----

// Rule 7: an event is newer than any `status` still answering; only the newest answer counts.
const statusLoads = latest()

/** A status from an event or a command's answer: it replaces ours, and a failure in it is told. */
export function applyStatus(s: CloudStatus) {
  statusLoads.next()
  cloud.status = s
  tellStatusError(s)
}

export async function loadStatus() {
  const fresh = statusLoads.next()
  const s = await emu.cloud.status()
  if (fresh()) {
    cloud.status = s
    tellStatusError(s)
  }
}

// ---- Failures, told once while they last ----
//
// E07 offline, E08 access expired, E09 Drive full… arrive in the status (`error`) and as the
// rejection of the command that met them: the toast says it once while the status keeps it, and
// again only after it cleared and came back.

let told: string | null = null

function tellStatusError(s: CloudStatus) {
  const key = errorKey(s.error)
  if (key !== null && key !== told) reportError(s.error)
  told = key
}

/** A command's failure: told, unless it is the failure the status already told. */
export function failCloud(e: unknown) {
  if (isErrorCode(e) && e.code === told) {
    console.warn('cloud: already told', e)
    return
  }
  reportError(e)
}

// ---- Transfers ----

/** How long G14's «Copiati su Google Drive» stays after the push ended. */
export const PUSHED_VISIBLE_MS = 1500
let pushedTimer: number | undefined

export function applyProgress(p: CloudProgress) {
  const before = cloud.progress
  if (p.phase !== 'done') {
    cloud.progress = p
    return
  }
  cloud.progress = null
  if (before?.phase === 'pushing') {
    cloud.pushed = before.game
    window.clearTimeout(pushedTimer)
    pushedTimer = window.setTimeout(() => (cloud.pushed = null), PUSHED_VISIBLE_MS)
  }
}

// ---- Games on Drive ----

const gameLoads = latest()

export async function loadGames() {
  const fresh = gameLoads.next()
  const games = await emu.cloud.games()
  if (!fresh()) return
  cloud.games = markRaw(games)
  cloud.gamesLoaded = true
}

/** One game's row as a command answered it (Carica / Scarica / Rimuovi): it replaces ours. */
export function applyGame(g: CloudGame) {
  gameLoads.next()
  const known = cloud.games.some(x => x.id === g.id)
  cloud.games = markRaw(known ? cloud.games.map(x => (x.id === g.id ? g : x)) : [...cloud.games, g])
}

// ---- G04: two versions ----

/** A conflict to ask about (`cloud://conflict`, or the status card's link): a newer question for
 * the same game replaces the one waiting. */
export function askConflict(c: CloudGameConflict) {
  const i = cloud.conflicts.findIndex(x => x.id === c.id)
  if (i >= 0) cloud.conflicts.splice(i, 1, c)
  else cloud.conflicts.push(c)
}

/** The question about `id` is answered, or put off (it stays in the status card). */
export function dropConflict(id: GameId) {
  cloud.conflicts = cloud.conflicts.filter(c => c.id !== id)
}

// ---- G05: open on another device ----

let answer: ((play: boolean) => void) | null = null
/** The game the user chose to play anyway: its own `cloud://in-use` is not asked again. */
let accepted: GameId | null = null

/** Shows G05; resolves with the answer (a newer question answers the older one «Annulla»). */
export function askInUse(ask: InUseAsk): Promise<boolean> {
  answer?.(false)
  cloud.inUse = ask
  return new Promise(resolve => (answer = resolve))
}

export function answerInUse(play: boolean) {
  const id = cloud.inUse?.id ?? null
  cloud.inUse = null
  if (play) accepted = id
  const a = answer
  answer = null
  a?.(play)
}

/** Whether `cloud://in-use` for `id` is still to ask (not already answered «Gioca comunque»). */
export const inUseToAsk = (id: GameId) => id !== accepted

/**
 * G05 before a library game opens: free, here, or the user's «Gioca comunque» → true. Asked only
 * while signed in. A check that fails never keeps the game from opening: the status card and
 * its toast say what is wrong with the sync.
 */
export async function clearToLaunch(game: { id: GameId; title: string }): Promise<boolean> {
  accepted = null
  if (!cloud.status?.signedIn) return true
  let inUse
  try {
    inUse = await emu.cloud.launchCheck(game.id)
  } catch (e) {
    // Deliberate: the open goes on with this device's files; the status tells the failure.
    console.warn('cloud: launch check failed', e)
    return true
  }
  if (inUse.kind !== 'elsewhere') return true
  return askInUse({
    id: game.id,
    title: game.title,
    device: inUse.name,
    since: inUse.since,
    opened: false,
  })
}
