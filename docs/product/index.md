# Product

Owner: `product`  
Audience: `user`, `operator`, `contributor`  
Type: `concept`  
Reason to exist: define the product promise, boundaries, and reader entrypoints.

## Product Story

`bijux-atlas` is a deterministic genomic data platform with a Rust-native control plane and stable contract surfaces.

Atlas is the stable platform surface for operating, evolving, and consuming the bijux ecosystem through explicit contracts, predictable workflows, and verifiable runtime behavior.

## Product Promise

- Deterministic ingest, registry, and query behavior.
- Immutable published dataset artifacts.
- Explicit dataset identity on all query surfaces.
- Contract-validated compatibility across supported v1 surfaces.

## Product Boundaries

- No implicit default dataset selection.
- No write APIs for genomic entities.
- No mutable in-place updates of published datasets.

## Canonical Product Pages

- [What Is Bijux Atlas](what-is-bijux-atlas.md)
- [Compatibility Promise](compatibility-promise.md)
- [Non Goals](non-goals.md)
- [User Stories](user-stories.md)
