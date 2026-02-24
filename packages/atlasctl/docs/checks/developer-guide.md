# Check Developer Guide

## Add a Check

1. Implement the check function in the target domain module under `packages/atlasctl/src/atlasctl/checks/domains/`.
2. Register it in that same module as a `CheckDef`.
3. Ensure the module exports `register()` and includes the check in `CHECKS`.
4. Run `atlasctl check doctor --json`.
5. Run targeted tests under `packages/atlasctl/tests/checksuite/checks/`.

## Command Surface

- Run checks: `atlasctl check run`
- List checks: `atlasctl check list`
- Explain check metadata: `atlasctl check explain <check_id>`
- Validate registry and invariants: `atlasctl check doctor`

## Selection

- `--domain <domain>`
- `--tag <tag>`
- `--suite <suite>`
- `--id <check_id>`

`--suite lint` is a canonical alias for lint-tagged checks.

## Output and Artifacts

- Canonical report path: `artifacts/atlasctl/check/<run_id>/report.unified.json`
- Timings path: `artifacts/atlasctl/check/<run_id>/timings.json`
- Use `--json` or `--jsonl` for machine-readable output.
