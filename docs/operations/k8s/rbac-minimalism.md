# Role-Based Access Control Minimalism

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines RBAC footprint policy for the chart.

## Why

Limits permissions surface for runtime security.

## Contracts

- Chart renders no Role/RoleBinding/ClusterRole/ClusterRoleBinding resources by default.
- Enforced by `ops/k8s/tests/checksuite/checks/security/test_rbac_minimalism.sh`.

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

- Chart values anchor: `values.server`
