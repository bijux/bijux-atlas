# Security Review Checklist

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: keep security-sensitive changes reviewable and tied to concrete checks.

## Prereqs

- The candidate change is available in a review branch
- The reviewer has access to the affected evidence and policy files

## Install

- Identify whether the change touches auth, secrets, network policy, workflows, release evidence, or image pins
- Run `bijux-dev-atlas security validate --format json`
- Run `bijux-dev-atlas security compliance validate --format json`
- For artifact-bearing changes, run `bijux-dev-atlas security scan-artifacts --dir ops/release/evidence --format json`

## Verify

- Confirm every newly introduced secret key is declared and covered by redaction
- Confirm any new dependency source matches the governed allowlist
- Confirm workflow action refs remain SHA-pinned and allowlisted
- Confirm release evidence still includes SBOM coverage for prod profiles

## Rollback

- Reject or revert the change if any security contract fails
- Restore the last passing policy, workflow, or evidence manifest
- Re-run the security commands and archive the passing reports
