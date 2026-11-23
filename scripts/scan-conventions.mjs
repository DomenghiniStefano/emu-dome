// Gate "design system conventions": the rules of docs/specs/2026-11-18-design-system-foundations.md
// (Decision 9) and of docs/frontend-conventions.md over the frontend sources, so a screen written
// outside the design system fails scripts/check instead of waiting for review.
//
// Run by scripts/check as `node scripts/scan-conventions.mjs` (from the repo root), and as
// `node scripts/scan-conventions.mjs --self-test`, which runs every rule on its fixture in
// scripts/scan-conventions/ and fails if a rule does not fire there: each rule is seen failing on
// every run, not once. Exit 0 = nothing found, 1 = findings, 2 = usage error.
//
// Every rule blocks, in every .vue and .ts file of app/src. Cycle 1 brought the rules with a LEGACY
// list of screens that were only counted; cycle 2 (Trello #62) migrated them all and cycle 3
// (Trello #63) removed the list, and with it the rule that guarded the legacy screens' scoped CSS
// from utilities of the same name (utility-collision): no screen has scoped CSS any more.
//
// The rules (heuristics on the source text, as the other gates; what each cannot see is beside it).
// The class rules read the whole file, not only class attributes: a comment in app/src that quotes
// a forbidden class fires too, so a comment names the rule instead of writing the class.
//   literal-colour    a colour written into a class: `bg-[#fff]`, `text-[rgb(…)]`, `[color-mix(…)]`.
//                     Colours are tokens (src/design-system/tokens/colors.css). Cannot see a colour put
//                     together at run time, one in a `style` binding, or a named one (`bg-[red]`).
//   alpha-modifier    a colour utility with an opacity: `bg-ink/50`. The alpha lives in a token
//                     (`scrim`, `accent-soft`), never in a class.
//   dark-variant      `dark:`. The theme is `data-theme` on <html> with values per theme on the
//                     tokens; Tailwind's built-in `dark:` follows the OS instead.
//   literal-variant   a static `variant="…"` / `size="…"` on a component (a capitalised tag):
//                     variants are the `as const` constants of the primitive's variants.ts
//                     (`:variant="ButtonVariant.Primary"`). Cannot see a string passed through a
//                     variable, or other props (`orientation`, `side`) that take closed sets.
//   raw-control       a raw <button> or <input> outside src/design-system/components: use the primitive.
//                     Cannot see one rendered by a render function or `<component :is="'button'">`.
//   style-block       a <style> block in any .vue file, the primitives' and the Kit page's too:
//                     styling is utilities over the tokens, compound looks are named utilities
//                     (src/design-system/styles/utilities.css).
//   arbitrary-length  a length in px / rem / em inside a class's brackets: `w-[13px]`,
//                     `text-[0.9rem]`, `shadow-[0_1px_2px_…]`, `top-[calc(100%+4px)]`, an arbitrary
//                     variant `[@media(max-width:900px)]:…`. Sizes are tokens (spacing.css,
//                     typography.css, radius.css, border.css, shadow.css); a breakpoint is a
//                     `--breakpoint-*` token, used as `max-<name>:`. Values built from tokens stay
//                     allowed: `calc(100%+var(--spacing-6))`, grid templates of fr / minmax / auto /
//                     var. Cannot see a length in % / vh / vw (proportions of a box or of the
//                     window: `top-[38%]`, `max-h-[50vh]`), a length in a `style` binding, a
//                     component prop (a Lucide icon's `:size="14"`), or one put together at run
//                     time from a number. Exceptions with no token form go in ALLOWED_LENGTHS by
//                     exact class, with the reason.
//   arbitrary-scale   an opacity, stacking order, duration, delay or curve in brackets:
//                     `opacity-[0.5]`, `z-[5]`, `duration-[300ms]`, `delay-[…]`, `ease-[…]`.
//                     They are tokens (opacity.css, the `z-*` utilities of layers.css, motion.css).
//                     Allows a variable reference `opacity-(--x)` (a token by construction).
//   numeric-scale     a Tailwind utility that compiles from a bare number, which the theme reset
//                     cannot switch off: `opacity-40` (not `opacity-0` / `opacity-100`, hidden and
//                     shown), `z-5`, `duration-300`, `delay-100`, `border-2` / `border-t-2`,
//                     `outline-2`, `outline-offset-2`, `ring-2`, `ring-offset-2`, `divide-x-2`,
//                     `underline-offset-2`, `decoration-2`, `scale-95` (a width of 0 stays: no
//                     edge). Use the tokens (`border-strong`, `outline-ring`, `outline-offset-ring`,
//                     `opacity-dim`…). Layout counts stay allowed: `grid-cols-3`, `col-span-2`,
//                     `aspect-3/2`, `line-clamp-2`, `order-1`, `rotate-90`. The rest of Tailwind's
//                     default palette and scales (`text-red-500`, `p-13`, `rounded-lg`,
//                     `shadow-md`, `text-sm`, `md:`) is reset in the theme (`--…-*: initial` in
//                     src/design-system/tokens/*.css) and generates no class at all: no rule needed.
//   important         a `!` override in a class: `w-48!` (Tailwind 4) or `!w-48` (Tailwind 3), with
//                     or without variants. A missing look is a variant; the few unlayered rules a
//                     look must beat are named utilities with `!important` inside (utilities.css).
//                     In a static class="…" it sees every class; in scripts, `:class` bindings and
//                     .ts files only a class with a `-` or a `:` (`w-48!`, `hover:!flex`), since
//                     `flex!` there reads as TypeScript's non-null assertion.
//   look-only-prop    a component prop named `phone` / `touch` whose every use is inside a
//                     `:class` binding: a prop only to change a look. The touch look is
//                     `data-density="touch"` on <html> and the `touch:` variant. Cannot see a prop
//                     read in <script> (a computed that only returns classes counts as a real
//                     use), one passed down to a child that uses it only for looks, a prop of
//                     another name, or props declared without the `defineProps<{ … }>` form.
//   hand-role         a widget role written by hand outside src/design-system/components: `role="dialog"`,
//                     `alertdialog`, `radio`, `radiogroup`, `checkbox`, `menu`, `menuitem`,
//                     `listbox`, `option`, `switch`, `tab`, `tablist`, static or in a `:role`
//                     binding. Those widgets are Reka primitives (Dialog, RadioGroup, Checkbox,
//                     DropdownMenu, Popover, Select, Switch, Tabs), which bring the focus trap,
//                     `aria-modal`, the arrow roving and the keys. Files still written by hand
//                     are in HAND_ROLES_PENDING with the reason, until they move (Trello #82).
//                     Cannot see a role set from a script (`setAttribute`), one in a render
//                     function, or a `:role` built from a variable.
//   button-name       a <Button> outside src/design-system/components named twice, with an `aria-label` and a
//                     `title` (static or bound): an icon alone takes Button's `label` (Trello #104),
//                     which is its accessible name and the app's tooltip (shown on the controller's
//                     and the keyboard's focus too, which `title` never is). A `title` beside
//                     `label` stays allowed (the tooltip then shows it: a short word for a long
//                     accessible name). Cannot see a Button rendered by a render function or
//                     `<component :is>`, nor attributes passed in a `v-bind="{…}"` object.
//   dead-button       a control with no action (Trello #112): a <Button>, <LinkRow>,
//                     <DropdownMenuItem> or a <Card> rendered as a button (`as="button"`) outside
//                     src/design-system with none of `@click` / `@select` / `@pointerdown` / `v-on`,
//                     `to`, `type="submit"`, `as-child`, `v-bind="…"` or a static `disabled` (a bound
//                     `:disabled` is live whenever it is false). Allowed without one: the only child
//                     of a tag that gives it its action (RadioGroupItem, Checkbox, DropdownMenuItem,
//                     a `…Trigger` / `…Close`, anything `as-child`), the component's root element
//                     (its caller's listeners fall through), `as` another element (`as="div"`: not
//                     a control), and a control after `<!-- no-action: why -->`. Cannot see a
//                     handler that does nothing, an event emitted that no parent listens to, a
//                     root element whose callers pass no listener, a `v-bind` object without one,
//                     controls in a render function or `<component :is>`, nor the Kit page.
import { readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";

const ROOT = resolve(import.meta.dirname, "..");
const SRC = join(ROOT, "app", "src");
const FIXTURES = join(ROOT, "scripts", "scan-conventions");

// arbitrary-length: classes allowed with a literal length, by exact text, each with the reason it
// has no token form. Empty: the last one (`align-[-2px]`, LibraryView) became the `align-icon`
// utility (typography.css).
const ALLOWED_LENGTHS = new Map([]);

// hand-role: the files (under app/src) that still write a widget role by hand, each with why it
// has not moved to a primitive yet. A temporary list: it only shrinks.
const HAND_ROLES_PENDING = new Map([
  [
    "components/quick-menu/QuickMenu.vue",
    "the menu's two panels are the screen itself (G07 panel, G09 bar, phone sheet), not an overlay over a trigger: on Reka Dialog they would portal and trap differently; needs its own design pass",
  ],
  [
    "components/closing/ClosingAsk.vue",
    "the closing question sits in the closing overlay's layout (ClosingOverlay), not as a centred dialog",
  ],
  [
    "components/controls/ControlsOverview.vue",
    "device cards (DeviceCard) as a radio group: next to move onto RadioGroup",
  ],
  [
    "components/controls/DeviceCard.vue",
    "one device card as a radio: moves with ControlsOverview",
  ],
  [
    "components/first-run/ControllerStep.vue",
    "the A/B layout choice: next to move onto RadioGroup",
  ],
  [
    "components/quick-menu/QuickSaves.vue",
    "the quick-save slots as a radio group (G07): moves with QuickMenu",
  ],
  [
    "components/quick-menu/QuickSlotTile.vue",
    "one quick-save slot as a radio: moves with QuickSaves",
  ],
  [
    "components/settings/SettingCards.vue",
    "a setting's choice cards, a plain group in `apply` mode: next to move onto RadioGroup",
  ],
  [
    "components/settings/SettingChips.vue",
    "a setting's multi-choice chips as checkboxes: next to move onto Checkbox",
  ],
  [
    "components/settings/SettingSwatches.vue",
    "a setting's colour swatches as a radio group: next to move onto RadioGroup",
  ],
  [
    "design-system/kit/sections/ButtonSection.vue",
    "the Kit page shows the accent swatches as written today",
  ],
  [
    "design-system/kit/sections/CardSection.vue",
    "the Kit page shows the choice cards as the screens above write them today",
  ],
  [
    "design-system/kit/sections/ChipSection.vue",
    "the Kit page shows the chips as SettingChips writes them today",
  ],
]);

const WIDGET_ROLE =
  /(?<![\w:-])(?::role|v-bind:role|role)="[^"]*\b(?:alertdialog|dialog|radiogroup|radio|checkbox|menuitem|menu|listbox|option|switch|tablist|tab)\b[^"]*"/g;

export const RULES = [
  "literal-colour",
  "alpha-modifier",
  "dark-variant",
  "literal-variant",
  "raw-control",
  "style-block",
  "arbitrary-length",
  "arbitrary-scale",
  "numeric-scale",
  "important",
  "look-only-prop",
  "hand-role",
  "button-name",
  "dead-button",
];

/** The open tags `<Name …>` of a template, quotes respected (a `>` inside `@click="() => x"`
 * does not end the tag): [{ index, text }]. */
function openTags(tpl, name) {
  const tags = [];
  const re = new RegExp(`<${name}(?![\\w-])`, "g");
  for (const m of tpl.matchAll(re)) {
    let quote = null;
    let i = m.index + m[0].length;
    for (; i < tpl.length; i++) {
      const c = tpl[i];
      if (quote) {
        if (c === quote) quote = null;
      } else if (c === '"' || c === "'") quote = c;
      else if (c === ">") break;
    }
    tags.push({ index: m.index, text: tpl.slice(m.index, i + 1) });
  }
  return tags;
}

const ATTR = (name) => new RegExp(`(?<![\\w-])(?::|v-bind:)?${name}=`);

// dead-button: what gives a control its action on its own tag. A bound `:disabled` is not one
// (the control is live whenever it is false); a static `disabled` is.
const ACTION_BINDING =
  /(?<![\w-])(?:@|v-on:)(?:click|select|pointerdown)\b|(?<![\w-])v-on=|(?<![\w-])v-bind=|(?<![\w:-])(?::|v-bind:)?to=|(?<![\w:-])type="submit"|(?<![\w:-])as-child(?![\w-])|(?<![\w:-])disabled(?=[\s/>])/;
// The tags that give their only child its action (Reka's `as-child` wrappers of ours).
const ACTION_WRAPPER =
  /^<(?:RadioGroupItem|Checkbox|DropdownMenuItem|[A-Z]\w*Trigger|[A-Z]\w*Close)(?![\w-])|(?<![\w:-])as-child(?![\w-])/;
// `<!-- no-action: <why> -->` right before a control: it takes no action by design.
const NO_ACTION = /<!--\s*no-action:[^>]*-->\s*$/;

/** dead-button: the controls of a template with no action: a <Button>, <LinkRow>,
 * <DropdownMenuItem> or a <Card> rendered as a button, with no handler, link, submit, `as-child`
 * or static `disabled`, not wrapped by a tag that gives it one, not the component's root (whose
 * caller's listeners fall through), and with no `no-action` comment before it. */
function deadControls(tpl) {
  const found = [];
  // The component's root element: the first tag inside <template>.
  const open = tpl.search(/<template[\s>]/);
  const inside = tpl.indexOf(">", open) + 1;
  const firstTag = open < 0 ? -1 : inside + tpl.slice(inside).search(/<[A-Za-z]/);
  for (const name of ["Button", "LinkRow", "DropdownMenuItem", "Card"]) {
    for (const tag of openTags(tpl, name)) {
      const t = tag.text;
      if (name === "Card" && !/(?<![\w:-])as="button"|(?<![\w-]):as="[^"]*'button'/.test(t)) continue;
      if ((name === "Button" || name === "LinkRow") && /(?<![\w:-])as="(?!button")[^"]*"/.test(t)) continue;
      if (ACTION_BINDING.test(t)) continue;
      if (tag.index === firstTag) continue;
      const before = tpl.slice(0, tag.index);
      if (NO_ACTION.test(before)) continue;
      // The open tag right before it (comments aside) is its parent: does it give the action?
      const prev = before.replace(/<!--[\s\S]*?-->/g, "").trimEnd();
      const last = [...prev.matchAll(/<([A-Za-z][\w-]*)/g)].at(-1);
      if (last) {
        const parent = openTags(prev.slice(last.index), last[1])[0];
        if (
          parent &&
          last.index + parent.text.length === prev.length &&
          ACTION_WRAPPER.test(parent.text)
        )
          continue;
      }
      found.push(tag);
    }
  }
  return found;
}

/** The root <template> block of a .vue source, blanked outside it so offsets keep their lines. */
function template(source) {
  const start = source.search(/^<template[\s>]/m);
  const closes = [...source.matchAll(/^<\/template>/gm)];
  if (start < 0 || closes.length === 0) return "";
  const end = closes[closes.length - 1].index;
  return source
    .slice(0, end)
    .replace(/[^\n]/g, (c, i) => (i < start ? " " : c));
}

const COLOUR_UTILITY =
  "(?:bg|text|border(?:-[trblxyse])?|ring|ring-offset|outline|fill|stroke|shadow|inset-shadow|drop-shadow|from|via|to|divide|decoration|accent|caret|placeholder)";

// A class with brackets: variants and a utility name glued to `[…]` (one level of nested brackets),
// with no space or quote inside, as in a class.
const BRACKETS =
  /(?<![\w$])[\w@&:!/.-]*\[[^\s'"`\]]*(?:\[[^\s'"`\]]*\][^\s'"`\]]*)*\][\w:/.!-]*/g;
const LENGTH = /(?<![\w.-])-?(?:\d+\.?\d*|\.\d+)(?:px|rem|em)(?![\w-])/;

// numeric-scale: the utilities that compile from a bare number; a width of 0 and the opacities 0
// and 100 stay.
const NUMERIC_SCALE = new RegExp(
  "(?<![\\w-])-?(?:" +
    [
      "opacity-(?!(?:0|100)(?![\\w.]))\\d+",
      "z-\\d+",
      "(?:duration|delay)-\\d+",
      "(?:border|outline|ring|ring-offset|divide)(?:-[trblxyse])?-(?!0(?![\\w.]))\\d+",
      "(?:outline-offset|underline-offset)-\\d+",
      "decoration-(?!0(?![\\w.]))\\d+",
      "scale(?:-[xyz])?-\\d+",
    ].join("|") +
    ")(?![\\w./-])",
  "g",
);

// important: `!` before or after a class that has a `-` or a `:`, between spaces or quotes.
const CLASS_CHARS = String.raw`[\w:/.[\]()%,&-]*`;
const IMPORTANT = new RegExp(
  String.raw`(?<=^|[\s'"\x60])(?:[\w[\]&@-]+:)*(?:![a-z[]${CLASS_CHARS}[-:]${CLASS_CHARS}` +
    String.raw`|[a-z[]${CLASS_CHARS}[-:]${CLASS_CHARS}[\w\])]!)(?=$|[\s'"\x60])`,
  "gm",
);

/** The values of `name="…"` attributes in a template, with their offsets: [{ index, value }]. */
function attributes(tpl, name) {
  return [
    ...tpl.matchAll(new RegExp(`(?<![\\w:-])${name}="([^"]*)"`, "g")),
  ].map((m) => ({
    index: m.index + m[0].indexOf('"') + 1,
    value: m[1],
  }));
}

/** look-only-prop: the `phone` / `touch` props of a .vue source used only inside `:class`. */
function lookOnlyProps(source, tpl) {
  const props = source.match(/defineProps<\{([^}]*)\}>/);
  if (!props) return [];
  const declStart = props.index;
  const declEnd = declStart + props[0].length;
  const classRanges = [
    ...attributes(tpl, ":class"),
    ...attributes(tpl, "v-bind:class"),
  ].map((a) => [a.index, a.index + a.value.length]);
  const found = [];
  for (const name of ["phone", "touch"]) {
    if (!new RegExp(`(?<![\\w-])${name}\\??\\s*:`).test(props[1])) continue;
    // A use: the bare name or `props.<name>`; not the `touch:` variant nor `touch-none`, nor a
    // word of a comment.
    const code = source.replace(
      /<!--[\s\S]*?-->|\/\*[\s\S]*?\*\/|(?<![:'"])\/\/[^\n]*/g,
      (c) => c.replace(/[^\n]/g, " "),
    );
    const uses = [
      ...code.matchAll(new RegExp(`(?<![\\w-])${name}(?![\\w-]|:[\\w[])`, "g")),
    ]
      .map((m) => m.index)
      .filter((i) => i < declStart || i >= declEnd);
    const inClass = uses.filter((i) =>
      classRanges.some(([a, b]) => i >= a && i < b),
    );
    if (uses.length > 0 && inClass.length === uses.length)
      found.push({ name, index: declStart });
  }
  return found;
}

/** Findings of the text rules in one file: { rule, line, text }. `kind`: 'vue' or 'ts'. */
export function scanSource(
  source,
  kind,
  { insideUi = false, handRolesAllowed = false, insideKit = false } = {},
) {
  const found = [];
  const lineOf = (offset) => source.slice(0, offset).split("\n").length;
  const add = (rule, offset, text) =>
    found.push({ rule, line: lineOf(offset), text: text.trim() });
  const each = (rule, re, text = source) => {
    for (const m of text.matchAll(re)) add(rule, m.index, m[0]);
  };

  each(
    "literal-colour",
    /-\[(?:#|rgba?\(|hsla?\(|oklch\(|oklab\(|lab\(|lch\(|color-mix\()[^\]]*\]/g,
  );
  each(
    "alpha-modifier",
    new RegExp(
      `(?<![\\w-])${COLOUR_UTILITY}-[a-z][a-z0-9-]*\\/(?:\\d+|\\[[^\\]]*\\]|\\([^)]*\\))`,
      "g",
    ),
  );
  each("dark-variant", /(?<![\w-])dark:[a-z[]/g);
  for (const m of source.matchAll(BRACKETS)) {
    if (LENGTH.test(m[0]) && !ALLOWED_LENGTHS.has(m[0]))
      add("arbitrary-length", m.index, m[0]);
  }
  each(
    "arbitrary-scale",
    /(?<![\w-])-?(?:opacity|z|duration|delay|ease)-\[[^\]\s]*\]/g,
  );
  each("numeric-scale", NUMERIC_SCALE);
  each("important", IMPORTANT);

  if (kind === "vue") {
    const tpl = template(source);
    // important in a static class="…": every class, `flex!` and `!hidden` too.
    for (const { index, value } of attributes(tpl, "class")) {
      for (const m of value.matchAll(
        /(?<=^|\s)(?:\S*:)?!\S+|(?<=^|\s)\S+!(?=\s|$)/g,
      )) {
        const at = lineOf(index + m.index);
        if (
          !found.some(
            (f) => f.rule === "important" && f.line === at && f.text === m[0],
          )
        ) {
          add("important", index + m.index, m[0]);
        }
      }
    }
    each(
      "literal-variant",
      /<[A-Z][A-Za-z0-9]*\b[^>]*?\s(?:variant|size)="[^"]*"/g,
      tpl,
    );
    if (!insideUi) each("raw-control", /<(?:button|input)\b/g, tpl);
    if (!insideUi && !handRolesAllowed) each("hand-role", WIDGET_ROLE, tpl);
    if (!insideUi) {
      for (const tag of openTags(tpl, "Button")) {
        if (ATTR("aria-label").test(tag.text) && ATTR("title").test(tag.text))
          add("button-name", tag.index, "<Button> with aria-label and title: use `label`");
      }
    }
    if (!insideUi && !insideKit) {
      for (const tag of deadControls(tpl)) {
        add(
          "dead-button",
          tag.index,
          `${tag.text.match(/^<\w+/)[0]}> with no action: @click / @select / to / type="submit" / as-child, a static disabled, or <!-- no-action: why -->`,
        );
      }
    }
    each("style-block", /^<style\b[^>]*>/gm);
    for (const p of lookOnlyProps(source, tpl)) {
      add("look-only-prop", p.index, `prop \`${p.name}\` used only in :class`);
    }
  }
  return found;
}

/** Every .vue and .ts file under app/src, as paths relative to it with forward slashes. */
function sources(dir = SRC) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const p = join(dir, e.name);
    if (e.isDirectory()) return sources(p);
    return /\.(vue|ts)$/.test(e.name)
      ? [relative(SRC, p).split(sep).join("/")]
      : [];
  });
}

function scanRepo() {
  const findings = [];
  for (const file of sources()) {
    const source = readFileSync(join(SRC, file), "utf8");
    const kind = file.endsWith(".vue") ? "vue" : "ts";
    const found = scanSource(source, kind, {
      insideUi: file.startsWith("design-system/components/"),
      handRolesAllowed: HAND_ROLES_PENDING.has(file),
      insideKit: file.startsWith("design-system/kit/"),
    });
    findings.push(
      ...found.map((f) => `app/src/${file}:${f.line}: ${f.rule}: ${f.text}`),
    );
  }
  if (findings.length === 0) {
    console.log("none");
    return 0;
  }
  console.log(findings.join("\n"));
  console.log(
    "Write it with the design system: tokens, the primitives of app/src/design-system/components and their " +
      "variant constants (docs/frontend-conventions.md).",
  );
  return 1;
}

// Every rule on its fixture: `<rule>.vue.fixture` / `<rule>.ts.fixture` must produce that rule,
// and `clean.vue.fixture` / `clean.ts.fixture` nothing (they hold each rule's allowed look-alikes).
function selfTest() {
  const failures = [];
  const fixtures = readdirSync(FIXTURES).filter((f) => f.endsWith(".fixture"));
  const kindOf = (f) => (f.includes(".vue.") ? "vue" : "ts");
  for (const rule of RULES) {
    const mine = fixtures.filter((f) => f.startsWith(`${rule}.`));
    if (mine.length === 0)
      failures.push(`${rule}: no fixture in scripts/scan-conventions/`);
    for (const f of mine) {
      const found = scanSource(
        readFileSync(join(FIXTURES, f), "utf8"),
        kindOf(f),
      );
      if (!found.some((x) => x.rule === rule))
        failures.push(`${rule}: did not fire on ${f}`);
    }
  }
  for (const f of fixtures.filter((f) => f.startsWith("clean."))) {
    const found = scanSource(
      readFileSync(join(FIXTURES, f), "utf8"),
      kindOf(f),
    );
    if (found.length > 0) {
      failures.push(
        `${f}: ${found.map((x) => `${x.rule} line ${x.line} ${x.text}`).join(", ")}`,
      );
    }
  }
  if (failures.length > 0) {
    console.log(failures.join("\n"));
    return 1;
  }
  console.log(`every rule fires on its fixture (${RULES.length} rules)`);
  return 0;
}

const args = process.argv.slice(2);
if (args.length > 1 || (args.length === 1 && args[0] !== "--self-test")) {
  console.error("usage: node scripts/scan-conventions.mjs [--self-test]");
  process.exit(2);
}
process.exit(args[0] === "--self-test" ? selfTest() : scanRepo());
