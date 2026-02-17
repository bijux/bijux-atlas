# Testing

- Owner: `bijux-atlas-api`

## Purpose

Defines how test suites are executed and what they validate.

## Invariants

- Tests are deterministic and do not require network.
- Test fixtures are pinned and reproducible.

## Boundaries

- Unit tests validate crate-local behavior.
- Integration tests validate crate contracts with neighbors.

## Failure modes

- Flaky tests due to implicit environment dependencies.
- Incomplete coverage around contract edges.

## How to test

```bash
$ cargo nextest run -p bijux-atlas-api
```

Expected output: all tests pass for this crate.

```bash
$ cargo test -p bijux-atlas-api --doc
```

Expected output: doc tests pass.
