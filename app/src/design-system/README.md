# EmuDome Design System

Everything that decides how EmuDome looks lives in this folder: the tokens, the named looks, the primitives
(Button, Card, Dialog…), the few helpers they need, the mark, and the Kit page that shows all of it. The
screens (`app/src/components/<feature>/`) only compose what is here; they never invent a colour, a size or a
control of their own.

The look in words: modern console style, dark. Neutral graphite, the colour comes from the game covers; the
accent (amber, the app icon's `#faa40f`) only on the main action; glass for panels and menus; Lucide icons, no
emoji; compact and near-square, no empty bands, sober motion. The full rules and their reasons:
`docs/frontend-conventions.md`, and CLAUDE.md «Interface and tone» / «Interface structure».

## Map

```
design-system/
  README.md          this map
  index.css          the entry: Tailwind (no preflight), the `touch:` and `tv:` variants, the tokens,
                     the named looks, and the @source list (the folders Tailwind reads classes from)
  tokens/            one file per family, every visual value of the app
    colors.css         three themes (Scuro, Nero, Chiaro) and the accent
    typography.css     the type scale: each step a complete style (`text-label`, `text-title`…)
    spacing.css        spacing and sizes, in px (`p-12` is 12 px), role names (`card-pad`, `section-x`…)
    radius.css         corner radii, role names (`radius-control`, `radius-card`…)
    border.css         edge widths
    shadow.css         shadows, glows, the glass's blurs
    opacity.css        named opacities (`opacity-disabled`…)
    motion.css         durations and easings; appearing / pressing, keyed by the primitives' `data-slot`
    layers.css         stacking order, named (`z-menu`, `z-dialog`…)
  styles/
    utilities.css    the looks made of more than one property, by name (glass panels, accent gradients,
                     the cover haze, focus rings, glows)
  components/        the primitives, one folder each, all on Reka UI:
                     badge, button, capture-field, card, checkbox, chip, date-time-field, dialog,
                     dropdown-menu, empty-state, heading, input, kbd, labelled-block, link-row, popover,
                     progress, radio-group, search-field, section-page, segmented, select, slider,
                     stat-tile, switch, tabs, tooltip
  lib/
    cn.ts            merges class lists, knowing the token names (read from tokens/ by themeKeys.ts)
    themeKeys.ts     the token names each Tailwind namespace declares, read from the CSS itself
    colour.ts        colour arithmetic: OKLCH, WCAG contrast
    accent.ts        the accent swatches (`general.accent`) turned into the accent tokens of every theme
    useDensity.ts    sets `data-density="touch"` on <html> (the phone's look)
  brand/             EmuDome's mark (SVG)
  kit/               the Kit page (development only, never in the build)
```

## Seeing it: the Kit page

```
cd app
pnpm install
pnpm dev            then open http://localhost:5173/#kit
```

No Rust, no emulator cores and no Tauri are needed: in a plain browser `emu.ts` stands in for the backend. The
Kit shows every primitive in every state, the main feature components (`kit/features/`: library, game,
settings) with mock data, in the three themes and five accents, desktop and touch side by side. It is also the
review board: under a section, a `KitIssue` holds a problem, «Ora» (now) and «Proposta» (proposal) rendered
with the real components, and a status (open / proposed / decided); `KitIndex` counts the open ones.

The phone to check against first is 360 × 780 CSS px (and 780 × 360 on its side). TV layouts are drawn for
1920 × 1080.

## How a change is made

- **A value** (a colour, a size, a radius, a duration): edit its token in `tokens/`. Every screen follows. A
  value that is not a token generates no class at all: Tailwind's default scales are reset, so `p-[13px]`,
  `bg-red-500` or `rounded-lg` do not exist here, and `scripts/scan-conventions.mjs` refuses literals in
  classes.
- **A new value**: a new token, with a role name where it has a role (`spacing-card-pad`, not `spacing-10`).
- **A compound look** (glass, a glow): a named utility in `styles/utilities.css`.
- **A new look of a control**: a new variant in that primitive's `variants.ts` (`ButtonVariant.Primary`, never
  `variant="primary"`), shown in the Kit first. A screen never restyles a primitive.
- **A new primitive**: a folder in `components/` (`index.ts` re-exports, `variants.ts` the variants as
  `as const`, the parts as `.vue`), on Reka UI, with its section in `kit/sections/`.
- **Themes and accent** are CSS variables on `:root` / `[data-theme]`, so switching them needs no class.

## The three layouts

The same screens serve three layouts; a component never takes a `phone`, `touch` or `tv` prop just to change
its look.

- **Desktop**: the default.
- **Touch** (the phone, upright and on its side): `data-density="touch"` on <html>, styled with the `touch:`
  variant (`touch:h-44`). Touch targets 44 px.
- **TV**: `data-layout="tv"` on <html>, styled with the `tv:` variant and the «TV» tokens, grown by
  `--tv-scale` past 1920 × 1080.

What decides *when* a theme or a layout applies is the app's, not the design system's: `lib/theme.ts` and
`lib/appearance.ts` (they read the settings), `composables/useUiMode.ts` (TV),
`composables/useDeviceLayout.ts` (phone or desktop).

## Rules that hold everywhere

- Every user-visible text is in `app/src/i18n/` (`en.json`, `it.json`), never written in a component.
- Every menu is fully usable with a controller (d-pad, A, B) as well as mouse, keyboard and touch. The
  primitives already take their keys (`@/nav`); a new control must too.
- Fonts, icons and images need a licence compatible with GPL-3.0-or-later, listed in `THIRD_PARTY.md`.
- No Nintendo material, not even as a placeholder: no commercial game screenshots, covers or logos in the code,
  the Kit or a mockup. Mock covers are homebrew or generated.

## The boundary

This folder imports nothing of the app except `@/nav` (the controller keys the primitives take) and
`@/generated/…` (the Rust contract's types and the settings catalogue). ESLint refuses anything else
(`app/eslint.config.js`, tested in `boundary.lint.test.ts`). `kit/` is the exception: it shows the feature
components too.

## Checks

From `app/`: `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm format`; from the repo root:
`node scripts/scan-conventions.mjs` (the design-system conventions). `./scripts/check.ps1` runs everything.
