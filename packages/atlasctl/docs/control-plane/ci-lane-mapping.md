# CI Lane Mapping

Reference map for canonical GitHub Actions lanes and their atlasctl CLI front doors.

## PR / Push Lanes

- `ci`: canonical repo CI entrypoint (`make ci`) plus registry/bypass snapshots.
- `control-plane-conformance`: `atlasctl check run make|contracts|docs` and registry/help/bypass gates.
- `repo-hygiene-fast`: `atlasctl fix hygiene --apply`, `atlasctl doctor repo-hygiene`, `atlasctl suite run repo-hygiene`, and a clean git diff gate.
- `hygiene-fast`: `atlasctl suite run lint-fast` plus `atlasctl suite run checks-fast`.
- `suite-product`: `atlasctl suite run product --only fast` (PR-safe lane surface check).
- `suite-ops-fast`: `atlasctl suite run ops --only fast` (PR-safe lane surface check).

## Scheduled / Manual Lanes

- `bypass-burn-down`: weekly bypass inventory + trend + burn-down gates.
- `suite-slow-scheduled`: scheduled/manual slow suite execution with archived artifacts.

## Artifacts

- command inventory: `artifacts/reports/atlasctl/commands.snapshot.json`
- check inventory: `artifacts/reports/atlasctl/checks.snapshot.json`
- bypass report: `artifacts/reports/atlasctl/policies-bypass-report.json`
- suite plans/results: `artifacts/reports/atlasctl/*.json`
- control-plane gates: `artifacts/reports/atlasctl/control-plane-gates*.json`
- bypass governance gate: `artifacts/reports/atlasctl/bypass-governance.gate.json`
