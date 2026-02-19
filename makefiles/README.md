# Makefiles SSOT

- Owner: `build-and-release`

## What

Defines the single-source build and operations target surface for the repository.

## Why

Keeps operational entrypoints stable, discoverable, and auditable through `make`.

## Contracts

- Root `Makefile` is a thin dispatcher that only includes `makefiles/*.mk`.
- Public interfaces are make targets, not direct script paths.
- `makefiles/root.mk` is the publication surface for public targets.
- `makefiles/_macros.mk` centralizes shared run-id/isolation/logging/python helpers.
- `makefiles/CONTRACT.md` is the normative contract for make target boundaries.
- Tier model:
  - `root`: CI-fast deterministic gate.
  - `root-local`: local superset with parallel isolated lanes.
  - `ci`: workflow-equivalent full CI matrix.
  - `nightly`: CI plus long-running/nightly ops suites.
- Domain logic lives in:
  - `makefiles/root.mk`
  - `makefiles/help.mk`
  - `makefiles/layout.mk`
  - `makefiles/ci.mk`
  - `makefiles/cargo.mk`
  - `makefiles/cargo-dev.mk`
  - `makefiles/docs.mk`
  - `makefiles/ops.mk`
  - `makefiles/policies.mk`
- Any new public target must be listed in `docs/development/makefiles/surface.md`.

## Public Targets

- `make help`: Show curated public targets grouped by namespace.
- `make list`: List curated public targets with one-line descriptions.
- `make explain TARGET=<target>`: Show description, lanes, and expansion tree for one public target.
- `make graph TARGET=<target>`: Print a compact dependency tree for one public target.
- `make gates`: Print top-level areas and mapped public targets.
- `make quick`: Minimal loop (fmt + lint + test).
- `make cargo/all`: Local exhaustive Rust lane.
- `make docs/all`: Docs lane.
- `make ops/all`: Ops lint/schemas/contracts + bounded smoke lane.
- `make scripts/all`: Scripts lint/tests/audit lane.
- `make configs/all`: Config schema + drift lane.
- `make policies/all`: deny/audit/policy-relaxations lane.
- `make local/all`: Run all meaningful local lanes.
- `make ci/all`: Deterministic CI superset.
- `make nightly/all`: Slow nightly suites.

## Failure modes

- Direct script usage bypasses target contracts and drifts from CI behavior.
- Untracked target additions create undocumented and unstable interfaces.

## How to verify

```bash
$ make help
$ make gates
$ make list
$ make explain TARGET=ci/all
$ make graph TARGET=ci/all
$ make internal-list
$ make makefiles-contract
$ python3 scripts/docs/check_make_targets_documented.py
$ make ops-script-coverage
```

Expected output: target listing, target docs check, and ops script mapping check pass.

## See also

- [Makefiles Public Surface](../docs/development/makefiles/surface.md)
- [Repository Surface](../docs/development/repo-surface.md)
