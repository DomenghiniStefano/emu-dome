// The Kit page as a review board: every section registers itself here with the issues written
// under it (KitIssue.vue), so the page can list the sections, count what is open and show only the
// sections that still have something to decide. Development only, like the whole kit/ folder.
import type { InjectionKey, Ref } from 'vue'
import { computed, inject, onBeforeUnmount, provide, reactive, ref } from 'vue'

// Where a section sits on the page: the primitives of design-system/components, then the feature
// components, one group per area of the app.
export const KitGroup = {
  Primitives: 'primitives',
  Library: 'library',
  Settings: 'settings',
  Game: 'game',
} as const
export type KitGroup = (typeof KitGroup)[keyof typeof KitGroup]

export const KIT_GROUP_TITLES: Record<KitGroup, string> = {
  [KitGroup.Primitives]: 'Primitive',
  [KitGroup.Library]: 'Libreria, copertine, dettagli',
  [KitGroup.Settings]: 'Impostazioni e salvataggi',
  [KitGroup.Game]: 'In gioco, shell, controlli',
}

// Where an issue is: noticed and described, a solution shown beside it, or decided by the owner
// (kept until the decision is carried out, then the issue is deleted).
export const IssueStatus = {
  Open: 'open',
  Proposed: 'proposed',
  Decided: 'decided',
} as const
export type IssueStatus = (typeof IssueStatus)[keyof typeof IssueStatus]

export const ISSUE_STATUS_LABELS: Record<IssueStatus, string> = {
  [IssueStatus.Open]: 'Da vedere',
  [IssueStatus.Proposed]: 'Proposta',
  [IssueStatus.Decided]: 'Deciso',
}

export interface KitSectionEntry {
  id: string
  title: string
  group: KitGroup
  // Issues not decided yet.
  open: number
}

interface KitBoard {
  sections: KitSectionEntry[]
  onlyOpen: Ref<boolean>
}

const BOARD: InjectionKey<KitBoard> = Symbol('kit-board')
const SECTION: InjectionKey<KitSectionEntry> = Symbol('kit-section')

export const slug = (title: string) =>
  'kit-' +
  title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '')

/** KitPage: the board every section and issue registers into. */
export function provideKitBoard() {
  const board: KitBoard = { sections: reactive([]), onlyOpen: ref(false) }
  provide(BOARD, board)
  return board
}

/** KitSection: registers the section, and tells whether the filter hides it. */
export function useKitSection(title: () => string, group: () => KitGroup) {
  const board = inject(BOARD, null)
  const entry = reactive<KitSectionEntry>({
    id: slug(title()),
    title: title(),
    group: group(),
    open: 0,
  })
  board?.sections.push(entry)
  provide(SECTION, entry)
  onBeforeUnmount(() => {
    if (!board) return
    const at = board.sections.indexOf(entry)
    if (at >= 0) board.sections.splice(at, 1)
  })
  const hidden = computed(() => !!board?.onlyOpen.value && entry.open === 0)
  return { entry, hidden }
}

/** KitIssue: counts itself on its section while it is not decided. */
export function useKitIssue(status: () => IssueStatus) {
  const section = inject(SECTION, null)
  if (!section) return
  const counted = status() !== IssueStatus.Decided
  if (counted) section.open++
  onBeforeUnmount(() => {
    if (counted) section.open--
  })
}
