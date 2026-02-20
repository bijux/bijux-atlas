# Scripts Changelog

## Update Process
- Add one entry per shim addition/removal.
- Include: old command, new command, expiry date, owner, and issue link.
- Add a release-note snippet under `.github/templates/scripts-migration-release-note.md`.

## 2026-02-20

- Added tool-driven report command suite (`collect`, `validate`, `summarize`, `scorecard`, `diff`, `trend`, `export`).
- Added lock/venv/install gates for scripts package.
- Added `--version` output with git SHA suffix.
- Added hermetic scripts test mode and scripts SBOM generator.
- Added compatibility command surface: `atlasctl compat list|check`.
- Added root `bin/` migration shims with expiry enforcement.
