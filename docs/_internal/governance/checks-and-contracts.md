# Checks And Contracts

- Owner: `team:atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define the durable difference between quality checks and governance contracts.

## Checks

Checks are quality gates. They verify a bounded slice of repository health, toolchain policy, or delivery readiness and are allowed to aggregate multiple sub-steps as long as the sub-steps run in deterministic order and emit at least one machine-readable report artifact.

## Contracts

Contracts are governance invariants. They define product or operational guarantees that the control plane must enforce and explain, even when those guarantees are implemented by a lower-level check or effectful verification flow.

## Pure And Effect

- `pure` items may read the repository and compute deterministic results without writing, network access, git mutation, or external process side effects beyond controlled read-only inspection.
- `effect` items may require subprocesses, writable artifact roots, network access, or other bounded side effects that must be declared up front in the registry.

## Required registry rules

- Every check must declare at least one report artifact path in the registry.
- Every check must declare stable report ids and validate those reports against governed schemas.
- Every check command list must be stored in deterministic execution order.
- Every check must be idempotent when rerun against the same repository state and declared inputs.
- The checks registry is the SSOT for stable check ids exposed by Make and CI.
- The contracts registry is the SSOT for stable governance invariant ids exposed by the control plane.

## Checks As Product

Checks are product surfaces, not throwaway shell wrappers. A governed check must be interpretable by humans from its summary, owner, artifacts, and reference docs, and by machines from its stable id, report ids, and schema-validated JSON outputs.
