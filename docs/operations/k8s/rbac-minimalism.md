# RBAC Minimalism

- Owner: `bijux-atlas-operations`

## What

Defines RBAC footprint policy for the chart.

## Why

Limits permissions surface for runtime security.

## Contracts

- Chart renders no Role/RoleBinding/ClusterRole/ClusterRoleBinding resources by default.
- Enforced by `ops/e2e/k8s/tests/test_rbac_minimalism.sh`.

## Failure modes

Unreviewed RBAC additions can introduce privilege escalation paths.

## How to verify

```bash
$ make ops-k8s-tests
```

Expected output: RBAC minimalism gate passes.

## See also

- [Security Posture](../security/security-posture.md)
- [K8s Test Contract](k8s-test-contract.md)
- `ops-k8s-tests`
