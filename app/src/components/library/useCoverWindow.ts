// The cover grid's windowing in the page (Trello #91): which games CoverGrid renders, the padding
// standing for the rest, and the focus kept across re-renders. The sums are coverWindow.ts's.
//
// - The view is the grid's scroll container (the shell's main area): on its scroll and on a
//   resize of the grid, the container or the view around the grid, the rows in view plus
//   `MARGIN_ROWS` on each side are rendered, once per frame.
// - Columns, row gap and the first track's width come from the grid's resolved style (the CSS
//   `auto-fill` stays the one place that decides the columns); rendered rows are measured.
// - Focus: when the focused tile is about to leave the DOM (the user scrolled it away), the focus
//   moves to the grid itself (`parked`: no ring, tabindex -1 meanwhile) and goes back to that tile as soon
//   as it is rendered again; a controller step from the parked grid starts from that tile.
// - `anchor`: a game whose row is rendered first (back from its details, useLibraryFocus focuses
//   its cover once it is in the DOM).
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  onUpdated,
  ref,
  shallowRef,
  watch,
  type Ref,
} from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import { COVER_PARTS, type CoverCell, type CoverPart } from './coverGridNav'
import {
  averageHeight,
  firstTrack,
  padding,
  rowCount,
  rowOf,
  rowTops,
  rowsInView,
  sameRange,
  scrollToShow,
  trackCount,
  type RowLayout,
  type RowRange,
} from './coverWindow'

/** Rows rendered beyond the view on each side: a d-pad step always lands on a rendered row. */
export const MARGIN_ROWS = 2
/** The title and play time under a cover, for a row not measured yet (an estimate only). */
const TEXT_ESTIMATE = 48
/** A cover's height for its width (2:3). */
const COVER_ASPECT = 1.5

/** The attribute a tile's root carries, with its game's id. */
export const TILE_OF = 'data-game'
/** The attribute each focusable part of a tile carries: `cover`, `play` or `more`. */
export const PART_ATTR = 'data-cover-part'

const isPart = (s: string | undefined): s is CoverPart =>
  s !== undefined && (COVER_PARTS as readonly string[]).includes(s)

/** The nearest ancestor that scrolls (overflow auto / scroll), or null. */
function scrollParent(el: HTMLElement): HTMLElement | null {
  for (let p = el.parentElement; p; p = p.parentElement) {
    const style = getComputedStyle(p)
    if (/(auto|scroll)/.test(style.overflowY)) return p
  }
  return null
}

export function useCoverWindow(
  grid: Ref<HTMLElement | null>,
  games: () => LibraryGame[],
  anchor: GameId | null,
) {
  /** False until the grid's columns are read: nothing is rendered before. */
  const ready = ref(false)
  const cols = ref(1)
  const gap = ref(0)
  const track = ref(0)
  /** Measured row heights; `measures` says when they changed (the array is not reactive). */
  let heights: number[] = []
  const measures = ref(0)
  const range = shallowRef<RowRange>({ start: 0, end: 0 })
  /** The columns the DOM was last rendered with (a measure of another layout is wrong). */
  let renderedCols = 0
  let scroller: HTMLElement | null = null
  /** The tile that had the focus when it left the DOM. */
  let parked: { id: GameId; part: CoverPart } | null = null
  /** What the DOM shows now: its first row, that row's top (the padding), for which list. */
  let rendered: { start: number; top: number; list: LibraryGame[]; cols: number } | null = null
  /** A tile on screen and where it was before a re-render: the render must not move it. */
  let hold: { id: GameId; top: number } | null = null

  const indexOf = computed(() => new Map(games().map((g, i) => [g.id, i])))
  const rows = computed(() => rowCount(games().length, cols.value))
  const layout = computed<RowLayout>(() => {
    void measures.value
    return {
      rows: rows.value,
      heights,
      estimate: averageHeight(heights, track.value * COVER_ASPECT + TEXT_ESTIMATE),
      gap: gap.value,
    }
  })
  const tops = computed(() => rowTops(layout.value))
  /** The range clamped to the list as it is now (it may have shrunk since the last update). */
  const shown = computed<RowRange>(() => {
    const end = Math.min(range.value.end, rows.value)
    return { start: Math.min(range.value.start, end), end }
  })
  const visible = computed(() =>
    ready.value ? games().slice(shown.value.start * cols.value, shown.value.end * cols.value) : [],
  )
  const style = computed(() => {
    const p = padding(tops.value, rows.value, shown.value)
    return { paddingTop: `${p.top}px`, paddingBottom: `${p.bottom}px` }
  })

  /** The game index of a tile element, or undefined. */
  function tileIndex(el: Element | null): number | undefined {
    const id = el?.closest<HTMLElement>(`[${TILE_OF}]`)?.getAttribute(TILE_OF)
    return id ? indexOf.value.get(id) : undefined
  }

  /** The cell a focusable element of the grid is, or null. */
  function cellOf(el: HTMLElement): CoverCell | null {
    const index = tileIndex(el)
    const part = el.getAttribute(PART_ATTR) ?? undefined
    return index !== undefined && isPart(part) ? { index, part } : null
  }

  /** The element of `cell`, when it is rendered. */
  function cellElement(cell: CoverCell): HTMLElement | null {
    const game = games()[cell.index]
    if (!game || !grid.value) return null
    return grid.value.querySelector<HTMLElement>(
      `[${TILE_OF}="${CSS.escape(game.id)}"] [${PART_ATTR}="${cell.part}"]`,
    )
  }

  /** The tile parked on the grid, as a cell of the list as it is now. */
  function parkedCell(): CoverCell | null {
    if (!parked) return null
    const index = indexOf.value.get(parked.id)
    return index === undefined ? null : { index, part: parked.part }
  }

  function readColumns(el: HTMLElement) {
    const style = getComputedStyle(el)
    const n = trackCount(style.gridTemplateColumns)
    track.value = firstTrack(style.gridTemplateColumns) ?? el.clientWidth
    gap.value = parseFloat(style.rowGap) || 0
    if (n === cols.value) return
    cols.value = n
    heights = []
    measures.value++
  }

  /** Reads the height of each rendered row (its first tile: grid items stretch to their row). */
  function measureRows(el: HTMLElement) {
    if (renderedCols !== cols.value) return
    let changed = false
    for (const child of el.children) {
      const index = tileIndex(child)
      if (index === undefined || index % cols.value !== 0) continue
      const h = (child as HTMLElement).offsetHeight
      const row = rowOf(index, cols.value)
      if (h > 0 && heights[row] !== h) {
        heights[row] = h
        changed = true
      }
    }
    if (changed) measures.value++
  }

  /** Applies a new range; a focused tile about to leave the DOM hands the focus to the grid. */
  function setRange(next: RowRange) {
    if (sameRange(next, range.value)) return
    const el = grid.value
    const active = document.activeElement
    if (el && active instanceof HTMLElement && active !== el && el.contains(active)) {
      const index = tileIndex(active)
      const part = active.getAttribute(PART_ATTR) ?? undefined
      const first = next.start * cols.value
      const out = index === undefined || index < first || index >= next.end * cols.value
      const game = index === undefined ? undefined : games()[index]
      if (out && game && isPart(part)) {
        parked = { id: game.id, part }
        // Focusable only while it holds the parked focus: a click on a gap never focuses it.
        el.tabIndex = -1
        el.focus({ preventScroll: true })
      }
    }
    range.value = next
  }

  /** The view in the grid's coordinates. */
  function view(el: HTMLElement, box: HTMLElement) {
    const top = box.getBoundingClientRect().top - el.getBoundingClientRect().top
    return { top, height: box.clientHeight }
  }

  function update() {
    const el = grid.value
    if (!el || !scroller || !ready.value) return
    measureRows(el)
    const v = view(el, scroller)
    // Rows measured above the first rendered one (or a new estimate for those never rendered)
    // move it: the view is taken as moved with it, so the rows on screen stay rendered.
    const same = rendered && rendered.list === games() && rendered.cols === cols.value
    const shift = same && rendered ? (tops.value[rendered.start] ?? 0) - rendered.top : 0
    const next = rowsInView(
      layout.value,
      tops.value,
      v.top + shift,
      v.top + shift + v.height,
      MARGIN_ROWS,
    )
    hold = shift !== 0 || !sameRange(next, range.value) ? onScreen(el, scroller, next) : null
    setRange(next)
  }

  /**
   * A tile on screen that stays rendered in `next`: the focused one, else the first one in view.
   * After the render the view is scrolled so that it is where it was (the padding above it may
   * have changed): by hand, as WebKitGTK (Linux) has no scroll anchoring; where the browser
   * anchored it already, the difference is 0.
   */
  function onScreen(el: HTMLElement, box: HTMLElement, next: RowRange) {
    const stays = (index: number | undefined) =>
      index !== undefined && index >= next.start * cols.value && index < next.end * cols.value
    const tile = (child: Element | null) => {
      const t = child?.closest(`[${TILE_OF}]`)
      const id = t?.getAttribute(TILE_OF)
      return t && id && stays(indexOf.value.get(id))
        ? { id, top: t.getBoundingClientRect().top }
        : null
    }
    const active = document.activeElement
    if (active && active !== el && el.contains(active)) {
      const focused = tile(active)
      if (focused) return focused
    }
    const viewTop = box.getBoundingClientRect().top
    for (const child of el.children) {
      const t = tile(child)
      if (t && t.top + (child as HTMLElement).offsetHeight > viewTop) return t
    }
    return null
  }

  let frame = 0
  function schedule() {
    if (frame) return
    frame = requestAnimationFrame(() => {
      frame = 0
      update()
    })
  }

  /**
   * Scrolls the row of `cell` into view, renders it and resolves its element (null when the game
   * is gone meanwhile): a controller step to a tile that is not in the DOM.
   */
  async function reveal(cell: CoverCell): Promise<HTMLElement | null> {
    const el = grid.value
    if (el && scroller) {
      const row = rowOf(cell.index, cols.value)
      const t = tops.value
      const v = view(el, scroller)
      const top = t[row] ?? 0
      const bottom = (t[row + 1] ?? top) - gap.value
      scroller.scrollTop += scrollToShow(top, bottom, v.top, v.height)
      update()
    }
    await nextTick()
    return cellElement(cell)
  }

  // Keeps the tile `update` held where it was, and gives the focus back to a parked tile once it
  // is rendered again.
  onUpdated(() => {
    renderedCols = cols.value
    const held = hold && grid.value?.querySelector(`[${TILE_OF}="${CSS.escape(hold.id)}"]`)
    if (held && hold && scroller) {
      const moved = held.getBoundingClientRect().top - hold.top
      if (Math.abs(moved) >= 1) scroller.scrollTop += moved
    }
    hold = null
    rendered = {
      start: shown.value.start,
      top: tops.value[shown.value.start] ?? 0,
      list: games(),
      cols: cols.value,
    }
    const el = grid.value
    const cell = parkedCell()
    if (el && cell && document.activeElement === el)
      cellElement(cell)?.focus({ preventScroll: true })
    // The rows just rendered are measured, and the range checked again, before the next paint.
    schedule()
  })

  function onFocusIn(e: FocusEvent) {
    if (e.target !== grid.value) parked = null
  }
  function onFocusOut(e: FocusEvent) {
    const to = e.relatedTarget
    // To nowhere (the window in the background), the focus stays where it is.
    const el = grid.value
    if (!(to instanceof Node) || !el) return
    if (e.target === el) el.removeAttribute('tabindex')
    if (!el.contains(to)) parked = null
  }

  let lastWidth = -1
  const resized = new ResizeObserver(entries => {
    const el = grid.value
    for (const entry of entries) {
      if (entry.target !== el || entry.contentRect.width === lastWidth) continue
      lastWidth = entry.contentRect.width
      if (el) readColumns(el)
    }
    schedule()
  })

  onMounted(() => {
    const el = grid.value
    if (!el) return
    scroller = scrollParent(el)
    readColumns(el)
    lastWidth = el.getBoundingClientRect().width
    ready.value = true
    renderedCols = 0
    const at = anchor === null ? undefined : indexOf.value.get(anchor)
    if (at !== undefined) {
      const row = rowOf(at, cols.value)
      range.value = {
        start: Math.max(0, row - MARGIN_ROWS),
        end: Math.min(rows.value, row + 1 + MARGIN_ROWS),
      }
    } else update()
    scroller?.addEventListener('scroll', schedule, { passive: true })
    el.addEventListener('focusin', onFocusIn)
    el.addEventListener('focusout', onFocusOut)
    for (const box of [el, el.parentElement, scroller]) if (box) resized.observe(box)
  })
  onBeforeUnmount(() => {
    resized.disconnect()
    scroller?.removeEventListener('scroll', schedule)
    if (frame) cancelAnimationFrame(frame)
  })
  // Another list (a filter, an order, a scan): the rows it makes in view.
  watch(games, schedule)

  return {
    visible,
    style,
    cols,
    shown,
    cellOf,
    cellElement,
    parkedCell,
    reveal,
  }
}
