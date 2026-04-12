---
title: Serving Store Model
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Serving Store Model

The serving store is the durable runtime boundary for Atlas data.

It is not raw ingest input, temporary build output, or request-local cache
state. It is the published artifact-backed state the runtime resolves when
serving dataset and query traffic.

## Core Role

- hold published immutable content
- support dataset discovery and resolution
- remain separate from transient runtime caches
