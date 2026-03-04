# RBAC Structure

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define role and permission composition for authorization.

## Building Blocks

- Principal: runtime identity category (`user`, `service-account`, `operator`, `ci`)
- Role: named permission bundle with optional inheritance
- Permission: action/resource-kind capability unit
- Policy rule: route-level allow or deny constraint

## Resolution

1. Resolve assigned roles for principal.
2. Expand inherited roles.
3. Union all permission ids.
4. Match action/resource-kind permission.
5. Enforce route policy effect.

## Source Files

- `configs/security/roles.yaml`
- `configs/security/permissions.yaml`
- `configs/security/role-assignments.yaml`
- `configs/security/policy.yaml`
