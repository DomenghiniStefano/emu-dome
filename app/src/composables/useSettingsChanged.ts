// Following `settings://changed`: the one way a screen or a module reads a setting again when it
// changes elsewhere (Settings, the quick menu, another screen). A screen uses
// `useSettingsChanged`, which follows while the screen is mounted and never leaks a listener
// that resolves after it unmounted (useListeners.ts); a module that lives as long as the app uses
// `onSettingsChanged`.
import { onMounted } from 'vue'
import { emu, type SettingKey, type SettingsChanged } from '@/emu'
import { useListeners, type Unlisten } from './useListeners'

/** Which changes a follower cares about: one key, a list of keys, or a test on the change. */
export type SettingsFilter = SettingKey | readonly SettingKey[] | ((e: SettingsChanged) => boolean)

/** Every key: for a screen that reads whole pages again on any change. */
export const EVERY_SETTING: SettingsFilter = () => true

/** A test on the key's prefix (`pad.`: every controls setting). */
export const settingsUnder =
  (prefix: string): SettingsFilter =>
  e =>
    e.key.startsWith(prefix)

/** Whether change `e` concerns `filter`. */
export function settingMatches(filter: SettingsFilter, e: SettingsChanged): boolean {
  if (typeof filter === 'string') return e.key === filter
  if (typeof filter === 'function') return filter(e)
  return filter.includes(e.key)
}

/** Follows the changes `filter` takes, for the app's whole life. */
export function onSettingsChanged(
  filter: SettingsFilter,
  cb: (e: SettingsChanged) => void,
): Promise<Unlisten> {
  return emu.settings.onChanged(e => {
    if (settingMatches(filter, e)) cb(e)
  })
}

/** Follows the changes `filter` takes while the calling component is mounted. */
export function useSettingsChanged(filter: SettingsFilter, cb: (e: SettingsChanged) => void) {
  const listeners = useListeners()
  onMounted(() => void listeners.add(onSettingsChanged(filter, cb)))
}
