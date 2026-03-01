# Contracts

- Owner: `bijux-atlas-server`

## Purpose

Defines contract surfaces exported or enforced by this crate.

## Invariants

- Contract changes are additive unless explicitly versioned.
- Contract outputs are deterministic.

## Boundaries

- Only crate-owned contracts are documented here.

## Failure modes

- Contract drift between docs, tests, and generated artifacts.

## How to test

```bash
$ make ssot-check
```

Expected output: contract generation and drift checks pass.

```bash
$ make docs-freeze
```

Expected output: generated contract docs are up-to-date.

## Versioning

Contract changes are additive in v1 unless explicitly version-bumped.
