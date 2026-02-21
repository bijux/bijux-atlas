# Makefiles Contract

## Purpose
Define stable boundaries between public make surface and internal make implementation details.

## Public surface rules
- `configs/ops/public-surface.json` is the SSOT for public make targets.
- `makefiles/root.mk` is the publication surface for public targets (`.PHONY` includes public targets).
- `make help` prints only curated public targets from SSOT.
- Make recipes must call stable `atlasctl` entrypoints only (for CI suite use `atlasctl dev ci run`).
- Make recipes must not call internal suite plumbing directly (forbidden: `atlasctl suite run ci` from makefiles).
- Makefiles may not contain tool logic (tool install, toolchain orchestration, cleanup, ad-hoc scripts).
- Wrapper makefiles (`makefiles/dev.mk`) may only delegate to stable `./bin/atlasctl ...` entrypoints.
- Wrapper make recipes must be single-line delegations; multi-line shell blocks in wrapper recipes are forbidden.

## Internal target rules
- Non-root makefiles must not publish public targets.
- Internal target naming convention: prefer `_internal.*` (or `internal/*` for transitional aliases).
- Non-root makefile targets must use an internal namespace prefix (`_internal.`, `internal/`, `_`, `ci-`, `ops-`, `dev-`, `layout-`, `path-`, `policy-`, `crate-`, `cli-`, `e2e-`, `stack-`, `observability-`, `ingest-`).

## Artifact rules
- Make-driven lanes write into `artifacts/isolate/<lane>/<run_id>/...`.
- Gate checks emit JSON status in `artifacts/evidence/gates/<run_id>/`.

## Verification
- `make makefiles-contract`
- `make gates-check`
- `make help`
- `atlasctl check make-delegation-only`
