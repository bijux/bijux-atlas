# bijux-atlas Contract

## Scope

Defines behavior guarantees and boundaries for the `bijux-atlas` product CLI.

## Guarantees

- CLI entrypoint is `bijux-atlas`.
- Package layout uses `src/atlasctl/`.
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
- `pytest -q packages/bijux-atlas/tests`
- `bijux-atlas --help`
