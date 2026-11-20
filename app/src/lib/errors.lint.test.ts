// @vitest-environment node
// The ESLint rule against a dropped bridge call (app/eslint.config.js, DROPPED_EMU_CALL), seen
// failing on its fixture: each refused line is refused, each allowed one passes.
import { ESLint } from 'eslint'
import { describe, expect, it } from 'vitest'

const eslint = new ESLint()

async function refused(code: string): Promise<number> {
  const [result] = await eslint.lintText(`import { emu } from '@/emu'\n${code}\n`, {
    filePath: 'src/lib/lint-fixture.ts',
  })
  return result!.messages.filter(m => m.message.includes('report() / ignore()')).length
}

describe('no dropped emu call', () => {
  it('refuses `void emu.x()` and a bare `emu.x()` statement, nested commands too', async () => {
    expect(await refused('void emu.setPaused(true)')).toBe(1)
    expect(await refused('void emu.controls.watch(false)')).toBe(1)
    expect(await refused('emu.saveState(1)')).toBe(1)
    expect(await refused('emu.settings.reset("a.b", { level: "global" })')).toBe(1)
  })

  it('allows an awaited call, a handled one and one handed to report', async () => {
    expect(await refused('export async function f() { await emu.setPaused(true) }')).toBe(0)
    expect(await refused('void emu.rewind(true).catch(() => {})')).toBe(0)
    expect(
      await refused('declare function report(p: Promise<unknown>): void\nreport(emu.stats())'),
    ).toBe(0)
    expect(await refused('export const p = emu.stats()')).toBe(0)
  })
})
