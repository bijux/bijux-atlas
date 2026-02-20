# bijux-atlas-scripts

SSOT scripting product for `bijux-atlas`.

## Purpose

`bijux-atlas-scripts` is the stable Python CLI surface for script orchestration, diagnostics, and machine-readable report helpers.

## Command Surface

- `bijux-atlas-scripts run <script-path> [args...]`
- `bijux-atlas-scripts validate-output --schema <schema.json> --file <output.json> [--json]`
- `bijux-atlas-scripts surface [--json] [--out-file <path>]`
- `bijux-atlas-scripts doctor [--json] [--out-file <path>]`
- `bijux-atlas-scripts ops [--json] [--out-file <path>]`
- `bijux-atlas-scripts docs [--json] [--out-file <path>]`
- `bijux-atlas-scripts configs [--json] [--out-file <path>]`
- `bijux-atlas-scripts policies [--json] [--out-file <path>]`
- `bijux-atlas-scripts make [--json] [--out-file <path>]`
- `bijux-atlas-scripts inventory [--json] [--out-file <path>]`
- `bijux-atlas-scripts report [--json] [--out-file <path>]`

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
