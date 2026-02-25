# Generated Committed Mirror Policy

`ops/_generated.example/` is a committed mirror/output compatibility area during the migration window.

Rules:
- Generate primary outputs under `ops/_generated/`.
- Only explicit update commands may write `ops/_generated.example/`.
- Every committed file in `ops/_generated.example/` (except `.gitkeep` and runtime compatibility outputs explicitly allowlisted) must be declared in `ops/inventory/generated-committed-mirror.json`.
- `ops/_generated.example/inventory-index.json` is the generated inventory checksum index used for drift comparisons.
- `ops/_generated.example/control-plane.snapshot.md` is the control-plane snapshot used for drift checks.
- `ops/_generated.example/docs-drift-report.json` is the docs governance drift report artifact.

## Mirrored Artifacts

- `ops/_generated.example/ops-index.json`: canonical generated index of ops domains and reporting outputs.
- `ops/_generated.example/ops-evidence-bundle.json`: canonical generated evidence envelope with hashes and gate status.
- `ops/_generated.example/scorecard.json`: generated readiness score summary.
- `ops/_generated.example/pins.index.example.json`: generated pins-index example contract.
- `ops/_generated.example/inventory-index.json`: generated checksum index for inventory SSOT files.
- `ops/_generated.example/control-plane.snapshot.md`: generated control-plane snapshot for drift enforcement.
- `ops/_generated.example/docs-drift-report.json`: generated docs governance drift report.
