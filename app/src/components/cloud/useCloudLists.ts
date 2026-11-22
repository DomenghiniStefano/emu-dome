// «I tuoi dispositivi» and «Attività recenti» (Trello #31): read when their tab opens and again
// after each sync or status change that may have added to them. Rule 7: only the newest answer
// is shown.
import { onMounted, ref, watch, type Ref } from 'vue'
import { emu, type CloudActivity, type CloudDevice } from '@/emu'
import { latest } from '@/lib/latest'
import { cloud, failCloud } from './cloudState'

function useCloudList<T>(read: () => Promise<T[]>): { rows: Ref<T[] | null> } {
  const rows = ref<T[] | null>(null) as Ref<T[] | null>
  const loads = latest()
  async function load() {
    const fresh = loads.next()
    try {
      const got = await read()
      if (fresh()) rows.value = got
    } catch (e) {
      if (fresh()) failCloud(e)
    }
  }
  onMounted(load)
  watch(
    () => [cloud.status?.lastSync, cloud.status?.syncing, cloud.status?.conflicts.length],
    () => void load(),
  )
  return { rows }
}

/** Every device that published its library list, this one first. */
export const useCloudDevices = () => useCloudList<CloudDevice>(emu.cloud.devices)

/** What the sync did since the app started, newest first. */
export const useCloudActivity = () => useCloudList<CloudActivity>(emu.cloud.activity)
