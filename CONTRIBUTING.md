# Contributing to EmuDome

Thank you for helping. A few rules keep EmuDome legal and free.

## Licence

- EmuDome is licensed GPL-3.0-or-later (`LICENSE`). By sending a contribution you agree that it is
  released under GPL-3.0-or-later, and you confirm you have the right to release it that way.
- Code, assets, fonts, icons and dependencies you add must have a licence compatible with
  GPL-3.0-or-later. Check it **before** adding, and add the component to `THIRD_PARTY.md`.
- Third-party code keeps its own licence: never remove or alter its LICENSE files or copyright notices.

## Never include

- Nintendo material: games/ROMs, BIOS, firmware, encryption keys, system files, logos. Not in the
  repository, not in tests, not as a download link. Tests use homebrew or test ROMs with a free licence,
  kept with their licence file (see `tools/test-rom/`).
- Code that decrypts or bypasses copy protection, or that fetches keys or firmware.
- Secrets: tokens, passwords, signing keys, keystores.
- Paid features, ads, or anything that ties EmuDome to commercial games.

## Before a pull request

Run every check, the licence check included:

```powershell
./scripts/check.ps1        # Windows (PowerShell)
```

```sh
sh scripts/check           # Linux, Git Bash
```

It must end with "all green". The licence check needs `cargo-deny`
(`cargo install cargo-deny --locked`) and the frontend dependencies (`pnpm install` in `app/`).
Once per clone, enable the pre-commit hook: `git config core.hooksPath scripts/git-hooks`.

Project conventions (English code and docs, user-facing text only in the i18n files, no `unwrap()`,
no OS-specific code outside `crates/emudome-platform`) are described in `CLAUDE.md` and enforced by the
same checks where a script can see them.
