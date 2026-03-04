---
title: Architecture Summary
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
---

# Architecture Summary

Atlas architecture is deterministic by construction, with explicit boundaries between ingest, query, operations, and release workflows. The control plane enforces policy and contract invariants across these boundaries.

## Primary architectural properties

- Immutable artifact flow from ingest to query and release surfaces.
- Executable governance through checks and contracts.
- Reproducible outputs with traceable evidence bundles.

## Next steps

- [Architecture Overview](overview.md)
- [Architecture Diagrams](diagrams/index.md)
- [Internal Architecture](internal-architecture.md)
