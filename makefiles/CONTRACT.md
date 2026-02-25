# Makefiles Contract

## Purpose
Define stable boundaries between public make surface and internal make implementation details.

## Public surface rules
- `ops/schema/configs/public-surface.schema.json` is the SSOT for public make targets.
- `makefiles/root.mk` is the publication surface for public targets (`.PHONY` includes public targets).
- `make help` prints only curated public targets from SSOT.
- Make recipes must call stable `bijux` command surfaces only.
- Make recipes must not call internal suite plumbing directly from makefiles.
- Makefiles may not contain tool logic (tool install, toolchain orchestration, cleanup, ad-hoc scripts).
- Wrapper makefiles (`makefiles/dev.mk`, `makefiles/docs.mk`, `makefiles/ops.mk`, `makefiles/ci.mk`, `makefiles/policies.mk`) may only delegate to stable `bijux ...` entrypoints.
- Wrapper make recipes must be single-line delegations; multi-line shell blocks in wrapper recipes are forbidden.
- Make is wrapper-only: recipe bodies must not implement tool orchestration logic directly.
- Forbidden in make recipes: raw `cargo`, raw `pytest`, and ad-hoc script execution paths.
- Required governance entrypoint in recipes: `bijux dev atlas ...`.

## Target surface policy
- Public surface: only curated targets exposed through `make help` and documented in `docs/development/makefiles/surface.md`.
- Internal surface: all non-public/maintenance targets must remain outside help output and must not be required by CI workflows.
- Wrapper targets must use neutral naming and must not include banned marketing adjectives.

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
- `bijux dev atlas check run --domain make`
- `bijux dev atlas check run --suite ci_fast`
