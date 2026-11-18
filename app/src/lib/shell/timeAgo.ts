/**
 * «2 minuti fa» for the Unix ms `at` seen at `now`, in `locale`: minutes under an hour, then
 * hours (E11, when the last auto quick save was written).
 */
export function timeAgo(at: number, now: number, locale: string): string {
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })
  const minutes = Math.round((at - now) / 60000)
  return Math.abs(minutes) < 60
    ? rtf.format(minutes, 'minute')
    : rtf.format(Math.round(minutes / 60), 'hour')
}
