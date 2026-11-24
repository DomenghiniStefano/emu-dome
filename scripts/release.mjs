// One command for a release (docs/release.md): raises the version, checks, pushes, and lets the CI build and upload.
//
//   node scripts/release.mjs internal   [--version patch|minor|major|x.y.z] [--dry-run]
//       A test build: Google Play's internal testing track, through `stage` (.github/workflows/stage.yml).
//   node scripts/release.mjs production [--version patch|minor|major|x.y.z] [--dry-run]
//       A release (1.0.0 and later): Google Play's production track after the owner's approval in GitHub Actions,
//       through `main` (.github/workflows/main.yml), which tags the commit v<version>.
//
// Steps: on `develop`, clean, not behind origin/develop; the version raised in Cargo.toml (default: the patch;
// scripts/versions.mjs) and committed as `chore: <version>`; the full check on that commit (the merge rule: the
// same list as ./scripts/check.ps1); then `develop` pushed, and `develop` pushed to `stage` or `main` (a
// fast-forward: nothing is ever committed on those two). --dry-run prints the plan and changes nothing.
// A failed check stops before any push and leaves the version commit on develop, to fix or to drop
// (`git reset --soft HEAD~1`).
//
// Cannot see: whether the CI run then succeeds (it prints where to follow it), or whether Play takes the upload.

import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { currentVersion, nextVersion, raiseVersion } from './versions.mjs'

const root = fileURLToPath(new URL('..', import.meta.url))
const ACTIONS = 'https://github.com/DomenghiniStefano/emu-dome/actions'

const TARGETS = {
  internal: { branch: 'stage', workflow: 'stage.yml', what: "Google Play's internal testing track" },
  production: { branch: 'main', workflow: 'main.yml', what: "Google Play's production, after your approval" },
}

class Stop extends Error {}
const stop = (message) => {
  throw new Stop(message)
}

const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim()
const gitShown = (...args) => execFileSync('git', args, { cwd: root, stdio: 'inherit' })

function parseArgs(args) {
  const [target, ...rest] = args
  if (!(target in TARGETS)) stop('usage: release.mjs internal|production [--version patch|minor|major|x.y.z] [--dry-run]')
  const at = rest.indexOf('--version')
  const version = at === -1 ? 'patch' : rest[at + 1]
  if (!version) stop('--version needs a value: patch, minor, major or x.y.z')
  const unknown = rest.filter((a, i) => a !== '--dry-run' && (at === -1 || (i !== at && i !== at + 1)))
  if (unknown.length > 0) stop(`unknown argument ${unknown.join(' ')}`)
  return { target, version, dryRun: rest.includes('--dry-run') }
}

// Where the working copy stands; every problem as a sentence.
function preflight() {
  git('fetch', '--quiet', 'origin')
  const branch = git('rev-parse', '--abbrev-ref', 'HEAD')
  const dirty = git('status', '--porcelain')
  const behind = Number(git('rev-list', '--count', 'HEAD..origin/develop'))
  return [
    branch !== 'develop' && `you are on ${branch}: a release starts from develop`,
    dirty !== '' && 'the working copy has changes: commit or stash them first',
    behind > 0 && `develop is ${behind} commit(s) behind origin/develop: pull first`,
  ].filter(Boolean)
}

// A production release is 1.0.0 or later: the 0.x versions are test builds.
function versionProblem(target, next) {
  return target === 'production' && Number(next.split('.')[0]) < 1
    ? `${next} is a test version: a production release is 1.0.0 or later (--version 1.0.0)`
    : null
}

function runCheck() {
  const [command, args] =
    process.platform === 'win32'
      ? ['powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'scripts/check.ps1']]
      : ['sh', ['scripts/check']]
  try {
    execFileSync(command, args, { cwd: root, stdio: 'inherit' })
    return true
  } catch {
    return false
  }
}

function plan({ target, version }) {
  const next = nextVersion(version)
  const { branch, what } = TARGETS[target]
  return [
    `version ${currentVersion()} -> ${next}, committed on develop as «chore: ${next}»`,
    'the full check on that commit',
    `push develop, then develop -> ${branch}: the CI builds and uploads to ${what}`,
  ]
}

function release({ target, version, dryRun }) {
  const problems = preflight()
  if (problems.length > 0) stop(problems.join('\n'))
  const refused = versionProblem(target, nextVersion(version))
  if (refused) stop(refused)
  const steps = plan({ target, version })
  steps.forEach((s, i) => console.log(`${i + 1}. ${s}`))
  if (dryRun) {
    console.log('dry run: nothing changed')
    return
  }
  const next = raiseVersion(version)
  gitShown('add', 'Cargo.toml', 'Cargo.lock')
  gitShown('commit', '--quiet', '-m', `chore: ${next}`)
  if (!runCheck()) {
    stop(`the check failed on «chore: ${next}»: nothing was pushed. Fix and run again, or drop the commit (git reset --soft HEAD~1).`)
  }
  const { branch, workflow } = TARGETS[target]
  gitShown('push', '--quiet', 'origin', 'develop')
  gitShown('push', '--quiet', 'origin', `develop:${branch}`)
  console.log(`${next} pushed to ${branch}: follow it at ${ACTIONS}/workflows/${workflow}`)
  if (target === 'production') console.log('approve the deployment there (Review deployments) to publish it.')
}

try {
  release(parseArgs(process.argv.slice(2)))
} catch (e) {
  if (!(e instanceof Stop)) throw e
  console.error(`release: ${e.message}`)
  process.exitCode = 1
}
