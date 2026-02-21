# Atlasctl Goldens

All golden snapshots under this directory are SSOT test fixtures for atlasctl CLI behavior.

## Update policy

- Do not edit golden files manually.
- Regenerate goldens only via:
  - `python -m atlasctl.cli gen goldens`
  - or `atlasctl gen goldens` (console script)
- Review the diff and commit only intentional behavior changes.

## Current generated files

- `help.json.golden`
- `commands.json.golden`
- `surface.json.golden`
- `explain.check.json.golden`
- `check-list.json.golden`
- `cli_help_snapshot.txt`
- `cli_help_commands.expected.txt`
