# bijux-atlas

SSOT scripting product for `bijux-atlas`.

## Purpose

`bijux-atlas` is the stable Python CLI surface for script orchestration, diagnostics, and machine-readable report helpers.

`scripts/` is in deprecation mode for Python business logic and should converge to `scripts/bin/` shims only; new Python logic must live under `packages/bijux-atlas-scripts/`.

## Command Surface

- `bijux-atlas run <script-path> [args...]`
- `bijux-atlas validate-output --schema <schema.json> --file <output.json> [--json]`
- `bijux-atlas surface [--json] [--out-file <path>]`
- `bijux-atlas doctor [--json] [--out-file <path>]`
- `bijux-atlas ops [--json] [--out-file <path>]`
- `bijux-atlas docs [--json] [--out-file <path>]`
- `bijux-atlas configs [--json] [--out-file <path>]`
- `bijux-atlas policies [--json] [--out-file <path>]`
- `bijux-atlas make [--json] [--out-file <path>]`
- `bijux-atlas inventory [--json] [--out-file <path>]`
- `bijux-atlas report [--json] [--out-file <path>]`

Global context flags:
- `--run-id`
- `--profile`
- `--evidence-root`
- `--no-network`

## Guarantees

- Uses one `RunContext` with `run_id`, `profile`, `repo_root`, and `evidence_root`.
- JSON-producing commands support `--json`.
- Output file writes are enforced to `artifacts/evidence/**` only.
- Exit codes map to `ops/_meta/error-registry.json`.
- Structured logs emit timestamp, level, run_id, component, action, file, and line.

## Packaging And Locking

- Python dependency SSOT uses `pip-tools` style input/lock files:
- `packages/bijux-atlas-scripts/requirements.in`
- `packages/bijux-atlas-scripts/requirements.lock.txt`
- Validate lock consistency with `make scripts-lock-check`.
- Create deterministic virtualenv with `make scripts-venv` at `artifacts/isolate/py/scripts/.venv`.
- Install only from lock via `make scripts-install`.

## Local Development

- `python -m bijux_atlas_scripts --help`
- `make scripts-check`
- `make scripts-test`

The package intentionally has no local `Makefile`; repository-level make targets are the only supported entrypoints.

## Publish Policy

- `bijux-atlas` is internal-only and must not be published to PyPI until an explicit release policy is added.
