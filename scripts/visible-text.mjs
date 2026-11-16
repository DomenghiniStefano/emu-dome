// Gate "visible text": flags user-facing text written straight into a .vue template instead of
// going through vue-i18n (`t()`, `<I18nT>`), so a new hardcoded string fails scripts/check.
//
// Run by scripts/check as `node scripts/visible-text.mjs <file.vue>...`. Exit 0 = nothing found,
// 1 = findings (printed as file:line: text), 2 = usage error. No dependencies: a regex scan of the
// <template> block, not a Vue parser.
//
// What it flags, inside the template:
//   - text nodes with at least one letter, once `{{ ... }}` interpolations and comments are removed;
//   - static (unbound) placeholder / title / aria-label / alt / label attributes with a letter.
//     Bound ones (`:title="t('...')"`) are fine: the value is code.
// What it lets through:
//   - the content of <kbd> and <code>: key names and file paths are not translated;
//   - ALLOWED: words that are the same in every language (the product name, key names).
// What it cannot see: text built in <script> and bound with `{{ }}` or `:attr` (a hardcoded string
// in a ref or computed), text set from TypeScript (document.title, toasts), and render functions.
import { existsSync, readFileSync } from 'node:fs'

const ALLOWED = new Set(['EmuDome', 'Esc'])
const TEXT_ATTRS = ['placeholder', 'title', 'aria-label', 'alt', 'label']

// A tag, with quoted attribute values skipped whole: `@click="(a) => b"` holds a `>`.
const TAG = /<\/?[A-Za-z][^\s/>]*(?:\s+[^\s"'>=/]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'>]+))?)*\s*\/?>/g
const ATTR = /(?:^|\s)([^\s"'>=/]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g
const LETTER = /\p{L}/u

/** Replaces a match with spaces, keeping the newlines, so offsets still map to the right line. */
const blank = s => s.replace(/[^\n]/g, ' ')

function scan(file) {
  // A file git still lists but the working copy deleted (a rename not staged yet) has no text.
  if (!existsSync(file)) return []
  const source = readFileSync(file, 'utf8')
  // The root <template> block: from the first line that opens it to the last line that closes it
  // (nested <template #slot> tags are indented, the root ones start the line).
  const start = source.search(/^<template[\s>]/m)
  const closes = [...source.matchAll(/^<\/template>/gm)]
  if (start < 0 || closes.length === 0) return []
  const end = closes[closes.length - 1].index
  let tpl = source.slice(0, end).replace(/[^\n]/g, (c, i) => (i < start ? ' ' : c))

  tpl = tpl
    .replace(/<!--[\s\S]*?-->/g, blank)
    .replace(/\{\{[\s\S]*?\}\}/g, blank)
    .replace(/<(kbd|code)\b[^>]*>[\s\S]*?<\/\1>/g, blank)

  const lineOf = offset => source.slice(0, offset).split('\n').length
  const found = []
  const check = (text, offset) => {
    const value = text.trim()
    if (value && LETTER.test(value) && !ALLOWED.has(value)) {
      found.push(`${file}:${lineOf(offset)}: ${value.replace(/\s+/g, ' ')}`)
    }
  }

  let last = 0
  for (const tag of tpl.matchAll(TAG)) {
    check(tpl.slice(last, tag.index), last + tpl.slice(last, tag.index).search(/\S|$/))
    for (const attr of tag[0].matchAll(ATTR)) {
      if (TEXT_ATTRS.includes(attr[1])) check(attr[2] ?? attr[3], tag.index + attr.index)
    }
    last = tag.index + tag[0].length
  }
  check(tpl.slice(last), last)
  return found
}

const files = process.argv.slice(2)
if (files.length === 0) {
  console.error('usage: node scripts/visible-text.mjs <file.vue>...')
  process.exit(2)
}
const found = files.flatMap(scan)
if (found.length === 0) {
  console.log('none')
  process.exit(0)
}
console.log(found.join('\n'))
console.log('Move the text into app/src/i18n/en.json and it.json and render it with t().')
process.exit(1)
