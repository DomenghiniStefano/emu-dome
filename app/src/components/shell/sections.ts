// The shell's sections (desktop mode, board L01): the rail shows them in this order,
// with Settings on its own at the bottom. The id is also the `shell.<id>` message key.
export const SECTIONS = ['library', 'saves', 'controls', 'cloud', 'settings'] as const
export type Section = (typeof SECTIONS)[number]
