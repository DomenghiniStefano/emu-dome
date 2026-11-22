// The C06 editor's toolbar on a narrow phone (owner: it must never overflow or be clipped),
// pure: which tools fit in the toolbar's width and which go into «Altro». «Salva» and «Riduci»
// always stay; the others leave, least needed first, and «Altro» appears as soon as one has
// left. The screens measure each tool's button and pass the widths in.

export const TOOLS = [
  'undo',
  'redo',
  'add',
  'size',
  'opacity',
  'behaviour',
  'remove',
  'reset',
  'discard',
] as const
export type Tool = (typeof TOOLS)[number]

/** Into «Altro» in this order: the rare actions first, undo / redo last. */
const LEAVE_ORDER: readonly Tool[] = [
  'reset',
  'discard',
  'remove',
  'behaviour',
  'opacity',
  'size',
  'add',
  'redo',
  'undo',
]

export interface ToolbarWidths {
  /** Each tool's button, CSS pixels. */
  tools: Record<Tool, number>
  /** «Altro», «Riduci» and «Salva». */
  more: number
  collapse: number
  save: number
  /** Between two buttons. */
  gap: number
}

export interface ToolbarFit {
  /** In the toolbar, in their order. */
  shown: Tool[]
  /** In «Altro», in the toolbar's order. */
  more: Tool[]
}

/** What fits in `available` CSS pixels (the toolbar's inside width). */
export function fitToolbar(available: number, w: ToolbarWidths): ToolbarFit {
  const width = (tools: readonly Tool[], withMore: boolean) => {
    const buttons = [
      ...tools.map(t => w.tools[t]),
      ...(withMore ? [w.more] : []),
      w.collapse,
      w.save,
    ]
    return buttons.reduce((sum, b) => sum + b, 0) + w.gap * (buttons.length - 1)
  }
  let shown: Tool[] = [...TOOLS]
  if (width(shown, false) <= available) return { shown, more: [] }
  for (const leaving of LEAVE_ORDER) {
    shown = shown.filter(t => t !== leaving)
    if (width(shown, true) <= available) break
  }
  return { shown, more: TOOLS.filter(t => !shown.includes(t)) }
}
