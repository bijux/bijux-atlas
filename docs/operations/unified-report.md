# Unified Report

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

`artifacts/evidence/make/<run_id>/unified.json` is the canonical machine report for root/root-local lane outcomes.

The report always includes core fields: `schema_version`, `report_version`, `run_id`, `generated_at`, `lanes`, `summary`, and `budget_status`.

Use `make report` to collect lane data and print summary output, and use `bijux dev atlas report summarize --run-id <id>` to generate `summary.md` for PR text.

Use `bijux dev atlas report validate --run-id <id>` to validate against `ops/schema/report/unified.schema.json`.

Use `bijux dev atlas report diff --from-run <old> --to-run <new>` for quick lane status changes and `bijux dev atlas report export --run-id <id>` to bundle run evidence.
