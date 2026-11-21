// The generated part of THIRD_PARTY.md (the crate and npm tables), from the licence each package
// declares: `cargo metadata` for the crates, `pnpm licenses list --json` for the npm packages (app/,
// after `pnpm install`). Run from anywhere in the repo:
//   node scripts/third-party-tables.mjs           prints the tables to stdout
//   node scripts/third-party-tables.mjs --write   writes them into THIRD_PARTY.md
//   node scripts/third-party-tables.mjs --check   fails when THIRD_PARTY.md differs (scripts/check)
// The tables are the end of THIRD_PARTY.md: from the line `### Rust crates (` to the end of the file.
// Everything above it is written by hand and left alone.
//
// It only lists; the check that fails on a licence is scripts/check (cargo-deny and
// scripts/npm-licenses.mjs).
//
// What --check cannot see: the npm table comes from what is installed in app/node_modules, so a stale
// install (pnpm install not run after a lockfile change) compares the old packages; the crates come
// from Cargo.lock (`--locked`), the root workspace only (not crates/hw-render-spike, never shipped);
// the licence is the one each package declares, not its files; the hand-written sections above the
// tables (cores, fonts, icons, Android) are not compared with anything.

import { execSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('..', import.meta.url))
const docPath = root + 'THIRD_PARTY.md'
const marker = '### Rust crates ('

const run = (command, cwd = '') =>
  execSync(command, {
    cwd: root + cwd,
    encoding: 'utf8',
    maxBuffer: 256 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'inherit'],
  })

const cell = text => String(text ?? '').replace(/\|/g, '\\|')

const byName = (a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version)

const table = (header, rows) =>
  [`| ${header.join(' | ')} |`, `| ${header.map(() => '---').join(' | ')} |`, ...rows].join('\n')

function generate() {
  const crates = JSON.parse(run('cargo metadata --format-version 1 --locked'))
    .packages.filter(p => p.source !== null)
    .map(p => ({ name: p.name, version: p.version, licence: p.license ?? p.license_file }))
    .sort(byName)

  const npmList = extra => JSON.parse(run(`pnpm licenses list --json${extra}`, 'app'))

  const shipped = new Set(Object.values(npmList(' --prod')).flatMap(ps => ps.map(p => p.name)))

  const npm = Object.entries(npmList(''))
    .flatMap(([licence, ps]) =>
      ps.map(p => ({ name: p.name, version: p.versions.join(', '), licence })),
    )
    .sort(byName)

  return [
    `${marker}${crates.length})\n`,
    table(
      ['Crate', 'Version', 'Licence'],
      crates.map(c => `| ${cell(c.name)} | ${cell(c.version)} | ${cell(c.licence)} |`),
    ),
    `\n### npm packages (${npm.length}, ${shipped.size} shipped)\n`,
    table(
      ['Package', 'Version', 'Licence', 'Ships in the app'],
      npm.map(
        p =>
          `| ${cell(p.name)} | ${cell(p.version)} | ${cell(p.licence)} | ${shipped.has(p.name) ? 'yes' : 'no (dev tool)'} |`,
      ),
    ),
  ].join('\n')
}

// THIRD_PARTY.md split into the hand-written part and the generated tables; null without the marker.
function splitDoc() {
  const text = readFileSync(docPath, 'utf8').replace(/\r\n/g, '\n')
  const at = text.indexOf(`\n${marker}`)
  if (at === -1) return null
  return { head: text.slice(0, at + 1), tables: text.slice(at + 1).trimEnd() }
}

function main() {
  const mode = process.argv[2] ?? ''
  if (!['', '--check', '--write'].includes(mode)) {
    console.error('usage: node scripts/third-party-tables.mjs [--check | --write]')
    return 2
  }
  const generated = generate()
  if (mode === '') {
    console.log(generated)
    return 0
  }
  const doc = splitDoc()
  if (!doc) {
    console.error(`THIRD_PARTY.md has no line starting "${marker}": the generated tables end the file`)
    return 1
  }
  if (mode === '--write') {
    writeFileSync(docPath, `${doc.head}${generated}\n`)
    console.log('THIRD_PARTY.md: generated tables written')
    return 0
  }
  if (doc.tables === generated) {
    console.log('THIRD_PARTY.md: generated tables up to date')
    return 0
  }
  const have = doc.tables.split('\n')
  const want = generated.split('\n')
  const shown = []
  for (let i = 0; i < Math.max(have.length, want.length) && shown.length < 20; i++) {
    if (have[i] === want[i]) continue
    shown.push(`  line ${i + 1} of the tables`)
    if (have[i] !== undefined) shown.push(`    THIRD_PARTY.md: ${have[i]}`)
    if (want[i] !== undefined) shown.push(`    generated:      ${want[i]}`)
  }
  console.error(`THIRD_PARTY.md: the generated tables differ (first differences):\n${shown.join('\n')}`)
  console.error(
    'Write them again with: node scripts/third-party-tables.mjs --write (after pnpm install in app/), ' +
      'then review the diff.',
  )
  return 1
}

process.exitCode = main()
