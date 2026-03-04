# Authorization Troubleshooting

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: diagnose authorization denials quickly.

## Verify role and permission definitions

- Check `configs/security/roles.yaml` for assigned permissions.
- Check `configs/security/permissions.yaml` for action/resource-kind pairs.
- Check `configs/security/role-assignments.yaml` for principal bindings.

## Verify policy route rules

- Check `configs/security/policy.yaml` rule route prefixes and effects.
- Confirm action and resource kind match runtime route mapping.

## Runtime evidence

- `authorization_evaluation_started`
- `auth_policy_decision`
- `authorization_denied`

## CLI diagnostics

- `bijux-dev-atlas security authorization diagnostics`
- `bijux-dev-atlas security authorization validate`
- `bijux-dev-atlas security authorization permissions`
