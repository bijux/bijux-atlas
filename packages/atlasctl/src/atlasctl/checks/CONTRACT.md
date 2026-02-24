# Atlasctl Checks Contract

## Contract Summary

The checks subsystem is a single tree and single engine architecture.

- one runtime registry source of truth
- one execution engine surface
- no mirror taxonomies
- no compatibility shim without explicit expiry

## Terminology

- `domain`: policy ownership and intent boundary for checks
- `suite`: named selector manifest from suite registry, not a folder name
- `tag`: check metadata marker used for filtering
- `marker`: stable selector token used by suite and check selection paths
- `visibility`: `public` or `internal` execution exposure
- `effect`: declared capability required by a check

## Canonical Tree

Runtime shape for `atlasctl.checks`:

- `checks/model.py`
- `checks/policy.py`
- `checks/runner.py`
- `checks/report.py`
- `checks/selectors.py`
- `checks/registry.py`
- `checks/domains/*.py`
- `checks/tools/*.py`

## Deletion List

The following trees are migration-only and must be removed:

- `checks/layout/`
- `checks/repo/`
- `checks/tools/*domain*/`
- `checks/registry_legacy/`
- `checks/registry/` package directory

## Identifiers and Taxonomy

- Canonical check id format: `checks_<domain>_<area>_<intent>`
- Check IDs are stable, lowercase snake case, and must not include banned adjectives.
- Domain taxonomy for policy contracts:
  - `root`
  - `python`
  - `docs`
  - `make`
  - `ops`
  - `policies`
  - `product`
- Suite IDs are registry entries only.

## Visibility and Speed

- Visibility values: `PUBLIC` and `INTERNAL`
- Internal checks are hidden unless explicitly requested.
- Speed values: `FAST` and `SLOW`
- Default speed is `FAST`.

## Effect and Boundary Policy

- Default-allow effect: `fs_read`
- Default-deny effects: `subprocess`, `network`
- Writes require explicit declaration and capability enablement.
- Evidence writes are allowed only under `artifacts/...`.
- Checks must not print directly.
- Checks must not rely on cwd; runner passes explicit `repo_root`.
- Checks must not construct time-based output paths; `run_id` is explicit.

## Runtime and Reporting Contract

- Output formats: text, json, jsonl
- Output schemas are stable and versioned.
- Deterministic ordering uses stable check-id sort.
- Exit codes and failure codes are stable by contract.
- Engine error envelope must be typed and machine-readable.
- Budget hooks are available per check and runner profile.

## Registry and Generated Artifacts

- Runtime check selection uses python registry APIs.
- Registry metadata source is registry definition + generated json artifact.
- `REGISTRY.generated.json` is generated and read-only.
- Runtime execution must not read generated registry files as input.

## Compatibility Shim Policy

- Compatibility shims require explicit expiry date metadata.
- Expired shims are policy violations and must be removed.

## Module Budget Policy

- Root/module/file budgets are enforced by policy checks.
- LOC and module count budgets are enforced as ratchets.

## CI Contract

- CI entrypoints select suites.
- CI does not own independent check lists.
- Check and lint surfaces must use the same runner/report path.
