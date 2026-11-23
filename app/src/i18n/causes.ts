// The codes that carry a value the interface words (Trello #128), and those values' texts.
//
// - A file error carries an `IoCause`: its text is `errorCauses.<set>.<cause>` (labels.ts
//   `causeText`), where `<set>` is the code itself or a set several codes share (`data`: a file in
//   EmuDome's own data folder, where what to do is the same whatever failed).
// - A failure EmuDome tells apart further carries a `reason` of its own enum: its text is
//   `errorReasons.<code>.<reason>` (`reasonText`).
//
// Every list is a Record over the generated types, so a code, a cause or a reason Rust adds is a
// type error here until it is listed; parity.test.ts holds every text against both message files.
import type { ErrorCode, ImageProblem, IoCause, SignInProblem } from '@/emu'

/** An `ErrorCode` that carries a `cause`. */
export type WithCause = Extract<ErrorCode, { params: { cause: IoCause } }>
export type CauseCode = WithCause['code']

/** The text set of each code with a cause: its own (`errorCauses.<code>`) or a shared one. */
export const CAUSE_CODES: Record<CauseCode, string> = {
  romUnreadable: 'romUnreadable',
  saveFolderUnavailable: 'saveFolderUnavailable',
  biosUnreadable: 'biosUnreadable',
  logExportFailed: 'logExportFailed',
  stateSaveFailed: 'data',
  stateLoadFailed: 'data',
  settingsWriteFailed: 'data',
  settingsReadFailed: 'data',
  libraryWriteFailed: 'data',
  libraryReadFailed: 'data',
  notificationsWriteFailed: 'data',
  notificationsReadFailed: 'data',
  saveFileFailed: 'data',
  screenshotFailed: 'data',
  screenshotsUnreadable: 'data',
  biosFileFailed: 'data',
  cloudIndexReadFailed: 'data',
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

/** The codes whose `reason` has one text per value, with every value of their enum. Not
 * `stateLoadUnsupported`: its reason is the disabled loads' own text (`flow.loadBlock`). */
export const REASON_CODES = {
  coverInvalid: {
    format: true,
    tooLarge: true,
    missing: true,
    other: true,
  } satisfies Record<ImageProblem, true>,
  cloudSignInFailed: {
    browser: true,
    timedOut: true,
    notGranted: true,
    other: true,
  } satisfies Record<SignInProblem, true>,
}

export type ReasonCode = keyof typeof REASON_CODES
/** An `ErrorCode` whose `reason` has its own texts. */
export type WithReason = Extract<ErrorCode, { code: ReasonCode }>

/** Whether `e` carries a reason with texts of its own. */
export function hasReason(e: ErrorCode): e is WithReason {
  return e.code in REASON_CODES
}
