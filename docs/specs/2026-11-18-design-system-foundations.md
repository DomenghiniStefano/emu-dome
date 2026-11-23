# Design system, cycle 1: foundations and primitives (spec)

**Date:** 2026-11-18 · **Card:** Trello #61 (cycles 2 and 3: #62, #63)

> A record of cycle 1: its paths are those of the time. Since `2026-11-23-design-system-folder.md` the design
> system lives in `app/src/design-system/` (`assets/theme/` → `tokens/`, `assets/main.css` → `index.css`,
> `components/ui/` → `components/`, `kit/` → `kit/` inside it); its `README.md` is the current map.
**Source of names and values:** EmuDome Design System README and `tokens.json`
(https://claude.ai/artifact/TEz4yQgd1w27HRQLt4beAy), then `app/src/styles.css` and the screens.
**Status:** approved by the owner (2026-11-18), 0BSD allowed. Points marked **Cowork** still need
a design decision; until then the screens keep their values.

## What cycle 1 delivers

The vocabulary the screens will be written in, with **nothing on screen changing**: Tailwind v4
tokens, shadcn-vue primitives on Reka UI dressed in EmuDome's look, a development-only Kit page,
`cn()` that knows the tokens, a scanner for new code, and the one rule that makes Reka and the
controller agree (Decision 6). Cycle 2 migrates the screens one by one, cycle 3 removes `<style>`
from the SFCs and makes the scanner blocking everywhere.

## Decision 1: one standard for the interface

One structure, the same in every Dome project (owner, 2026-11-18):

- `app/src/assets/theme/{colors,typography,spacing,radius,shadow,opacity,motion,layers}.css`, each
  family's Tailwind namespace **reset** (`--color-*: initial` …): a class outside the system
  generates nothing (`bg-red-500`, `text-sm`, `rounded-md`, `p-4`).
- `app/src/components/ui/<name>/`: `index.ts`, `variants.ts` with `as const` constants keyed into
  `cva`, the `.vue` parts. **Every primitive is shadcn-vue on Reka UI**: no hand-made primitives,
  no mix. `components.json` written by hand (`style: reka-vega`, `utils: @/lib/cn`); never
  `shadcn-vue init` (it rewrites the stylesheet).
- `lib/cn.ts` built with `extendTailwindMerge`, its lists read from the theme CSS by
  `lib/design/themeKeys.ts` (`?raw` imports): the CSS stays the one place a token is declared.
- Vitest for the pure functions and for each primitive with the controller (Decision 6).
- Kit page `#kit`, development only (Decision 8). Scanner `scripts/scan-conventions.mjs` (Decision 9).

EmuDome's choices and why:

| choice | why |
| --- | --- |
| three themes and a run-time accent: values on `:root` / `[data-theme]`, mapped with `@theme inline reference` | Decision 2 |
| glass, blur, glows and the accent dome are tokens and a few named `@utility` rules | the look of the README («Vetro», «Forme») |
| px sizes; the interface scale is `zoom` on `<html>` (`a11y.textsize`) | the root's font size is 13px, so a rem is 13px and rem steps come out wrong |
| Tailwind's 4px `--spacing` multiplier off; named steps, `p-12` = 12px = `space-12` | the design system names its grid in px (`space-2` … `space-48`) |
| no preflight; Tailwind's utilities in `@layer utilities` | Decision 3: nothing on screen changes |
| every primitive driven by `nav.ts` (controller and arrows), not only the keyboard | Decision 6 |

## Decision 2: themes and the run-time accent

- **Values** stay CSS variables on `:root` (Scuro), `:root[data-theme='black']`, `[data-theme='light']`,
  `[data-transp='reduce']`, under the names the screens already read (`--bg`, `--ink`, `--accent`…),
  same values, same order. They move from `styles.css` to `theme/colors.css` (and `shadow.css`,
  `opacity.css`, `motion.css`); `styles.css` keeps only the document base and the legacy classes.
- **Tailwind names** map onto them: `@theme inline reference { --color-ink: var(--ink); … }`.
  `inline`: `text-ink` compiles to `color: var(--ink)`, so it follows `data-theme`, the accent
  `<style>` that `appearance.ts` writes after the stylesheet (`accent.ts`), and «Riduci la
  trasparenza». `reference`: Tailwind emits no `--color-*` variable of its own, so a value lives in
  one place, and `--shadow-float` can point at the per-theme variable of the same name without a cycle.
- Tokens whose value is the same in every theme (radii, type, spacing, motion) are plain `@theme`;
  `@theme static` where the screens' scoped CSS reads them by name (`--radius-6`, `--ease-out`).
- The bridge is temporary: cycle 3 drops the old names once no scoped CSS reads them.

## Decision 3: additive and visually neutral

- `main.css` imports `tailwindcss/theme.css` and `tailwindcss/utilities.css` only (no preflight),
  declares `@layer theme, base, components, utilities`, and `source(none)` with
  `@source '../components/ui'` and `@source '../kit'`: Tailwind reads only the design system's code.
  Cycle 2 adds each migrated screen; cycle 3 scans all of `src/`.
- `styles.css`'s global `button { font: inherit; color: inherit }` moves into `@layer base`, or it
  (unlayered) would beat every primitive's `text-*` utility. Neutral for the screens: their scoped
  rules are unlayered and more specific anyway.
- **Collision guard.** A class name a screen uses for its scoped CSS can also be a Tailwind utility
  (`block`, `grid`, `table`, `grow`, `inline`, `underline`, `overline`); if the word appears in the
  sources Tailwind reads, the utility applies to that screen's element. Measured today: `block`,
  `grid`, `underline`, `glass-thick` are generated and neutral (the scoped rule sets the same
  property); `table`, `grow`, `inline`, `overline` are latent. The scanner checks it (Decision 9).
- **Proof**, measured on the prototype (2026-11-18): 11 screens (Library in Scuro / Nero / Chiaro,
  empty Library, Saves, Controls, Cloud, Settings, a Settings page, Settings in Chiaro, First run)
  shot headless before (develop 8ca989f) and after, 1280×800, compared pixel by pixel:
  **0 different pixels in all 11**. Cycle 1 must end with the same result.

## Decision 4: token catalogue (names from the design system)

| family | file | tokens |
| --- | --- | --- |
| colour | `colors.css` | `bg`, `surface`, `surface-raised`, `surface-hover`, `glass-thin`, `glass`, `glass-thick`, `glass-edge`, `line`, `line-strong`, `scrim`, `ink`, `ink-muted`, `ink-faint`, `violet`, `accent`, `accent-hover`, `accent-deep`, `accent-crown`, `on-accent`, `accent-text`, `accent-soft`, `focus-ring`, `success`, `warning`, `danger`, `cover-1..3`; new, each a value a screen writes by hand today: `knob` (#fff of switch and slider thumbs), `accent-edge` (40% accent, LibraryView chip). DS tokens not used yet (`scene-scrim`, `cover-scrim`, `skeleton`, `on-cover`, `*-soft` states) arrive with the screen that needs them. |
| type | `typography.css` | Banco: `title` 20/26/600/-0.01em, `heading` 15/20/600, `body` 13/18/400, `label` 13/18/500, `caption` 12/16/400, `overline` 11/14/600/0.08em. Scena: `display` 32/38/600/-0.02em, `lead` 15/22/400, `stat` 48/52/600/-0.03em. Each step carries line height, weight and tracking. Weights `regular`, `medium`, `semibold`. |
| spacing | `spacing.css` | grid `0 2 4 6 8 12 16 20 24 32 48` (`space-*`); sizes `control-sm` 28, `control` 32, `control-lg` 40 (`size` in tokens.json), plus named sizes a primitive needs (`field` 200, `switch` 36, `dialog` 520, `menu` 320). |
| radius | `radius.css` | `4`, `6`, `10`, `16` (`radius-4` …); `rounded-full` for switches and round keys. |
| shadow | `shadow.css` | `card`, `lift`, `float` (per theme), `glow-sm`, `glow-md`, `glow-lg` (mixed from `--violet`, so they follow the accent); `blur-glass` 24, `blur-scene` 32. |
| opacity | `opacity.css` | `disabled`, `locked` (Decision 5); `--haze-opacity` stays a per-theme variable. |
| motion | `motion.css` | `duration-fast` 150, `-panel` 250, `-scene` 500, `-ambient` 1200; `ease-out`, `ease-spring`. Reduced motion stays `data-motion` in `styles.css`. |
| layers | `layers.css` | `@utility z-raised` 10, `z-menu` 20, `z-overlay` 30, `z-dialog` 40. |

Named utilities (`utilities.css`), each one look, not a long class string: `glass-thick` (fill,
edge, top light, float shadow, blur 24 + saturate 140%), `bg-accent-dome` (the main button's
gradient), `bg-accent-track` (switch on).

## Decision 5: one scale per family (for Cowork)

Every value in use today mapped to one token. Rows where a screen moves by ≥1px need Cowork's
explicit yes; until then cycle 2 keeps those screens as they are (the migration of that value waits).

**Type** (font-size)

| in use | where | proposed | change |
| --- | --- | --- | --- |
| 10px | AppShell | `overline` 11 | +1 |
| 11px | 11 files | `overline` 11 | none (line heights to 14) |
| 12px | 19 files | `caption` 12 | none |
| 13px | 7 files, root | `body` / `label` 13 | none |
| 14px | AppShell, DetailsSaves, GameDetails, QuickMenu, `.btn-primary` | `heading` 15 (`.btn-primary`: `label` 13?) | +1 / −1 **Cowork** |
| 15px | 7 files | `heading` / `lead` 15 | none |
| 16px | GameDetails, QuickMenu | `heading` 15 | −1 |
| 17px | QuickMenu | `heading` 15 | −2 |
| 18px | FlowDialog, TouchControls | `title` 20 | +2 |
| 20px | 8 files | `title` 20 | none |
| 22px | GameDetails | `title` 20 | −2 |
| 24px, 26px | FirstRun | `display` 32 (first run is a Scena) or `title` 20 | ±6..8 **Cowork** |
| 32px | FirstRun, GameDetails, GameFlowOverlays, LibraryView | `display` 32 | none |

Line heights 17 and 19 (QuickMenu, GameCover) become the step's own. Letter spacing 0.04em
(GameCover) → `overline` 0.08em; −0.025em → `display` −0.02em.

**Control heights**

| in use | where | proposed | change |
| --- | --- | --- | --- |
| 28 / 30 | 8 files / ControlsRemap | `control-sm` 28 | 30 → −2 |
| 32 | 11 files, `.btn` | `control` 32 | none |
| 36 | ControlsView, CoverPicker, FlowDialog, GameDetails, LibraryEmpty, LibraryView, QuickMenu, SavesView, SettingRow | `control` 32, or a fourth step `control-md` 36 | −4, or a DS addition **Cowork** |
| 40 | `.btn-primary`, ControlsView tabs, FirstRun, GameDetails, SettingsView | `control-lg` 40 | none |
| 44 | GameDetails tabs, QuickMenu | tabs → 40 (ControlsView's); QuickMenu rows stay (README: 44 is the phone's touch minimum) | −4 **Cowork** |

**Radius**: 4, 6, 10, 16 stay. `8px` (AppShell, ButtonTester, LibraryView, QuickMenu, SavesView,
SettingsView, ToastStack) → `6` (−2); `12px` (QuickMenu, SettingsView, ToastStack) → `10` (−2);
`20px` (FirstRun, QuickMenu sheet) → `16` (−4); `999px` / `50%` → `rounded-full`. **Cowork**.

**Spacing**: grid 2/4/6/8/12/16/20/24/32/48. Off-grid today: `3` (AppShell, QuickMenu gap) → 4;
`5` (QuickMenu padding) → 4; `18` (ControlsRemap, ControlsView, SavesView gap; QuickMenu, `.btn-primary`
padding) → 16 or 20; `28` (GameDetails) → 24. **`10` and `14` are used in ~25 files each**: folding
them moves most screens, so the proposal is to **add `space-10` and `space-14` to the design
system** rather than fold. **Cowork**.

**Shadows**: `0 0 12px violet 45%` (DetailsFiles, SettingRow, `.btn-primary`) = `glow-sm`;
`0 0 60px violet 35%` (FirstRun) = `glow-lg`; `0 1px 2px #00000052` (LibraryView) = `shadow-card`
(in Chiaro it becomes the DS's lighter value: a change); slider thumb `0 1px 3px #00000080`
(SettingRow) → `shadow-card` (subtle change); QuickMenu sheet `0 -24px 64px` → new `shadow-sheet`;
GameDetails cover glow `0 0 40px tint 66%` → new `glow-cover`; tab underline / rail marker
(`inset 0 -2px` accent-text, `inset 2px 0` violet) → `shadow-tab`, `shadow-rail`; LibraryView's
own focus `0 0 0 2px focus-ring` → the global ring (README: 2px with 2px gap). **Cowork** for the
new names and the two changes.

**Opacity**: 0.35 / 0.4 / 0.45 → `disabled` 0.4; 0.5 / 0.55 → `locked` 0.55; 0.8 (LibraryView,
QuickMenu) stays until its meaning is named. **Blur**: 24 = `blur-glass`; 8px (App, CoverPicker)
and 2px (QuickMenu) are not in the DS: **Cowork**. **Motion**: `transition 600ms` → `duration-scene`
500 (−100ms); the 1.4s skeleton sweep (FirstRun, LibraryView) → new `duration-loop`. **Layers**:
5 (SettingsView) → `raised` 10; 35 (GameFlowOverlays) → `overlay` 30 or a fifth layer.

## Decision 6: Reka and the controller, one rule in `nav.ts`

EmuDome moves the focus spatially with `nav.ts`: the controller sends `emu://nav` steps (no key
events), and App.vue turns the arrows and Enter into the same steps. Reka components handle some
keys themselves. Measured before the rule existed (2026-11-18):

- Reka Tabs / ToggleGroup in headless Chrome with trusted keys: one ArrowRight moved **two** tabs;
  at the last tab Reka looped to the first and nav.ts moved on to the second, instead of leaving
  the list.
- Reka Switch toggles on its own Enter, and App.vue's `confirm` clicked it again: two toggles per
  press.
- Reka Slider's thumb takes the arrows itself; a controller step moved the focus away instead.
- Reka Select: `element.click()` does not open the field (it opens on pointerdown or on its own
  Enter / Space / ArrowDown), and its options select on pointerup or Enter, not on click; they are
  `tabindex=-1` divs, so nav.ts cannot land on them.
- Reka Select renders its **closed** content too, out of the page, to know its options: a nav scope
  registered there took every step of the screen behind it (found in the real-browser check).
- Reka Dialog: after closing, the focus went to `<body>` instead of the trigger.

**The rule** (generic, by the attributes Reka renders, never by component name; verified on the
DOM of reka-ui 2.11.0, and absent from every current screen, so no legacy screen changes):

1. **Arrow owner.** The focused element owns the arrows along one axis when it matches
   `[data-reka-collection-item][data-orientation]` (Tabs triggers, ToggleGroup items, Slider thumb;
   the axis is its `data-orientation`). It owns both axes when it is inside an open listbox,
   `[role="listbox"][data-dismissable-layer]` (Select).
   - Along the owned axis, `keyToNav` returns `null` (the keyboard event goes to Reka alone) and
     `dispatchNav` **sends Reka the same key**: a `keydown` with `key: 'ArrowRight'` … dispatched on
     the focused element (the controller path). App.vue's listener sees that event too, and
     `keyToNav` returns `null` for it by the same rule: no double move.
   - **Edges**: for a roving group (the owner is the closest `[data-orientation]` ancestor that is
     not itself an item; its items are its `[data-reka-collection-item]:not([data-disabled])`), a
     step past the first or last item goes back to nav.ts and moves spatially out of the group. The
     wrappers set `loop: false`, so Reka never wraps. A slider thumb (`role="slider"`) and an open
     listbox have no edge: the value clamps, the list stays.
   - On the other axis, nav.ts moves spatially as everywhere else.
2. **Enter owner.** `[role="tab"][data-state]`, `[role="option"][data-state]`,
   `[role="combobox"][data-state]`, `[role="switch"][data-state]` act on their own Enter:
   `keyToNav` returns `null` for Enter there, and `dispatchNav('confirm')` sends a `keydown` Enter to
   the element instead of `click()`. (`data-state` is Reka's; the screens' own `role="tab"` /
   `role="switch"` buttons have none and keep today's click.)
3. **Layers.** Dialog and Select content register a nav scope while open (`useNavScope`), whose
   `back` calls Reka's `onOpenChange(false)`: B closes the layer, never the screen behind it. The
   Select's scope has a root only while the list is open (`SelectNavScope`), so its closed content
   takes nothing.
   Escape from the keyboard is closed by Reka itself; App.vue's Escape outside a game is a no-op
   (`setMenu` returns without a game), and in the quick menu it is `back`, which closes the same
   layer: idempotent.
4. Nothing else changes in nav.ts: native `<input type="range">` stepping stays for the screens.
   `inputMode.ts` ignores untrusted key events, so a controller step sent as a key keeps the
   controller as the input in use (its key hints).
5. **Dialog focus.** The first focus goes to the `data-nav-initial` control (else Reka's first
   control); on closing, the dialog puts the focus back on what was focused when it opened
   (`DialogContent`), since Reka's own return lost it.

**Per primitive.** Vitest (`components/ui/nav.test.ts`, happy-dom) checks nav.ts's side for each
primitive: the key handed over, nav.ts not moving, the ends and the other axis left to nav.ts, A and
B. happy-dom does not run Reka's roving focus or slider keys, so the moves themselves are checked
in headless Chrome by `scripts/headless/kit-nav.mjs` (trusted keys and `dispatchNav` on the Kit
page, 24 checks, all passing on 2026-11-18):

| primitive | Reka parts | what the rule does |
| --- | --- | --- |
| Button, Card (as button), Chip | `Primitive` | nothing: native buttons, click on confirm |
| Tabs | TabsRoot/List/Trigger/Content, `loop: false`, activation manual | rule 1 (axis) + 2 (Enter); L1/R1 via the screen's `tab` |
| Segmented | ToggleGroup `type="single"`, `orientation` set, `loop: false`; a chosen item cannot be clicked off (the wrapper ignores the empty value) | rule 1; A is a click |
| Switch | SwitchRoot/Thumb | rule 2 |
| Slider | SliderRoot/Track/Range/Thumb; `v-model` is one number, `commit` when a change ends | rule 1, no edge |
| Select | SelectRoot/Trigger/Value/Content/Item, `position: popper` | rule 2 on the trigger and options, rule 1 both axes in the list, rule 3 |
| Dialog | DialogRoot/Portal/Overlay/Content/Title/Description/Close | rule 3; focus starts on `data-nav-initial`, returns to the trigger |

The rule serves all nine primitives; none is hand-made. (Had one not been servable, the
implementation would have stopped and reported it: the alternative is a single all-our-own
standard, the owner's decision.)

## Decision 7: primitives dressed in today's look

Values copied from the screens, written with the tokens (variants are colours, sizes are sizes):

- **Button** `ButtonVariant` Primary (`.btn-primary`: accent dome, `on-accent`, semibold, glow-sm,
  hover flat `accent-hover`) · Soft (`.btn`: `surface-raised`, `line` edge, medium, hover
  `surface-hover`) · Ghost / GhostAccent (SettingRow `.ghost`) · Danger (QuickSaveDialog);
  `ButtonSize` Small 28 · Default 32 · Large 40 · Icon / IconSmall. Disabled `opacity-disabled`.
- **Card** Surface (Details `.card`: `surface`, `line`, radius-10, p-16) · Raised (SettingRow choice
  card, `selected` = accent-soft + accent-text edge) · Glass (`glass-thick`).
- **Chip** Filter (LibraryView: 32, chosen on accent-soft with `accent-edge`) · Option (SettingRow:
  28, chosen edge `accent-text`). **Cowork**: the two chosen edges differ today.
- **Tabs** (GameDetails / ControlsView: ink-muted, active ink with `shadow-tab`), **Segmented**
  (SettingRow `.segments`), **Switch** (36×20, knob 16, on = `bg-accent-track` + glow-sm),
  **Slider** (4px track filled in `accent-text`, 16px `knob` thumb), **Select** (SettingSelect field
  and glass menu), **Dialog** (FlowDialog: `scrim`, glass-thick, max 520, p-24, title 18/24).
- No strings inside primitives; an accessible label a primitive needs (a close control) goes through
  `t()` in both message files.

## Decision 8: the Kit page

`src/kit/KitPage.vue` with one `sections/<Primitive>Section.vue` each, mounted by `main.ts` only
when `import.meta.env.DEV` and the hash is `#kit` (dropped from the production build: measured).
It is a nav scope like any screen, so each primitive's controller behaviour can be tried by hand.

- **Every primitive × every state × the three themes.** A theme switch (Scuro / Nero / Chiaro) at
  the top sets `data-theme`; each section lays its states side by side: Button (each variant ×
  size, disabled, with icon), Card (each variant, selected / not, as button), Chip
  (each variant, selected / not, disabled, as span), Tabs (both sizes, a disabled tab), Segmented
  (chosen, a disabled item, all disabled), Switch (on / off × enabled / disabled), Slider (min, mid,
  max, disabled), Select (closed, open, disabled), Dialog (closed trigger, open), plus the accent
  swatches of `general.accent` applied live, to see the run-time tokens on every primitive.
- Headless screenshots of the Kit in the three themes, with another accent, and with Select and
  Dialog open, are part of the cycle's evidence. The focus ring is the global one (`styles.css`)
  and shows while moving with the arrows or the controller. Outside Tauri the page needs the bridge mocked (`main.ts` imports App.vue,
  whose bridge reads `__TAURI_INTERNALS__` at load).
- `scripts/check` exempts `app/src/kit/` from the visible-text gate only (the labels name states
  for whoever builds them; the page never ships).

## Decision 9: the scanner, for new code

`scripts/scan-conventions.mjs`, run by `scripts/check`, with `--self-test` first (each rule must
fire on its fixture in `scripts/scan-conventions/`, every run). Rules: `literal-colour`
(`bg-[#…]`, `[rgb(`…), `alpha-modifier` (`bg-ink/50`), `dark-variant`, `literal-variant`
(`variant="…"` / `size="…"` on a component), `raw-control` (`<button>` / `<input>` outside
`components/ui`), `style-block`, `utility-collision` (Decision 3, through Tailwind's compiler).
The screens of today are listed in `LEGACY` and reported as counts, not failures (measured: 35
`<style>` blocks, 173 raw controls); cycle 2 removes a file from the list when it migrates it,
cycle 3 deletes the list. Each rule's blind spots are written beside it.

## Dependencies and licences

`reka-ui`, `class-variance-authority` (Apache-2.0), `clsx`, `tailwind-merge` ship; `tailwindcss`,
`@tailwindcss/vite`, `vitest`, `@vue/test-utils`, `happy-dom` are dev tools (all MIT).
`reka-ui` ships `tslib` (**0BSD**) through `aria-hidden`. 0BSD has no conditions at all; the owner
allowed it on 2026-11-18 and `scripts/npm-licenses.mjs` lists it.

## Handed back to design (Cowork)

1. The Decision 5 rows marked **Cowork**: 14px, 24/26px text, the 36px control, 44px tabs, radii
   8/12/20, spacing 10/14/18, the new shadow names, blur 8/2, 600ms.
2. The DS has no `knob` (white thumb), no `accent-edge`, no slider-thumb shadow: added from the
   screens' values, to be named or folded.
3. Two chosen-chip edges (filter: soft accent; option: accent-text).
4. `.btn-primary` is 14px on a 18px line in the DS's 13px `label` world.

## Verified at the end of cycle 1 (2026-11-18)

- The 11 screens of Decision 3, before (develop 8ca989f) and after, pixel by pixel: **0 different
  pixels** in all 11.
- `scripts/headless/kit-nav.mjs`: 24 checks, all passing.
- `pnpm typecheck`, `lint`, `format:check`, `test` (37 tests), `build` (no Kit module in `dist`);
  scanner self-test and scan; licence check; visible-text gate.
