# Bypass Policy

Bypass files under `configs/policy/` are temporary exceptions, not a second policy system.

## Rules
- Every bypass entry must include: `owner`, `issue_id`, `expiry`, `justification`, and `removal_plan`.
- Expiry must not be in the past.
- Expiry horizon above 90 days requires explicit approval in `configs/policy/bypass-horizon-approvals.json`.
- New bypass source files are forbidden unless explicitly registered.

## Commands
- Inventory: `./bin/atlasctl policies bypass list --report json`
- Consolidated report: `./bin/atlasctl policies bypass report --out artifacts/reports/atlasctl/policies-bypass-report.json`
- Entry drill-down: `./bin/atlasctl policies bypass entry --id <source:key>`

## CI Contract
- PRs upload `artifacts/reports/atlasctl/policies-bypass-report.json`.
- Repo checks enforce bypass metadata quality and budget ratchet.
