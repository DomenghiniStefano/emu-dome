// G09 quick picks (design on #43, from #38): each sets image size, smoothing and the screen
// effect together, «in prova» like every other change in the quick menu.
import type { SettingKey, SettingValue } from '@/emu'

export const ScreenPickId = {
  Sharp: 'sharp',
  Soft: 'soft',
  Lcd: 'lcd',
  Crt: 'crt',
} as const
export type ScreenPickId = (typeof ScreenPickId)[keyof typeof ScreenPickId]

/** The values each pick sets, by setting key. */
export const SCREEN_PICKS: Record<ScreenPickId, Partial<Record<SettingKey, string>>> = {
  [ScreenPickId.Sharp]: { 'video.scale': 'integer', 'video.filter': 'none', 'fx.fxpick': 'none' },
  [ScreenPickId.Soft]: { 'video.scale': 'fit', 'video.filter': 'soft', 'fx.fxpick': 'soft_pixels' },
  [ScreenPickId.Lcd]: { 'video.scale': 'integer', 'video.filter': 'none', 'fx.fxpick': 'lcd' },
  [ScreenPickId.Crt]: { 'video.scale': 'fit', 'video.filter': 'none', 'fx.fxpick': 'crt' },
}

export interface ScreenPick {
  id: ScreenPickId
  /** i18n key of its label. */
  label: string
  /** Every value it sets is the current one. */
  on: boolean
  /**
   * It changes no setting that is locked here: on the Compatible method the effects cannot
   * change, so only a pick that keeps the effect as it is can be used (its card says why).
   */
  available: boolean
}

export interface ScreenPickInput {
  value: (key: SettingKey) => SettingValue | null
  /** The setting cannot change at the Session level (`lockReason` is not null). */
  locked: (key: SettingKey) => boolean
}

/** The picks as the bar shows them. */
export function screenPicks(c: ScreenPickInput): ScreenPick[] {
  return Object.values(ScreenPickId).map(id => {
    const same = changesOf(id).map(([key, v]) => ({
      key,
      same: c.value(key) === v,
    }))
    return {
      id,
      label: `qm.picks.${id}`,
      on: same.every(s => s.same),
      available: same.every(s => s.same || !c.locked(s.key)),
    }
  })
}

/** The changes a pick makes: only the values that differ from the current ones. */
export function pickChanges(
  id: ScreenPickId,
  value: (key: SettingKey) => SettingValue | null,
): [SettingKey, string][] {
  return changesOf(id).filter(([key, v]) => value(key) !== v)
}

/** Every value a pick sets, by its setting. */
const changesOf = (id: ScreenPickId) => Object.entries(SCREEN_PICKS[id]) as [SettingKey, string][]
