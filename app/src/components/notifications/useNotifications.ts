// X03: the notifications the bell shows, kept in step with Rust (`notifications://changed` sends
// the whole list after every change), and the two ways to read them. A failure reaches the user
// (useErrorSink); an answer older than the latest event or load is dropped (rule 7).
import { computed, onMounted, ref } from 'vue'
import { emu, type Notification } from '@/emu'
import { useListeners } from '@/composables/useListeners'
import { useErrorSink } from '@/composables/useToasts'
import { latest } from '@/lib/latest'
import { unreadCount } from './notifications'

export function useNotifications() {
  const failWith = useErrorSink()
  const listeners = useListeners()
  const list = ref<Notification[]>([])
  const loads = latest()

  async function load() {
    const fresh = loads.next()
    try {
      const answer = await emu.notifications.list()
      if (fresh()) list.value = answer
    } catch (e) {
      if (fresh()) failWith(e)
    }
  }

  onMounted(async () => {
    try {
      await listeners.add(
        emu.notifications.onChanged(l => {
          // Newer than any list still on its way.
          loads.next()
          list.value = l
        }),
      )
    } catch (e) {
      failWith(e)
    }
    await load()
  })

  async function markRead(id: number) {
    try {
      await emu.notifications.markRead(id)
    } catch (e) {
      failWith(e)
    }
  }

  async function markAllRead() {
    try {
      await emu.notifications.markAllRead()
    } catch (e) {
      failWith(e)
    }
  }

  return { list, unread: computed(() => unreadCount(list.value)), markRead, markAllRead }
}
