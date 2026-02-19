# Makefiles Contract

## Purpose
Define stable boundaries between public make surface and internal make implementation details.

## Public surface rules
- `configs/ops/public-surface.json` is the SSOT for public make targets.
- `makefiles/root.mk` is the publication surface for public targets (`.PHONY` includes public targets).
- `make help` prints only curated public targets from SSOT.

## Internal target rules
- Non-root makefiles must not publish public targets.
- Internal target naming convention: prefer `_internal.*` (or `internal/*` for transitional aliases).
- Non-root makefile targets must use an internal namespace prefix (`_internal.`, `internal/`, `_`, `ci-`, `ops-`, `dev-`, `layout-`, `path-`, `policy-`, `crate-`, `cli-`, `e2e-`, `stack-`, `observability-`, `ingest-`).

## Artifact rules
- Make-driven lanes write into `artifacts/isolate/<lane>/<run_id>/...`.
- Gate checks emit JSON status in `ops/_generated/gates/<run_id>/`.

## Verification
- `make makefiles-contract`
- `make gates-check`
- `make help`
