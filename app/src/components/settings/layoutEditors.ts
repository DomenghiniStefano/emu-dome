// The two layout editors a settings row opens (Trello #111): C09, the DS / 3DS screens
// (`screens.custom`), and C06, the touch controls (`touch.tedit`). Each edits one console's layout
// (C09 also one game's): a row at a scope with such a console opens the editor at once; at «Tutti
// i giochi», or at a console the editor has nothing for, it asks for the console first
// (LayoutConsoleChooser). Pure, tested.
import type { ConsoleId, SettingKey, SettingsScope } from '@/emu'

export type LayoutEditor = 'screens' | 'touch'

/** The editor a row opens: its kind says it (the touch controls' row, or the screens'). */
export const editorOf = (key: SettingKey): LayoutEditor =>
  key === 'touch.tedit' ? 'touch' : 'screens'

/** The consoles each editor has layouts for, in the interface's order: two screens for C09,
 * every console with touch controls for C06. */
export const EDITOR_CONSOLES: Record<LayoutEditor, readonly ConsoleId[]> = {
  screens: ['nds', '3ds'],
  touch: ['gb', 'gbc', 'gba', 'nds', '3ds'],
}

/** What the editor opens on: the console and the scope «Salva» stores at. */
export interface EditorTarget {
  editor: LayoutEditor
  console: ConsoleId
  scope: SettingsScope
}

/**
 * The editor opened from a row at `scope`: at once when the scope is of a console it has layouts
 * for (C09 at a game's scope saves for that game; C06 is the console's only, so a game's scope
 * saves for its console), else null: the console is chosen first.
 */
export function directTarget(editor: LayoutEditor, scope: SettingsScope): EditorTarget | null {
  if (!('console' in scope) || !EDITOR_CONSOLES[editor].includes(scope.console)) return null
  const at: SettingsScope =
    editor === 'touch' ? { level: 'console', console: scope.console } : scope
  return { editor, console: scope.console, scope: at }
}

/** The editor opened for a console picked in the chooser: saved for that console's games. */
export const chosenTarget = (editor: LayoutEditor, console: ConsoleId): EditorTarget => ({
  editor,
  console,
  scope: { level: 'console', console },
})
