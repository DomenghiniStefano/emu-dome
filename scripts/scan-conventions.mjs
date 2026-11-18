// Gate "design system conventions": the rules of docs/specs/2026-11-18-design-system-foundations.md
// (Decision 9) over the frontend sources, so a screen written outside the design system fails
// scripts/check instead of waiting for review.
//
// Run by scripts/check as `node scripts/scan-conventions.mjs` (from the repo root), and as
// `node scripts/scan-conventions.mjs --self-test`, which runs every rule on its fixture in
// scripts/scan-conventions/ and fails if a rule does not fire there: each rule is seen failing on
// every run, not once. Exit 0 = nothing found in new code, 1 = findings, 2 = usage error.
//
// New code and the screens of today. Cycle 1 brings the rules; the screens written before it break
// most of them by design (scoped <style>, raw <button>), and are migrated in cycle 2 (Trello #62).
// So the files in LEGACY are reported as a count per rule and do not fail; every other file fails.
// Cycle 2 removes a file from LEGACY when it migrates it; cycle 3 (Trello #63) deletes the list.
//
// The rules (heuristics on the source text, as the other gates; what each cannot see is beside it):
//   literal-colour    a colour written into a class: `bg-[#fff]`, `text-[rgb(…)]`, `[color-mix(…)]`.
//                     Colours are tokens (src/assets/theme/colors.css). Cannot see a colour put
//                     together at run time, or one in a `style` binding.
//   alpha-modifier    a colour utility with an opacity: `bg-ink/50`. The alpha lives in a token
//                     (`scrim`, `accent-soft`), never in a class. Cannot see an opacity applied with
//                     `opacity-*` on the element (allowed: it fades the element, not the colour).
//   dark-variant      `dark:`. The theme is `data-theme` on <html> with values per theme on the
//                     tokens; Tailwind's built-in `dark:` follows the OS instead.
//   literal-variant   a static `variant="…"` / `size="…"` on a component (a capitalised tag):
//                     variants are the `as const` constants of the primitive's variants.ts
//                     (`:variant="ButtonVariant.Primary"`). Cannot see a string passed through a
//                     variable, or other props (`orientation`, `side`) that take closed sets.
//   raw-control       a raw <button> or <input> outside src/components/ui: use the primitive.
//                     Cannot see one rendered by a render function or `<component :is="'button'">`.
//   style-block       a <style> block in a .vue file: styling is utilities over the tokens. The
//                     primitives and the Kit page have none either.
//   utility-collision a class name a LEGACY screen uses for its scoped CSS that Tailwind also
//                     generates as a utility (because the word appears in the sources Tailwind
//                     reads, src/components/ui and src/kit): the utility would apply to the
//                     screen's element too. Neutral only when the screen's scoped rule sets the
//                     same property; those are listed in NEUTRAL with the reason. Reads the words
//                     of the source files, a superset of what Tailwind takes as candidates, so it
//                     can report a word Tailwind would not use; it cannot see a class a screen adds
//                     from <script>. Needs app/node_modules (skipped before `pnpm install`).
import { existsSync, readdirSync, readFileSync, realpathSync } from 'node:fs'
import { createRequire } from 'node:module'
import { basename, join, relative, resolve, sep } from 'node:path'
import { pathToFileURL } from 'node:url'

const ROOT = resolve(import.meta.dirname, '..')
const SRC = join(ROOT, 'app', 'src')
const FIXTURES = join(ROOT, 'scripts', 'scan-conventions')

// The screens as they were when cycle 1 landed (paths under app/src/). Cycle 2 shrinks this list.
const LEGACY = new Set([
  'App.vue',
  'components/AppShell.vue',
  'components/ButtonTester.vue',
  'components/ControlsOverview.vue',
  'components/ControlsRemap.vue',
  'components/ControlsShortcuts.vue',
  'components/ControlsSticks.vue',
  'components/ControlsView.vue',
  'components/CoverMenu.vue',
  'components/CoverPicker.vue',
  'components/DetailsControls.vue',
  'components/DetailsFiles.vue',
  'components/DetailsSaves.vue',
  'components/DetailsSettings.vue',
  'components/FirstRun.vue',
  'components/FlowDialog.vue',
  'components/GameCanvas.vue',
  'components/GameCover.vue',
  'components/GameDetails.vue',
  'components/GameFlowOverlays.vue',
  'components/GameSurround.vue',
  'components/LibraryEmpty.vue',
  'components/LibraryFilters.vue',
  'components/LibraryView.vue',
  'components/QuickMenu.vue',
  'components/SettingRow.vue',
  'components/SettingSelect.vue',
  'components/SettingSelectMenu.vue',
  'components/SettingsView.vue',
  'components/ToastStack.vue',
  'components/TouchControls.vue',
])

// utility-collision: legacy class names whose utility is harmless, each checked by hand.
const NEUTRAL = new Map([
  ['block', 'the scoped `.block` sets `display: flex` (ControlsOverview, QuickMenu, SavesView)'],
  ['grid', 'the scoped `.grid` sets `display: grid` (ControlsRemap, LibraryView)'],
  ['underline', 'QuickMenu `.link.underline` sets `text-decoration: underline` itself'],
  ['glass-thick', 'utilities.css `glass-thick` is styles.css `.glass-thick`, same values'],
])

const RULES = [
  'literal-colour',
  'alpha-modifier',
  'dark-variant',
  'literal-variant',
  'raw-control',
  'style-block',
  'utility-collision',
]

/** The root <template> block of a .vue source, blanked outside it so offsets keep their lines. */
function template(source) {
  const start = source.search(/^<template[\s>]/m)
  const closes = [...source.matchAll(/^<\/template>/gm)]
  if (start < 0 || closes.length === 0) return ''
  const end = closes[closes.length - 1].index
  return source.slice(0, end).replace(/[^\n]/g, (c, i) => (i < start ? ' ' : c))
}

const COLOUR_UTILITY =
  '(?:bg|text|border(?:-[trblxyse])?|ring|ring-offset|outline|fill|stroke|shadow|inset-shadow|drop-shadow|from|via|to|divide|decoration|accent|caret|placeholder)'

/** Findings of the text rules in one file: { rule, line, text }. `kind`: 'vue' or 'ts'. */
export function scanSource(source, kind, { insideUi = false } = {}) {
  const found = []
  const lineOf = offset => source.slice(0, offset).split('\n').length
  const add = (rule, offset, text) => found.push({ rule, line: lineOf(offset), text: text.trim() })
  const each = (rule, re, text = source) => {
    for (const m of text.matchAll(re)) add(rule, m.index, m[0])
  }

  each('literal-colour', /-\[(?:#|rgba?\(|hsla?\(|oklch\(|oklab\(|lab\(|lch\(|color-mix\()[^\]]*\]/g)
  each(
    'alpha-modifier',
    new RegExp(`(?<![\\w-])${COLOUR_UTILITY}-[a-z][a-z0-9-]*\\/(?:\\d+|\\[[^\\]]*\\]|\\([^)]*\\))`, 'g'),
  )
  each('dark-variant', /(?<![\w-])dark:[a-z[]/g)

  if (kind === 'vue') {
    const tpl = template(source)
    each('literal-variant', /<[A-Z][A-Za-z0-9]*\b[^>]*?\s(?:variant|size)="[^"]*"/g, tpl)
    if (!insideUi) each('raw-control', /<(?:button|input)\b/g, tpl)
    each('style-block', /^<style\b[^>]*>/gm)
  }
  return found
}

/** Every .vue and .ts file under app/src, as paths relative to it with forward slashes. */
function sources(dir = SRC) {
  return readdirSync(dir, { withFileTypes: true }).flatMap(e => {
    const p = join(dir, e.name)
    if (e.isDirectory()) return sources(p)
    return /\.(vue|ts)$/.test(e.name) ? [relative(SRC, p).split(sep).join('/')] : []
  })
}

/** The class names a template sets statically or in a `:class` object / string. */
function classNames(source) {
  const tpl = template(source)
  const names = new Set()
  for (const m of tpl.matchAll(/(?<![:\w-])class="([^"]*)"/g)) {
    for (const t of m[1].split(/\s+/)) if (t) names.add(t)
  }
  for (const m of tpl.matchAll(/:class="([^"]*)"/g)) {
    for (const k of m[1].matchAll(/([A-Za-z][\w-]*)'?\s*:/g)) names.add(k[1])
    for (const k of m[1].matchAll(/'([A-Za-z][\w -]*)'/g)) {
      for (const t of k[1].split(/\s+/)) if (t) names.add(t)
    }
  }
  return names
}

/** Tailwind's compiler for app/src/assets/main.css, or null without app/node_modules. */
async function tailwind() {
  const pkg = join(ROOT, 'app', 'node_modules', '@tailwindcss', 'vite', 'package.json')
  if (!existsSync(pkg)) return null
  const req = createRequire(realpathSync(pkg))
  const { compile } = await import(pathToFileURL(req.resolve('@tailwindcss/node')).href)
  const base = join(SRC, 'assets')
  return compile(readFileSync(join(base, 'main.css'), 'utf8'), { base, onDependency() {} })
}

/** Whether `name` compiles to a utility class of that very name. */
const isUtility = (compiler, name) =>
  new RegExp(`\\.${name.replace(/[^\w-]/g, '\\$&')}\\s*\\{`).test(compiler.build([name]))

/**
 * utility-collision: `legacy` maps a class name to the legacy files using it, `words` is the set
 * of words in the sources Tailwind reads. A finding per class that is a utility, is in `words`,
 * and is not NEUTRAL.
 */
export function collisions(compiler, legacy, words) {
  return [...legacy]
    .filter(([name]) => words.has(name) && !NEUTRAL.has(name) && isUtility(compiler, name))
    .map(([name, files]) => ({
      rule: 'utility-collision',
      line: 0,
      text: `\`${name}\` is a utility now and a scoped class of ${[...files].join(', ')}`,
    }))
}

async function scanRepo() {
  const files = sources()
  const fresh = []
  const legacyCounts = new Map()
  for (const file of files) {
    const source = readFileSync(join(SRC, file), 'utf8')
    const kind = file.endsWith('.vue') ? 'vue' : 'ts'
    const found = scanSource(source, kind, { insideUi: file.startsWith('components/ui/') })
    if (LEGACY.has(file)) {
      for (const f of found) legacyCounts.set(f.rule, (legacyCounts.get(f.rule) ?? 0) + 1)
    } else {
      fresh.push(...found.map(f => `app/src/${file}:${f.line}: ${f.rule}: ${f.text}`))
    }
  }

  const compiler = await tailwind()
  if (compiler) {
    const legacy = new Map()
    for (const file of files.filter(f => LEGACY.has(f))) {
      for (const name of classNames(readFileSync(join(SRC, file), 'utf8'))) {
        if (!legacy.has(name)) legacy.set(name, new Set())
        legacy.get(name).add(basename(file))
      }
    }
    const words = new Set()
    for (const file of files.filter(f => f.startsWith('components/ui/') || f.startsWith('kit/'))) {
      for (const w of readFileSync(join(SRC, file), 'utf8').match(/[A-Za-z][\w-]*/g) ?? []) {
        words.add(w)
      }
    }
    fresh.push(...collisions(compiler, legacy, words).map(f => `app/src: ${f.rule}: ${f.text}`))
  } else {
    console.log('utility-collision skipped: no app/node_modules (run pnpm install in app/)')
  }

  if (legacyCounts.size > 0) {
    const counts = [...legacyCounts].map(([rule, n]) => `${rule} ${n}`).join(', ')
    console.log(`legacy screens (not failing until cycle 3, Trello #63): ${counts}`)
  }
  if (fresh.length === 0) {
    console.log('none')
    return 0
  }
  console.log(fresh.join('\n'))
  console.log(
    'Write it with the design system: tokens, the primitives of app/src/components/ui and their ' +
      'variant constants (docs/specs/2026-11-18-design-system-foundations.md).',
  )
  return 1
}

// Every rule on its fixture: `<rule>.vue.fixture` / `<rule>.ts.fixture` must produce that rule,
// and `clean.vue.fixture` nothing. utility-collision runs on a made-up legacy screen whose scoped
// class `table` meets the word `table` in the design system's sources.
async function selfTest() {
  const failures = []
  const fixtures = readdirSync(FIXTURES).filter(f => f.endsWith('.fixture'))
  for (const rule of RULES.filter(r => r !== 'utility-collision')) {
    const mine = fixtures.filter(f => f.startsWith(`${rule}.`))
    if (mine.length === 0) failures.push(`${rule}: no fixture in scripts/scan-conventions/`)
    for (const f of mine) {
      const kind = f.includes('.vue.') ? 'vue' : 'ts'
      const found = scanSource(readFileSync(join(FIXTURES, f), 'utf8'), kind)
      if (!found.some(x => x.rule === rule)) failures.push(`${rule}: did not fire on ${f}`)
    }
  }
  const clean = scanSource(readFileSync(join(FIXTURES, 'clean.vue.fixture'), 'utf8'), 'vue')
  if (clean.length > 0) {
    failures.push(`clean.vue.fixture: ${clean.map(x => `${x.rule} line ${x.line}`).join(', ')}`)
  }
  const compiler = await tailwind()
  if (compiler) {
    const legacy = new Map([['table', new Set(['Fixture.vue'])]])
    if (collisions(compiler, legacy, new Set(['table'])).length !== 1) {
      failures.push('utility-collision: did not fire on `table`')
    }
    if (collisions(compiler, legacy, new Set(['tabular'])).length !== 0) {
      failures.push('utility-collision: fired on a word the sources do not hold')
    }
  } else {
    console.log('utility-collision self-test skipped: no app/node_modules')
  }
  if (failures.length > 0) {
    console.log(failures.join('\n'))
    return 1
  }
  console.log(`every rule fires on its fixture (${RULES.length} rules)`)
  return 0
}

const args = process.argv.slice(2)
if (args.length > 1 || (args.length === 1 && args[0] !== '--self-test')) {
  console.error('usage: node scripts/scan-conventions.mjs [--self-test]')
  process.exit(2)
}
process.exit(args[0] === '--self-test' ? await selfTest() : await scanRepo())
