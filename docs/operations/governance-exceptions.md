# Governance Exceptions

- Owner: `bijux-atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last reviewed: `2026-03-03`
- Reason to exist: define the single governed process for temporary exceptions to executable controls.

## How exceptions work

- The SSOT registry is `configs/governance/exceptions.yaml`.
- Every exception targets exactly one governed `scope.kind` and `scope.id`.
- Exceptions are temporary approvals to proceed while a mitigation remains in place.
- Exceptions are validated by `cargo run -q -p bijux-dev-atlas -- governance exceptions validate --format json`.

## How to renew

- Confirm the original reason still applies and the mitigation is still active.
- Shorten scope where possible instead of extending duration.
- Update `expires_at`, keep the same `id`, and document the current mitigation state in the same entry.
- Keep the same mitigation issue link in `tracking_link` so reviewers can follow the history without searching across systems.

## When exceptions are forbidden

- No exception may target a listed no-exception zone from `configs/governance/exceptions.yaml`.
- Current no-exception zones cover secrets in evidence, GitHub Action SHA pinning, and the runtime env allowlist.
- Exceptions are not a substitute for removing a broken control, rewriting a contract, or making a release default less safe.
- Atlas does not use a separate waiver registry today. One-time approvals still go through the same exception registry so they remain visible and expiring.

## Exception SLA

- `low`: maximum 30 days.
- `medium`: maximum 21 days.
- `high`: maximum 7 days unless explicit governance approval is recorded in the entry.

## Governance dashboard inputs

- Summary report: `artifacts/governance/exceptions-summary.json`
- Read-only table: `artifacts/governance/exceptions-table.md`
- Expiry warning report: `artifacts/governance/exceptions-expiry-warning.json`
- Churn report: `artifacts/governance/exceptions-churn.json`

## Review rules

- Every exception must have an owner, mitigation, mitigation issue link, risk acceptor, and verification plan.
- Every exception must target a real contract or check id.
- Expired exceptions fail validation.
- Archived exceptions move to `configs/governance/exceptions-archive.yaml` and stay frozen by content digest.
