---
title: Dataset Model
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Dataset Model

Atlas treats a dataset as a release-shaped serving unit, not as a loose bundle
of files.

The stable identity usually combines release, species, and assembly. That
identity is the anchor for ingest, publication, catalog lookup, query routing,
diff workflows, and rollback reasoning.

## What A Dataset Owns

- source-derived validated content
- immutable release artifacts
- catalog-visible identity
- queryable runtime state after publication

## Why It Matters

If the dataset boundary stays clear, Atlas can keep ingest, serving, and
operations honest about what is actually being changed.
