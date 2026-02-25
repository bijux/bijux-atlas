# Engine Architecture

## Scope
`bijux-dev-atlas-core` is the deterministic engine for repository checks. It owns check selection, execution order, report aggregation, and rendering helpers.

## Check Lifecycle
1. Load and validate the registry (`ops/inventory/registry.toml`).
2. Select checks by suite, domain, tag, and id glob.
3. Resolve each selected check to a `Check` implementation with a stable check id.
4. Enforce effect capabilities before execution.
5. Execute checks and collect violations/evidence.
6. Sort violations and check results deterministically.
7. Aggregate counts, timings, and serialized report output.

## Determinism Rules
- Selection order is sorted by `CheckId`.
- Violation ordering is stable (`code`, `message`, `path`, `line`).
- Report JSON output is tested with golden coverage.
- Evidence paths must not contain timestamp-like segments.

## Effects Boundary
- Check logic receives `CheckContext` and the explicit `EffectsBoundary` contract.
- Filesystem and process behavior flow only through adapters (`Fs`, `ProcessRunner`).
- Capability gates (`fs_write`, `subprocess`, `git`, `network`) are enforced by the runner.

## Inventory Source Of Truth
- Ops inventory manifests are loaded through `load_ops_inventory_cached`.
- Cache invalidation is content-fingerprint based across all inventory inputs.
- Validation remains explicit and fail-fast for missing or malformed manifests.

## Stable Contracts
- `Check` trait contract (`id`, `description`, `tags`, `inputs`, `run`).
- `RunReport`, violations, and evidence are serializable API outputs.
- Registry ids are treated as machine-stable integration points.
