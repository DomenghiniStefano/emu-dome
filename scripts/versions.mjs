// Fails when EmuDome's version is not the same everywhere it is written (Trello #83). The one source
// is `[workspace.package] version` in the root Cargo.toml; two files repeat it and must agree:
// - app/package.json `version`: the frontend package (pnpm, the licence scripts);
// - app/src-tauri/tauri.conf.json `version`: kept although Tauri falls back to Cargo's version for
//   the runtime (`PackageInfo`) and the Windows resource, because the Tauri CLI writes the Android
//   versionName / versionCode only from this field (tauri-cli `generate_tauri_properties`; without it
//   Gradle falls back to "1.0" and code 1, app/src-tauri/gen/android/app/build.gradle.kts).
// It also asks every workspace member to take the version from the workspace (`version.workspace =
// true`), so the binaries carry that one number.
//
// Usage: node scripts/versions.mjs [--self-test]   (--self-test first feeds the rules broken inputs)
//
// Cannot see: a version written somewhere else (an installer script, a doc), the spike in
// crates/hw-render-spike (its own workspace, never shipped), or a TOML layout it does not parse
// (it reads `version = "..."` lines inside the `[workspace.package]` table, and the `[package]`
// table of each member, written as plain lines).

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('..', import.meta.url))
const read = (path) => readFileSync(root + path, 'utf8')

// The lines of one `[table]` of a TOML file, up to the next table header.
function tableLines(text, name) {
  const lines = text.split(/\r?\n/)
  const start = lines.findIndex((l) => l.trim() === `[${name}]`)
  if (start === -1) return null
  const rest = lines.slice(start + 1)
  const end = rest.findIndex((l) => /^\s*\[/.test(l))
  return end === -1 ? rest : rest.slice(0, end)
}

function workspaceVersion(cargoToml) {
  const lines = tableLines(cargoToml, 'workspace.package') ?? []
  const m = lines.map((l) => /^\s*version\s*=\s*"([^"]+)"/.exec(l)).find(Boolean)
  return m ? m[1] : null
}

function workspaceMembers(cargoToml) {
  const m = /^\s*members\s*=\s*\[([^\]]*)\]/m.exec(cargoToml)
  return m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : []
}

function inheritsVersion(memberToml) {
  const lines = tableLines(memberToml, 'package') ?? []
  return lines.some((l) => /^\s*version\s*\.\s*workspace\s*=\s*true\b/.test(l))
}

// The problems found, as sentences; empty when everything agrees.
function problems({ cargoToml, members, packageJson, tauriConf }) {
  const found = []
  const version = workspaceVersion(cargoToml)
  if (!version) {
    found.push('Cargo.toml: no version in [workspace.package]')
    return found
  }
  const pkg = JSON.parse(packageJson).version
  if (pkg !== version) {
    found.push(`app/package.json version is ${JSON.stringify(pkg)}, the workspace's is "${version}"`)
  }
  const conf = JSON.parse(tauriConf).version
  if (conf !== version) {
    found.push(`app/src-tauri/tauri.conf.json version is ${JSON.stringify(conf)}, the workspace's is "${version}"`)
  }
  for (const [path, text] of Object.entries(members)) {
    if (!inheritsVersion(text)) found.push(`${path}: [package] must say version.workspace = true`)
  }
  return found
}

function selfTest() {
  const cargoToml = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "1.2.3"\n'
  const good = {
    cargoToml,
    members: { 'a/Cargo.toml': '[package]\nname = "a"\nversion.workspace = true\n' },
    packageJson: '{"version": "1.2.3"}',
    tauriConf: '{"version": "1.2.3"}',
  }
  const cases = [
    ['all equal', good, 0],
    ['package.json differs', { ...good, packageJson: '{"version": "1.2.4"}' }, 1],
    ['tauri.conf.json differs', { ...good, tauriConf: '{"version": "1.2.4"}' }, 1],
    ['tauri.conf.json without version', { ...good, tauriConf: '{}' }, 1],
    ['member with its own version', { ...good, members: { 'a/Cargo.toml': '[package]\nversion = "1.2.3"\n' } }, 1],
    ['no workspace version', { ...good, cargoToml: '[workspace]\nmembers = []\n' }, 1],
  ]
  const wrong = cases.filter(([, input, expected]) => problems(input).length !== expected).map(([name]) => name)
  if (wrong.length > 0) {
    console.error(`versions self-test failed: ${wrong.join(', ')}`)
    return false
  }
  return true
}

function main() {
  if (process.argv.includes('--self-test') && !selfTest()) return 1
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
    console.error('Write the version once in [workspace.package] of Cargo.toml and copy it into the files above.')
    return 1
  }
  console.log(`version ${workspaceVersion(cargoToml)} everywhere (${Object.keys(members).length} crates)`)
  return 0
}

process.exitCode = main()
