// The libretro cores, pinned by cores.lock (repo root): one implementation for every platform.
// The fetch scripts (fetch-cores.ps1, fetch-cores.sh, fetch-cores-android.ps1) call this file, the
// packaging configs (app/src-tauri/package/) call `verify`, and scripts/check calls `check`.
//
//   node scripts/cores.mjs fetch    [--platform P] [--cores a,b] [--lock file]
//       Downloads each core's archive, checks its sha256, takes the listed files out of it, checks
//       theirs, and writes them (temp file + rename). Also the core's licence file at the pinned
//       commit, sha256-checked. A file already in place with the right hash is kept. On any
//       mismatch: the core's files for that platform are deleted and the command fails.
//       Default platform: this machine's (windows-x86_64 / linux-x86_64). Default cores: the ones
//       cores.lock ships (`ship: true`); `--cores all` for every core in the lock.
//   node scripts/cores.mjs verify   --platform P [--cores a,b] [--package]
//       Checks the files on disk against cores.lock, downloads nothing. --package (run by the
//       packaging configs before bundling): the shipped cores, cores/SOURCES.md up to date, and
//       no `url_stable: false` entry unless EMUDOME_ALLOW_UNSTABLE_CORES=1 (a local test package).
//   node scripts/cores.mjs generate
//       Writes the files derived from cores.lock: cores/SOURCES.md (the source offer shipped next
//       to the cores) and app/src-tauri/package/<platform>.json (the desktop bundle lists).
//   node scripts/cores.mjs check [--self-test]
//       Offline, in scripts/check: cores.lock is well formed, the derived files are what
//       `generate` would write, tauri.conf.json lists no resource of its own, the Gradle build
//       reads cores.lock, and THIRD_PARTY.md names every core's commit. --self-test feeds each
//       rule a broken copy first and fails if a rule does not fire.
//       Blind spots: it cannot tell whether a URL still serves the pinned bytes (only `fetch` can),
//       and THIRD_PARTY.md's prose is matched by the commit hash only.
//
// No dependency: Node's fetch, crypto and zlib (zip archives are read here, store and deflate).

import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import zlib from 'node:zlib'

const ROOT = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const LOCK = path.join(ROOT, 'cores.lock')
const SOURCES = path.join(ROOT, 'cores', 'SOURCES.md')
const PACKAGE_DIR = path.join(ROOT, 'app', 'src-tauri', 'package')
const TAURI_CONF = path.join(ROOT, 'app', 'src-tauri', 'tauri.conf.json')
const GRADLE = path.join(ROOT, 'crates', 'emudome-platform', 'android', 'build.gradle.kts')
const THIRD_PARTY = path.join(ROOT, 'THIRD_PARTY.md')
const EMUDOME_REPO = 'https://github.com/DomenghiniStefano/emu-dome'

const DESKTOP = ['windows-x86_64', 'linux-x86_64']
const PLATFORMS = [...DESKTOP, 'android-arm64-v8a']
const ALLOW_UNSTABLE = 'EMUDOME_ALLOW_UNSTABLE_CORES'

class Failure extends Error {}
const fail = message => {
  throw new Failure(message)
}

// --- Paths (relative to the repo root, with forward slashes: they also go into the configs)

/** Folder of a platform's libraries: where the app and the Android build look for them. */
const libraryDir = platform =>
  platform.startsWith('android-') ? `cores/android/${platform.slice('android-'.length)}` : 'cores'

/** Folder of a platform's licence files (kept apart: the same name differs per platform). */
const noticesDir = platform => `cores/notices/${platform}`

/** Every file a core installs for a platform: library, licence, notices from the archive. */
const installedFiles = (core, platform) => {
  const build = core.builds[platform]
  return [
    { path: `${libraryDir(platform)}/${build.library.file}`, sha256: build.library.sha256 },
    { path: `${noticesDir(platform)}/${core.license.file}`, sha256: core.license.sha256 },
    ...build.notices.map(n => ({ path: `${noticesDir(platform)}/${n.file}`, sha256: n.sha256 })),
  ]
}

const abs = relative => path.join(ROOT, ...relative.split('/'))

// --- The lock

const HEX64 = /^[0-9a-f]{64}$/
const HEX40 = /^[0-9a-f]{40}$/
const FILE_NAME = /^[A-Za-z0-9._-]+$/

/** The problems of a parsed lock, as sentences; empty when it is well formed. */
const lockProblems = lock => {
  const problems = []
  const need = (ok, what) => {
    if (!ok) problems.push(what)
  }
  need(lock?.schema === 1, 'schema must be 1')
  const cores = Object.entries(lock?.cores ?? {})
  need(cores.length > 0, 'no cores')
  for (const [id, core] of cores) {
    const at = `cores.${id}`
    need(typeof core.name === 'string', `${at}.name missing`)
    need(typeof core.ship === 'boolean', `${at}.ship must be true or false`)
    need(typeof core.spdx === 'string', `${at}.spdx missing`)
    need(/^https:\/\/github\.com\/[^/]+\/[^/]+$/.test(core.source?.repo ?? ''), `${at}.source.repo`)
    need(HEX40.test(core.source?.commit ?? ''), `${at}.source.commit must be a full commit hash`)
    need(typeof core.source?.version === 'string', `${at}.source.version missing`)
    need(
      core.source?.archive === undefined || HEX64.test(core.source?.archive_sha256 ?? ''),
      `${at}.source.archive needs archive_sha256`,
    )
    need(
      (core.license?.url ?? '').includes(core.source?.commit ?? '-'),
      `${at}.license.url must point at the pinned commit`,
    )
    need(HEX64.test(core.license?.sha256 ?? ''), `${at}.license.sha256`)
    need(FILE_NAME.test(core.license?.file ?? ''), `${at}.license.file`)
    const builds = Object.entries(core.builds ?? {})
    need(builds.length > 0, `${at}.builds empty`)
    for (const [platform, build] of builds) {
      const bt = `${at}.builds.${platform}`
      need(PLATFORMS.includes(platform), `${bt}: unknown platform`)
      need(/^https:\/\//.test(build.url ?? ''), `${bt}.url`)
      need(typeof build.url_stable === 'boolean', `${bt}.url_stable must be true or false`)
      need(!build.url_stable || !build.url.includes('/latest/'), `${bt}: a latest/ URL is not stable`)
      need(HEX64.test(build.sha256 ?? ''), `${bt}.sha256`)
      for (const [i, file] of [build.library, ...(build.notices ?? [])].entries()) {
        const ft = i === 0 ? `${bt}.library` : `${bt}.notices[${i - 1}]`
        need(typeof file?.member === 'string', `${ft}.member`)
        need(FILE_NAME.test(file?.file ?? ''), `${ft}.file`)
        need(HEX64.test(file?.sha256 ?? ''), `${ft}.sha256`)
      }
      need(Array.isArray(build.notices), `${bt}.notices must be a list`)
      need(
        !platform.startsWith('android-') || /^lib.+\.so$/.test(build.library?.file ?? ''),
        `${bt}.library.file: Android packages only lib*.so`,
      )
    }
  }
  return problems
}

const readLock = file => {
  const lock = JSON.parse(fs.readFileSync(file, 'utf8'))
  const problems = lockProblems(lock)
  if (problems.length > 0) fail(`${file} is not valid:\n  ${problems.join('\n  ')}`)
  return lock
}

// --- Hashes, downloads, zip

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex')

const fileSha256 = file => (fs.existsSync(file) ? sha256(fs.readFileSync(file)) : null)

const download = async url => {
  const response = await fetch(url, { headers: { 'User-Agent': 'emudome-fetch-cores' } })
  if (!response.ok) fail(`${url}: HTTP ${response.status}`)
  return Buffer.from(await response.arrayBuffer())
}

/** The bytes of one member of a zip archive (stored or deflated), from its central directory. */
const unzipMember = (zip, name) => {
  const eocd = zip.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]))
  if (eocd < 0) fail('not a zip archive')
  const count = zip.readUInt16LE(eocd + 10)
  const start = zip.readUInt32LE(eocd + 16)
  const entries = Array.from({ length: count }).reduce(
    ({ offset, list }) => {
      if (zip.readUInt32LE(offset) !== 0x02014b50) fail('broken zip central directory')
      const nameLength = zip.readUInt16LE(offset + 28)
      const extraLength = zip.readUInt16LE(offset + 30)
      const commentLength = zip.readUInt16LE(offset + 32)
      const entry = {
        method: zip.readUInt16LE(offset + 10),
        size: zip.readUInt32LE(offset + 20),
        local: zip.readUInt32LE(offset + 42),
        name: zip.toString('utf8', offset + 46, offset + 46 + nameLength),
      }
      return { offset: offset + 46 + nameLength + extraLength + commentLength, list: [...list, entry] }
    },
    { offset: start, list: [] },
  ).list
  const entry = entries.find(e => e.name === name) ?? fail(`${name} is not in the archive`)
  if (zip.readUInt32LE(entry.local) !== 0x04034b50) fail('broken zip local header')
  const data = entry.local + 30 + zip.readUInt16LE(entry.local + 26) + zip.readUInt16LE(entry.local + 28)
  const raw = zip.subarray(data, data + entry.size)
  if (entry.method === 0) return Buffer.from(raw)
  if (entry.method === 8) return zlib.inflateRawSync(raw)
  return fail(`${name}: unsupported zip method ${entry.method}`)
}

/** Writes through a temp file and a rename: a reader never sees half a file. */
const writeAtomic = (file, bytes) => {
  fs.mkdirSync(path.dirname(file), { recursive: true })
  const temp = `${file}.part`
  const fd = fs.openSync(temp, 'w')
  try {
    fs.writeSync(fd, bytes)
    fs.fsyncSync(fd)
  } finally {
    fs.closeSync(fd)
  }
  fs.renameSync(temp, file)
}

const checked = (what, bytes, expected) => {
  const actual = sha256(bytes)
  if (actual !== expected) fail(`${what}: sha256 ${actual}, cores.lock says ${expected}`)
  return bytes
}

// --- Commands

const hostPlatform = () => {
  const key = `${process.platform}-${process.arch}`
  const known = { 'win32-x64': 'windows-x86_64', 'linux-x64': 'linux-x86_64' }
  return known[key] ?? fail(`no default platform for ${key}: pass --platform (${PLATFORMS.join(', ')})`)
}

/** The cores a command works on: --cores, else the shipped ones; only those built for the platform. */
const selectCores = (lock, platform, wanted) => {
  const ids = Object.keys(lock.cores)
  const chosen =
    wanted === undefined
      ? ids.filter(id => lock.cores[id].ship)
      : wanted === 'all'
        ? ids
        : wanted.split(',').map(s => s.trim()).filter(Boolean)
  const unknown = chosen.filter(id => !ids.includes(id))
  if (unknown.length > 0) fail(`unknown core(s): ${unknown.join(', ')} (cores.lock has ${ids.join(', ')})`)
  const missing = chosen.filter(id => !lock.cores[id].builds[platform])
  if (missing.length > 0) fail(`cores.lock has no ${platform} build of: ${missing.join(', ')}`)
  return chosen
}

const fetchCore = async (id, core, platform) => {
  const build = core.builds[platform]
  const files = installedFiles(core, platform)
  if (files.every(f => fileSha256(abs(f.path)) === f.sha256)) {
    console.log(`${id} (${platform}): already in place, sha256 verified`)
    return
  }
  if (!build.url_stable) {
    console.warn(
      `warning: ${id} (${platform}) comes from a URL that changes with every upstream build;` +
        ' when it no longer serves the pinned bytes this fetch fails (see cores.lock).',
    )
  }
  try {
    console.log(`${id} (${platform}): downloading ${build.url}`)
    const archive = checked(`${id} archive`, await download(build.url), build.sha256)
    const [library, ...notices] = [build.library, ...build.notices].map(m =>
      checked(`${id} ${m.member}`, unzipMember(archive, m.member), m.sha256),
    )
    console.log(`${id}: downloading its licence ${core.license.url}`)
    const licence = checked(`${id} licence`, await download(core.license.url), core.license.sha256)
    // Same order as installedFiles: library, licence, notices.
    const bytes = [library, licence, ...notices]
    files.forEach((f, i) => writeAtomic(abs(f.path), bytes[i]))
    console.log(`${id} (${platform}): ${files.map(f => f.path).join(', ')}`)
  } catch (error) {
    files.filter(f => fs.existsSync(abs(f.path))).forEach(f => fs.rmSync(abs(f.path)))
    throw error
  }
}

const runFetch = async (lock, options) => {
  const platform = options.platform ?? hostPlatform()
  const errors = []
  for (const id of selectCores(lock, platform, options.cores)) {
    try {
      await fetchCore(id, lock.cores[id], platform)
    } catch (error) {
      if (!(error instanceof Failure)) throw error
      errors.push(`${id}: ${error.message}`)
    }
  }
  if (errors.length > 0) {
    fail(`FETCH FAILED, the files of these cores were removed:\n  ${errors.join('\n  ')}`)
  }
  console.log('Done: every file matches cores.lock.')
}

const runVerify = (lock, options) => {
  const platform = options.platform ?? fail('verify needs --platform')
  const ids = selectCores(lock, platform, options.package ? undefined : options.cores)
  const problems = ids.flatMap(id =>
    installedFiles(lock.cores[id], platform).flatMap(f => {
      const actual = fileSha256(abs(f.path))
      if (actual === null) return [`${f.path} is missing (run the fetch script)`]
      return actual === f.sha256 ? [] : [`${f.path}: sha256 ${actual}, cores.lock says ${f.sha256}`]
    }),
  )
  if (options.package) {
    derivedFiles(lock)
      .filter(d => read(d.file) !== d.text)
      .forEach(d =>
        problems.push(`${path.relative(ROOT, d.file)} is out of date: run node scripts/cores.mjs generate`),
      )
    const unstable = ids.filter(id => !lock.cores[id].builds[platform].url_stable)
    if (unstable.length > 0 && process.env[ALLOW_UNSTABLE] !== '1') {
      problems.push(
        `${unstable.join(', ')}: pinned from a URL that is not stable (cores.lock url_stable false).` +
          ` A package to share needs a stable source; for a local test package set ${ALLOW_UNSTABLE}=1.`,
      )
    }
  }
  if (problems.length > 0) fail(`VERIFY FAILED (${platform}):\n  ${problems.join('\n  ')}`)
  console.log(`${platform}: ${ids.join(', ')} match cores.lock.`)
}

// --- Derived files

const shipped = lock => Object.entries(lock.cores).filter(([, core]) => core.ship)

const sourcesMarkdown = lock => {
  const line = (id, core) => {
    const builds = Object.entries(core.builds)
      .map(([platform, b]) => `  - ${platform}: \`${b.library.file}\`, from ${b.url} (sha256 \`${b.sha256}\`)`)
      .join('\n')
    return [
      `## ${core.name} (${core.consoles})`,
      '',
      `- Licence: ${core.spdx}; its text is \`notices/${core.license.file}\`` +
        (Object.values(core.builds).some(b => b.notices.length > 0)
          ? ', and the licences of what it bundles are next to it.'
          : '.'),
      `- Source: ${core.source.repo}, commit \`${core.source.commit}\`` +
        ` (${core.source.ref}; version ${core.source.version}).`,
      core.source.archive
        ? `  Full source with its submodules: ${core.source.archive} (sha256 \`${core.source.archive_sha256}\`).`
        : `  Download it as ${core.source.repo}/archive/${core.source.commit}.zip`,
      `- Builds:`,
      builds,
      '',
    ].join('\n')
  }
  return [
    '# Sources of the libretro cores',
    '',
    '<!-- Generated from cores.lock by `node scripts/cores.mjs generate`. Do not edit by hand. -->',
    '',
    'EmuDome ships these libretro cores unmodified, as separate programs it loads at run time.',
    'Each keeps its own licence, whose text is in `notices/`. The Corresponding Source of each',
    'binary is the upstream commit named below. If a link stops working, open an issue at',
    `${EMUDOME_REPO}/issues and the project will provide that source.`,
    '',
    ...shipped(lock).map(([id, core]) => line(id, core)),
  ].join('\n')
}

/** The Tauri config merged by `tauri build --config` for one desktop platform. */
const packageConfig = (lock, platform) => {
  const resources = Object.fromEntries([
    ['../../LICENSE', 'LICENSE.txt'],
    ['../../cores/SOURCES.md', 'cores/SOURCES.md'],
    ...shipped(lock).flatMap(([, core]) =>
      installedFiles(core, platform).map(f => [
        `../../${f.path}`,
        f.path.startsWith(`${noticesDir(platform)}/`)
          ? `cores/notices/${path.posix.basename(f.path)}`
          : f.path,
      ]),
    ),
  ])
  return {
    build: { beforeBundleCommand: `node ../scripts/cores.mjs verify --platform ${platform} --package` },
    bundle: { active: true, resources },
  }
}

const derivedFiles = lock => [
  { file: SOURCES, text: sourcesMarkdown(lock) },
  ...DESKTOP.map(platform => ({
    file: path.join(PACKAGE_DIR, `${platform}.json`),
    text: `${JSON.stringify(packageConfig(lock, platform), null, 2)}\n`,
  })),
]

const runGenerate = lock => {
  derivedFiles(lock).forEach(d => writeAtomic(d.file, d.text))
  derivedFiles(lock).forEach(d => console.log(`wrote ${path.relative(ROOT, d.file)}`))
}

// --- check

const read = file => (fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : null)

/** Every disagreement between the lock and the files that must follow it. */
const checkProblems = (lock, files) => {
  const problems = lockProblems(lock)
  if (problems.length > 0) return problems.map(p => `cores.lock: ${p}`)
  const stale = derivedFiles(lock)
    .filter(d => files.read(d.file) !== d.text)
    .map(d => `${path.relative(ROOT, d.file)} is not what cores.lock gives: run node scripts/cores.mjs generate`)
  const tauri = JSON.parse(files.read(TAURI_CONF) ?? '{}')
  const ownResources = tauri.bundle?.resources
    ? ['tauri.conf.json lists bundle.resources: the cores go only through app/src-tauri/package/<platform>.json']
    : []
  const gradle = (files.read(GRADLE) ?? '').includes('cores.lock')
    ? []
    : ['build.gradle.kts does not read cores.lock: the APK would take whatever is in cores/android']
  const thirdParty = files.read(THIRD_PARTY) ?? ''
  const undocumented = Object.entries(lock.cores)
    .filter(([, core]) => !thirdParty.includes(core.source.commit))
    .map(([id, core]) => `THIRD_PARTY.md does not name ${id}'s commit ${core.source.commit}`)
  return [...stale, ...ownResources, ...gradle, ...undocumented]
}

const disk = { read }

/** Each rule, fed a broken copy of the real inputs, must report something. */
const selfTest = lock => {
  const override = (file, text) => ({ read: f => (f === file ? text : read(f)) })
  const mutated = change => {
    const copy = JSON.parse(JSON.stringify(lock))
    change(Object.values(copy.cores)[0])
    return copy
  }
  const newCommit = 'f'.repeat(40)
  const cases = [
    ['malformed lock', mutated(core => (core.source.commit = 'abc')), disk],
    [
      'commit changed, derived files not regenerated',
      mutated(core => {
        core.source.commit = newCommit
        core.license.url += newCommit
      }),
      disk,
    ],
    ['a hand-edited bundle list', lock, override(path.join(PACKAGE_DIR, 'windows-x86_64.json'), '{}\n')],
    ['resources back in tauri.conf.json', lock, override(TAURI_CONF, '{"bundle":{"resources":["../../cores/"]}}')],
    ['Gradle not reading the lock', lock, override(GRADLE, 'jniLibs.srcDir("../../../cores/android")')],
    ['THIRD_PARTY.md without the commits', lock, override(THIRD_PARTY, '# Third-party components\n')],
    [
      'a latest/ URL marked stable',
      mutated(core => {
        Object.values(core.builds)[0].url = 'https://example.org/latest/core.zip'
        Object.values(core.builds)[0].url_stable = true
      }),
      disk,
    ],
  ]
  const silent = cases.filter(([, l, files]) => checkProblems(l, files).length === 0).map(([name]) => name)
  if (silent.length > 0) fail(`self-test: these cases were not caught:\n  ${silent.join('\n  ')}`)
  console.log(`self-test: all ${cases.length} broken inputs caught`)
}

const runCheck = (lock, options) => {
  if (options.selfTest) selfTest(lock)
  const problems = checkProblems(lock, disk)
  if (problems.length > 0) fail(`cores check:\n  ${problems.join('\n  ')}`)
  console.log('cores.lock, the bundle lists, cores/SOURCES.md, Gradle and THIRD_PARTY.md agree.')
}

// --- Entry point

const parseArgs = argv =>
  argv.reduce(
    (acc, arg, i) => {
      if (acc.skip) return { ...acc, skip: false }
      const value = argv[i + 1]
      if (arg === '--platform') return { ...acc, platform: value, skip: true }
      if (arg === '--cores') return { ...acc, cores: value, skip: true }
      if (arg === '--lock') return { ...acc, lock: value, skip: true }
      if (arg === '--package') return { ...acc, package: true }
      if (arg === '--self-test') return { ...acc, selfTest: true }
      if (!acc.command) return { ...acc, command: arg }
      return fail(`unknown argument ${arg}`)
    },
    { skip: false },
  )

const main = async () => {
  const options = parseArgs(process.argv.slice(2))
  if (options.platform !== undefined && !PLATFORMS.includes(options.platform)) {
    fail(`unknown platform ${options.platform} (${PLATFORMS.join(', ')})`)
  }
  const lock = readLock(options.lock ? path.resolve(options.lock) : LOCK)
  const commands = {
    fetch: () => runFetch(lock, options),
    verify: () => runVerify(lock, options),
    generate: () => runGenerate(lock),
    check: () => runCheck(lock, options),
  }
  const command = commands[options.command] ?? fail('usage: node scripts/cores.mjs fetch|verify|generate|check')
  await command()
}

main().catch(error => {
  if (!(error instanceof Failure)) throw error
  console.error(error.message)
  process.exitCode = 1
})
