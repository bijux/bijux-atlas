# Dev Control Plane Failure Modes

## Purpose

Maps common `bijux dev atlas` failure categories to the primary places to inspect.

## Exit Codes

- `0`: success
- `1`: policy/check failure
- `2`: usage error (invalid arguments / command shape)
- `3`: execution or internal error (tool failure, contract failure, runtime error)

See also: `EXIT_CODES.md`.

## Common Error Shapes

- `*_PARSE_ERROR`
  - Input/config/docs parse failed.
  - Inspect the referenced file path and line in the JSON/text payload.
- `*_SCHEMA_ERROR`
  - Structured input failed schema/contract validation.
  - Inspect config/docs contract files and command-specific schema docs.
- `*_DRIFT_ERROR`
  - Generated/expected output mismatch.
  - Re-run the matching `generate`/`build` command in explicit write mode and compare hashes.
- `OPS_TOOL_ERROR`
  - External tool invocation failed (`kind`, `kubectl`, `helm`, etc).
  - Re-run with `--format json` and inspect subprocess status/stderr details.

## Capability Denials

- Missing `--allow-subprocess`
  - Ops/docs/docker commands that shell out will refuse to run.
- Missing `--allow-write`
  - Commands that write artifacts or apply changes will refuse to run.
- Missing `--allow-network`
  - Network access is denied by default.

## CI Triage Order

1. Re-run the failing command with `--format json`.
2. Check `error_code`, `summary`, and `rows`.
3. Confirm capability flags and selected suite/domain/tag filters.
4. Inspect artifact/report paths under `artifacts/atlas-dev/...`.
