# Docs As Runbooks

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define the required structure for operator runbooks.

## Required Headings

- `Prereqs`
- `Install`
- `Verify`
- `Rollback`

## Why These Headings Matter

- `Prereqs` prevents hidden assumptions during incidents.
- `Install` captures the action or setup path being executed.
- `Verify` defines the concrete success signal.
- `Rollback` gives the bounded escape hatch when the primary action fails.

## Writing Rule

If a page is an operator runbook, keep those headings even when the page is an index or a generic
playbook.
