// EmuDome's version, written in one place only: `[workspace.package] version` in the root Cargo.toml.
// Every reader takes it from there:
// - the crates: `version.workspace = true` in each member, so every binary carries that number;
// - Tauri on the desktop (`PackageInfo`, the bundles, the Windows resource): with no `version` in
//   tauri.conf.json the Tauri CLI and tauri-build fall back to the Cargo package's version;
// - Android: the Tauri CLI writes versionName / versionCode only from the config's `version`
//   (tauri-cli `generate_tauri_properties`; without it Gradle falls back to "1.0" and code 1), so the
//   build scripts pass it with `--config <file>`, the file written by `tauri-config` below. A bare
//   `pnpm tauri android build` gets "1.0" / code 1: never upload that one.
// Cargo.lock repeats it for the workspace crates; cargo writes it, `bump` refreshes it.
//
// Usage:
//   node scripts/versions.mjs [--self-test]   the check (a step of scripts/check): the version is a plain
//                                             x.y.z in Cargo.toml and written nowhere else
//   node scripts/versions.mjs print           the version
//   node scripts/versions.mjs bump [patch|minor|major|x.y.z]
//                                             raises it in Cargo.toml (and Cargo.lock), prints the new one
//                                             (a release goes through scripts/release.mjs, which calls it)
//   node scripts/versions.mjs tauri-config    writes target/emudome-version.tauri.json ({ "version" }) and
//                                             prints its path, for `pnpm tauri ... --config <path>`
//
// Google Play's versionCode is major * 1000000 + minor * 1000 + patch (tauri-cli), hence minor and
// patch stay under 1000. 0.0.x are test builds (Play's internal track), 1.0.0 the first release.
//
// Cannot see: a version written somewhere else (an installer script, a doc), the spike in
// crates/hw-render-spike (its own workspace, never shipped), or a TOML layout it does not parse
// (it reads `version = "..."` lines inside the `[workspace.package]` table, and the `[package]`
// table of each member, written as plain lines).

import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = fileURLToPath(new URL('..', import.meta.url))
const read = (path) => readFileSync(root + path, 'utf8')

const PLAIN = /^(\d+)\.(\d+)\.(\d+)$/

// The lines of one `[table]` of a TOML file, up to the next table header.
function tableLines(text, name) {
  const lines = text.split(/\r?\n/)
  const start = lines.findIndex((l) => l.trim() === `[${name}]`)
  if (start === -1) return null
  const rest = lines.slice(start + 1)
  const end = rest.findIndex((l) => /^\s*\[/.test(l))
  return end === -1 ? rest : rest.slice(0, end)
}

const VERSION_LINE = /^(\s*version\s*=\s*")([^"]+)(".*)$/

function workspaceVersion(cargoToml) {
  const lines = tableLines(cargoToml, 'workspace.package') ?? []
  const m = lines.map((l) => VERSION_LINE.exec(l)).find(Boolean)
  return m ? m[2] : null
}

// Cargo.toml with the workspace version replaced; every other line as it was.
function withWorkspaceVersion(cargoToml, version) {
  const lines = cargoToml.split('\n')
  const start = lines.findIndex((l) => l.trim() === '[workspace.package]')
  const at = lines.findIndex((l, i) => i > start && VERSION_LINE.test(l))
  const header = lines.findIndex((l, i) => i > start && /^\s*\[/.test(l))
  if (start === -1 || at === -1 || (header !== -1 && header < at)) {
    throw new Error('Cargo.toml: no version in [workspace.package]')
  }
  return lines.map((l, i) => (i === at ? l.replace(VERSION_LINE, `$1${version}$3`) : l)).join('\n')
}

const parts = (version) => PLAIN.exec(version).slice(1).map(Number)

// Whether a is a higher version than b (both plain x.y.z).
const higher = (a, b) => {
  const [x, y] = [parts(a), parts(b)]
  const at = x.findIndex((n, i) => n !== y[i])
  return at !== -1 && x[at] > y[at]
}

// The version after `version`: a part raised (patch, minor, major) or a plain x.y.z above it.
function bumped(version, part) {
  if (!PLAIN.test(version)) throw new Error(`the version ${version} is not a plain x.y.z`)
  if (PLAIN.test(part)) {
    if (!higher(part, version)) throw new Error(`${part} is not higher than ${version}`)
    return part
  }
  const [major, minor, patch] = parts(version)
  if (part === 'major') return `${major + 1}.0.0`
  if (part === 'minor') return `${major}.${minor + 1}.0`
  if (part === 'patch') return `${major}.${minor}.${patch + 1}`
  throw new Error(`unknown version ${part} (patch, minor, major or a plain x.y.z)`)
}

function workspaceMembers(cargoToml) {
  const m = /^\s*members\s*=\s*\[([^\]]*)\]/m.exec(cargoToml)
  return m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : []
}

function inheritsVersion(memberToml) {
  const lines = tableLines(memberToml, 'package') ?? []
  return lines.some((l) => /^\s*version\s*\.\s*workspace\s*=\s*true\b/.test(l))
}

// The problems found, as sentences; empty when the version is written once.
function problems({ cargoToml, members, packageJson, tauriConf }) {
  const found = []
  const version = workspaceVersion(cargoToml)
  if (!version) {
    found.push('Cargo.toml: no version in [workspace.package]')
    return found
  }
  const m = PLAIN.exec(version)
  if (!m) {
    found.push(`Cargo.toml: the version ${version} is not a plain x.y.z (Google Play's versionCode is made from it)`)
  } else if (Number(m[2]) > 999 || Number(m[3]) > 999) {
    found.push(`Cargo.toml: the version ${version} has a part over 999 (Google Play's versionCode would collide)`)
  }
  if ('version' in JSON.parse(packageJson)) {
    found.push('app/package.json has a version: the one version is in Cargo.toml')
  }
  if ('version' in JSON.parse(tauriConf)) {
    found.push('app/src-tauri/tauri.conf.json has a version: the one version is in Cargo.toml')
  }
  for (const [path, text] of Object.entries(members)) {
    if (!inheritsVersion(text)) found.push(`${path}: [package] must say version.workspace = true`)
  }
  return found
}

function selfTest() {
  const cargoToml = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "1.2.3"\nedition = "2024"\n'
  const good = {
    cargoToml,
    members: { 'a/Cargo.toml': '[package]\nname = "a"\nversion.workspace = true\n' },
    packageJson: '{"name": "ui"}',
    tauriConf: '{"identifier": "x"}',
  }
  const cases = [
    ['written once', good, 0],
    ['package.json with a version', { ...good, packageJson: '{"version": "1.2.3"}' }, 1],
    ['tauri.conf.json with a version', { ...good, tauriConf: '{"version": "1.2.3"}' }, 1],
    ['member with its own version', { ...good, members: { 'a/Cargo.toml': '[package]\nversion = "1.2.3"\n' } }, 1],
    ['no workspace version', { ...good, cargoToml: '[workspace]\nmembers = []\n' }, 1],
    ['pre-release version', { ...good, cargoToml: cargoToml.replace('1.2.3', '1.2.3-beta') }, 1],
    ['patch over 999', { ...good, cargoToml: cargoToml.replace('1.2.3', '1.2.1000') }, 1],
  ]
  const wrong = cases.filter(([, input, expected]) => problems(input).length !== expected).map(([name]) => name)
  const bumps = [
    ['patch', '1.2.4'],
    ['minor', '1.3.0'],
    ['major', '2.0.0'],
    ['1.10.0', '1.10.0'],
  ]
  for (const [part, expected] of bumps) {
    if (bumped('1.2.3', part) !== expected) wrong.push(`bump ${part}`)
  }
  const refuses = (part) => {
    try {
      bumped('1.2.3', part)
      return false
    } catch {
      return true
    }
  }
  wrong.push(...['1.2.3', '1.2.2', '0.9.9', 'next'].filter((p) => !refuses(p)).map((p) => `bump to ${p} refused`))
  const rewritten = withWorkspaceVersion(cargoToml, '1.2.4')
  if (workspaceVersion(rewritten) !== '1.2.4' || rewritten.replace('1.2.4', '1.2.3') !== cargoToml) {
    wrong.push('rewrite keeps the other lines')
  }
  if (wrong.length > 0) {
    console.error(`versions self-test failed: ${wrong.join(', ')}`)
    return false
  }
  return true
}

function check() {
  const cargoToml = read('Cargo.toml')
  const members = Object.fromEntries(
    workspaceMembers(cargoToml).map((m) => [`${m}/Cargo.toml`, read(`${m}/Cargo.toml`)]),
  )
  const found = problems({
    cargoToml,
    members,
    packageJson: read('app/package.json'),
    tauriConf: read('app/src-tauri/tauri.conf.json'),
  })
  if (found.length > 0) {
    for (const p of found) console.error(p)
    console.error('Write the version once, in [workspace.package] of Cargo.toml (node scripts/versions.mjs bump).')
    return 1
  }
  console.log(`version ${workspaceVersion(cargoToml)}, written once (${Object.keys(members).length} crates inherit it)`)
  return 0
}

// The version for other scripts (the updater manifest, the build scripts).
export function currentVersion() {
  const version = workspaceVersion(read('Cargo.toml'))
  if (!version) throw new Error('Cargo.toml: no version in [workspace.package]')
  return version
}

// The version `raiseVersion(part)` would write (scripts/release.mjs's dry run).
export const nextVersion = (part) => bumped(currentVersion(), part)

// Raises the version in Cargo.toml (and Cargo.lock) and returns the new one.
export function raiseVersion(part) {
  const next = nextVersion(part)
  writeFileSync(root + 'Cargo.toml', withWorkspaceVersion(read('Cargo.toml'), next))
  // Cargo.lock lists the workspace crates with their version: refresh only those, without the network.
  execFileSync('cargo', ['update', '--workspace', '--offline'], { cwd: root, stdio: 'inherit' })
  return next
}

function bump(part) {
  console.log(raiseVersion(part))
  return 0
}

function tauriConfig() {
  const dir = root + 'target'
  mkdirSync(dir, { recursive: true })
  const file = `${dir}/emudome-version.tauri.json`
  writeFileSync(file, `${JSON.stringify({ version: currentVersion() })}\n`)
  console.log(file)
  return 0
}

function main() {
  const [command, arg] = process.argv.slice(2)
  if (command === 'print') {
    console.log(currentVersion())
    return 0
  }
  if (command === 'bump') return bump(arg ?? 'patch')
  if (command === 'tauri-config') return tauriConfig()
  if (command === '--self-test' && !selfTest()) return 1
  if (command === undefined || command === '--self-test') return check()
  console.error('usage: versions.mjs [--self-test] | print | bump [patch|minor|major|x.y.z] | tauri-config')
  return 1
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) process.exitCode = main()
