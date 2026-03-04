# Audit Procedure

1. Generate audit bundle: `bijux-dev-atlas audit bundle generate --output-format json`.
2. Validate audit bundle: `bijux-dev-atlas audit bundle validate --output-format json`.
3. Generate compliance report: `bijux-dev-atlas audit compliance report --output-format json`.
4. Run final readiness gate: `bijux-dev-atlas audit readiness validate --output-format json`.

Artifacts are written under `artifacts/audit/`.
