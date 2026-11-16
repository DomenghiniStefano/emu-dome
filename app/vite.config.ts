import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Configuration made for Tauri: fixed port, no browser opening.
export default defineConfig({
  plugins: [vue()],
  // vue-i18n feature flags for the esm-bundler build: set explicitly so the bundle drops the
  // legacy API and the production devtools hooks (and Vite stops warning about them).
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_'],
  build: { target: 'es2022', outDir: 'dist' },
})
