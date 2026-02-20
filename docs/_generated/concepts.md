# Concept Graph

- Owner: `docs-governance`

## What

Generated mapping of concept IDs to canonical and pointer pages.

## Why

Provides a deterministic lookup for concept ownership.

## Scope

Concept registry entries from `docs/_style/concepts.yml`.

## Non-goals

No semantic interpretation beyond declared links.

## Contracts

- Exactly one canonical page per concept.
- Pointer pages must reference canonical page.

## Failure modes

Registry drift causes stale concept ownership.

## How to verify

```bash
$ atlasctl docs concept-registry-check --report text
$ make docs
```

Expected output: concept checks pass.

## Concepts

### `concept.crate-boundaries`

- Canonical: [architecture/boundaries.md](../architecture/boundaries.md)
- Pointer: none

### `concept.effects-boundary`

- Canonical: [architecture/effects.md](../architecture/effects.md)
- Pointer: [architecture/boundary-maps.md](../architecture/boundary-maps.md)

### `concept.query-pagination`

- Canonical: [reference/querying/pagination.md](../reference/querying/pagination.md)
- Pointer: none

### `concept.store-integrity`

- Canonical: [reference/store/integrity-model.md](../reference/store/integrity-model.md)
- Pointer: none

### `concept.dataset-immutability`

- Canonical: [product/immutability-and-aliases.md](../product/immutability-and-aliases.md)
- Pointer: [reference/store/immutability-guarantee.md](../reference/store/immutability-guarantee.md)

### `concept.latest-release-alias`

- Canonical: [product/immutability-and-aliases.md](../product/immutability-and-aliases.md)
- Pointer: [reference/registry/latest-release-alias-policy.md](../reference/registry/latest-release-alias-policy.md)

### `concept.registry-federation`

- Canonical: [reference/registry/federation-semantics.md](../reference/registry/federation-semantics.md)
- Pointer: [reference/registry/deterministic-merge.md](../reference/registry/deterministic-merge.md)
- Pointer: [reference/registry/conflict-resolution.md](../reference/registry/conflict-resolution.md)

### `concept.compatibility-matrix`

- Canonical: [contracts/compatibility.md](../contracts/compatibility.md)
- Pointer: [reference/compatibility/bijux-dna-atlas.md](../reference/compatibility/bijux-dna-atlas.md)
- Pointer: [reference/compatibility/umbrella-atlas-matrix.md](../reference/compatibility/umbrella-atlas-matrix.md)
- Pointer: [reference/compatibility/cross-project-compatibility-policy.md](../reference/compatibility/cross-project-compatibility-policy.md)
- Pointer: [reference/registry/plugin-versioning-and-release-independence.md](../reference/registry/plugin-versioning-and-release-independence.md)

### `concept.error-codes`

- Canonical: [contracts/errors.md](../contracts/errors.md)
- Pointer: [reference/registry/error-code-registry.md](../reference/registry/error-code-registry.md)

### `concept.security-coordination`

- Canonical: [operations/security/advisory-process.md](../operations/security/advisory-process.md)
- Pointer: [reference/registry/security-response-coordination.md](../reference/registry/security-response-coordination.md)

### `concept.no-dna-dependency`

- Canonical: [reference/registry/no-dna-dependency-policy.md](../reference/registry/no-dna-dependency-policy.md)
- Pointer: none

### `concept.shared-core-extraction`

- Canonical: [reference/registry/shared-core-extraction.md](../reference/registry/shared-core-extraction.md)
- Pointer: none

### `concept.plugin-contract`

- Canonical: [contracts/plugin/spec.md](../contracts/plugin/spec.md)
- Pointer: [contracts/plugin/mode.md](../contracts/plugin/mode.md)

## See also

- [Concept Registry](../_style/CONCEPT_REGISTRY.md)
- [Concept IDs](../_style/concept-ids.md)
- [Docs Home](../index.md)
