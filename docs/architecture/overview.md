---
title: Architecture Overview
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - overview
related:
  - docs/architecture/index.md
  - docs/architecture/dataflow.md
---

# Architecture Overview

Atlas architecture is organized around deterministic data movement and explicit control boundaries.

## Layers

1. Source ingest and validation.
2. Versioned artifact and manifest production.
3. Query/runtime serving from approved artifacts.
4. Ops/release/audit evidence generation.
5. Control-plane contract and policy enforcement.

## Entry points

- [System Architecture Specification](system-architecture-specification.md)
- [Architecture Index](index.md)
- [System Dataflow](dataflow.md)
- [Component Documentation](components/index.md)
- [Architecture Diagrams](diagrams/index.md)
