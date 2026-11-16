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
