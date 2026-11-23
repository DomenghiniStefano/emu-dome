# The design system in one folder

## Why

A UX collaborator joins the project, working with his own Claude. Today the design system is spread over six
places (`assets/theme/`, `assets/main.css` and `utilities.css`, `components/ui/`, `lib/cn.ts` and `lib/design/`,
`lib/colour.ts` and `lib/accent.ts`, `composables/useDensity.ts`, `assets/brand/`, `kit/`): nothing says «this is
all of it». One folder, with a README that maps it, is what someone arriving from outside opens first. Nothing
changes on screen.

## Approaches

- **A, chosen: `app/src/design-system/` inside the app.** One folder, an ESLint boundary that keeps it from
  importing the app. Cost: every import of a primitive changes (230 files, mechanical).
- B: a separate pnpm package (`app/packages/design-system`). A real boundary, but a second Vite / tsconfig /
  Tailwind source / licence setup for one app. A is a step towards it if it is ever needed.
- C: leave the files where they are, write only a map. Cheapest, but the structure stays scattered: not what was
  asked.

## The folder

```
app/src/design-system/
  README.md      the map: what is here, where, how a value is changed, the rules
  index.css      the entry (was assets/main.css): Tailwind, the tokens, the variants, @source
  tokens/        colors, typography, spacing, radius, border, shadow, opacity, motion, layers (.css)
  styles/        utilities.css: the named compound looks (glass, glow…)
  components/    the primitives on Reka UI (was components/ui/), one folder each
  lib/           cn.ts, themeKeys.ts, colour.ts, accent.ts, useDensity.ts
  brand/         the mark
  kit/           the #kit page (development only): every primitive in every state, the review board
```

- `components/` is the name the big design systems use (Primer, Carbon, Polaris). The app's feature folders
  stay `src/components/<feature>/`; the path tells them apart.
- What decides *when* a look applies stays in the app: `lib/theme.ts` and `lib/appearance.ts` (they read the
  settings through the bridge), `composables/useUiMode.ts` and `lib/uiMode.ts` (TV: the question, the
  controllers), `composables/useDeviceLayout.ts`. The design system holds the looks; the app switches them.
  (The `themes/` folder discussed earlier is dropped: once the switching stays in the app, what is left fits
  in `lib/`.)
- **Boundary** (ESLint, `app/eslint.config.js`): a file under `src/design-system/` imports only
  `@/design-system/…`, `@/nav` (the primitives' controller keys) and `@/generated/…` (the Rust contract's types
  and the settings catalogue `accent.ts` reads). `src/design-system/kit/` is the declared exception: it shows the
  feature components too. Seen failing on a fixture test.

## Not changed

Class names, tokens' names and values, the primitives' API, the Kit's content, `#kit`. Every rule of CLAUDE.md
«Interface structure» holds; only paths move. `docs/specs/2026-11-18-design-system-foundations.md` is history and
keeps its paths, with a note at the top pointing here.

## Plan

1. `git mv` the files into the folder above (history kept).
2. Rewrite the imports: `@/components/ui/` → `@/design-system/components/`, `@/lib/cn` → `@/design-system/lib/cn`,
   `@/lib/design/themeKeys`, `@/lib/colour`, `@/lib/accent`, `@/composables/useDensity` → `@/design-system/lib/…`,
   `@/assets/theme/` → `@/design-system/tokens/`, `@/assets/main.css` → `@/design-system/index.css`,
   `@/kit/` → `@/design-system/kit/`; `index.css`'s own `@import` / `@source` paths; `index.html`'s mark;
   `components.json`; `accent.ts` takes `Theme` from `@/generated/app/Theme`.
3. Config and scripts: the ESLint single-word rule's path, the boundary rule and its fixture test,
   `scripts/scan-conventions.mjs` (`insideUi`, its fixtures' list, messages).
4. Docs: `design-system/README.md`; CLAUDE.md, `docs/frontend-conventions.md`, `docs/customization.md`,
   `THIRD_PARTY.md`, the comments that name the old paths.
5. Verify: typecheck, lint, Vitest, `scan-conventions.mjs --self-test` and run, `pnpm build`, the Kit opens in
   `pnpm dev`; `./scripts/check.ps1 --fast` at the end.
