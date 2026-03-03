# Triage Validation Failures

- Owner: `team:atlas-governance`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`

## Where To Look

- `artifacts/suites/<suite>/<run_id>/suite-summary.json`
- `artifacts/suites/<suite>/<run_id>/<id>/result.json`
- `artifacts/suites/<suite>/<run_id>/<id>/stdout.log`
- `artifacts/suites/<suite>/<run_id>/<id>/stderr.log`
- `artifacts/suites/history/<suite>.jsonl`
- `artifacts/governance/registry-status.json`
- `artifacts/governance/registry-work-remaining.json`

## Fast Triage Flow

1. Run `bijux dev atlas suites report --suite <suite> --run <run_id> --failed-only`.
2. Open the failed entry `result.json`.
3. Inspect `stderr.log` for the first actionable line.
4. If the failure is inventory-related, run `bijux dev atlas registry doctor --fix-suggestions`.
5. If the failure regressed recently, run `bijux dev atlas suites diff --suite <suite> --a <older_run> --b <run_id>`.
6. If timing drift is the issue, inspect `bijux dev atlas suites history --suite <suite> --id <check_id>`.

## Interpretation Rules

- A suite failure with a valid `result.json` is an underlying gate failure, not a suite-runner failure.
- A missing or invalid report means the check definition is incomplete.
- A registry doctor failure means suite execution should not be trusted until inventory drift is fixed.
