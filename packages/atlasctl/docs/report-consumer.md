# Report Consumer Contract

`atlasctl report` commands emit canonical JSON for downstream tooling.

## Canonical File

- Unified report path: `artifacts/evidence/ci/<run_id>/report.unified.json`
- Gate run path: `artifacts/evidence/gates/<run_id>/report.unified.json`

## Required Fields

- `schema_version`: integer schema revision.
- `tool`: always `atlasctl`.
- `status`: `ok|pass|fail|error`.
- `run_id`: canonical run identifier.
- `results`: stable ordered result rows.

## Validation

- Validate reports with: `./bin/atlasctl report validate --run-id <run_id>`
- Validate an explicit file with: `./bin/atlasctl report validate --file <path>`

## Consumer Rules

- Treat unknown fields as forward-compatible extensions.
- Do not infer status from text logs; use JSON `status` only.
- Use `run_id` + artifact path as immutable lookup key.
