/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

// Configuration made for Tauri: fixed port, no browser opening.
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  // vue-i18n feature flags for the esm-bundler build: set explicitly so the bundle drops the
  // legacy API and the production devtools hooks (and Vite stops warning about them).
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
  // `@/` is src/: the design system's modules (design-system/components, lib) import each other through it,
  // as shadcn-vue's components.json expects.
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_'],
  build: { target: 'es2022', outDir: 'dist' },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'happy-dom',
    // Vitest does not load CSS by default and hands a `?raw` import an empty string. cn() reads
    // its token names from the theme files, so those load as source. A file missing here fails
    // as a wrong answer, not as an error: the import yields '' and the test reads "no tokens".
    css: { include: [/src\/design-system\/tokens\/.+\.css/] },
  },
})
