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
- `make cargo/fmt`: Cargo fmt gate (CI-safe).
- `make cargo/lint`: Cargo lint gate (CI-safe).
- `make cargo/test-fast`: Cargo fast unit-focused tests.
- `make cargo/test`: Cargo CI-profile test lane.
- `make cargo/test-all`: Cargo full nextest lane.
- `make cargo/test-contracts`: Cargo contract-focused tests.
- `make cargo/audit`: Cargo audit gate.
- `make cargo/bench-smoke`: Cargo benchmark smoke lane.
- `make cargo/coverage`: Cargo coverage lane (kept out of root).
- `make lane-cargo`: Rust lane (fmt/lint/check/test/coverage/audit) under isolated artifacts.
- `make lane-docs`: Docs lane (docs, freeze, hardening) under isolated artifacts.
- `make lane-ops`: Ops lane without cluster bring-up (lint/contracts) under isolated artifacts.
- `make lane-scripts`: Scripts lane (lint/tests/audit) under isolated artifacts.
- `make lane-configs-policies`: Config + policy lane under isolated artifacts.
- `make cargo/all`: Local exhaustive Rust lane.
- `make docs/all`: Docs lane.
- `make ops/all`: Ops lint/schemas/contracts lane.
- `make scripts/all`: Scripts lint/tests/audit lane.
- `make configs/all`: Config schema + drift lane.
- `make policies/all`: deny/audit/policy-relaxations lane.
- `make root`: CI-fast lane subset (no cluster bring-up).
- `make root-local`: Run all lanes in parallel plus ops smoke lane (`PARALLEL=0` for serial).
- `make root-local-summary`: Print pass/fail summary and artifact paths per lane.
- `make root-local-open`: Open the latest summary file (or print its path).
- `make report/merge`: Merge lane reports into one unified JSON report.
- `make report/print`: Print a CI-style human summary.
- `make report/md`: Generate markdown summary for PR comments.
- `make report/junit`: Generate optional JUnit XML from lane reports.
- `make doctor`: Print tool versions/paths/env and store doctor report.
- `make prereqs`: Validate required tools/versions before heavy lanes.
- `make print-env`: Print key lane/gate environment variables.
- `make clean-safe`: Clean only safe generated make artifact directories.
- `make clean-all CONFIRM=YES`: Clean all allowed generated directories.
- `make local/all`: Run all meaningful local lanes.
- `make ci/all`: Deterministic CI superset.
- `make nightly/all`: Slow nightly suites.
- `make repro TARGET=<lane-target> [SEED=<n>]`: Deterministic lane replay with seed propagation.

## Cargo Boundary

- `makefiles/cargo.mk`: CI-safe, deterministic cargo targets only.
- `makefiles/cargo-dev.mk`: local convenience targets only (`DEV_ONLY=1` required per target).
- Profile SSOT: `docs/development/cargo-profiles-ssot.md`.

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
$ make root-local
$ make root-local-summary
$ make report/print
$ make internal-list
$ make makefiles-contract
$ python3 scripts/docs/check_make_targets_documented.py
$ make ops-script-coverage
```

Expected output: target listing, target docs check, and ops script mapping check pass.

## See also

- [Makefiles Public Surface](../docs/development/makefiles/surface.md)
- [Repository Surface](../docs/development/repo-surface.md)
