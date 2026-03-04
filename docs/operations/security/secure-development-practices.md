# Secure Development Practices

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Practices

1. Default-deny for authn/authz changes.
2. Require explicit threat and abuse-case updates for new endpoints.
3. Block merges on security contract failures and vulnerability budget failures.
4. Keep dependency pinning and allowlists current.
5. Treat integrity and tamper alerts as release blockers.

## Verification

- `bijux-dev-atlas security validate`
- `cargo audit`
- security k6 suites under `ops/load/k6/suites/security-*.js`
