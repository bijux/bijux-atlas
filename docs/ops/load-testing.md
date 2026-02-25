> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Load Testing

- Owner: bijux-atlas-operations
- Stability: stable

Load suites are declared in SSOT manifests and executed via `ops load` commands.

- Validate inputs: `bijux dev atlas ops load check --suite <name>`
- Run: `bijux dev atlas ops load run --suite <name>`
- Compare outcomes: `bijux dev atlas ops load compare --suite <name>`
