# Add a New Dataset Type

- Owner: `docs-governance`
- Stability: `evolving`

## What

Procedure to add support for a new dataset type in atlas.

## Why

Ensures new dataset forms stay contract-driven and release-indexed.

## Scope

Contract, parser/ingest, fixtures, and ops validation updates.

## Non-goals

Does not define scientific curation policy.

## Contracts

- Update [Config Keys Contract](../contracts/config-keys.md) if new config knobs are required.
- Keep dataset identity release-indexed (`release/species/assembly` compatible).
- Add/refresh fixtures under `ops/datasets/fixtures/`.

## Steps

1. Add parser + ingest support in relevant crates.
2. Add deterministic fixture(s) in `ops/datasets/fixtures/`.
3. Extend tests and golden snapshots.
4. Update operations docs if behavior changes.

## Failure modes

- Missing fixture and snapshot updates cause non-deterministic regressions.
- Contract omissions cause docs/code drift.

## How to verify

```bash
$ make contracts
$ make test
$ make ops-smoke
```

Expected output: contract checks and tests pass with no drift.

## See also

- [Contracts SSOT](../contracts/INDEX.md)
- [Operations Index](../operations/INDEX.md)
- [Terms Glossary](../_style/terms-glossary.md)
