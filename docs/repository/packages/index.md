---
title: Repository Packages
audience: mixed
type: inventory
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Repository Packages

`bijux-atlas` ships two owned Rust packages with different responsibilities.
The split is deliberate: product-facing dataset delivery stays in
`bijux-atlas`, while repository governance and maintainer automation stay in
`bijux-dev-atlas`.

## Package Map

| Package | Owns | Open Next |
| --- | --- | --- |
| `bijux-atlas` | GFF3 and FASTA ingest, immutable dataset artifacts, CLI and HTTP runtime behavior, OpenAPI export | [Runtime Handbook](../../runtime/index.md) |
| `bijux-dev-atlas` | repository governance, docs tooling, policy validation, report generation, operational automation | [Maintainer Handbook](../../maintainer/index.md) |

## Reading Rule

- open [Runtime Handbook](../../runtime/index.md) when the issue is product behavior, data flow, APIs, or runtime contracts
- open [Maintainer Handbook](../../maintainer/index.md) when the issue is repository automation, release proof, or policy enforcement
- stay in [Repository Handbook](../index.md) when the question crosses those boundaries

## Code Anchors

- `Cargo.toml`
- `crates/bijux-atlas/README.md`
- `crates/bijux-dev-atlas/README.md`

## Review Lens

- every published package should appear here once
- product and maintainer ownership should be easy to distinguish without reading the full tree
- package routing should match the handbook structure rendered in navigation
