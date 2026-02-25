# Layer Rule: No Cross-Layer Fixups

- Owner: `architecture`

## What

States the invariant: no layer may fix another layer's problem.

## Why

Preserves clear ownership boundaries and prevents hidden coupling.

## Contracts

- `ops/e2e` must not patch cluster resources to "fix" deployment issues.
- `ops/load` must not mutate k8s values to make perf tests pass.
- `ops/observe` must not redeploy stack components to recover missing telemetry.
- Boundary gates enforce this rule through lint and contract checks.

## Failure modes

Cross-layer fixups hide root causes and create non-deterministic operations behavior.

## How to verify

```bash
make policies/boundaries-check
make ops/contract-check
make ops/check
```

Expected output: all boundary and contract gates pass with no layer-fixup violations.

See also:
- [Layer Boundary Rules](boundary-rules.md)
- [What E2E Is Not](../../operations/e2e/what-e2e-is-not.md)
