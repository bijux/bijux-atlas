# Repository Automation Inventory

This inventory is the canonical index for automation entrypoints as of 2026-03-05.

## Directories and files discovered

### `tools/` inventory

| Path | Classification | Current role | Replacement dev-atlas command |
| --- | --- | --- | --- |
| `tools/cli/atlas_cli_runner.py` | cli-wrapper | invoke dev-atlas from python wrapper | `bijux-dev-atlas <command>` |
| `tools/cli/discover_subcommands.py` | cli-discovery | command discovery and grouping output | `bijux-dev-atlas help --format json` |
| `tools/cli/interactive_help.py` | cli-help | interactive help display | `bijux-dev-atlas help --format text` |
| `tools/cli/observability.py` | cli-telemetry | command telemetry helper | `bijux-dev-atlas observe diagnostics --format json` |
| `tools/cli/completions/*` | cli-completions | prebuilt shell completion assets | `bijux-dev-atlas help --format text` (runtime discovery baseline) |
| `tools/cli/shell-integration/*.sh` | shell-integration | completion installation helpers | `bijux-dev-atlas help --format text` + shell-native completion install process |
| `tools/cli/fixtures/*` | test-fixture | discovery/help fixtures | moved into docs and contract snapshots |
| `tools/cli/tests/*.py` | test | python tests for wrappers | replaced by Rust CLI contract tests |
| `tools/cli/usability/*` | docs-fixture | usability scenarios | `ops/cli/usability/` owned artifacts |

### `scripts/` inventory

- Directory does not exist in repository root.

### Root shell scripts (`*.sh`) outside `ops/` and `docs/`

- None.

### Root python scripts (`*.py`) outside `docs/`

- None.

### Python/shell automation found outside root

| Path | Classification | Notes |
| --- | --- | --- |
| `tutorials/scripts/*.sh` | tutorial-helper | tutorial-local; not root tooling surface |
| `tutorials/scripts/validate_example_dataset.py` | tutorial-helper | tutorial-local dataset validation helper |

### Make target helper call sites (node/python)

- `Makefile`: no direct `python`, `python3`, or `node` invocations.
- `make/` includes: no direct `tools/` or `scripts/` invocations for current wrapper surfaces.

## Migration status for tasks 1-20

- `tools/` migrated to direct dev-atlas command surface and removed.
- `scripts/` root directory absent.
- Root-level `*.sh` and `*.py` absent.
