import { describe, expect, it } from 'vitest'
import type { Notification } from '@/emu'
import { NoticeTone, badgeText, noticeView, noticeWhen, unreadCount } from './notifications'

const t = (key: string) => `<${key}>`
const pad = (id: number, read: boolean, name = 'Xbox Wireless Controller'): Notification => ({
  id,
  at: 0,
  read,
  notice: { kind: 'controllerConnected', name },
})

describe('notifications', () => {
  it('a controller connected names it, or says «controller» when it has no name', () => {
    expect(noticeView(pad(1, false), { t })).toEqual({
      title: '<notifications.controllerConnected>',
      sub: 'Xbox Wireless Controller',
      tone: NoticeTone.Faint,
    })
    expect(noticeView(pad(1, false, ''), { t }).sub).toBe('<toast.gamepadUnnamed>')
  })

  it('an update ready names its version, in the accent', () => {
    const tv = (key: string, p?: Record<string, unknown>) => `<${key}${p ? JSON.stringify(p) : ''}>`
    const n: Notification = {
      id: 2,
      at: 0,
      read: false,
      notice: { kind: 'updateReady', version: '0.0.2' },
    }
    expect(noticeView(n, { t: tv })).toEqual({
      title: '<notifications.updateReady{"version":"0.0.2"}>',
      sub: '<notifications.updateReadySub>',
      tone: NoticeTone.Accent,
    })
  })

  it('counts the unread ones, and the badge stops at 99+', () => {
    expect(unreadCount([pad(1, false), pad(2, true), pad(3, false)])).toBe(2)
    expect(badgeText(0)).toBe('')
    expect(badgeText(7)).toBe('7')
    expect(badgeText(100)).toBe('99+')
  })

  it('says minutes ago under an hour, else the day and time', () => {
    const labels = { ago: () => 'ago', time: () => 'time' }
    const now = 10_000_000
    expect(noticeWhen(now - 59 * 60_000, now, labels)).toBe('ago')
    expect(noticeWhen(now - 61 * 60_000, now, labels)).toBe('time')
  })
})
