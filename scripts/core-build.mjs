// Builds a libretro core from its pinned source, for the cores EmuDome builds itself instead of
// downloading a binary (cores.lock: `source.archive` + `build`, and `built` on a platform's entry;
// today mGBA, Trello #88, and melonDS DS for Android). Used by scripts/cores.mjs: its `build` command
// (behind scripts/build-mgba.ps1 / .sh and build-melondsds.ps1 / .sh) and `fetch`, when a built
// core's files are not in place and cannot be downloaded from EmuDome's release of the build
// (`built.release`), or with --build.
//
// Steps, every one checked:
//   1. the source archive of the pinned commit, downloaded once into the work folder, sha256-checked
//      against cores.lock (`source.archive_sha256`) before anything reads it;
//   2. extracted by this file (no tool from the archive runs, only the core's own CMake build);
//   3. `build.git_depth`: the commit's history fetched with git to that depth, so the core reports
//      the same version string as the libretro buildbot's build of the same commit (mGBA computes it
//      from `git describe` / `git rev-list --count` at build time; the buildbot clones 50 deep). The
//      fetched commit must be the pinned one and the extracted tree must equal it (`git status` clean);
//   4. `build.dependencies`: what the core's CMake would git-clone at configure time (FetchContent;
//      melonDS DS: cmake/FetchDependencies.cmake), each pinned to a commit and its source archive,
//      downloaded and sha256-checked like the core's, extracted where FetchContent puts its own
//      checkout (<build>/_deps/<name in lowercase>-src: melonDS DS's CMake reads modules from
//      there), and handed to CMake with FETCHCONTENT_SOURCE_DIR_<NAME> and
//      FETCHCONTENT_FULLY_DISCONNECTED=ON: CMake downloads nothing, and a dependency missing from
//      the lock fails the configure instead of being cloned. Their symbolic links are skipped
//      (libslirp's fuzzing corpus has some; nothing built reads them);
//   5. CMake with `build.cmake` plus the platform's toolchain (below), `--parallel <jobs>` (default 2);
//   6. the library (`build.output` + the platform's extension, found anywhere in the build tree)
//      copied to <work>/<platform>/out/ (stripped on Linux and Android), its sha256 compared with
//      cores.lock; only a match is installed into cores/ (temp file + rename), with the licence and
//      the entry's `notices` (licences of code the core compiles in), each checked against its
//      sha256: `member` is a path in the source archive, `output` a file the core's own build
//      writes in its build folder (melonDS DS: melondsds-LICENSE.txt, every bundled licence). Then
//      the source and build trees are removed (the archives and out/ stay); after a failure they stay.
//
// Toolchains (the one used is printed, and cores.lock records the one the pinned sha256 came from in
// `built.toolchain`: another one builds the same source but almost surely other bytes, so the
// install is refused and the built file left in out/ for a deliberate re-pin):
//   windows-x86_64     clang-cl of the Android NDK (why: at `toolchain` below) with the headers,
//                      libraries and link.exe of Visual Studio (Build Tools) 2022 (vswhere,
//                      vcvars64.bat) and the Windows SDK, Ninja; static CRT (no VC++ redistributable
//                      needed), /Brepro (no time stamps). CMake: CMAKE, else `cmake` on PATH, else
//                      the one inside Visual Studio (with its Ninja).
//   android-<abi>      the NDK (NDK_HOME / ANDROID_NDK_HOME / ANDROID_NDK_ROOT, else
//                      <sdk>/ndk/<the version cores.lock names>, else the newest there), API 26 (the
//                      app's minSdk), Ninja; CMake and Ninja: CMAKE, else <sdk>/cmake/<newest>/bin,
//                      else PATH. -g0 and -ffile-prefix-map (C, C++ and assembly) keep the
//                      source and build folders out of the binary.
//   linux-x86_64       the system compiler with cmake and ninja (or make) on PATH, on Linux (WSL).
//                      Not built yet (Trello #92): cores.lock still pins the buildbot's Linux file.
//
// Work folder: --work-dir, else EMUDOME_CORE_BUILD_DIR, else <temp>/emudome-cb. Keep it short on
// Windows: MSBuild fails on paths over 260 characters.

import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import zlib from 'node:zlib'

export class BuildFailure extends Error {}
const fail = message => {
  throw new BuildFailure(message)
}

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex')

const ANDROID_API = 26

// --- Running tools

/** Runs a program (no shell), output on this console; fails on a non-zero exit. */
const run = (program, args, options = {}) => {
  console.log(`> ${[program, ...args].join(' ')}`)
  const result = spawnSync(program, args, { stdio: 'inherit', ...options })
  if (result.error) fail(`${program}: ${result.error.message}`)
  if (result.status !== 0) fail(`${path.basename(program)} ${args[0] ?? ''} failed (exit ${result.status})`)
}

/** A program's standard output, trimmed; null when it cannot run or fails. */
const output = (program, args, options = {}) => {
  const result = spawnSync(program, args, { encoding: 'utf8', ...options })
  return result.error || result.status !== 0 ? null : result.stdout.trim()
}

const exe = name => (process.platform === 'win32' ? `${name}.exe` : name)

const isFile = file => {
  try {
    return fs.statSync(file).isFile()
  } catch {
    return false
  }
}

// --- The source archive (.tar.gz), read here: no tar program, nothing from the archive runs

const tarString = (block, start, length) => {
  const raw = block.subarray(start, start + length)
  const end = raw.indexOf(0)
  return raw.toString('utf8', 0, end < 0 ? raw.length : end)
}

const paxPath = body =>
  body
    .toString('utf8')
    .split('\n')
    .map(record => record.replace(/^\d+ /, ''))
    .find(record => record.startsWith('path='))
    ?.slice('path='.length)

/**
 * Extracts a gzipped tar into `into`, dropping the archive's top folder (GitHub's `<repo>-<commit>/`).
 * Only plain files and folders; a path that would leave `into` or any other entry kind fails, except
 * links with `skipLinks` (a dependency's archive): those are skipped, never created, and counted.
 */
const extractTarGz = (archive, into, { skipLinks = false } = {}) => {
  const tar = zlib.gunzipSync(archive)
  let offset = 0
  let longPath
  let count = 0
  let skipped = 0
  while (offset + 512 <= tar.length) {
    const header = tar.subarray(offset, offset + 512)
    if (header.every(b => b === 0)) break
    const size = parseInt(tarString(header, 124, 12).trim() || '0', 8)
    const type = String.fromCharCode(header[156] || 0x30)
    const body = tar.subarray(offset + 512, offset + 512 + size)
    offset += 512 + Math.ceil(size / 512) * 512
    if (type === 'g') continue
    if (type === 'x') {
      longPath = paxPath(body)
      continue
    }
    if (type === 'L') {
      longPath = tarString(body, 0, body.length)
      continue
    }
    const prefix = tarString(header, 345, 155)
    const name = longPath ?? (prefix ? `${prefix}/${tarString(header, 0, 100)}` : tarString(header, 0, 100))
    longPath = undefined
    const relative = name.split('/').slice(1).filter(Boolean)
    if (relative.length === 0) continue
    if (relative.some(part => part === '..' || part.includes(':') || part.includes('\\'))) {
      fail(`archive entry ${name} leaves the source folder`)
    }
    const target = path.join(into, ...relative)
    if (type === '5') {
      fs.mkdirSync(target, { recursive: true })
    } else if (type === '0' || type === '7') {
      fs.mkdirSync(path.dirname(target), { recursive: true })
      fs.writeFileSync(target, body)
      const mode = parseInt(tarString(header, 100, 8).trim() || '644', 8)
      if (process.platform !== 'win32') fs.chmodSync(target, mode & 0o111 ? 0o755 : 0o644)
      count += 1
    } else if (skipLinks && (type === '1' || type === '2')) {
      skipped += 1
    } else {
      fail(`archive entry ${name}: unsupported kind '${type}' (links are refused)`)
    }
  }
  if (count === 0) fail('the source archive holds no file')
  return { count, skipped }
}

/** A pinned source archive (`archive`, `archive_sha256`, `commit`), downloaded once into `dir`. */
const downloadArchive = async (what, source, dir) => {
  fs.mkdirSync(dir, { recursive: true })
  const file = path.join(dir, `${source.commit}.tar.gz`)
  if (isFile(file) && sha256(fs.readFileSync(file)) === source.archive_sha256) {
    console.log(`${what}: ${file} (cached, sha256 verified)`)
    return fs.readFileSync(file)
  }
  console.log(`${what}: downloading ${source.archive}`)
  const response = await fetch(source.archive, { headers: { 'User-Agent': 'emudome-build-cores' } })
  if (!response.ok) fail(`${source.archive}: HTTP ${response.status}`)
  const bytes = Buffer.from(await response.arrayBuffer())
  const actual = sha256(bytes)
  if (actual !== source.archive_sha256) {
    fail(`${what}: sha256 ${actual}, cores.lock says ${source.archive_sha256}; nothing was built`)
  }
  fs.writeFileSync(`${file}.part`, bytes)
  fs.renameSync(`${file}.part`, file)
  return bytes
}

// --- Version metadata (git), for the version string the core reports

const addGitHistory = (src, source, depth) => {
  const git = (...args) => run('git', ['-C', src, ...args])
  console.log(`git history: ${source.repo} at ${source.commit}, ${depth} deep (the version string)`)
  git('init', '--quiet')
  git('config', 'core.autocrlf', 'false')
  git('config', 'core.fileMode', 'false')
  git('fetch', '--quiet', '--no-tags', `--depth=${depth}`, source.repo, source.commit)
  const fetched = output('git', ['-C', src, 'rev-parse', 'FETCH_HEAD'])
  if (fetched !== source.commit) fail(`git fetched ${fetched}, cores.lock pins ${source.commit}`)
  git('update-ref', 'refs/heads/master', source.commit)
  git('symbolic-ref', 'HEAD', 'refs/heads/master')
  git('read-tree', 'HEAD')
  run('git', ['-C', src, 'update-index', '-q', '--refresh'], { stdio: 'ignore' })
  const status = output('git', ['-C', src, 'status', '--porcelain', '--untracked-files=all'])
  if (status === null) fail('git status failed in the source folder')
  if (status !== '') {
    fail(`the source archive is not the commit's tree:\n${status.split('\n').slice(0, 20).join('\n')}`)
  }
}

// --- Toolchains

const androidSdk = () =>
  [
    process.env.ANDROID_HOME,
    process.env.ANDROID_SDK_ROOT,
    process.env.LOCALAPPDATA && path.join(process.env.LOCALAPPDATA, 'Android', 'Sdk'),
    path.join(os.homedir(), 'Android', 'Sdk'),
  ].find(dir => dir && fs.existsSync(dir))

const newestVersionDir = dir => {
  if (!dir || !fs.existsSync(dir)) return null
  const versions = fs
    .readdirSync(dir)
    .filter(name => /^\d+(\.\d+)*$/.test(name))
    .sort((a, b) => {
      const pa = a.split('.').map(Number)
      const pb = b.split('.').map(Number)
      const i = pa.findIndex((n, k) => n !== (pb[k] ?? 0))
      return i < 0 ? pa.length - pb.length : pa[i] - (pb[i] ?? 0)
    })
  return versions.length > 0 ? path.join(dir, versions[versions.length - 1]) : null
}

const ndkVersion = ndk => {
  const properties = path.join(ndk, 'source.properties')
  if (!isFile(properties)) return null
  return /Pkg\.Revision\s*=\s*(\S+)/.exec(fs.readFileSync(properties, 'utf8'))?.[1] ?? null
}

const findNdk = pinned => {
  const fromEnv = [process.env.NDK_HOME, process.env.ANDROID_NDK_HOME, process.env.ANDROID_NDK_ROOT].find(
    dir => dir && ndkVersion(dir),
  )
  if (fromEnv) return fromEnv
  const sdk = androidSdk()
  const root = sdk && path.join(sdk, 'ndk')
  const exact = pinned && root && path.join(root, pinned)
  if (exact && ndkVersion(exact)) return exact
  return newestVersionDir(root) ?? fail('Android NDK not found: set NDK_HOME, or install one under <sdk>/ndk')
}

/** The NDK version a cores.lock toolchain description names («... Android NDK 27.1.12297006 ...»). */
const pinnedNdk = built => /NDK (\d+(?:\.\d+)*)/.exec(built.toolchain ?? '')?.[1]

const cmakeVersion = cmake => /cmake version (\S+)/.exec(output(cmake, ['--version']) ?? '')?.[1] ?? null

/** CMAKE, else the first of `candidates` that runs. */
const findCmake = candidates => {
  const all = [process.env.CMAKE, ...candidates].filter(Boolean)
  return all.find(c => cmakeVersion(c)) ?? fail(`CMake not found (tried ${all.join(', ')}): set CMAKE`)
}

const visualStudio = () => {
  const vswhere = path.join(
    process.env['ProgramFiles(x86)'] ?? 'C:\\Program Files (x86)',
    'Microsoft Visual Studio',
    'Installer',
    'vswhere.exe',
  )
  const where = isFile(vswhere)
    ? output(vswhere, [
        '-latest',
        '-products',
        '*',
        '-version',
        '[17.0,18.0)',
        '-requires',
        'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
        '-property',
        'installationPath',
      ])
    : null
  if (!where) fail('Visual Studio 2022 (or its Build Tools) with the C++ tools not found (vswhere)')
  const tools = path.join(where, 'VC', 'Auxiliary', 'Build', 'Microsoft.VCToolsVersion.default.txt')
  const msvc = isFile(tools) ? fs.readFileSync(tools, 'utf8').trim() : 'unknown'
  return { where, msvc }
}

/** The environment of Visual Studio's x64 developer prompt (vcvars64.bat): INCLUDE, LIB, PATH. */
const vcvarsEnv = vs => {
  const bat = path.join(vs.where, 'VC', 'Auxiliary', 'Build', 'vcvars64.bat')
  if (!isFile(bat)) fail(`${bat} not found`)
  const result = spawnSync('cmd.exe', ['/d', '/s', '/c', `""${bat}" >nul && set"`], {
    encoding: 'utf8',
    windowsVerbatimArguments: true,
  })
  if (result.status !== 0) fail('vcvars64.bat failed')
  const env = Object.fromEntries(
    result.stdout
      .split(/\r?\n/)
      .map(line => /^([^=]+)=(.*)$/.exec(line))
      .filter(Boolean)
      .map(([, key, value]) => [key, value]),
  )
  if (!env.INCLUDE || !env.LIB) fail('vcvars64.bat gave no INCLUDE / LIB')
  return env
}

/**
 * What a platform's build needs: the CMake program, its configure arguments, the environment
 * (`baseEnv` replaces this process's when set), the toolchain description cores.lock records, where
 * the library lands and how it is stripped.
 */
const toolchain = (platform, built, dirs) => {
  if (platform === 'windows-x86_64') {
    // clang-cl, not cl: libretro/mgba writes a preprocessor directive inside a macro's arguments
    // (src/gba/dma.c, the mLOG of a starting DMA), which GCC and Clang accept and MSVC refuses with
    // either preprocessor. The libretro buildbot builds it with GCC (MinGW). clang-cl is taken from
    // the Android NDK (needed for the Android build anyway, cores.lock names its version); the
    // headers, libraries and linker are MSVC's and the Windows SDK's, so the DLL is an ordinary
    // MSVC-ABI DLL with the static CRT (no VC++ redistributable needed).
    if (process.platform !== 'win32') fail('windows-x86_64 is built on Windows')
    const vs = visualStudio()
    const ndk = findNdk(pinnedNdk(built))
    const clangCl = path.join(ndk, 'toolchains', 'llvm', 'prebuilt', 'windows-x86_64', 'bin', 'clang-cl.exe')
    if (!isFile(clangCl)) fail(`${clangCl} not found`)
    const clangVersion = /clang version (\S+)/.exec(output(clangCl, ['--version']) ?? '')?.[1] ?? 'unknown'
    const vsCmake = path.join(vs.where, 'Common7', 'IDE', 'CommonExtensions', 'Microsoft', 'CMake')
    const cmake = findCmake(['cmake', path.join(vsCmake, 'CMake', 'bin', 'cmake.exe')])
    const besideCmake = path.join(path.dirname(cmake), 'ninja.exe')
    const ninja = [besideCmake, path.join(vsCmake, 'Ninja', 'ninja.exe')].find(isFile) ?? 'ninja'
    const vcEnv = vcvarsEnv(vs)
    const linker = path.join(vcEnv.VCToolsInstallDir ?? '', 'bin', 'Hostx64', 'x64', 'link.exe')
    if (!isFile(linker)) fail(`MSVC's link.exe not found at ${linker}`)
    return {
      cmake,
      description:
        `clang-cl ${clangVersion} (Android NDK ${ndkVersion(ndk)}), MSVC ${vs.msvc} libraries and linker,` +
        ` Windows SDK ${(vcEnv.WindowsSDKVersion ?? 'unknown').replace(/\\$/, '')}, CMake ${cmakeVersion(cmake)}`,
      configure: [
        '-G',
        'Ninja',
        `-DCMAKE_MAKE_PROGRAM=${ninja}`,
        `-DCMAKE_C_COMPILER=${clangCl}`,
        `-DCMAKE_CXX_COMPILER=${clangCl}`,
        `-DCMAKE_LINKER=${linker}`,
        '-DCMAKE_BUILD_TYPE=Release',
        '-DCMAKE_POLICY_DEFAULT_CMP0091=NEW',
        '-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded',
      ],
      baseEnv: vcEnv,
      // /Brepro: no time stamps in the objects and the DLL, so the same toolchain gives the same bytes.
      env: { CFLAGS: '/Brepro', CXXFLAGS: '/Brepro', LDFLAGS: '/Brepro' },
      extension: '.dll',
      strip: null,
    }
  }
  // No folder of this machine in the binary, so the bytes do not depend on where the build ran:
  // -g0 (the NDK's toolchain file adds -g; the debug info is stripped anyway, but it feeds the build
  // id the linker writes, and it names the NDK's and the work folder's paths), and the source and
  // build folders mapped to `.` for anything else (both slash forms: Windows paths reach the
  // compiler either way). Passed as CMAKE_<LANG>_FLAGS (C, C++ and assembly: melonDS DS is C++ with
  // some assembly): the NDK's toolchain file ignores CFLAGS.
  const reproducible = [
    '-g0',
    ...[dirs.src, dirs.build]
      .flatMap(dir => [...new Set([dir, dir.replaceAll('\\', '/')])])
      .map(dir => `-ffile-prefix-map=${dir}=.`),
  ].join(' ')
  if (platform.startsWith('android-')) {
    const abi = platform.slice('android-'.length)
    const ndk = findNdk(pinnedNdk(built))
    const sdkCmake = newestVersionDir(androidSdk() && path.join(androidSdk(), 'cmake'))
    const cmake = findCmake([sdkCmake && path.join(sdkCmake, 'bin', exe('cmake')), 'cmake'].filter(Boolean))
    const ninja =
      [path.dirname(cmake), sdkCmake && path.join(sdkCmake, 'bin')]
        .filter(Boolean)
        .map(dir => path.join(dir, exe('ninja')))
        .find(isFile) ?? 'ninja'
    const host = { win32: 'windows-x86_64', linux: 'linux-x86_64', darwin: 'darwin-x86_64' }[process.platform]
    const llvmStrip = path.join(ndk, 'toolchains', 'llvm', 'prebuilt', host ?? '', 'bin', exe('llvm-strip'))
    return {
      cmake,
      description: `Android NDK ${ndkVersion(ndk)}, API ${ANDROID_API}, CMake ${cmakeVersion(cmake)}`,
      configure: [
        '-G',
        'Ninja',
        `-DCMAKE_MAKE_PROGRAM=${ninja}`,
        `-DCMAKE_TOOLCHAIN_FILE=${path.join(ndk, 'build', 'cmake', 'android.toolchain.cmake')}`,
        `-DANDROID_ABI=${abi}`,
        `-DANDROID_PLATFORM=android-${ANDROID_API}`,
        // 16 KB-aligned segments: Google Play requires them for apps targeting Android 15+ (the
        // app targets SDK 37); NDK r27's switch, still loading on 4 KB-page devices.
        '-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON',
        '-DCMAKE_BUILD_TYPE=Release',
        ...['C', 'CXX', 'ASM'].map(lang => `-DCMAKE_${lang}_FLAGS=${reproducible}`),
      ],
      env: {},
      extension: '.so',
      strip: file => run(llvmStrip, ['--strip-unneeded', file]),
    }
  }
  if (platform === 'linux-x86_64') {
    if (process.platform !== 'linux') fail('linux-x86_64 is built on Linux (or WSL)')
    const cmake = findCmake(['cmake'])
    const compiler = (output('cc', ['--version']) ?? 'cc unknown').split('\n')[0]
    return {
      cmake,
      description: `${compiler}, CMake ${cmakeVersion(cmake)}`,
      configure: [
        '-G',
        output('ninja', ['--version']) ? 'Ninja' : 'Unix Makefiles',
        '-DCMAKE_BUILD_TYPE=Release',
        ...['C', 'CXX', 'ASM'].map(lang => `-DCMAKE_${lang}_FLAGS=${reproducible}`),
      ],
      env: {},
      extension: '.so',
      strip: file => run('strip', ['--strip-unneeded', file]),
    }
  }
  return fail(`no toolchain for ${platform}`)
}

// --- Dependencies the core's CMake would fetch (FetchContent), pinned

/** Every file named `name` under `dir`. */
const findFiles = (dir, name) =>
  fs
    .readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter(entry => entry.isFile() && entry.name === name)
    .map(entry => path.join(entry.parentPath ?? entry.path, entry.name))

/**
 * Downloads each pinned dependency (sha256-checked), extracts it where FetchContent would have
 * cloned it, and returns the CMake arguments that make the configure use these trees and fetch
 * nothing. The archives are cached in <work>/deps/.
 */
const placeDependencies = async (dependencies, work, buildDir) => {
  if (dependencies.length === 0) return []
  const args = ['-DFETCHCONTENT_FULLY_DISCONNECTED=ON']
  for (const dep of dependencies) {
    const archive = await downloadArchive(`dependency ${dep.name}`, dep, path.join(work, 'deps'))
    const into = path.join(buildDir, '_deps', `${dep.name.toLowerCase()}-src`)
    fs.mkdirSync(into, { recursive: true })
    const { count, skipped } = extractTarGz(archive, into, { skipLinks: true })
    console.log(
      `dependency ${dep.name} ${dep.commit}: ${count} files into ${into}` +
        (skipped > 0 ? ` (${skipped} symbolic links skipped)` : ''),
    )
    args.push(`-DFETCHCONTENT_SOURCE_DIR_${dep.name.toUpperCase()}=${into.replaceAll('\\', '/')}`)
  }
  return args
}

// --- The build

const defaultWorkDir = () => process.env.EMUDOME_CORE_BUILD_DIR || path.join(os.tmpdir(), 'emudome-cb')

/**
 * Builds core `id` for `platform` and calls `install(library, licence, notices)` with the bytes to
 * install, only when the library's sha256 is the one cores.lock pins (else it fails). Returns the
 * built file, its sha256 and the toolchain used.
 */
export const buildCore = async ({ id, core, platform, workDir, jobs = 2, install }) => {
  const build = core.builds[platform]
  if (!build?.built) fail(`${id}: cores.lock does not build ${platform} from source`)
  const work = path.resolve(workDir || defaultWorkDir(), id)
  const dirs = {
    src: path.join(work, platform, 's'),
    build: path.join(work, platform, 'b'),
    out: path.join(work, platform, 'out'),
  }
  const tools = toolchain(platform, build.built, dirs)
  console.log(`${id} (${platform}): building ${core.source.commit} with ${tools.description}`)
  if (tools.description !== build.built.toolchain) {
    console.warn(
      `warning: cores.lock's sha256 comes from "${build.built.toolchain}"; another toolchain gives` +
        ' other bytes, and then nothing is installed (the built file stays in the work folder).',
    )
  }

  const archive = await downloadArchive('source archive', core.source, work)
  fs.rmSync(path.join(work, platform), { recursive: true, force: true })
  fs.mkdirSync(dirs.src, { recursive: true })
  console.log(`extracted ${extractTarGz(archive, dirs.src).count} files into ${dirs.src}`)
  if (core.build.git_depth) addGitHistory(dirs.src, core.source, core.build.git_depth)
  const dependencies = await placeDependencies(core.build.dependencies ?? [], work, dirs.build)

  const env = {
    ...(tools.baseEnv ?? process.env),
    ...tools.env,
    // git (run by the core's CMake for its version) never climbs above the source folder.
    GIT_CEILING_DIRECTORIES: path.dirname(dirs.src),
  }
  run(tools.cmake, ['-S', dirs.src, '-B', dirs.build, ...tools.configure, ...dependencies, ...core.build.cmake], {
    env,
  })
  run(
    tools.cmake,
    ['--build', dirs.build, '--config', 'Release', '--target', core.build.target, '--parallel', String(jobs)],
    { env },
  )

  const name = `${core.build.output}${tools.extension}`
  const found = findFiles(dirs.build, name)
  if (found.length !== 1) {
    fail(`the build produced ${found.length === 0 ? 'no' : found.length} ${name} in ${dirs.build}`)
  }
  const produced = found[0]
  fs.mkdirSync(dirs.out, { recursive: true })
  // A notice the build writes is kept in out/ beside the library, with its sha256 printed: a re-pin
  // needs both.
  const outputs = build.notices
    .filter(n => n.output !== undefined)
    .map(n => {
      const from = path.join(dirs.build, ...n.output.split('/'))
      if (!isFile(from)) fail(`${id}: the build did not write ${n.output} (cores.lock notices)`)
      const to = path.join(dirs.out, n.file)
      fs.copyFileSync(from, to)
      console.log(`${id} (${platform}): ${to}\n  sha256 ${sha256(fs.readFileSync(to))}`)
      return to
    })
  const file = path.join(dirs.out, build.library.file)
  fs.copyFileSync(produced, file)
  if (tools.strip) tools.strip(file)
  const bytes = fs.readFileSync(file)
  const actual = sha256(bytes)
  console.log(`${id} (${platform}): ${file}\n  sha256 ${actual}`)
  if (actual !== build.library.sha256) {
    fail(
      `${id} (${platform}): the build's sha256 is ${actual}, cores.lock says ${build.library.sha256}.` +
        ` Nothing was installed; the file is ${file} (toolchain: ${tools.description}).` +
        ' Re-pin only on purpose: update cores.lock (library.sha256, built.toolchain), then generate.',
    )
  }
  // The licence and the notices (licences of what the core compiles in) come from the same archive,
  // or from the build's own output (`output`, copied into out/ above).
  const verified = (file, expected, what) => {
    const content = fs.readFileSync(file)
    if (sha256(content) !== expected) fail(`${id}: ${file} does not match cores.lock's ${what}`)
    return content
  }
  const fromSource = member => path.join(dirs.src, ...member.split('/'))
  const licence = verified(fromSource(core.build.license), core.license.sha256, 'license.sha256')
  const notices = build.notices.map((n, i) =>
    verified(
      n.output !== undefined ? outputs.find(o => path.basename(o) === n.file) : fromSource(n.member),
      n.sha256,
      `notices[${i}].sha256`,
    ),
  )
  install(bytes, licence, notices)
  // Installed: the source and build trees (about 100 MB) go; the archive and out/ stay. After a
  // failure they stay, to look at.
  fs.rmSync(dirs.src, { recursive: true, force: true })
  fs.rmSync(dirs.build, { recursive: true, force: true })
  return { file, sha256: actual, toolchain: tools.description }
}
