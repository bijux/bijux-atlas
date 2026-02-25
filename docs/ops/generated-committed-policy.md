> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Generated Committed Policy

Primary generated outputs are written under `ops/_generated/`.
Committed compatibility/mirror outputs under `ops/_generated.example/` must be declared in `ops/inventory/generated-committed-mirror.json`.

Required-files enforcement:
- Every ops domain declares machine-checkable `required_files` in `REQUIRED_FILES.md`.
- Validation fails if declared files are missing, empty, or not linked from domain `INDEX.md`.
- Tight inventory/schema surfaces fail on undeclared extra files.
