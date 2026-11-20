// Fails when the running Node is outside `engines.node` in app/package.json, the range the
// frontend tools in the lockfile need (Vite, Vitest, ESLint), or when devEngines.runtime.version
// says something else. scripts/check runs it first and stops there: on a wrong Node every frontend
// step would fail later with errors that do not say why.
//
// Usage: node scripts/node-version.mjs [--self-test]
//
// Written for old Nodes too (it must run on the Node it rejects): no dependencies, no recent syntax.
// It understands the range forms the file uses: alternatives joined by `||`, each one `^x.y.z`
// or `>=x.y.z` (missing minor / patch count as 0). Any other form is an error, not a pass.

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

function parseVersion(text) {
  const m = /^v?(\d+)(?:\.(\d+))?(?:\.(\d+))?$/.exec(text.trim())
  if (!m) return null
  return [Number(m[1]), Number(m[2] || 0), Number(m[3] || 0)]
}

function compare(a, b) {
  const i = a.findIndex((part, k) => part !== b[k])
  if (i === -1) return 0
  return a[i] < b[i] ? -1 : 1
}

// Whether `version` ([major, minor, patch]) satisfies one alternative; throws on a form it does not know.
function satisfiesOne(version, alternative) {
  const text = alternative.trim()
  if (text.startsWith('>=')) {
    const min = parseVersion(text.slice(2))
    if (min) return compare(version, min) >= 0
  } else if (text.startsWith('^')) {
    const min = parseVersion(text.slice(1))
    if (min) return version[0] === min[0] && compare(version, min) >= 0
  }
  throw new Error(`unsupported range "${text}" (use ^x.y.z or >=x.y.z joined by ||)`)
}

function satisfies(versionText, range) {
  const version = parseVersion(versionText)
  if (!version) throw new Error(`not a version: "${versionText}"`)
  return range.split('||').some((alternative) => satisfiesOne(version, alternative))
}

function throws(fn) {
  try {
    fn()
    return false
  } catch {
    return true
  }
}

function selfTest() {
  const range = '^22.13.0 || ^24.0.0 || >=26.0.0'
  const cases = [
    ['v18.17.1', false],
    ['v22.12.0', false],
    ['v22.13.0', true],
    ['v23.1.0', false],
    ['v24.7.0', true],
    ['v25.0.0', false],
    ['v26.1.0', true],
  ]
  const wrong = cases.filter(([v, expected]) => satisfies(v, range) !== expected)
  const refused = throws(() => satisfies('v24.0.0', '~24.1.0'))
  if (wrong.length > 0 || !refused) {
    console.error(`node-version self-test failed: ${JSON.stringify(wrong)}, unknown form refused: ${refused}`)
    return false
  }
  return true
}

function main() {
  if (process.argv.includes('--self-test') && !selfTest()) return 1
  const pkgPath = fileURLToPath(new URL('../app/package.json', import.meta.url))
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
  const range = pkg.engines?.node
  if (!range) {
    console.error('app/package.json has no engines.node')
    return 1
  }
  // pnpm reads devEngines.runtime, npm and nvm-aware tools engines.node: one range, written twice.
  if (pkg.devEngines?.runtime?.version !== range) {
    console.error('app/package.json: devEngines.runtime.version must be the same range as engines.node')
    return 1
  }
  if (satisfies(process.version, range)) {
    console.log(`node ${process.version} (needs ${range})`)
    return 0
  }
  console.error(
    `Node ${process.version} is not supported: the frontend tools need ${range} ` +
      '(app/package.json engines.node). Switch Node (nvm use, .nvmrc at the repo root) and run the check again.',
  )
  return 1
}

process.exitCode = main()
