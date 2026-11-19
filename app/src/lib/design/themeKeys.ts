// The custom names a Tailwind namespace declares in a theme file, read from the CSS itself, so
// `cn()` (lib/cn.ts) learns the tokens from the one place they are declared instead of a
// TypeScript copy that could drift.

// The Tailwind namespaces whose names `cn()` has to know. A namespace is the prefix of a theme
// variable: `--text-body` belongs to `text`.
export const ThemeNamespace = {
  Text: 'text',
  Leading: 'leading',
  Tracking: 'tracking',
  Font: 'font',
  FontWeight: 'font-weight',
  Spacing: 'spacing',
  Radius: 'radius',
  BorderWidth: 'border-width',
  OutlineWidth: 'outline-width',
  OutlineOffset: 'outline-offset',
  Shadow: 'shadow',
  Blur: 'blur',
  Ease: 'ease',
  TransitionDuration: 'transition-duration',
  Opacity: 'opacity',
  Color: 'color',
} as const
export type ThemeNamespace = (typeof ThemeNamespace)[keyof typeof ThemeNamespace]

// `--<namespace>-<name>:` at a declaration. The name stops before a `--` sub-property, so
// `--text-body--line-height` still names `body`; the `--text-*: initial` reset has no name and
// does not match; `var(--text-body)` is not followed by a colon and does not either. The `(?<![\w-])`
// keeps `--default-transition-duration` out of `transition-duration`, and the name must not run
// into a longer namespace (`--font-weight-medium` is not the font `weight-medium`).
const declaration = (namespace: ThemeNamespace) =>
  new RegExp(`(?<![\\w-])--${namespace}-([a-z0-9]+(?:-[a-z0-9]+)*)(?:--[a-z0-9-]+)?\\s*:`, 'g')

const unique = (names: string[]) => [...new Set(names)]

/** The names declared in `namespace` by a theme file's source, in order, once each. */
export const themeKeys = (css: string, namespace: ThemeNamespace): string[] => {
  const longer = Object.values(ThemeNamespace).filter(
    other => other !== namespace && other.startsWith(`${namespace}-`),
  )
  return unique(
    [...css.matchAll(declaration(namespace))]
      .map(([, name]) => name ?? '')
      .filter(name => !longer.some(other => `${namespace}-${name}`.startsWith(`${other}-`))),
  )
}

/** `@utility <prefix>-<name> {`: a token where Tailwind has no theme namespace (layers.css). */
export const utilityKeys = (css: string, prefix: string): string[] =>
  unique(
    [...css.matchAll(new RegExp(`@utility ${prefix}-([a-z0-9-]+)\\s*\\{`, 'g'))].map(
      ([, name]) => name ?? '',
    ),
  )
