# Security Incident Response

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@a98808392299dfcbf57f73e25722d2b7070f72e4`
- Reason to exist: provide a repeatable response skeleton for compromised dependencies or leaked secrets.

## Prereqs

- Access to the affected release evidence bundle
- Access to CI workflow history and repository audit trail
- Ability to rotate or revoke the impacted credential, artifact, or workflow

## Install

- Freeze promotions and stop publishing new release artifacts
- Identify whether the incident is a dependency compromise, a leaked secret, or both
- Run `bijux-dev-atlas security validate --format json`
- Run `bijux-dev-atlas security scan-artifacts --dir release/evidence --format json`

## Verify

- Confirm the compromised artifact, dependency, or secret identifier is isolated
- Record the failing contract IDs and affected files
- Generate a fresh evidence bundle after mitigation and confirm the security checks pass

## Rollback

- Revert to the last known good pinned dependency set or last known good evidence bundle
- Revoke any superseded emergency credentials
- Document the final state in the incident record before resuming promotion
