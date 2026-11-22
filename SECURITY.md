# Security

## Reporting a problem

Please do not open a public issue for a security problem. Report it privately through GitHub:
on the repository page, open **Security**, then **Report a vulnerability** (GitHub private
vulnerability reporting). The report stays private until a fix is ready.

Include what you found, how to reproduce it, and the version or commit you tested.

## Scope

EmuDome has no release yet: there are no supported versions, and fixes go into the `develop`
branch. Problems in the emulation cores (mGBA, melonDS DS, Azahar) belong to their own projects:
report them upstream, and tell us if EmuDome needs to update a core.

## How EmuDome handles keys, code and data

- **No secrets in the repository.** Signing keys, keystores, passwords and tokens are never
  committed; `scripts/check` refuses keystore files (`*.jks`, `*.keystore`, `*.p12`, `*.pfx`).
  The Google OAuth client id (and its installed-app secret) and the public update key are
  build-time values (`EMUDOME_GOOGLE_CLIENT_ID`, `EMUDOME_GOOGLE_CLIENT_SECRET`,
  `EMUDOME_UPDATER_PUBKEY`), read from the build's environment.
- **Updates** (desktop only): the app reads `latest.json` from the repository's latest GitHub
  release over HTTPS and installs a package only if its signature matches the public key built
  into the app (Tauri's updater). The private key lives only in the release build's environment,
  outside the repository (`docs/release.md`). A build without the public key cannot update
  itself. The updater's TLS is the system's (SChannel on Windows, OpenSSL on Linux). Android
  updates come from Google Play.
- **Android release signing**: the keystore stays outside the repository and is named by
  environment variables (`EMUDOME_KEYSTORE` and its passwords); the release build stops when they
  are missing or when the keystore is inside the repository.
- **Windows packages are not code-signed yet**: Windows SmartScreen warns on the first run.
- **Cores**: a release build loads only the cores in its own resource folder, each checked against
  the sha256 `cores.lock` pins (`crates/emudome-platform/src/cores.rs`); the fetch scripts check
  every downloaded or built file the same way.
- **Google Drive sync** (optional, nothing syncs until the user signs in): sign-in is OAuth with
  PKCE in the system browser, the redirect on `127.0.0.1` at a random port with its state checked,
  and the `drive.file` scope only (EmuDome sees the files it created, nothing else on the user's
  Drive). The refresh token is kept in the system's credential store (Windows Credential Manager,
  the Secret Service on Linux), never in a file; access tokens stay in memory. Drive traffic is
  HTTPS (rustls) checked against the system's root certificates. Sign-in is not available on
  Android yet.
- **Network**: besides Drive (when signed in) and the update check (desktop), EmuDome makes no
  network requests. It sends no telemetry and downloads no games, BIOS, firmware or keys.
- **Local data** (saves, states, settings, library, covers, logs) stays in the app's data folders
  (`it.domehub.emudome`) on the device, written atomically. The `emu://` protocol that serves
  frames, covers and screenshots to the interface lets only the app's own page read them from
  script.
