# Validation Entrypoints

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@a4f9ebad44bca62517d2fcb77f8f2a38e4c72f54`
- Last changed: `2026-03-03`
- Reason to exist: explain the canonical suite and Make entrypoints for deterministic validation.

## Taxonomy

`contracts` verifies declarative repository and product contracts.

`checks` runs deterministic quality gates such as formatting, linting, and static validation.

`tests` runs deterministic executable test suites that do not require external network access.

## Control Plane Entrypoints

- `bijux dev atlas suites list`
- `bijux dev atlas suites describe --suite checks`
- `bijux dev atlas suites run --suite checks --jobs auto`
- `bijux dev atlas suites run --suite contracts --jobs auto`
- `bijux dev atlas check run CHECK-RUSTFMT-001`
- `bijux dev atlas contract run OPS-DATASETS-001`

`--jobs auto` is the default on the control-plane surface. Use `--jobs <n>` only when a lane needs
an explicit cap. Use `--fail-fast` when you want the first blocking failure to stop the suite.

## Make Entrypoints

- `make suites-list` prints the governed suite ids.
- `make checks-all` runs the full checks suite.
- `make contracts-all` runs the full contracts suite.
- `make suites-all` runs checks then contracts.
- `make checks-group GROUP=rust` runs one checks group.
- `make contracts-group GROUP=datasets` runs one contracts group.
- `make checks-tag TAG=rust` runs one checks tag slice.
- `make contracts-tag TAG=datasets` runs one contracts tag slice.
- `make checks-pure` runs only pure checks.
- `make checks-effect` runs only effectful checks.
- `make contracts-pure` runs only pure contracts.
- `make contracts-effect` runs only effectful contracts.

All suite Make entrypoints accept:

- `JOBS=<n|auto>` to override the suite worker count.
- `FAIL_FAST=1` to stop after the first blocking failure.

## Effects Boundary

These entrypoints keep effectful work explicit:

- `checks-pure` and `contracts-pure` stay within pure registry entries.
- `checks-effect` and `contracts-effect` intentionally include effectful entries.
- `checks-all` and `contracts-all` respect the registry mode metadata and write per-entry artifacts
  under `artifacts/suites/<suite>/<run_id>/`.

## When To Use Each

- Use `make checks-all` before pushing implementation, docs, or config changes that affect quality
  gates.
- Use `make contracts-all` when touching contracts, governance, schemas, or release surfaces.
- Use `make suites-all` when you want the full deterministic non-test validation lane locally.
- Use `make tests-all` when you need executable test coverage in addition to checks and contracts.
