// ESLint for the frontend (app/src and the config files of app/). Formatting belongs to Prettier:
// eslint-config-prettier comes last and switches off every rule that would fight it.
import js from '@eslint/js'
import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import prettier from 'eslint-config-prettier/flat'
import globals from 'globals'

export default withVueTs(
  {
    ignores: ['dist', 'node_modules', 'src-tauri'],
  },
  js.configs.recommended,
  pluginVue.configs['flat/essential'],
  vueTsConfigs.recommended,
  {
    // `const { a: _a, ...rest } = obj` is how a field is left out of a copy.
    rules: { '@typescript-eslint/no-unused-vars': ['error', { ignoreRestSiblings: true }] },
  },
  {
    // The Vite and ESLint configs run in Node, outside the webview.
    files: ['vite.config.ts', 'eslint.config.js'],
    languageOptions: { globals: globals.node },
  },
  {
    files: ['src/**'],
    languageOptions: { globals: globals.browser },
  },
  {
    // The design system's primitives are one word each (Button.vue, Card.vue), as shadcn-vue names
    // them. The rule exists to avoid clashing with HTML elements; a folder of primitives imported
    // by name (`import { Button } from '@/components/ui/button'`) is where single words belong.
    files: ['src/components/ui/**/*.vue'],
    rules: { 'vue/multi-word-component-names': 'off' },
  },
  prettier,
)
