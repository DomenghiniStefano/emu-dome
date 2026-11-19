# Frontend conventions

How `app/src` is put together, and which rules a script holds and which only review holds. The reasons
behind the design system's choices are in `docs/specs/2026-11-18-design-system-foundations.md` (a dated
record of cycle 1; where it and this file disagree, this file is current). The look itself is Cowork's:
the EmuDome Design System artifact linked from `CLAUDE.md`.

## Layout of `app/src`

- `components/ui/<name>/`: the **primitives**, the only place a raw `<button>` / `<input>` is written.
- `components/<feature>/`: a feature's components, composables and pure modules live together in its
  folder; a screen composes panels, a panel composes primitives.
- `composables/`: the `useX` functions shared by more than one feature; `lib/`: pure helpers (`cn`,
  `lib/design/`), with their tests beside them.
- `kit/`: the Kit page. `i18n/`: messages and the code → text labels. `assets/`: tokens and named
  utilities.
- `emu.ts` is the only file that calls `invoke` (the typed bridge to Rust); `nav.ts` is the focus
  navigation.

> TODO (manager, after ds3/placement merges): the exact folder layout and the one import style.

## Primitives (`components/ui/`)

- All on **Reka UI**; none hand-made. One folder per primitive: `index.ts` re-exports, the parts are
  `.vue` files, `variants.ts` holds the looks.
- Variants are `as const` objects (`ButtonVariant.Primary`, `ButtonSize.Small`) mapped to classes
  with `cva`. A screen passes the constant (`:variant="ButtonVariant.Primary"`), never a string.
- A screen never restyles a primitive (no extra colours, sizes or `!` overrides on it): a missing look
  is a new variant, added to the Kit page first.

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
  `accent.ts`); classes point at them, so a theme change needs no class.
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

Allowed by construction: values built only from tokens (`calc(100%+var(--spacing-6))`, grid templates
of `fr` / `minmax` / `auto` / `var`), keywords (`text-[length:inherit]`), property lists
(`transition-[background]`). An exception with no token form goes in the scanner's `ALLOWED_LENGTHS`
by exact class, with its reason (empty today).

The rest of Tailwind's default palette and scales (`text-red-500`, `rounded-lg`, `shadow-md`,
`text-sm`, `md:`) generates nothing after the theme reset, so no rule is needed for it.

Other gates on the frontend: `scripts/visible-text.mjs` (hardcoded text in templates), `invoke` only in
`emu.ts`, no emoji, `vue-tsc`, ESLint, Prettier, Vitest.

## What only review holds

- Text built in `<script>` or TypeScript goes through `t()`.
- The look (graphite, accent only on the main action, glass, Lucide icons) and every value the scanner
  cannot see: % / viewport lengths, icon sizes as props, `style` bindings.
- A `phone` / `touch` prop that only changes the look but escapes `look-only-prop`.
- Clean code: small components with one job, logic in composables and tested pure functions,
  nothing duplicated across screens, names that say what a thing is for.
