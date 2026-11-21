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

// Errors never vanish (CLAUDE.md, engineering rule 6): a bridge call whose promise is dropped,
// `void emu.x()` or a bare `emu.x()` statement, loses its failure. Hand it to `report` / `ignore`
// (src/lib/errors.ts), or await it where the error is handled. Seen failing on its fixture in
// src/lib/errors.lint.test.ts. Blind spots: a call through an alias (`const c = emu.controls`),
// through a wrapper (`void load()` where `load` lets the bridge's error out), and `.then()`
// without a `.catch`; the global handlers (lib/errors.ts `catchEverything`) still report those.
const DROPPED_EMU_CALL = 'Hand the promise to report() / ignore() (src/lib/errors.ts) or await it.'
const EMU_CALL =
  "CallExpression:matches([callee.object.name='emu'], [callee.object.object.name='emu'])"

export default withVueTs(
  {
    // src/generated: written by ts-rs from the Rust types (crates/emudome-core/src/contract.rs)
    // and compared byte for byte by a cargo test; its files import each other across its own
    // subfolders (`../X`), which the import rule below refuses in hand-written code.
    ignores: ['dist', 'node_modules', 'src-tauri', 'src/generated'],
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
    // a plain string literal (built at runtime, a template string). The same rule list refuses a
    // dropped bridge call (DROPPED_EMU_CALL above).
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
        {
          selector: `:matches(UnaryExpression[operator='void'], ExpressionStatement) > ${EMU_CALL}`,
          message: DROPPED_EMU_CALL,
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
