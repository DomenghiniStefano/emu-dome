// X03, the notification centre's words and looks, pure: what a notification says (its code and
// values from Rust, `Notice`, turned into text here), the colour of its dot, when it happened,
// and the bell's count.
import type { Notification } from '@/emu'
import type { Translator } from '@/i18n/message'

/** The colour of a notification's dot, by what it is about (mockup «Ricerca e notifiche»). */
export const NoticeTone = {
  /** Something done (a sync). */
  Success: 'success',
  /** Something to look at (a cover not found). */
  Warning: 'warning',
  /** Something offered (an update ready). */
  Accent: 'accent',
  /** A fact (a controller connected). */
  Faint: 'faint',
} as const
export type NoticeTone = (typeof NoticeTone)[keyof typeof NoticeTone]

/** One notification in words. */
export interface NoticeView {
  title: string
  sub: string
  tone: NoticeTone
}

export function noticeView(n: Notification, { t }: Pick<Translator, 't'>): NoticeView {
  switch (n.notice.kind) {
    case 'controllerConnected':
      return {
        title: t('notifications.controllerConnected'),
        sub: n.notice.name || t('toast.gamepadUnnamed'),
        tone: NoticeTone.Faint,
      }
  }
}

export const unreadCount = (list: readonly Notification[]) => list.filter(n => !n.read).length

/** The bell's badge: the count, «99+» past it; empty when nothing is unread (no badge). */
export function badgeText(unread: number): string {
  if (unread <= 0) return ''
  return unread > 99 ? '99+' : String(unread)
}

/** Under an hour: «5 minuti fa»; else the day and time (`time`: «oggi alle 09:12», «ieri…»). */
export function noticeWhen(
  at: number,
  now: number,
  labels: { ago: (at: number, now: number) => string; time: (at: number, now: number) => string },
): string {
  return now - at < 3_600_000 ? labels.ago(at, now) : labels.time(at, now)
}
