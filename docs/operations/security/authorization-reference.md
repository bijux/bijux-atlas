# Authorization Reference

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define runtime authorization controls and diagnostics.

## Runtime Evaluation Order

1. Authentication establishes principal and request identity.
2. RBAC role and permission evaluation checks action/resource eligibility.
3. Route policy rule evaluation checks principal/action/resource/route effect.
4. Denied requests emit structured `authorization_denied` records.

## Authorization Metrics And Counters

Authorization denials increment:

- `atlas_policy_violations_total{policy="authorization.denied"}`

## Authorization Logs And Audit

Structured events include:

- `authorization_evaluation_started`
- `auth_policy_decision`
- `authorization_denied`

If audit is enabled, denial decisions emit `authorization_denied` audit records
with action, resource kind, and route context.

## Role And Permission Sources

- `configs/security/roles.yaml`
- `configs/security/permissions.yaml`
- `configs/security/policy.yaml`
