// Prints the generated part of THIRD_PARTY.md (the crate and npm tables) to stdout, from the
// licence each package declares: `cargo metadata` for the crates, `pnpm licenses list --json`
// for the npm packages (app/, after `pnpm install`). Run from the repo root and paste the output
// over the "Generated tables" section:
//   node scripts/third-party-tables.mjs > tables.md
// It only reports; the check that fails on a licence is scripts/check (cargo-deny and
// scripts/npm-licenses.mjs).

import { execSync } from 'node:child_process'

const run = (command, cwd = '.') =>
  execSync(command, {
    cwd,
    encoding: 'utf8',
    maxBuffer: 256 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'inherit'],
  })

const cell = (text) => String(text ?? '').replace(/\|/g, '\\|')

const byName = (a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version)

const crates = JSON.parse(run('cargo metadata --format-version 1 --locked'))
  .packages.filter((p) => p.source !== null)
  .map((p) => ({ name: p.name, version: p.version, licence: p.license ?? p.license_file }))
  .sort(byName)

const npmList = (extra) => JSON.parse(run(`pnpm licenses list --json${extra}`, 'app'))

const shipped = new Set(
  Object.values(npmList(' --prod')).flatMap((ps) => ps.map((p) => p.name)),
)

const npm = Object.entries(npmList(''))
  .flatMap(([licence, ps]) =>
    ps.map((p) => ({ name: p.name, version: p.versions.join(', '), licence })),
  )
  .sort(byName)

const table = (header, rows) =>
  [`| ${header.join(' | ')} |`, `| ${header.map(() => '---').join(' | ')} |`, ...rows].join('\n')

console.log(`### Rust crates (${crates.length})\n`)
console.log(
  table(
    ['Crate', 'Version', 'Licence'],
    crates.map((c) => `| ${cell(c.name)} | ${cell(c.version)} | ${cell(c.licence)} |`),
  ),
)
console.log(`\n### npm packages (${npm.length}, ${shipped.size} shipped)\n`)
console.log(
  table(
    ['Package', 'Version', 'Licence', 'Ships in the app'],
    npm.map(
      (p) =>
        `| ${cell(p.name)} | ${cell(p.version)} | ${cell(p.licence)} | ${shipped.has(p.name) ? 'yes' : 'no (dev tool)'} |`,
    ),
  ),
)
