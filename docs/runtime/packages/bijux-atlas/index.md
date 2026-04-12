---
title: bijux-atlas Package
audience: mixed
type: package
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# bijux-atlas

`bijux-atlas` is the product-facing Atlas runtime crate. It owns genomics
dataset delivery behavior: GFF3 and FASTA ingest, immutable dataset artifacts,
dataset and catalog workflows, CLI and HTTP surfaces, and OpenAPI export.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| ingest and datasets | GFF3 and FASTA ingest, dataset artifact building, dataset verification |
| product interfaces | `bijux-atlas` CLI, HTTP server, OpenAPI export, stable runtime outputs |
| runtime behavior | configuration, policy-governed workflows, diff and garbage-collection flows |
| boundary | does not own repository governance, docs publishing, or maintainer report automation |

## Source Layout

- `crates/bijux-atlas/src/adapters`
- `crates/bijux-atlas/src/app`
- `crates/bijux-atlas/src/contracts`
- `crates/bijux-atlas/src/domain`
- `crates/bijux-atlas/src/runtime`

## Open Next

- open the [Runtime Handbook](../../index.md) for the full product docs path
- open the [Repository Handbook](../../../repository/index.md) when the change crosses into maintainer or repository policy
- open [bijux-dev-atlas](../../../maintainer/packages/bijux-dev-atlas/index.md) when the issue is control-plane automation rather than runtime behavior

## Code Anchors

- `crates/bijux-atlas/README.md`
- `crates/bijux-atlas/Cargo.toml`
- `docs/03-user-guide/index.md`
- `docs/04-operations/index.md`
- `docs/08-contracts/index.md`

## Review Lens

- product behavior should stay owned by the runtime crate, not by repository tooling
- stable runtime contracts should be documented in handbook and contract pages, not hidden in implementation details
- CLI, HTTP, and OpenAPI surfaces should stay aligned as one runtime story
