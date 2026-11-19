// The game details' tabs (L05–L10), in their order on screen.
export const DETAILS_TABS = ['saves', 'settings', 'controls', 'cheats', 'patches', 'files'] as const
export type DetailsTab = (typeof DETAILS_TABS)[number]

/** Whether a value Tabs reports is one of the details' tabs. */
export function isDetailsTab(value: unknown): value is DetailsTab {
  return (DETAILS_TABS as readonly unknown[]).includes(value)
}

/** The i18n key of a tab's label; the phone in portrait has a shorter one for «Impostazioni». */
export function tabLabelKey(tab: DetailsTab, phone: boolean): string {
  return phone && tab === 'settings' ? 'details.tabs.settingsShort' : `details.tabs.${tab}`
}
