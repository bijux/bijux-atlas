# Scenario Artifact Layout

- Root: `artifacts/ops/scenarios/<scenario-id>/<run-id>/`

## Required Files

- `result.json`
- `summary.md`

## Determinism

- `run-id = sha256("scenario::<id>::<mode>")[0..12]`
- Result payload is stable and schema-versioned.
