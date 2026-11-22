// The licences page's words (`about.lic`, Trello #44), pure: a row's name and line from the
// backend's `Licence` (EmuDome, a core, the other components). Nothing about a component is typed
// here: the names, licences and versions come from cores.lock and Cargo.toml through Rust.
import type { Licence } from '@/emu'
import type { Message } from '@/i18n/message'

/** A row's name: the app's or the core's own, the components' from the message files. */
export function licenceName(l: Licence): Message | string {
  switch (l.kind) {
    case 'app':
    case 'core':
      return l.name
    case 'components':
      return { key: 'about.licences.components' }
  }
}

/** A row's line: its licence and version (a core adds its consoles); the components say where
 * each one's licence is. */
export function licenceLine(l: Licence): Message {
  switch (l.kind) {
    case 'app':
      return { key: 'about.licences.appLine', params: { spdx: l.spdx, version: l.version } }
    case 'core':
      return {
        key: 'about.licences.coreLine',
        params: { spdx: l.spdx, version: l.version, consoles: l.consoles },
      }
    case 'components':
      return { key: 'about.licences.componentsLine' }
  }
}

/** A licence text cut at its blank lines, none empty: the page's steps through it. */
export function paragraphs(text: string): string[] {
  return text
    .replace(/\r\n/g, '\n')
    .split(/\n(?:[ \t]*\n)+/)
    .map(p => p.replace(/^\n+|\n+$/g, ''))
    .filter(p => p.trim() !== '')
}
