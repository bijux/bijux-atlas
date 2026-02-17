# E2E Kubernetes Tests

- Owner: `bijux-atlas-operations`

## What

Canonical description of chart validation tests under `e2e/k8s/tests`.

## Why

Ensures install, policy, and operational semantics are validated consistently.

## Scope

Install, network policy, secrets, cached-only mode, rollout/rollback, HPA, and warmup job checks.

## Non-goals

Does not duplicate each test script implementation.

## Contracts

- Runner: `e2e/k8s/tests/run_all.sh`
- Install gate: `e2e/k8s/tests/test_install.sh`
- Cached-only mode: `e2e/k8s/tests/test_cached_only_mode.sh`

## Failure modes

Chart drift can break runtime semantics while unit tests remain green.

## How to verify

```bash
$ ./e2e/k8s/tests/run_all.sh
```

Expected output: all K8s e2e tests pass.

## See also

- [E2E Index](INDEX.md)
- [K8s Operations](../k8s/INDEX.md)
- [Load CI Policy](../load/ci-policy.md)
