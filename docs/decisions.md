# Decisions

Summary of tooling, product and design decisions, with the reason behind each one.
The analysis and the roadmap are in the artifact linked from `CLAUDE.md`.

## Tooling

- Packages with pnpm, not npm (2026-11-16): npm was only the scaffold's default. pnpm keeps one content-addressed
  store shared by every worktree (each worktree used to carry a full `node_modules` of its own), installs faster, and
  its strict `node_modules` fails on a dependency the code uses without declaring it. `packageManager` in
  `app/package.json` pins the version; `devEngines.packageManager` with `onFail: "error"` makes `npm install` refuse,
  so a second lockfile cannot appear by accident. The native packages (Tauri CLI, rolldown) come as prebuilt optional
  dependencies: no build script needs allowing.

## Product

Answers to the ten owner questions of `docs/customization.md` section 12, given on 2026-11-17 (Cowork session).
The desk checks that followed are in `docs/feasibility.md`.

- **Default face layout: positional** (RetroArch style, GBA A = right face button). Switchable per controller
  model, as already decided. On Nintendo-layout pads positional and label are the same thing, so the choice only
  matters for Xbox/PlayStation layouts.
- **NDS/3DS screens are composed by EmuDome (option B)**, so positions, sizes and layouts are ours to decide.
  **No fallback** to the cores' own layouts: NDS waits for the compositor (personal project, no deadline).
- **GB/GBC: full support**, a first-class console next to GBA, NDS and 3DS (mGBA runs them). Needs its own touch
  layouts (no L/R), core options and tests; the project description ("GBA, NDS and 3DS") must be updated.
- **Sync: built in from the start, Google Drive**, so PC and phone can be used in parallel. Model taken from
  Rapportino (Google sign-in, `drive.file` scope only, a visible `EmuDome/` folder, resumable uploads, ask only
  when the two sides differ, never lose data), reimplemented in Rust; desktop sign-in uses the installed-app flow
  (loopback redirect + PKCE). What syncs:
  - always: in-game saves, save states, portable settings (never the machine-local ones), patches, and a
    **library index** (so the phone shows "on the PC, not on this device");
  - **games: optional, per game, as a setting** ("Upload to Drive" / "Download to this device"), with the Drive
    space shown before uploading. It must be explained well in the app (design for Cowork). A game added by hand
    with the same identity picks up its saves automatically;
  - sync points: pull before launch, push after exit (and periodically while playing); conflict = changed on both
    sides since the last sync → ask, showing date, device and play time, keep both; a non-blocking "in use on
    <device>" marker.
  - Later idea: PC → phone transfer over the local network (no Drive quota, good for large 3DS games).
  - Uploading the user's own games to the user's own Drive is not EmuDome distributing Nintendo files; the
    project rule in `CLAUDE.md` should say so explicitly.
- **Game identity: content, never the file name.** GBA/GB/GBC: hash of the file; 3DS: title ID; NDS: to decide
  so that trimmed and untrimmed dumps of the same game match (header game code + revision, or a hash of the used
  area). Different regions/revisions are different games.
- **ROM hacks are a different game**: identity = the patched content, own saves; metadata keeps the link to the
  original so a save can be copied by hand. The patch file syncs, not the patched ROM.
- **Covers: user images, generated covers and icons from the ROM only. Never scraping.**
- **RetroAchievements: very low priority** (Phase 5 or later); no hardcore constraints in today's design.
- **Hardware rendering (`SET_HW_RENDER`): a spike right after Phase 0**, to learn early whether 3DS holds up on
  phones; full integration stays in Phase 3.
- **Touch skins: our own pack format with import/export from day one**; `.deltaskin` and RetroArch overlay
  importers in Phase 5. Built-in skins contain only our own art.
- **Macros: always allowed** (a future hardcore mode would disable them).
- **No emulation core of our own.** The existing cores (mGBA, melonDS DS, Azahar) are mature; writing one
  would only make sense if they were poor, and testing it would cost too much (2026-11-17).
- **Keyboard F11 = fullscreen**, the usual convention; no controller combo for it. Swap screens stays L3 + Y
  (2026-11-17).
- **NDS/3DS lid: L3 + Select**, and also from the quick menu (Tools) (2026-11-17).
- **Reference Android phones: the Galaxy A71** as the minimum for GBA and NDS, **plus a powerful recent phone**
  for 3DS, model still to choose (2026-11-17).
