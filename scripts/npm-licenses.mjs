// The npm half of the licence check in scripts/check: every package installed in app/ must
// carry a licence compatible with GPL-3.0-or-later. Run from the repo root:
// `node scripts/npm-licenses.mjs`. Exit 0 = every package allowed, 1 = something is not
// (each offender is printed), 2 = the list could not be read (e.g. no `pnpm install` yet).
//
// It reads `pnpm licenses list --json` (every dependency, dev tools included) and
// `pnpm licenses list --json --prod` (what ships in the interface bundle). A licence in ALLOWED
// is fine anywhere; one in DEV_ONLY is accepted only for tools that never ship (the build and
// lint chain) and fails if a shipped package uses it. SPDX expressions are evaluated: `A OR B`
// needs one allowed side, `A AND B` both, `A WITH exception` is matched as a whole.
//
// Proved failing: with "BSD-2-Clause" removed from ALLOWED it printed
// `entities@7.0.1 (prod): BSD-2-Clause` and exited 1 (2026-11-18).
//
// Blind spots: pnpm reports the `license` field each package declares, not what its files say;
// it lists only the packages installed on this machine, so optional native packages for other
// platforms (e.g. lightningcss-linux-*) are not seen; packages bundled inside another package's
// dist/ are invisible. Assets that are not npm packages (fonts, icons, cores) are in
// THIRD_PARTY.md and reviewed by hand.

import { execSync } from 'node:child_process'

// Compatible with GPL-3.0-or-later (FSF license list) and needed by the current graph.
const ALLOWED = new Set([
  'MIT',
  'ISC',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'MPL-2.0',
])

// Permissive, but with no FSF verdict on GPL compatibility: accepted only for dev tools.
const DEV_ONLY = new Set(['BlueOak-1.0.0'])

const listLicences = (extra) =>
  JSON.parse(
    execSync(`pnpm licenses list --json${extra}`, {
      cwd: 'app',
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'inherit'],
    }),
  )

// { licence: [{ name, versions }] } -> [{ id: 'name@v', licence }]
const flatten = (byLicence) =>
  Object.entries(byLicence).flatMap(([licence, packages]) =>
    packages.flatMap((p) => p.versions.map((v) => ({ id: `${p.name}@${v}`, licence }))),
  )

// SPDX expression -> tokens: '(', ')', 'AND', 'OR', or a licence id (with its WITH clause).
const tokenize = (expression) =>
  expression
    .replace(/[()]/g, ' $& ')
    .trim()
    .split(/\s+/)
    .reduce((tokens, word) => {
      const last = tokens[tokens.length - 1]
      if (word === 'WITH' || (last && last.endsWith(' WITH'))) {
        return [...tokens.slice(0, -1), `${last} ${word}`]
      }
      return [...tokens, word]
    }, [])

// Recursive descent: or := and ('OR' and)*, and := atom ('AND' atom)*, atom := id | '(' or ')'.
const parseOr = (tokens, allowed) => {
  const [first, rest] = parseAnd(tokens, allowed)
  if (rest[0] !== 'OR') return [first, rest]
  const [next, after] = parseOr(rest.slice(1), allowed)
  return [first || next, after]
}

const parseAnd = (tokens, allowed) => {
  const [first, rest] = parseAtom(tokens, allowed)
  if (rest[0] !== 'AND') return [first, rest]
  const [next, after] = parseAnd(rest.slice(1), allowed)
  return [first && next, after]
}

const parseAtom = (tokens, allowed) => {
  if (tokens[0] !== '(') return [allowed.has(tokens[0]), tokens.slice(1)]
  const [value, rest] = parseOr(tokens.slice(1), allowed)
  return [value, rest.slice(1)]
}

const satisfies = (expression, allowed) => {
  const [value, rest] = parseOr(tokenize(expression), allowed)
  return value && rest.length === 0
}

const everyAllowed = new Set([...ALLOWED, ...DEV_ONLY])

const readLists = () => {
  try {
    return { all: flatten(listLicences('')), prod: flatten(listLicences(' --prod')) }
  } catch (error) {
    console.error(`cannot read the npm licences (${error.message.split('\n')[0]}).`)
    console.error('Run `pnpm install` in app/ first.')
    process.exit(2)
  }
}

const { all, prod } = readLists()
const prodIds = new Set(prod.map((p) => p.id))

const offenders = [
  ...prod.filter((p) => !satisfies(p.licence, ALLOWED)).map((p) => ({ ...p, where: 'prod' })),
  ...all
    .filter((p) => !prodIds.has(p.id) && !satisfies(p.licence, everyAllowed))
    .map((p) => ({ ...p, where: 'dev' })),
]

if (offenders.length > 0) {
  offenders.forEach((p) => console.log(`${p.id} (${p.where}): ${p.licence}`))
  console.log(
    'Licence not in the allow-list of scripts/npm-licenses.mjs. Replace the package, or, if the ' +
      'licence is GPL-3.0-compatible, add it to the list and to THIRD_PARTY.md.',
  )
  process.exit(1)
}
console.log(`${all.length} packages (${prod.length} shipped): every licence allowed`)
