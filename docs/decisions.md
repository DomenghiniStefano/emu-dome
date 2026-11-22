# Decisions

Summary of tooling, product and design decisions, with the reason behind each one.
The analysis and the roadmap are in the artifact linked from `CLAUDE.md`.

## Tooling

- Packages with pnpm, not npm: npm was only the scaffold's default. pnpm keeps one content-addressed
  store shared by every worktree (each worktree used to carry a full `node_modules` of its own), installs faster, and
  its strict `node_modules` fails on a dependency the code uses without declaring it. `packageManager` in
  `app/package.json` pins the version; `devEngines.packageManager` with `onFail: "error"` makes `npm install` refuse,
  so a second lockfile cannot appear by accident. The native packages (Tauri CLI, rolldown) come as prebuilt optional
  dependencies: no build script needs allowing.
- App identifier `it.domehub.emudome`, was `com.domenghini.emudome` (Trello #90): EmuDome is
  published under DomeHub (domehub.it, the DomeHub Google Play account, `docs/release.md`), and Google Play never
  lets a package name change once an app is published, so it changed before the first release. The desktop data
  folders are named after it: a newer build copies the old ones once and leaves them in place
  (`emudome_core::data_move`). On Android the new identifier is another app, whose data starts empty.

## Product

Answers to the ten questions of `docs/customization.md` section 12. The desk checks that followed are in
`docs/feasibility.md`.

- **Default face layout: positional** (RetroArch style, GBA A = right face button). Switchable per controller
  model, as already decided. On Nintendo-layout pads positional and label are the same thing, so the choice only
  matters for Xbox/PlayStation layouts.
- **NDS/3DS screens are composed by EmuDome (option B)**, so positions, sizes and layouts are ours to decide.
  **No fallback** to the cores' own layouts (the compositor is `emudome_core::screens`).
- **GB/GBC: full support**, a first-class console next to GBA, NDS and 3DS (mGBA runs them), with its own touch
  layouts (no L/R), core options and tests.
- **Sync: built in from the start, Google Drive**, so PC and phone can be used in parallel. Model taken from
  Rapportino (Google sign-in, `drive.file` scope only, a visible `EmuDome/` folder, resumable uploads, ask only
  when the two sides differ, never lose data), reimplemented in Rust; desktop sign-in uses the installed-app flow
  (loopback redirect + PKCE). What syncs:
  - always: in-game saves, save states, portable settings (never the machine-local ones), patches, and a
    **library index** (so the phone shows "on the PC, not on this device");
  - **games: optional, per game, as a setting** ("Upload to Drive" / "Download to this device"), with the Drive
    space shown before uploading. It must be explained well in the app. A game added by hand with the same
    identity picks up its saves automatically;
  - sync points: pull before launch, push after exit (and periodically while playing: not built yet, only the
    «in use» marker beats while playing); conflict = changed on both
    sides since the last sync → ask, showing date, device and play time, keep both; a non-blocking "in use on
    <device>" marker.
  - Later idea: PC → phone transfer over the local network (no Drive quota, good for large 3DS games).
  - Uploading the user's own games to the user's own Drive is not EmuDome distributing Nintendo files; the
    project rules in `CLAUDE.md` say so.
- **Game identity: content, never the file name.** GBA/GB/GBC: hash of the file; 3DS: title ID and remaster
  version (`.3dsx` homebrew: hash of the file); NDS: hash of the used area, so that trimmed and untrimmed dumps of
  the same game match (`emudome_core::library::identity`). Different regions/revisions are different games (the
  library groups regional versions on one tile, each keeping its own identity and saves).
- **ROM hacks are a different game**: identity = the patched content, own saves; metadata keeps the link to the
  original so a save can be copied by hand. The patch file syncs, not the patched ROM.
- **Covers: user images, generated covers and icons from the ROM only. Never scraping.**
- **RetroAchievements: very low priority** (Phase 5 or later); no hardcore constraints in today's design.
- **Hardware rendering (`SET_HW_RENDER`): a spike right after Phase 0**, to learn early whether 3DS holds up on
  phones; full integration stays in Phase 3.
- **Touch skins: our own pack format with import/export from day one**; `.deltaskin` and RetroArch overlay
  importers in Phase 5. Built-in skins contain only our own art. Not built yet: today the touch controls have two
  built-in styles and no import or export.
- **Macros: always allowed** (a future hardcore mode would disable them).
- **No emulation core of our own.** The existing cores (mGBA, melonDS DS, Azahar) are mature; writing one
  would only make sense if they were poor, and testing it would cost too much.
- **Keyboard F11 = fullscreen**, the usual convention; no controller combo for it. Swap screens stays L3 + Y.
- **NDS/3DS lid: L3 + Select**, and also from the quick menu (Tools). Only melonDS DS has a lid; on 3DS it is
  shown disabled.
- **Reference Android phones: the Galaxy A71** as the minimum for GBA and NDS, **plus a powerful recent phone**
  for 3DS, model still to choose.
