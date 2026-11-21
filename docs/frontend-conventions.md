# Frontend conventions

How `app/src` is put together, and which rules a script holds and which only review holds. The reasons
behind the design system's choices are in `docs/specs/2026-11-18-design-system-foundations.md` (a dated
record of cycle 1; where it and this file disagree, this file is current). The look itself is Cowork's:
the EmuDome Design System artifact linked from `CLAUDE.md`.

## Layout of `app/src`

- `components/ui/<name>/`: the **primitives**, the only place a raw `<button>` / `<input>` is written.
- `components/<feature>/`: everything one feature is, together: its screen root (`library/LibraryView.vue`,
  `quick-menu/QuickMenu.vue`…), its parts, its composables (`useX.ts`), its pure modules (`x.ts`) and their
  tests. A screen composes panels, a panel composes primitives. A component another feature reuses stays
  with the feature it belongs to and is imported from there (`settings/SettingRow.vue`,
  `cover/GameCover.vue`, `saves/SlotTile.vue`). Features today: `controls`, `cover`, `details`,
  `first-run`, `game`, `library`, `quick-menu`, `saves`, `settings`, `shell`, `toasts`.
- `composables/`: the `useX` functions used by two or more features or by the app root (`useDensity`,
  `useDeviceLayout`, `useListeners`, `useSettingsChanged`, `useToasts` with `useErrorSink`); `lib/`: plain modules and
  helpers shared the same way (`cn`, `cycle`, `design/`, `settingsModel`, `controls`, `theme`…). A
  module only one feature uses moves into that feature's folder.
- The root holds the app's wiring only: `main.ts`, `App.vue`, `emu.ts` (the only file that calls
  `invoke`, the typed bridge to Rust), `nav.ts` (the focus navigation), `styles.css`, `i18n/`
  (messages, `Message` / `useSay`, code → text labels), `assets/` (tokens and named utilities), `kit/`
  (the Kit page), `generated/` (the bridge's types and the catalogue's keys, written by the Rust
  contract tests, never edited by hand; import them through `@/emu`, and type a setting key as
  `SettingKey`, never `string`).

## Imports

`./name` for a file in the same folder, `@/path` for anything else; never `../`, never `./sub/x`. A path
reads the same in every file, and a move changes only the moved file's importers. `vi.mock` and
`import()` name the module the same way. Checked by ESLint (`no-restricted-imports` and
`no-restricted-syntax` in `app/eslint.config.js`); blind spot: a path that is not a plain string literal.

## Bridge calls (`emu.*`)

- **Errors never vanish**: a call is awaited where its error is handled (try / catch, a dialog, a
  row's message), or handed to `lib/errors.ts`: `report(p)` (logged and toasted), `ignore(p, why)`
  (logged only, with the reason), `settle(p)` (awaited, reported, the sequence goes on). `main.ts`
  installs `catchEverything` (Vue's `errorHandler`, `unhandledrejection`) as the last net. ESLint
  refuses `void emu.x()` and a bare `emu.x()` statement; blind spots: an alias, a wrapper, a
  `.then()` without `.catch` (`app/eslint.config.js`, fixture in `lib/errors.lint.test.ts`).
- **No error relays**: a component that does not show a failure itself sends it to `useErrorSink()`
  (logged, then a toast) and a notice to `useToasts().notify`; both are `provide`d once by App.vue
  (`composables/useToasts.ts`). No `error` / `notice` emits passed up from parent to parent. The toast
  list lives in the composable, so a toast pushed before the stack mounts is shown once it does.
- **Stale answers are dropped**: a load that can be overtaken (scope, game, reload) takes a token
  from `lib/latest.ts` (`latest()`, `latestBy()`) and applies its answer, or shows its error, only
  if still fresh. Held by review.
- A count goes through vue-i18n's plural form (`"{n} game | {n} games"`, `t(key, { n }, n)`).

## Primitives (`components/ui/`)

- All on **Reka UI**; none hand-made. One folder per primitive: `index.ts` re-exports, the parts are
  `.vue` files, `variants.ts` holds the looks.
- Variants are `as const` objects (`ButtonVariant.Primary`, `ButtonSize.Small`) mapped to classes
  with `cva`. A screen passes the constant (`:variant="ButtonVariant.Primary"`), never a string.
- A screen never restyles a primitive (no extra colours, sizes or `!` overrides on it): a missing look
  is a new variant, added to the Kit page first.
- Widgets with a role (dialog, popover, menu, radio group, checkbox, select, switch, tabs) are always
  the primitive: Dialog, Popover, DropdownMenu, RadioGroup, Checkbox, Select, Switch, Tabs. RadioGroup and
  Checkbox are behaviour only: each item is `as-child` on the screen's Card or Button, which keeps its
  look. Popover and DropdownMenu hang from the screen's own button (`anchor`) and the screen holds
  `open`, so A and a click toggle them the same way. Checked by `hand-role` (below).

## Kit page (`kit/`)

`#kit` in `pnpm dev`, development only (`lib/devRoutes.ts`, mounted by `main.ts` only under
`import.meta.env.DEV`). Every primitive in every state, the three themes, the five accents, the touch
density. It is where a primitive or variant is tried before a screen uses it, and what
`scripts/headless/kit-nav.mjs` drives in a real browser.

## Tokens (`assets/theme/`)

Tailwind v4 reads the token files in `@theme` with its default scales **reset** (`--<namespace>-*:
initial`): a value that is not a token generates no class. Preflight is off; the document's base is
`styles.css`. One file per family, imported by `assets/main.css`:

| File | Family | Classes |
| --- | --- | --- |
| `colors.css` | colours, per theme on `:root` / `[data-theme]`, mapped with `@theme inline` | `bg-surface`, `text-ink`, `border-line` |
| `typography.css` | type steps (size + line + weight + tracking), fonts, weights; `align-icon` | `text-label`, `font-semibold` |
| `spacing.css` | the spacing grid, sizes, role names, breakpoints | `p-12`, `gap-section-gap`, `h-control`, `max-controls-stack:` |
| `radius.css` | radii | `rounded-6`, `rounded-control` |
| `border.css` | edge widths, outline widths and offsets | `border`, `border-strong`, `outline-ring`, `outline-offset-ring` |
| `shadow.css` | shadows, glows, blurs | `shadow-card`, `backdrop-blur-*` |
| `opacity.css` | named opacities | `opacity-disabled`, `opacity-dim` |
| `motion.css` | durations, curves | `duration-panel`, `ease-out` |
| `layers.css` | stacking order (named utilities: z-index has no theme namespace) | `z-menu`, `z-dialog` |

- **Every visual value is a token**: colours, borders (width and colour), margins, paddings, gaps,
  radii, shadows, opacity, type, durations, stacking, breakpoints. Tokens are internal design
  variables, never user settings.
- **Role tokens**: where a value has a role it gets a role name pointing at a step
  (`--spacing-card-pad: var(--spacing-16)`, `radius-control`, `spacing-section-x`), so changing every
  card's padding is one edit.
- **«Pending Cowork» tokens**: a value the design system has not decided yet is a token under a
  `Pending Cowork` comment in its family's file, named by what it is for, never a literal in a screen.
  Cowork's decision is then an edit of that one line.
- Compound looks (glass, glows, gradients, focus rings drawn inside a frame) are named utilities in
  `assets/utilities.css`, written through the tokens.
- Themes (Scuro / Nero / Chiaro) and the runtime accent are CSS variables (`theme.ts`, `appearance.ts`,
  `accent.ts`); classes point at them, so a theme change needs no class. The default accent (amber)
  is written in `colors.css`, every other swatch computed by `accent.ts`'s rules; `accent.test.ts`
  holds the default's values to the same rules, so a change to one shows in the other. Text on the
  accent is `on-accent` (white or dark per swatch), never a fixed white.
- `cn()` (`lib/cn.ts`) merges classes; it learns the token names from the theme CSS itself
  (`lib/design/themeKeys.ts`), so `cn('text-label', 'text-ink')` keeps both.

## Touch density

The phone layout sets `data-density="touch"` on `<html>` once (`composables/useDensity.ts`, called by
`App.vue`). Primitives and screens style their touch look with the `touch:` variant (`touch:h-touch`).
A component never takes a `phone` / `touch` prop only to change its look; such a prop is fine only
when it changes structure (what is rendered, `v-if`).

## Controller navigation (`nav.ts`)

- Every screen calls `useNavScope(root, { back })` and marks its first element `data-nav-initial`;
  every menu works with d-pad / A / B as well as mouse and keyboard.
- Scopes form a tree that follows the component tree (provide / inject): the steps go down to the newest
  shown child at each level, so a child wins over its parent whatever the mount order. A part of a
  screen with its own `back` / `tab` (the shell's Settings, Controls, a game's details) calls
  `useNavSection(root, { back, tab })`: it does not hold the focus, and a step it leaves (its handler
  returns `false`, or it has none) bubbles to the screen; bubbling stops at a scope, so a screen behind
  a dialog gets nothing. Never drive a child's steps through a template ref.
- A step looks in the focused element's scroll container first and in the whole scope only when nothing
  is there in that direction; disabled, `aria-disabled`, `inert` and `visibility:hidden` elements are
  never landed on. Entering a radio group lands on its chosen radio.
- Where the shell is, is one typed `Route` (`components/shell/route.ts`) held by App.vue and passed as
  props; a section reads what it was opened for (its arrival) once, on mount. No module-level refs a
  screen reads and clears.
- Steps from `emu://nav` (the controller, routed in Rust) and the arrows / Enter move the focus
  spatially; `confirm` clicks; `back` calls the scope's `back`. The focus ring shows only while moving
  without the mouse (`data-nav` on `<html>`).
- One generic rule for elements that own their keys, recognised by the attributes Reka renders (never
  by component name): they take the arrows along their axis and Enter; at their ends the focus moves
  spatially. Select and Dialog take B while open and give the focus back on close.
- Tested in Vitest (`nav.test.ts`, `components/ui/nav.test.ts`) and in a real browser
  (`scripts/headless/kit-nav.mjs`).

## Text (`i18n/`)

- No visible text in a template or a script: everything goes through vue-i18n (`t()`, `<I18nT>`);
  `en.json` is the reference and fallback, `it.json` has the same keys.
- The backend sends codes and values; `i18n/labels.ts` turns them into text (`errorText`,
  `speedLabel`, `gamepadName`).
- Tone: address the user directly, verb first on buttons, no exclamation marks; errors say what to do.

## What the scripts hold

`scripts/scan-conventions.mjs` (run by `scripts/check`, `--self-test` first: every rule must fire on its
fixture in `scripts/scan-conventions/`, and `clean.*.fixture` must stay silent). Every rule blocks in
every `.vue` and `.ts` file of `app/src`. It reads source text, so a comment that quotes a forbidden
class fires too.

| Rule | Forbids | Cannot see |
| --- | --- | --- |
| `literal-colour` | a colour in brackets: `bg-[#fff]`, `text-[rgb(…)]` | colours built at run time, in `style`, named ones (`bg-[red]`) |
| `alpha-modifier` | `bg-ink/50` | — |
| `dark-variant` | `dark:` | — |
| `literal-variant` | `variant="primary"` / `size="…"` on a component | a string through a variable; other closed-set props |
| `raw-control` | `<button>` / `<input>` outside `components/ui` | render functions, `<component :is>` |
| `style-block` | any `<style>` block | — |
| `arbitrary-length` | px / rem / em in brackets: `w-[13px]`, `top-[calc(100%+4px)]`, `[@media(max-width:900px)]:` | % / vh / vw, `style` bindings, component props (`:size="14"` on an icon), classes built from numbers |
| `arbitrary-scale` | `opacity-[…]`, `z-[…]`, `duration-[…]`, `delay-[…]`, `ease-[…]` | `style` bindings |
| `numeric-scale` | bare-number utilities the theme cannot reset: `opacity-40`, `z-5`, `duration-300`, `border-2`, `outline-offset-2`, `ring-2`, `scale-95` | layout counts are allowed on purpose (`grid-cols-3`, `aspect-3/2`, `rotate-90`) |
| `important` | `w-48!`, `!w-48` | in scripts and `.ts`, a `!` class with no `-` or `:` (`flex!`) |
| `look-only-prop` | a `phone` / `touch` prop used only in `:class` | a prop read in `<script>`, passed down, or of another name |
| `hand-role` | `role="dialog"`, `alertdialog`, `radio`, `radiogroup`, `checkbox`, `menu`, `menuitem`, `listbox`, `option`, `switch`, `tab`, `tablist` (static or `:role`) outside `components/ui`; the files not moved yet are in `HAND_ROLES_PENDING` with the reason | a role set from a script, a render function, a `:role` built from a variable |

Allowed by construction: values built only from tokens (`calc(100%+var(--spacing-6))`, grid templates
of `fr` / `minmax` / `auto` / `var`), keywords (`text-[length:inherit]`), property lists
(`transition-[background]`). An exception with no token form goes in the scanner's `ALLOWED_LENGTHS`
by exact class, with its reason (empty today).

The rest of Tailwind's default palette and scales (`text-red-500`, `rounded-lg`, `shadow-md`,
`text-sm`, `md:`) generates nothing after the theme reset, so no rule is needed for it.

Other gates on the frontend: `scripts/visible-text.mjs` (hardcoded text in templates), `invoke` only in
`emu.ts`, no emoji, `vue-tsc` (strict, with `noUncheckedIndexedAccess`: an index read is
`T | undefined` until checked), ESLint, Prettier, Vitest.

## What only review holds

- Text built in `<script>` or TypeScript goes through `t()`.
- The look (graphite, accent only on the main action, glass, Lucide icons) and every value the scanner
  cannot see: % / viewport lengths, icon sizes as props, `style` bindings.
- A `phone` / `touch` prop that only changes the look but escapes `look-only-prop`.
- Clean code: small components with one job, logic in composables and tested pure functions,
  nothing duplicated across screens, names that say what a thing is for.
