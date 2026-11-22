// Quick saves in the menu (G07, G08): the slots, the one chosen, Carica and Salva, and the line
// «Salvato nello slot N · Annulla» a save made here leaves until the next choice.
import { computed, onMounted, ref } from 'vue'
import { useListeners } from '@/composables/useListeners'
import { AUTO_SLOT, emu, type EmuEvent, type SlotInfo } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { isEmptySlot } from './slots'

export interface QuickSlotsOptions {
  /** The number of slots `saves.slots` asks for. */
  count: () => number
  /** The slot the controller shortcuts use: chosen when the menu opens. */
  initial: number
  onSave: (slot: number) => void
  onLoad: (slot: number) => void
  onError: ErrorSink
}

export function useQuickSlots(o: QuickSlotsOptions) {
  const slots = ref<SlotInfo[]>([])
  const chosen = ref(o.initial)
  /** The slot a save made here went to, while its «Annulla» is offered. */
  const savedSlot = ref<number | null>(null)
  /** The slot of the last save seen while the menu is open (here or with a shortcut), until it is
   * undone: the phone's «Annulla l'ultimo salvataggio». */
  const lastSaved = ref<number | null>(null)
  /** The slot a save asked here is going to, until its event arrives. */
  let saving: number | null = null

  async function load() {
    try {
      slots.value = await emu.listSlots(o.count())
    } catch (e) {
      o.onError(e)
    }
  }

  const chosenEmpty = computed(() => isEmptySlot(slots.value.find(s => s.slot === chosen.value)))
  const autoChosen = computed(() => chosen.value === AUTO_SLOT)

  function pick(slot: number) {
    chosen.value = slot
    savedSlot.value = null
  }
  /** Saves to the chosen slot, or to `slot` (the phone's «Salva rapido»: the quick slot). */
  function save(slot: number = chosen.value) {
    saving = slot
    savedSlot.value = null
    o.onSave(slot)
  }
  /** Carica loads and resumes at once (App.vue shows G17). */
  function loadChosen() {
    if (!chosenEmpty.value) o.onLoad(chosen.value)
  }
  async function undoSave() {
    try {
      await emu.undoSave()
    } catch (e) {
      o.onError(e)
    }
  }

  /** A save done here or with a shortcut: refresh the dates and thumbnails. */
  function onEvent(ev: EmuEvent) {
    if (ev.type === 'stateSaved') {
      if (ev.slot === saving) savedSlot.value = ev.slot
      if (ev.slot !== AUTO_SLOT) lastSaved.value = ev.slot
      saving = null
      void load()
    }
    if (ev.type === 'saveUndone') {
      savedSlot.value = null
      lastSaved.value = null
      void load()
    }
    if (ev.type === 'error') saving = null
  }
  const listeners = useListeners()
  onMounted(() => void listeners.add(emu.onEvent(onEvent)))

  return {
    slots,
    chosen,
    chosenEmpty,
    autoChosen,
    savedSlot,
    lastSaved,
    load,
    pick,
    save,
    loadChosen,
    undoSave,
  }
}
