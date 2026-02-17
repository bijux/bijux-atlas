# E2E Kubernetes Tests

- Owner: `bijux-atlas-operations`

## What

Canonical description of chart validation tests under `ops/e2e/k8s/tests`.

## Why

Ensures install, policy, and operational semantics are validated consistently.

## Scope

Install, network policy, secrets, cached-only mode, rollout/rollback, HPA, and warmup job checks.

## Non-goals

Does not duplicate each test script implementation.

## Contracts

- Runner: `ops-k8s-tests`
- Report on failure: `ops/k8s/tests/report.sh`
- Full contract list: `../k8s/k8s-test-contract.md`

## Failure modes

Chart drift can break runtime semantics while unit tests remain green.

## How to verify

```bash
$ make ops-k8s-tests
```

Expected output: all K8s e2e tests pass.

## See also

- [E2E Index](INDEX.md)
- [K8s Operations](../k8s/INDEX.md)
- [Load CI Policy](../load/ci-policy.md)
- `ops-ci`
