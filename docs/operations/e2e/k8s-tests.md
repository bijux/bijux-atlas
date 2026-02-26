# E2E Kubernetes Tests

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Canonical description of chart validation tests under `ops/k8s/tests`.

## Why

Ensures install, policy, and operational semantics are validated consistently.

## Scope

Install, network policy, secrets, cached-only mode, rollout/rollback, HPA, and warmup job checks.

## Non-goals

Does not duplicate each test script implementation.

## Contracts

- Runner: `ops-k8s-tests`
- Local one-command smoke runner: `k8s-smoke`
- Report on failure: `bin/bijux dev atlas ./crates/bijux-dev-atlas/src/commands/ops/k8s/tests/report.py`
- Full contract list: `../k8s/k8s-test-contract.md`

## Failure modes

Chart drift can break runtime semantics while unit tests remain green.

## How to verify

```bash
$ make k8s-smoke
$ make ops-k8s-tests
```

Expected output: smoke or full K8s suite passes with evidence under `artifacts/evidence/k8s/<run_id>/`.

## See also

- [E2E Index](INDEX.md)
- [K8s Operations](../k8s/INDEX.md)
- [Load CI Policy](../load/ci-policy.md)
- `ops-ci`
