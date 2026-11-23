// A button of a section that leads to another section (#112: Settings' rows of a feature that
// lives elsewhere, C01's «Controlli touch»): the shell provides the move (route.ts `jumpTo`), a
// row or a card asks for it by its `Jump`. Outside the shell (the quick menu, the Kit page, a test)
// there is none: the caller then has nothing to lead to and keeps its button disabled.
import { inject, provide, type InjectionKey } from 'vue'
import type { Jump } from './route'

export type ShellJump = (j: Jump) => void

const JUMP: InjectionKey<ShellJump> = Symbol('shellJump')

/** AppShell: how a jump moves the route. */
export function provideShellJump(jump: ShellJump) {
  provide(JUMP, jump)
}

/** The shell's jump, or null outside the shell. */
export function useShellJump(): ShellJump | null {
  return inject(JUMP, null)
}
