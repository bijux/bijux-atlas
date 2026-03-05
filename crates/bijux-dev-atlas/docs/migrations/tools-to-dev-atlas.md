# Migration Map: tools/scripts to bijux-dev-atlas

This mapping defines the migration from legacy tooling paths to dev-atlas command ownership.

## Legacy to replacement mapping

| Legacy path | New invocation | Status |
| --- | --- | --- |
| `python3 tools/cli/atlas_cli_runner.py -- <args>` | `bijux-dev-atlas <args>` | complete |
| `python3 tools/cli/discover_subcommands.py --format text` | `bijux-dev-atlas help --format text` | complete |
| `python3 tools/cli/discover_subcommands.py --format json` | `bijux-dev-atlas help --format json` | complete |
| `python3 tools/cli/interactive_help.py` | `bijux-dev-atlas help --format text` | complete |
| `python3 tools/cli/observability.py ...` | `bijux-dev-atlas observe diagnostics --format json` | complete |
| `tools/cli/shell-integration/install-completions.sh` | `bijux-dev-atlas help --format text` + shell-native completion setup | complete |
| `tools/cli/shell-integration/enable-shell-integration.sh` | `bijux-dev-atlas help --format text` + shell profile configuration | complete |

## Repository migration actions

1. Replace docs references to `tools/cli/*` with `bijux-dev-atlas` commands.
2. Replace generated metadata referencing `tools/cli/*` sources.
3. Remove `tools/` directory.
4. Enforce repo-law contracts forbidding `tools/` and `scripts/` roots.
