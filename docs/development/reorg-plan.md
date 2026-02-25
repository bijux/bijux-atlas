# Reorganization Invariants Plan

## Goal
Keep repository operations discoverable and stable by capping public surface and enforcing SSOT-driven contracts.

## Invariants
1. Public surface is SSOT in `ops/schema/configs/public-surface.schema.json`.
2. Human entrypoints are only `make <public-target>` and `bijux dev atlas ...`.
3. Docs may reference only public make targets and bijux dev atlas command surfaces (allowlist exceptions only).
4. Shell script naming is kebab-case; Python naming is snake_case.
5. Mixed underscore/dash duplicate naming forms are forbidden.
6. Public entrypoint surface is capped to at most 10 entrypoints per area.
7. Config concepts have one canonical source (`no shadow config` rule).
8. Tooling outputs use `artifacts/isolate/<lane>/<run_id>/...`.
9. Gates emit machine-readable JSON results for unified reporting.
10. Unified run report must follow `ops/schema/report/unified.schema.json`.
11. Inventories are generated and verified for drift.
12. Generated directories are restricted to `docs/_generated`, `ops/_generated`, and `artifacts/*`.
13. Root dumping is forbidden: new top-level files must be explicitly allowlisted.

## Enforcement
- `make gates`:
  - public surface contract
  - docs surface contract
  - suite-id docs contract
  - JSON gate reports under `artifacts/evidence/gates/<run_id>/`
  - naming convention checks
  - duplicate naming form checks
  - no-shadow-config checks
- `make check-gates`:
  - generated-dir policy
  - root-shape and root-dump checks
- `make inventory`:
  - regenerate ops/make/docs/naming/repo-surface inventories
- `make verify-inventory`:
  - fail when generated inventories are stale

## Operational policy
- Cap public surface, keep internals flexible.
- Additions to public surface require updating SSOT and docs in the same change.
- Exception entries must be tracked in explicit allowlist files.
