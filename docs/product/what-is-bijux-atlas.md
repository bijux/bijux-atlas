# What Is Bijux Atlas

- Owner: `bijux-atlas-product`

## What

Atlas is a release-indexed genomic dataset product composed of three parts: ingest, registry/store, and read-only query serving.

## Why

Institutes need deterministic, versioned query behavior over immutable datasets without hidden transforms.

## Scope

- Dataset build from GFF3/FASTA inputs.
- Immutable artifact publication with checksums.
- Query API for genes, transcripts, sequence, and release diffs.

## Non-goals

- Variant calling, alignment, or annotation editing.
- Mutable in-place datasets.
- Implicit default dataset selection.

## Contracts

- Dataset identity is explicit: `release/species/assembly`.
- Artifacts are immutable after publish.
- All public surfaces are contract-driven from `docs/reference/contracts/`.

## Failure modes

- Missing dataset dimensions => reject request.
- Artifact checksum mismatch => reject dataset open.
- Contract drift => CI fails.

## How to verify

```bash
$ make ssot-check
$ make test
```

Expected output: contract checks and full test suite pass.

## See also

- [Compatibility Promise](compatibility-promise.md)
- [Non Goals](non-goals.md)
- [Glossary](../glossary.md)
