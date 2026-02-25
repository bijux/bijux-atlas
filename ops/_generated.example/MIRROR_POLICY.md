# Generated Committed Mirror Policy

`ops/_generated.example/` is a committed mirror/output compatibility area during the migration window.

Rules:
- Generate primary outputs under `ops/_generated/`.
- Only explicit update commands may write `ops/_generated.example/`.
- Every committed file in `ops/_generated.example/` (except `.gitkeep` and runtime compatibility outputs explicitly allowlisted) must be declared in `ops/inventory/generated-committed-mirror.json`.
- `ops/_generated.example/inventory-index.json` is the generated inventory checksum index used for drift comparisons.
