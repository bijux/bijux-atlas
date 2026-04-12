---
title: bijux-atlas Home
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# bijux-atlas

`bijux-atlas` is the product runtime handbook.

## Scope

Use this handbook for product behavior, data workflows, runtime interfaces,
architecture, and compatibility contracts.

Atlas is the repository-owned product surface for:

- ingesting governed GFF3 and FASTA inputs into immutable dataset artifacts
- publishing those artifacts into a serving store and catalog
- serving dataset identity, gene, transcript, sequence, and diff workflows
- exposing a stable CLI, HTTP, and OpenAPI surface around those artifacts

This handbook is intentionally separate from:

- `bijux-atlas-ops`, which explains how Atlas is deployed and operated
- `bijux-atlas-dev`, which explains the repository control plane and maintainer automation

## Reading Paths

Choose a path based on the question you are trying to answer:

- start in [Foundations](foundations/index.md) when you need the product model, terminology, or repository scope
- move to [Workflows](workflows/index.md) when you need to install Atlas, build data, start a server, or run queries
- use [Interfaces](interfaces/index.md) when the question is about exact commands, endpoints, flags, outputs, or env vars
- use [Runtime](runtime/index.md) when you need architecture, lifecycle, storage, request flow, or source-layout explanations
- use [Contracts](contracts/index.md) when you need the strongest compatibility promises and review rules

## Product Boundary

Atlas is artifact-first. The runtime is not meant to serve mutable, partially
built local state directly from ad hoc ingest output. The normal path is:

1. validate and build source inputs into release-shaped artifacts
2. publish artifacts into a serving store
3. resolve catalog state from that store
4. expose queries and metadata through the CLI and HTTP surfaces

That boundary is why repository docs, operations docs, and maintainer docs must
stay distinct. Product readers need to understand the runtime promise without
being forced through Helm, CI, or repository-governance material first.

## Sections

- [Foundations](foundations/index.md)
- [Workflows](workflows/index.md)
- [Interfaces](interfaces/index.md)
- [Runtime](runtime/index.md)
- [Contracts](contracts/index.md)

## Source Anchors

- `crates/bijux-atlas/`
- `crates/bijux-atlas/src/bin/bijux-atlas.rs`
- `crates/bijux-atlas/src/bin/bijux-atlas-server.rs`
- `crates/bijux-atlas/src/bin/bijux-atlas-openapi.rs`
