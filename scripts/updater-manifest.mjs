// The update manifest of a release (Trello #44): the `latest.json` the installed apps read
// (emudome-platform/src/updater.rs, `DEFAULT_ENDPOINT`: the newest GitHub release that is not a
// pre-release), and the check that a release build has its signing keys. docs/release.md says
// when each runs.
//
//   node scripts/updater-manifest.mjs check-env
//       Run by app/src-tauri/package/updater.json before the release build: the private key
//       (TAURI_SIGNING_PRIVATE_KEY, its content or a path) and the public key the app is built with
//       (EMUDOME_UPDATER_PUBKEY) are set, and a key file is outside the repository (licence rule 6:
//       no key in the repository). Prints no value.
//   node scripts/updater-manifest.mjs write --version V --out FILE [--notes TEXT] <platform>=<file>...
//       Writes the manifest for release v<V>: for each platform (windows-x86_64, linux-x86_64) the
//       package the updater installs (the NSIS setup .exe, the .AppImage), its signature (the
//       `.sig` file Tauri writes beside it) and its download address on the release
//       (https://github.com/DomenghiniStefano/emu-dome/releases/download/v<V>/<file name>).
//       Refuses a version that is not the one in app/src-tauri/tauri.conf.json, or a missing
//       signature.
//   node scripts/updater-manifest.mjs --self-test
//       Each rule on a broken input first.
//
// Blind spots: it cannot see whether the files were uploaded to the release under these names,
// nor whether the signature matches the public key the app was built with (the app checks that
// before installing, and refuses a mismatch).

import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const TAURI_CONF = path.join(ROOT, 'app', 'src-tauri', 'tauri.conf.json')
const RELEASES = 'https://github.com/DomenghiniStefano/emu-dome/releases/download'
const PLATFORMS = ['windows-x86_64', 'linux-x86_64']

class Failure extends Error {}
const fail = message => {
  throw new Failure(message)
}

/** The problems of the release build's environment, as sentences; empty when it is ready. */
export function envProblems(env, root = ROOT) {
  const problems = []
  const key = env.TAURI_SIGNING_PRIVATE_KEY ?? ''
  if (key.trim() === '')
    problems.push('TAURI_SIGNING_PRIVATE_KEY is not set (the private key, or the path of its file)')
  else if (!key.includes('\n') && fs.existsSync(key)) {
    const rel = path.relative(root, path.resolve(key))
    if (!rel.startsWith('..') && !path.isAbsolute(rel))
      problems.push('TAURI_SIGNING_PRIVATE_KEY names a file inside the repository: keep it outside')
  }
  if ((env.EMUDOME_UPDATER_PUBKEY ?? '').trim() === '')
    problems.push('EMUDOME_UPDATER_PUBKEY is not set (the public key: the app checks updates with it)')
  return problems
}

/** The manifest for `version`, from `{ platform: { file, signature } }`. */
export function manifest({ version, notes, date, packages }) {
  const platforms = {}
  for (const [platform, { file, signature }] of Object.entries(packages)) {
    if (!PLATFORMS.includes(platform)) fail(`unknown platform ${platform} (${PLATFORMS.join(', ')})`)
    if (!signature || signature.trim() === '') fail(`${platform}: the signature is empty`)
    platforms[platform] = {
      signature: signature.trim(),
      url: `${RELEASES}/v${version}/${encodeURIComponent(path.basename(file))}`,
    }
  }
  if (Object.keys(platforms).length === 0) fail('no package given (<platform>=<file>)')
  return { version, notes, pub_date: date, platforms }
}

function parseWrite(args) {
  const opts = { packages: {} }
  for (let i = 0; i < args.length; i++) {
    const a = args[i]
    if (a === '--version') opts.version = args[++i]
    else if (a === '--out') opts.out = args[++i]
    else if (a === '--notes') opts.notes = args[++i]
    else if (a.includes('=')) {
      const [platform, file] = [a.slice(0, a.indexOf('=')), a.slice(a.indexOf('=') + 1)]
      opts.packages[platform] = file
    } else fail(`unknown argument ${a}`)
  }
  if (!opts.version) fail('--version is missing')
  if (!opts.out) fail('--out is missing')
  return opts
}

function runWrite(args, confFile = TAURI_CONF) {
  const opts = parseWrite(args)
  const conf = JSON.parse(fs.readFileSync(confFile, 'utf8'))
  if (conf.version !== opts.version)
    fail(`--version ${opts.version} is not tauri.conf.json's ${conf.version}: build the release first`)
  const packages = {}
  for (const [platform, file] of Object.entries(opts.packages)) {
    if (!fs.existsSync(file)) fail(`${platform}: ${file} does not exist`)
    const sig = `${file}.sig`
    if (!fs.existsSync(sig))
      fail(`${platform}: ${sig} does not exist (built without package/updater.json?)`)
    packages[platform] = { file, signature: fs.readFileSync(sig, 'utf8') }
  }
  const out = manifest({
    version: opts.version,
    notes: opts.notes ?? `EmuDome ${opts.version}`,
    date: new Date().toISOString(),
    packages,
  })
  fs.writeFileSync(opts.out, `${JSON.stringify(out, null, 2)}\n`)
  console.log(`wrote ${opts.out}: ${Object.keys(out.platforms).join(', ')}`)
}

function selfTest() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'emudome-updater-manifest-'))
  const caught = []
  const expectFail = (what, f) => {
    try {
      f()
    } catch (e) {
      if (e instanceof Failure) {
        caught.push(what)
        return
      }
      throw e
    }
    throw new Error(`self-test: ${what} was not refused`)
  }
  try {
    // Environment.
    const inside = path.join(ROOT, 'LICENSE')
    const ok = { TAURI_SIGNING_PRIVATE_KEY: 'untrusted comment\nKEY', EMUDOME_UPDATER_PUBKEY: 'PUB' }
    if (envProblems(ok).length !== 0) throw new Error('self-test: a ready environment was refused')
    for (const [what, env] of [
      ['no private key', { EMUDOME_UPDATER_PUBKEY: 'PUB' }],
      ['no public key', { TAURI_SIGNING_PRIVATE_KEY: 'KEY\nX' }],
      ['a key file in the repository', { ...ok, TAURI_SIGNING_PRIVATE_KEY: inside }],
    ]) {
      expectFail(what, () => {
        const p = envProblems(env)
        if (p.length > 0) fail(p.join('; '))
      })
    }
    // Manifest.
    const exe = path.join(dir, 'EmuDome_0.0.2_x64-setup.exe')
    fs.writeFileSync(exe, 'installer')
    const conf = path.join(dir, 'tauri.conf.json')
    fs.writeFileSync(conf, '{"version":"0.0.2"}')
    const out = path.join(dir, 'latest.json')
    expectFail('a missing signature', () =>
      runWrite(['--version', '0.0.2', '--out', out, `windows-x86_64=${exe}`], conf),
    )
    fs.writeFileSync(`${exe}.sig`, 'SIG\n')
    expectFail('another version', () =>
      runWrite(['--version', '0.0.3', '--out', out, `windows-x86_64=${exe}`], conf),
    )
    expectFail('an unknown platform', () =>
      runWrite(['--version', '0.0.2', '--out', out, `darwin-aarch64=${exe}`], conf),
    )
    expectFail('no package', () => runWrite(['--version', '0.0.2', '--out', out], conf))
    runWrite(['--version', '0.0.2', '--out', out, `windows-x86_64=${exe}`], conf)
    const written = JSON.parse(fs.readFileSync(out, 'utf8'))
    const win = written.platforms['windows-x86_64']
    if (
      written.version !== '0.0.2' ||
      win.signature !== 'SIG' ||
      win.url !== `${RELEASES}/v0.0.2/EmuDome_0.0.2_x64-setup.exe`
    )
      throw new Error(`self-test: unexpected manifest ${JSON.stringify(written)}`)
  } finally {
    fs.rmSync(dir, { recursive: true, force: true })
  }
  console.log(`self-test: all ${caught.length} broken inputs caught`)
}

const [command, ...rest] = process.argv.slice(2)
try {
  if (command === 'check-env') {
    const problems = envProblems(process.env)
    if (problems.length > 0) fail(`release signing:\n  ${problems.join('\n  ')}\n  (docs/release.md)`)
    console.log('release signing: the private and the public update keys are set.')
  } else if (command === 'write') runWrite(rest)
  else if (command === '--self-test') selfTest()
  else fail('usage: updater-manifest.mjs check-env | write --version V --out FILE [--notes T] <platform>=<file>... | --self-test')
} catch (e) {
  if (!(e instanceof Failure)) throw e
  console.error(`updater-manifest: ${e.message}`)
  process.exit(1)
}
