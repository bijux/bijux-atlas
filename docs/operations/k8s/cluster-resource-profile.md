# Cluster Resource Profile

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Recommended local cluster CPU/memory profile for deterministic k8s tests.

## Why

Under-provisioned clusters produce false failures in rollout, HPA, and liveness checks.

## Contracts

- Kind node resources configured in `ops/stack/kind/cluster.yaml`.
- Suggested minimum for local runs: 4 vCPU, 8 GiB memory.
- Perf profile should use `ops/k8s/values/perf.yaml`.

## Failure modes

Insufficient resources cause timeout flakes and non-actionable failures.

## How to verify

```bash
$ make ops-up
$ make ops-k8s-tests
$ make ops-down
```

Expected output: cluster boots and tests complete without resource-related failures.

## See also

- [E2E Stack](../e2e/stack.md)
- [K8s Test Contract](k8s-test-contract.md)
- `ops-k8s-tests`

- Chart values anchor: `values.server`
