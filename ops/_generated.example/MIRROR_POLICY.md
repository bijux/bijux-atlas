# Generated Committed Mirror Policy

`ops/_generated_committed/` is a committed mirror/output compatibility area during the migration window.

Rules:
- Generate primary outputs under `ops/_generated/`.
- Only explicit update commands may write `ops/_generated_committed/`.
- Every committed file in `ops/_generated_committed/` (except `.gitkeep` and runtime compatibility outputs explicitly allowlisted) must be declared in `ops/inventory/generated-committed-mirror.json`.
