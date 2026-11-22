// The Kit page in a plain browser (`pnpm dev`, then `#kit`): there is no Tauri there, and emu.ts
// reads Tauri's internals as soon as it loads, so the page would stop before mounting. This stands
// in for the bridge, answering every command with nothing, as scripts/headless/kit-nav.mjs does.
// Imported first by main.ts so it runs before any module that reads the bridge; development and
// the Kit page only (the production build keeps an empty module).
import { DevRoute } from '@/lib/devRoutes'

if (
  import.meta.env.DEV &&
  window.location.hash === DevRoute.Kit &&
  !('__TAURI_INTERNALS__' in window)
) {
  Object.assign(window, {
    __TAURI_INTERNALS__: {
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { windowLabel: 'main', label: 'main' },
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
      convertFileSrc: (path: string, protocol = 'asset') =>
        `http://${protocol}.localhost/${encodeURIComponent(path)}`,
      invoke: async (cmd: string) => (cmd === 'plugin:event|listen' ? 1 : null),
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => undefined },
  })
}
