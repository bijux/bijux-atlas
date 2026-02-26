# Network Policy Correctness

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines allow/deny network policy behaviors verified in k8s e2e.

## Why

Prevents policy regressions that silently widen egress scope.

## Contracts

- Allowed egress: DNS resolution must work when enabled.
- Forbidden egress: outbound external access must fail by default.
- Enforced by: `ops/k8s/tests/checksuite/checks/network/test_networkpolicy.sh`.

## Failure modes

Policy drift can allow unexpected external connectivity or block required DNS.

## How to verify

```bash
$ make ops-k8s-tests
$ make ops-values-validate
```

Expected output: network policy gate and chart values gate pass.

## See also

- [K8s Test Contract](k8s-test-contract.md)
- [Helm Chart Contract](chart.md)
- `ops-k8s-tests`

- Chart values anchor: `values.server`
