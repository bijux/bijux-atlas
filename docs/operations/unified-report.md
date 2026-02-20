# Unified Report

`artifacts/evidence/make/<run_id>/unified.json` is the canonical machine report for root/root-local lane outcomes.

The report always includes core fields: `schema_version`, `report_version`, `run_id`, `generated_at`, `lanes`, `summary`, and `budget_status`.

Use `make report` to collect lane data and print summary output, and use `./scripts/bin/bijux-atlas-scripts report summarize --run-id <id>` to generate `summary.md` for PR text.

Use `./scripts/bin/bijux-atlas-scripts report validate --run-id <id>` to validate against `ops/_schemas/report/unified.schema.json`.

Use `./scripts/bin/bijux-atlas-scripts report diff --from-run <old> --to-run <new>` for quick lane status changes and `./scripts/bin/bijux-atlas-scripts report export --run-id <id>` to bundle run evidence.
