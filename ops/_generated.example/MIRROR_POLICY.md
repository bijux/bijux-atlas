# Generated Committed Mirror Policy

`ops/_generated.example/` is a committed mirror/output compatibility area during the migration window.

Rules:
- Generate primary outputs under `ops/_generated/`.
- Only explicit update commands may write `ops/_generated.example/`.
- Every committed file in `ops/_generated.example/` must be declared in `ops/_generated.example/ALLOWLIST.json`.
- `ops/_generated.example/inventory-index.json` is the generated inventory checksum index used for drift comparisons.
- `ops/_generated.example/control-plane.snapshot.md` is the control-plane snapshot used for drift checks.
- `ops/_generated.example/docs-drift-report.json` is the docs governance drift report artifact.
- `ops/_generated.example/contract-coverage-report.json` is the generated contract coverage summary.
- Binary artifacts are forbidden in this directory.
- Every committed JSON artifact in this directory must include `schema_version`.

## Generator Commands

- `ops/_generated.example/ops-index.json`: `bijux dev atlas report build-index --write-example`
- `ops/_generated.example/ops-evidence-bundle.json`: `bijux dev atlas report build-evidence --write-example`
- `ops/_generated.example/scorecard.json`: `bijux dev atlas report build-scorecard --write-example`
- `ops/_generated.example/pins.index.example.json`: `bijux dev atlas inventory pins index --write-example`
- `ops/_generated.example/inventory-index.json`: `bijux dev atlas inventory index --write-example`
- `ops/_generated.example/control-plane.snapshot.md`: `bijux dev atlas ops control-plane snapshot --write-example`
- `ops/_generated.example/docs-drift-report.json`: `bijux dev atlas docs drift --write-example`

## Mirrored Artifacts

- `ops/_generated.example/ALLOWLIST.json`: machine-checkable whitelist for committed artifacts in this directory.
- `ops/_generated.example/ops-index.json`: canonical generated index of ops domains and reporting outputs.
- `ops/_generated.example/ops-evidence-bundle.json`: canonical generated evidence envelope with hashes and gate status.
- `ops/_generated.example/scorecard.json`: generated readiness score summary.
- `ops/_generated.example/pins.index.example.json`: generated pins-index example contract.
- `ops/_generated.example/inventory-index.json`: generated checksum index for inventory SSOT files.
- `ops/_generated.example/control-plane.snapshot.md`: generated control-plane snapshot for drift enforcement.
- `ops/_generated.example/docs-drift-report.json`: generated docs governance drift report.
- `ops/_generated.example/contract-coverage-report.json`: generated contract coverage report for domain contract invariants and check linkage.
