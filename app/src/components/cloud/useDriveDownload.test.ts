// «Scarica e gioca» waits for the library to list the downloaded game: at once when it already
// does, as soon as a change brings it, and never longer than its limit.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { markRaw, nextTick } from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import { library } from '@/lib/library'
import { LIBRARY_WAIT_MS, libraryRow } from './useDriveDownload'

vi.mock('@/emu', () => ({ emu: {} }))

const row = (hex: string, available = true) =>
  ({ id: `3ds:${hex}` as GameId, title: hex, available }) as LibraryGame

afterEach(() => {
  library.games = markRaw([])
  vi.useRealTimers()
})

describe('libraryRow', () => {
  it('answers at once when the library already lists the game', async () => {
    library.games = markRaw([row('01')])
    expect(await libraryRow(row('01').id)).toEqual(row('01'))
  })

  it('answers when a change of the library brings the game', async () => {
    const found = libraryRow(row('02').id)
    library.games = markRaw([row('01')])
    await nextTick()
    library.games = markRaw([row('01'), row('02')])
    expect(await found).toEqual(row('02'))
  })

  it('waits for a game removed from this device to have its file again (#29)', async () => {
    library.games = markRaw([row('04', false)])
    const found = libraryRow(row('04').id)
    await nextTick()
    library.games = markRaw([row('04')])
    expect(await found).toEqual(row('04'))
  })

  it('gives up after its limit', async () => {
    vi.useFakeTimers()
    const found = libraryRow(row('03').id)
    vi.advanceTimersByTime(LIBRARY_WAIT_MS)
    expect(await found).toBeNull()
  })
})
