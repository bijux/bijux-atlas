---
title: Runtime Surfaces
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Runtime Surfaces

`bijux-atlas` exposes several runtime surfaces that need to be understood
together.

## Main Surfaces

- CLI workflows for ingest, validation, and local inspection
- HTTP endpoints for serving dataset content
- OpenAPI export for published API shape
- runtime configuration and environment inputs
- structured machine-readable output for automation

## Why Group Them

Atlas users usually touch more than one surface. A local ingest workflow often
ends in server startup or API use, so the docs should describe them as one
product system instead of scattered isolated pages.
