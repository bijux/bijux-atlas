# Validation Entrypoints

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@a4f9ebad44bca62517d2fcb77f8f2a38e4c72f54`
- Last changed: `2026-03-03`
- Reason to exist: explain the three canonical validation aggregators and when to use them.

## Taxonomy

`contracts` verifies declarative repository and product contracts.

`checks` runs deterministic quality gates such as formatting, linting, and static validation.

`tests` runs deterministic executable test suites that do not require external network access.

## Exact Membership

`contracts-all` runs: `contracts-root`, `contracts-repo`, `contracts-crates`,
`contracts-runtime`, `contracts-configs`, `contracts-docs`, `contracts-docker`,
`contracts-make`, `contracts-ops`.

`checks-all` runs: `fmt`, `lint`, `lint-policy-enforce`, `configs-lint`, `docs-validate`,
`k8s-validate`.

`tests-all` runs: `test`.

## Effects Boundary

These entrypoints intentionally exclude effectful operations. They do not invoke network, Docker,
or cluster mutating commands unless a future explicit opt-in surface is added and documented.

## When To Use Each

- Use `contracts-all` when changing contracts, schemas, docs governance, or make surfaces.
- Use `checks-all` when changing implementation or docs and you need the deterministic quality
  gates.
- Use `tests-all` when changing executable code and you need the deterministic test suite.
