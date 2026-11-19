import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { extendTailwindMerge } from 'tailwind-merge'
import colors from '@/assets/theme/colors.css?raw'
import layers from '@/assets/theme/layers.css?raw'
import motion from '@/assets/theme/motion.css?raw'
import opacity from '@/assets/theme/opacity.css?raw'
import border from '@/assets/theme/border.css?raw'
import radius from '@/assets/theme/radius.css?raw'
import shadow from '@/assets/theme/shadow.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys, utilityKeys } from '@/lib/design/themeKeys'

// shadcn's cn() is twMerge(clsx(...)) with no configuration, and with our tokens that drops
// classes silently: tailwind-merge only knows t-shirt font sizes, reads `text-label` as a colour,
// and cn('text-label', 'text-ink') keeps only the second, so a button would lose its type the
// moment a caller passes a colour. The names come from the theme files themselves, so a token is
// still declared in one place. tailwind-merge has no theme key for durations, opacity,
// z-index, edge widths or outline offsets: those extend their class groups instead.
const merge = extendTailwindMerge({
  extend: {
    theme: {
      color: themeKeys(colors, ThemeNamespace.Color),
      text: themeKeys(typography, ThemeNamespace.Text),
      font: themeKeys(typography, ThemeNamespace.Font),
      'font-weight': themeKeys(typography, ThemeNamespace.FontWeight),
      spacing: themeKeys(spacing, ThemeNamespace.Spacing),
      radius: themeKeys(radius, ThemeNamespace.Radius),
      leading: themeKeys(typography, ThemeNamespace.Leading),
      tracking: themeKeys(typography, ThemeNamespace.Tracking),
      shadow: themeKeys(shadow, ThemeNamespace.Shadow),
      blur: themeKeys(shadow, ThemeNamespace.Blur),
      ease: themeKeys(motion, ThemeNamespace.Ease),
    },
    classGroups: {
      duration: [{ duration: themeKeys(motion, ThemeNamespace.TransitionDuration) }],
      opacity: [{ opacity: themeKeys(opacity, ThemeNamespace.Opacity) }],
      z: [{ z: utilityKeys(layers, 'z') }],
      'border-w': [{ border: themeKeys(border, ThemeNamespace.BorderWidth) }],
      'outline-w': [{ outline: themeKeys(border, ThemeNamespace.OutlineWidth) }],
      'outline-offset': [{ 'outline-offset': themeKeys(border, ThemeNamespace.OutlineOffset) }],
    },
  },
})

export const cn = (...inputs: ClassValue[]) => merge(clsx(inputs))
