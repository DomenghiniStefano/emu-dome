// ESLint for the frontend (app/src and the config files of app/). Formatting belongs to Prettier:
// eslint-config-prettier comes last and switches off every rule that would fight it.
import js from '@eslint/js'
import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import prettier from 'eslint-config-prettier/flat'
import globals from 'globals'

const OTHER_FOLDER = 'Import from another folder through `@/`; `./name` is for the same folder.'
// A relative path that leaves the file's folder (`../x`, `./sub/x`), as an esquery attribute
// (`\x2F` is `/`: esquery ends a regex at a bare slash).
const RELATIVE_ELSEWHERE = '[value=/^\\.\\.\\x2F|^\\.\\x2F[^?]*\\x2F/]'

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
    // One import style (CLAUDE.md «Interface structure»): `./name` for a file in the same folder,
    // `@/path` for anything else, so a path reads the same in every file and survives a move.
    // Covers `import` / `export … from`, `import()` and `vi.mock`. Blind spot: a path that is not
    // a plain string literal (built at runtime, a template string).
    files: ['src/**/*.{ts,vue}'],
    rules: {
      'no-restricted-imports': [
        'error',
        { patterns: [{ regex: '^\\.\\./|^\\./[^?]*/', message: OTHER_FOLDER }] },
      ],
      'no-restricted-syntax': [
        'error',
        { selector: `ImportExpression > Literal${RELATIVE_ELSEWHERE}`, message: OTHER_FOLDER },
        {
          selector: `CallExpression[callee.object.name='vi'][callee.property.name='mock'] > Literal${RELATIVE_ELSEWHERE}`,
          message: OTHER_FOLDER,
        },
      ],
    },
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
