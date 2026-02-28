# Control-plane

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: provide the canonical control-plane product surface for contributors and CI operators.

## Why this section exists

The control-plane replaces ad-hoc scripts with explicit, contract-governed commands, reports, and gates.

## Curated guide map

- [Why a control-plane](why-a-control-plane.md)
- [How suites work](how-suites-work.md)
- [Static and effect mode](static-and-effect-mode.md)
- [Capabilities model](capabilities-model.md)
- [Reports contract](reports-contract.md)
- [CI report consumption](ci-report-consumption.md)
- [Add a check in 30 minutes](add-a-check-in-30-minutes.md)
- [Add a contract registry in 30 minutes](add-a-contract-registry-in-30-minutes.md)
- [Add a gate policy](add-a-gate-policy.md)
- [Debug failing checks](debug-failing-checks.md)
- [CLI reference](cli-reference.md)
- [Lane matrix](lane-matrix.md)

## Stable entrypoints

- `cargo run -q -p bijux-dev-atlas -- --help`
- `cargo run -q -p bijux-dev-atlas -- check --help`
- `cargo run -q -p bijux-dev-atlas -- docs --help`
- `make ci-pr`
- `make docs-build`

## Verify success

A contributor can discover command surfaces, reproduce CI checks locally, and understand output contracts without reading governance internals.

## Next steps

- [Development](../development/index.md)
- [Architecture boundaries](../architecture/boundaries.md)
- [Operations release workflow](../operations/release-workflow.md)
- [Glossary](../glossary.md)
