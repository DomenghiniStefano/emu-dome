// «Gestisci lo spazio» (Trello #29): the screen's state. The sizes come first (`storage_breakdown`,
// Rust walks the folders off the event loop); which games Drive has comes after, when the user is
// signed in (`cloud_games`, a network call the screen never waits for: until it answers no game
// offers «Togli dal telefono»). Every deletion asks Rust, then reads the sizes again. An older
// answer than the latest request is dropped (rule 7).
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type GameId, type StorageBreakdown, type StorageGame } from '@/emu'
import { cloud } from '@/components/cloud/cloudState'
import { useErrorSink, useToasts } from '@/composables/useToasts'
import { sizeLabel } from '@/i18n/labels'
import { ignore } from '@/lib/errors'
import { latest } from '@/lib/latest'
import { reloadLibrary } from '@/lib/library'
import { gameAction, onDriveSet } from './storage'

export function useStorage() {
  const { t } = useI18n()
  const toasts = useToasts()
  const failWith = useErrorSink()
  const breakdown = ref<StorageBreakdown | null>(null)
  const onDrive = ref<ReadonlySet<GameId>>(new Set())
  /** A deletion running: every action waits. */
  const busy = ref(false)
  const loads = latest()
  const driveLoads = latest()

  async function load() {
    const fresh = loads.next()
    try {
      const b = await emu.storage.breakdown()
      if (fresh()) breakdown.value = b
    } catch (e) {
      if (fresh()) failWith(e)
    }
  }

  function loadDrive() {
    if (!cloud.status?.signedIn) return
    const fresh = driveLoads.next()
    ignore(
      emu.cloud.games().then(games => {
        if (fresh()) onDrive.value = onDriveSet(games)
      }),
      'Drive unreachable: no game offers «Togli dal telefono» (the screen says nothing else)',
    )
  }

  const actionOf = (g: StorageGame) => gameAction(g, onDrive.value.has(g.id))
  /** «Giochi, dal più pesante»: the games here, the heaviest first (Rust's order). */
  const games = computed(() => breakdown.value?.games ?? [])

  /** Runs a deletion, says how much went, and measures again. */
  async function run(what: () => Promise<number | void>, done: (n: number) => string) {
    if (busy.value) return
    busy.value = true
    try {
      const n = await what()
      toasts.notify(done(typeof n === 'number' ? n : 0))
    } catch (e) {
      failWith(e)
    } finally {
      busy.value = false
      await load()
    }
  }

  const removeGame = (g: StorageGame) =>
    run(
      async () => {
        const freed = await emu.storage.removeGame(g.id)
        void reloadLibrary()
        return freed
      },
      freed => t('storage.page.removed', { title: g.title, size: sizeLabel(freed) }),
    )

  const clearTemp = () =>
    run(
      () => emu.storage.clearTemp(),
      freed => t('storage.page.cleared', { size: sizeLabel(freed) }),
    )

  async function openFolder(g: StorageGame) {
    try {
      await emu.storage.openFolder(g.id)
    } catch (e) {
      failWith(e)
    }
  }

  onMounted(() => {
    void load()
    loadDrive()
  })

  return { breakdown, games, busy, actionOf, load, run, removeGame, clearTemp, openFolder }
}
