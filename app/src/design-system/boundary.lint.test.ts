// @vitest-environment node
// The design system's boundary (app/eslint.config.js, DESIGN_SYSTEM_BOUNDARY), seen failing on its
// fixture: inside src/design-system an import of the app is refused, its own modules, nav.ts and the
// generated contract pass, and the Kit may import the features.
import { ESLint } from 'eslint'
import { describe, expect, it } from 'vitest'

const eslint = new ESLint()

async function refused(source: string, filePath: string): Promise<number> {
  const [result] = await eslint.lintText(`${source}\nexport { x }\n`, { filePath })
  return result!.messages.filter(m => m.message.includes('The design system imports only')).length
}

const INSIDE = 'src/design-system/lib/lint-fixture.ts'
const KIT = 'src/design-system/kit/lint-fixture.ts'
const FEATURE = 'src/components/library/lint-fixture.ts'

// The first lint loads the TypeScript parser and the whole config: slow on a busy machine.
describe('design system boundary', { timeout: 60_000 }, () => {
  it('refuses an import of the app from inside the design system', async () => {
    expect(await refused("import { x } from '@/emu'", INSIDE)).toBe(1)
    expect(await refused("import { x } from '@/lib/settingsModel'", INSIDE)).toBe(1)
    expect(await refused("import { x } from '@/components/cover/GameCover.vue'", INSIDE)).toBe(1)
    expect(await refused("import { x } from '@/navigation'", INSIDE)).toBe(1)
  })

  it('allows its own modules, nav.ts and the generated contract', async () => {
    expect(await refused("import { x } from '@/design-system/lib/cn'", INSIDE)).toBe(0)
    expect(await refused("import { x } from '@/nav'", INSIDE)).toBe(0)
    expect(await refused("import { x } from '@/generated/app/Theme'", INSIDE)).toBe(0)
    expect(await refused("import { x } from './colour'", INSIDE)).toBe(0)
  })

  it('lets the Kit and the features import anything', async () => {
    expect(await refused("import { x } from '@/components/cover/GameCover.vue'", KIT)).toBe(0)
    expect(await refused("import { x } from '@/emu'", FEATURE)).toBe(0)
  })
})
