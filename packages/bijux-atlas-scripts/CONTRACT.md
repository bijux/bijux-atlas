# bijux-atlas-scripts Contract

## Scope

Defines behavior guarantees and boundaries for the `bijux-atlas-scripts` product CLI.

## Guarantees

- CLI entrypoint is `bijux-atlas-scripts`.
- Package layout uses `src/bijux_atlas_scripts/`.
- Global run context includes: `run_id`, `profile`, `repo_root`, `evidence_root`, `no_network`.
- Machine-oriented commands provide `--json` output.
- Any `--out-file` write must resolve under `artifacts/evidence/**`.
- Exit-code names and values are loaded from `ops/_meta/error-registry.json`.
- Structured logs always include:
  - `ts`, `level`, `run_id`, `component`, `action`, `file`, `line`.

## Boundaries

- The CLI must not write runtime outputs under `ops/**`.
- The CLI may execute legacy scripts only through `run` with context propagation.
- New command families must preserve help output and JSON contract tests.
- Packaging is internal-only; publishing to PyPI is forbidden until an explicit release policy is added.

## Verification

- `make scripts-check`
- `pytest -q packages/bijux-atlas-scripts/tests`
- `bijux-atlas-scripts --help`
