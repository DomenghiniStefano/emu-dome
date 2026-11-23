// S06's Google Drive row (#112): what it shows for the sync's status, the same rule as the Cloud
// section's «Accedi con Google» (CloudSignedOut): a build without the Google client or a system
// that cannot sign in yet (Android) keeps the button disabled with the reason. Pure, tested.
import type { CloudStatus, ErrorCode } from '@/emu'

export type DriveRow =
  /** The status has not answered yet: the button waits, disabled. */
  | { kind: 'unknown' }
  /** Sign-in is not possible here: the reason, the button disabled. */
  | { kind: 'blocked'; reason: ErrorCode }
  | { kind: 'signedOut' }
  /** The browser is open, waiting for the user. */
  | { kind: 'signingIn' }
  /** Signed in: the account's name or address, null when Google gave neither. */
  | { kind: 'signedIn'; name: string | null }

export function driveRow(status: CloudStatus | null, signingIn: boolean): DriveRow {
  if (!status) return { kind: 'unknown' }
  if (status.signedIn)
    return { kind: 'signedIn', name: status.account?.name ?? status.account?.email ?? null }
  if (!status.configured) return { kind: 'blocked', reason: { code: 'cloudNotConfigured' } }
  if (!status.supported) return { kind: 'blocked', reason: { code: 'cloudSignInUnsupported' } }
  return signingIn ? { kind: 'signingIn' } : { kind: 'signedOut' }
}
