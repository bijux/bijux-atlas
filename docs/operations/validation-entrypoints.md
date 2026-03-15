# Validation Entrypoints

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
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
- `bijux dev atlas suites history --suite checks --id CHECK-RUSTFMT-001`
- `bijux dev atlas suites last --suite checks`
- `bijux dev atlas suites report --suite checks --run <run_id>`
- `bijux dev atlas suites diff --suite checks --a <run_id> --b <run_id>`
- `bijux dev atlas suites lint`
- `bijux dev atlas registry status`
- `bijux dev atlas registry doctor`
- `bijux dev atlas check run CHECK-RUSTFMT-001`
- `bijux dev atlas contract run OPS-DATASETS-001`

`--jobs auto` is the default on the control-plane surface. Use `--jobs <n>` only when a lane needs
an explicit cap. Use `--fail-fast` when you want the first blocking failure to stop the suite. Use
`--strict` when `severity=info` checks should fail the suite instead of surfacing as warnings.

## Make Entrypoints

- `make suites-list` prints the governed suite ids.
- `make suites-all` runs the deep validation suite and the contracts suite.
- `make registry-doctor` validates suite registries and mappings.
- `make ops-fast` runs the fast CI validation suite.
- `make ops-pr` runs the pull-request validation suite.
- `make ops-nightly` runs the nightly validation suite.

All suite Make entrypoints accept:

- `JOBS=<n|auto>` to override the suite worker count.
- `FAIL_FAST=1` to stop after the first blocking failure.

`make registry-doctor` should run before suite execution when you want registry drift to fail
before the worker pool starts.

## Effects Boundary

These entrypoints keep effectful work explicit:

- `make suites-all`, `make ops-fast`, `make ops-pr`, and `make ops-nightly` execute through the
  governed suite runner.
- `make suites-all` respects the registry mode metadata and writes per-entry artifacts
  under `artifacts/suites/<suite>/<run_id>/`.
- The deep and contracts suites keep effectful work isolated according to the governed concurrency
  policy.

## When To Use Each

- Use `make suites-all` when you want the full deterministic non-test validation lane locally.
- Use `make ops-fast` before pushing implementation, docs, or config changes that affect quality
  gates.
- Use `make ops-pr` when touching governance, schemas, or release surfaces.
- Use `make tests-all` when you need executable test coverage in addition to checks and contracts.
