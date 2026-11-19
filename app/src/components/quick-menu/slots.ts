// The quick saves of the menu (G07, G08): how many tiles the main screen shows, the labels of
// Carica and of the slots. (How many slots there are: details/gameSaves.ts.)
import { AUTO_SLOT, type SlotInfo } from '@/emu'
import type { Message } from '@/i18n/message'

/** Tiles on the main screen: Automatico + slots 1–4 on desktop, Automatico + 1–2 on the phone. */
export function mainSlotCount(sheet: boolean): number {
  return sheet ? 3 : 5
}

export function isEmptySlot(slot: SlotInfo | undefined): boolean {
  return !slot || slot.savedAt === null
}

export function slotName(slot: number): Message {
  return slot === AUTO_SLOT ? { key: 'qm.auto' } : { key: 'qm.slot', params: { slot } }
}

/** Carica's label: from the auto slot, from slot N, or that slot N is empty. */
export function loadLabel(slot: number, empty: boolean): Message {
  if (slot === AUTO_SLOT) return { key: 'qm.loadAuto' }
  if (empty) return { key: 'qm.emptySlot', params: { slot } }
  return { key: 'qm.loadSlot', params: { slot } }
}

/** The line of a load that can be undone (G17): from the auto slot or from slot N. */
export function loadedLine(slot: number): Message {
  return slot === AUTO_SLOT
    ? { key: 'toast.autoLoaded' }
    : { key: 'toast.stateLoaded', params: { slot } }
}
