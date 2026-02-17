# Makefiles Public Surface

- Owner: `docs-governance`

## What

Defines stable make target interfaces exported by the repository root `Makefile`.

## Why

Make targets are operational interfaces used by CI and local workflows.

## Scope

Public targets printed by `make help`.

## Non-goals

Does not document internal helper targets prefixed with `_`.

## Contracts

Stable targets:

- `fmt`
- `lint`
- `check`
- `test`
- `test-all`
- `coverage`
- `audit`
- `openapi-drift`
- `ci`
- `fetch-fixtures`
- `fetch-real-datasets`
- `load-test`
- `load-test-1000qps`
- `cold-start-bench`
- `memory-profile-load`
- `run-medium-ingest`
- `run-medium-serve`
- `crate-structure`
- `crate-docs-contract`
- `cli-command-surface`
- `culprits-all`
- `culprits-max_loc`
- `culprits-max_depth`
- `culprits-file-max_rs_files_per_dir`
- `culprits-file-max_modules_per_dir`
- `e2e-local`
- `e2e-k8s-install-gate`
- `e2e-k8s-suite`
- `e2e-perf`
- `e2e-realdata`
- `ops-up`
- `ops-down`
- `ops-reset`
- `ops-publish-medium`
- `ops-deploy`
- `ops-warm`
- `ops-smoke`
- `ops-metrics-check`
- `ops-traces-check`
- `ops-k8s-tests`
- `ops-load-smoke`
- `ops-load-full`
- `ops-drill-store-outage`
- `ops-drill-corruption`
- `ssot-check`
- `observability-check`
- `docs`
- `docs-serve`
- `docs-freeze`
- `docs-hardening`
- `layout-check`
- `layout-migrate`
- `bootstrap`
- `doctor`
- `help`

Perf targets:

- `perf-nightly`

Dev targets:

- `dev-fmt`
- `dev-lint`
- `dev-check`
- `dev-test`
- `dev-test-all`
- `dev-coverage`
- `dev-audit`
- `dev-ci`
- `dev-clean`

## Failure modes

Undocumented target changes break CI, scripts, or developer workflows.

## How to verify

```bash
$ make help
$ python3 scripts/docs/check_make_targets_documented.py
```

Expected output: make target documentation check passes.

## See also

- [Repo Surface](../repo-surface.md)
- [Scripts Index](../scripts/INDEX.md)
- [Crate Layout Contract](../../architecture/crate-layout-contract.md)
