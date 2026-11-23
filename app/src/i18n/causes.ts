// The codes that tell a file by an `IoCause` (Trello #128), and the causes: each such code has
// one text per cause, `errorCauses.<code>.<cause>` (labels.ts `causeText`; held against both
// message files by parity.test.ts). Both lists are Records over the generated types, so a code
// or a cause Rust adds is a type error here until it is listed.
import type { ErrorCode, IoCause } from '@/emu'

/** An `ErrorCode` that carries a `cause`. */
export type WithCause = Extract<ErrorCode, { params: { cause: IoCause } }>
export type CauseCode = WithCause['code']

export const CAUSE_CODES: Record<CauseCode, true> = {
  romUnreadable: true,
  saveFolderUnavailable: true,
  biosUnreadable: true,
}

export const IO_CAUSES: Record<IoCause, true> = {
  notFound: true,
  permissionDenied: true,
  other: true,
}

/** Whether `e` carries a cause. */
export function hasCause(e: ErrorCode): e is WithCause {
  return e.code in CAUSE_CODES
}
